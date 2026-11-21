use crate::{
    LineageKind, Result, Store, StoreConfig, TemporaryRoot,
    error::require,
    objects::Objects,
    platform, schema,
    store::{check_header, current, head},
};
use core_types::{
    WireMessage,
    generated::{CoreTransactionV2, EventEnvelope, ObjectRef},
    scalar::hex,
};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use std::{marker::PhantomData, path::Path, sync::Arc};

#[cfg(test)]
std::thread_local! {
    pub(crate) static FULL_CHECKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub struct Reader<L: LineageKind> {
    pub(crate) db: Connection,
    pub(crate) objects: Objects,
    verified_head: (Option<String>, u64, u64),
    writer_data_version: Option<i64>,
    proof: Option<PublicationProof>,
    _lineage: PhantomData<L>,
}

type Head = (Option<String>, u64, u64);

#[derive(PartialEq)]
enum CheckMode {
    Automatic,
    Full,
    Cached,
}

pub(crate) struct PublicationProof {
    version: i64,
    head: Head,
    prefix: crate::recovery::Prefix,
    projection: Connection,
    tables: crate::derived::Frontiers,
}

impl PublicationProof {
    fn empty() -> Result<Self> {
        Ok(Self {
            version: 0,
            head: (None, 0, 0),
            prefix: Default::default(),
            projection: crate::derived::empty()?,
            tables: Default::default(),
        })
    }
    fn extend(
        &mut self,
        db: &Connection,
        objects: &Objects,
        header: &core_types::generated::LineageHeaderV2,
        branch: &str,
    ) -> Result<()> {
        let inputs = self.prefix.inputs;
        let processed = self.prefix.position;
        crate::recovery::verify_tail(db, objects, header, branch, &mut self.prefix)?;
        crate::derived::extend(&self.projection, db, objects, inputs, processed)?;
        self.tables.compare(db, &self.projection)?;
        self.head = head(db)?;
        Ok(())
    }
}

impl<L: LineageKind> Store<L> {
    fn writer_version(&self) -> Result<i64> {
        Ok(self
            .db
            .pragma_query_value(None, "data_version", |r| r.get(0))?)
    }

    /// Check the observation again on the sole writer after a background check.
    pub fn publication_matches(&self, reader: &Reader<L>) -> Result<bool> {
        Ok(Arc::ptr_eq(&self.objects.pins, &reader.objects.pins)
            && reader.writer_data_version == Some(self.writer_version()?)
            && reader.verified_head == self.head()?)
    }

    pub fn has_verified_publication(&self) -> Result<bool> {
        let version = self.writer_version()?;
        let head = self.head()?;
        Ok(self
            .proof
            .borrow()
            .as_ref()
            .is_some_and(|p| p.version == version && p.head == head))
    }

    /// The service preflights misses on a blocking worker. A callback racing an
    /// external writer must fail closed instead of running recovery on its loop.
    pub fn set_cached_reads_only(&mut self, enabled: bool) {
        self.cached_reads_only = enabled;
    }

    pub fn verified_reader(&self) -> Result<Reader<L>> {
        let mode = if self.cached_reads_only {
            CheckMode::Cached
        } else {
            CheckMode::Automatic
        };
        let check = self.prepare_check(None, mode, || {})?;
        let mut reader = check()?;
        self.accept_publication(&mut reader)?;
        Ok(reader)
    }

    pub fn receipt_check<F: FnOnce() + Send + 'static>(
        &self,
        cached: Option<Reader<L>>,
        snapshot_pinned: F,
    ) -> Result<impl FnOnce() -> Result<Reader<L>> + Send + 'static + use<L, F>> {
        self.prepare_check(cached, CheckMode::Automatic, snapshot_pinned)
    }

    pub fn full_publication_check(
        &self,
    ) -> Result<impl FnOnce() -> Result<Reader<L>> + Send + 'static + use<L>> {
        self.prepare_check(None, CheckMode::Full, || {})
    }

    fn prepare_check<F: FnOnce() + Send + 'static>(
        &self,
        cached: Option<Reader<L>>,
        mode: CheckMode,
        snapshot_pinned: F,
    ) -> Result<impl FnOnce() -> Result<Reader<L>> + Send + 'static + use<L, F>> {
        let version = self.writer_version()?;
        let published = self.head()?;
        let root = self.root.clone();
        let config = self.objects.config;
        let pins = Arc::clone(&self.objects.pins);
        let temporary: bool = self
            .db
            .query_row("SELECT temporary FROM metadata", [], |r| r.get(0))?;
        let warm = mode != CheckMode::Full
            && self
                .proof
                .borrow()
                .as_ref()
                .is_some_and(|p| p.version == version && p.head == published);
        if !warm && mode == CheckMode::Cached {
            return Err(crate::Error::VerificationPending);
        }
        let previous = if warm {
            None
        } else {
            self.proof.borrow_mut().take()
        };
        Ok(move || {
            // Discarding an invalid projection may release the whole history.
            // Keep that work on the same worker as the replacement full check.
            let prefix = previous.filter(|p| {
                mode != CheckMode::Full && p.version == version && p.head.1 < published.1
            });
            let mut reader = if warm {
                if let Some(reader) = cached
                    && reader.verified_head == published
                    && reader.writer_data_version == Some(version)
                    && Arc::ptr_eq(&pins, &reader.objects.pins)
                {
                    return Ok(reader);
                }
                let db = Connection::open_with_flags(
                    root.join("store.sqlite"),
                    OpenFlags::SQLITE_OPEN_READ_ONLY,
                )?;
                schema::configure(&db, config, false)?;
                require(head(&db)? == published, "read handle publication mismatch")?;
                Reader {
                    db,
                    objects: Objects::new(root.join("objects"), config),
                    verified_head: published.clone(),
                    writer_data_version: None,
                    proof: None,
                    _lineage: PhantomData,
                }
            } else {
                Reader::check_prefix(&root, config, temporary, prefix, || {}, snapshot_pinned)?
            };
            require(
                reader.verified_head == published,
                "publication changed during receipt verification",
            )?;
            reader.objects.pins = pins;
            reader.writer_data_version = Some(version);
            if let Some(p) = &mut reader.proof {
                p.version = version;
            }
            Ok(reader)
        })
    }

    /// Install only on the originating Store, after rechecking the writer epoch.
    pub fn accept_publication(&self, reader: &mut Reader<L>) -> Result<()> {
        require(
            self.publication_matches(reader)?,
            "publication changed during receipt verification",
        )?;
        if let Some(proof) = reader.proof.take() {
            self.proof.replace(Some(proof));
        }
        Ok(())
    }

    pub fn receipt_reader<'a>(&self, slot: &'a mut Option<Reader<L>>) -> Result<&'a Reader<L>> {
        let check = self.receipt_check(slot.take(), || {})?;
        let mut reader = check()?;
        self.accept_publication(&mut reader)?;
        Ok(slot.insert(reader))
    }
}
impl<L: LineageKind> Reader<L> {
    pub fn open_temporary(root: &TemporaryRoot, name: &str, config: StoreConfig) -> Result<Self> {
        Self::open(&root.store_path(name)?, config, true)
    }
    pub(crate) fn open(root: &Path, config: StoreConfig, temporary: bool) -> Result<Self> {
        Self::open_checked(root, config, temporary, || {}, || {})
    }
    // Internal seam for deterministic read/append interleavings; no locks or
    // transactions are passed to callbacks. Standalone opens use no-op hooks.
    pub(crate) fn open_checked(
        root: &Path,
        config: StoreConfig,
        temporary: bool,
        after_capture: impl FnOnce(),
        after_snapshot: impl FnOnce(),
    ) -> Result<Self> {
        let mut reader =
            Self::check_prefix(root, config, temporary, None, after_capture, after_snapshot)?;
        reader.proof = None;
        Ok(reader)
    }

