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
    _lineage: PhantomData<L>,
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

    /// Capture a publication on the writer. The returned job owns only reader
    /// resources and can perform its SQL, file and witness checks off-thread.
    /// On a full check, the observer runs after the read transaction is pinned
    /// and before recovery traverses it. Finally call publication_matches on
    /// the original Store before consuming the results.
    pub fn receipt_check<F: FnOnce() + Send + 'static>(
        &self,
        cached: Option<Reader<L>>,
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
        Ok(move || {
            if let Some(reader) = cached
                && reader.verified_head == published
                && reader.writer_data_version == Some(version)
                && Arc::ptr_eq(&pins, &reader.objects.pins)
            {
                return Ok(reader);
            }
            let mut reader =
                Reader::open_checked(&root, config, temporary, || {}, snapshot_pinned)?;
            require(
                reader.verified_head == published,
                "publication changed during receipt verification",
            )?;
            reader.objects.pins = pins;
            reader.writer_data_version = Some(version);
            Ok(reader)
        })
    }

    pub fn receipt_reader<'a>(&self, slot: &'a mut Option<Reader<L>>) -> Result<&'a Reader<L>> {
        let check = self.receipt_check(slot.take(), || {})?;
        let reader = check()?;
        require(
            self.publication_matches(&reader)?,
            "publication changed during receipt verification",
        )?;
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
        let receipts = crate::witness::capture(&root, &objects)?;
        after_capture();
        db.execute_batch("BEGIN")?;
        let (header, branch) = check_header::<L>(&db, &root, config, temporary)?;
        after_snapshot();
        crate::recovery::verify(&db, &objects, &header, &branch)?;
        crate::witness::check(&db, &objects, &receipts)?;
        let verified_head = head(&db)?;
        #[cfg(test)]
        FULL_CHECKS.update(|n| n + 1);
        db.execute_batch("ROLLBACK")?;
        // The open check is a coherent snapshot; subsequent queries remain live.
        // The sole writer only appends immutable, fully verified publications.
        Ok(Self {
            db,
            objects,
            verified_head,
            writer_data_version: None,
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