    fn check_prefix(
        root: &Path,
        config: StoreConfig,
        temporary: bool,
        prefix: Option<PublicationProof>,
        after_capture: impl FnOnce(),
        after_snapshot: impl FnOnce(),
    ) -> Result<Self> {
        platform::no_links(root)?;
        let root = root.canonicalize()?;
        let db = Connection::open_with_flags(
            root.join("store.sqlite"),
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        schema::configure(&db, config, false)?;
        // Check lineage before inspecting that store's external receipt namespace;
        // repeat inside the later pinned snapshot for coherent full validation.
        check_header::<L>(&db, &root, config, temporary)?;
        let objects = Objects::new(root.join("objects"), config);
        // Capture immutable receipts FIRST, then pin a SQL read snapshot. SQL
        // publication always precedes its receipt, so this snapshot cannot lag
        // any captured receipt. Later receipts must not be scanned against it.
        crate::witness::admit(&db, &root, &objects, false)?;
        let receipts = if prefix.is_none() {
            Some(crate::witness::capture(&root, &objects)?)
        } else {
            None
        };
        after_capture();
        db.execute_batch("BEGIN")?;
        let (header, branch) = check_header::<L>(&db, &root, config, temporary)?;
        after_snapshot();
        let proof = if let Some(mut prefix) = prefix {
            let audit = prefix.prefix.audit;
            let inputs = prefix.prefix.inputs;
            let position = prefix.prefix.position;
            // verify_tail starts from the saved parent, position, anchor and
            // stable-key fingerprints. Retry independently from genesis on a gap.
            let valid = prefix
                .extend(&db, &objects, &header, &branch)
                .and_then(|()| {
                    crate::recovery::check_new_witnesses(
                        &db,
                        &root,
                        &objects,
                        inputs..prefix.prefix.inputs,
                        position..prefix.prefix.position,
                        audit..prefix.prefix.audit,
                    )
                });
            if valid.is_err() {
                db.execute_batch("ROLLBACK")?;
                return Self::check_prefix(
                    &root,
                    config,
                    temporary,
                    None,
                    (|| {}) as fn(),
                    (|| {}) as fn(),
                );
            }
            prefix
        } else {
            let mut proof = PublicationProof::empty()?;
            proof.extend(&db, &objects, &header, &branch)?;
            crate::witness::check(&db, &objects, &receipts.unwrap())?;
            #[cfg(test)]
            FULL_CHECKS.update(|n| n + 1);
            proof
        };
        let verified_head = proof.head.clone();
        db.execute_batch("ROLLBACK")?;
        // The open check is a coherent snapshot; subsequent queries remain live.
        // The sole writer only appends immutable, fully verified publications.
        Ok(Self {
            db,
            objects,
            verified_head,
            writer_data_version: None,
            proof: Some(proof),
            _lineage: PhantomData,
        })
    }
    pub fn current(&self) -> Result<Option<CoreTransactionV2>> {
        current(&self.db, &self.objects)
    }
    /// Read the exact accepted transaction from the prefix verified at open.
    /// Object visibility alone would also admit objects referenced by a commit.
    pub fn read_published_transaction(
        &self,
        snapshot: &ObjectRef,
    ) -> Result<(CoreTransactionV2, EventEnvelope)> {
        let tx: CoreTransactionV2 = self.read(snapshot)?;
        let event: EventEnvelope = self.read(&tx.staged_h)?;
        let position = schema::integer(tx.binding.raw_log_position.0)?;
        let accepted: bool = self.db.query_row(
            "SELECT EXISTS (
                SELECT 1 FROM branch_head h JOIN commits c ON c.position < h.processed
                JOIN snapshots s ON s.commit_id = c.id AND s.reference = c.snapshot
                JOIN facts f ON f.commit_id = c.id AND f.position = c.position
                WHERE h.singleton = 1 AND h.commit_id = ?1 AND h.processed = ?2
                  AND c.id = ?3 AND c.position = ?4 AND c.decision = 'accepted'
                  AND c.snapshot = ?5 AND f.event_id = ?6 AND f.reference = ?7)",
            params![
                self.verified_head.0,
                schema::integer(self.verified_head.1)?,
                hex(&snapshot.digest.0),
                position,
                snapshot.canonical_bytes(self.objects.config.encoding)?,
                event.id,
                tx.staged_h.canonical_bytes(self.objects.config.encoding)?
            ],
            |r| r.get(0),
        )?;
        require(accepted, "transaction is outside the verified publication")?;
        self.objects.closure(snapshot)?;
        Ok((tx, event))
    }
    pub fn read<T: WireMessage>(&self, r: &ObjectRef) -> Result<T> {
        let stored: Option<Vec<u8>> = self.db.query_row("SELECT reference FROM object_refs JOIN visible_objects USING(digest) WHERE digest=?1", [hex(&r.digest.0)], |r| r.get(0)).optional()?;
        require(
            stored.as_deref() == Some(r.canonical_bytes(self.objects.config.encoding)?.as_slice()),
            "object is not visible in this lineage",
        )?;
        Ok(T::from_canonical(
            &self.objects.read(r)?,
            self.objects.config.encoding,
        )?)
    }
    pub fn facts(&self, limit: usize) -> Result<Vec<core_types::generated::EventEnvelope>> {
        require(
            limit <= self.objects.config.encoding.max_items,
            "query budget exceeded",
        )?;
        let mut stmt = self
            .db
            .prepare("SELECT reference FROM facts ORDER BY position LIMIT ?1")?;
        let values = stmt.query_map([limit as i64], |r| r.get::<_, Vec<u8>>(0))?;
        values
            .map(|v| {
                self.read(&ObjectRef::from_canonical(
                    &v?,
                    self.objects.config.encoding,
                )?)
            })
            .collect()
    }
}
