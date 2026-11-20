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
pub(crate) mod counts {
    std::thread_local! { pub static FULL_CHECKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
}

pub struct Reader<L: LineageKind> {
    pub(crate) db: Connection,
    pub(crate) objects: Objects,
    verified_head: (Option<String>, u64, u64),
    writer_data_version: Option<i64>,
    _lineage: PhantomData<L>,
}

impl<L: LineageKind> Store<L> {
    /// Reuse one full open check for receipts at this writer's current publication.
    pub fn receipt_reader<'a>(&self, slot: &'a mut Option<Reader<L>>) -> Result<&'a Reader<L>> {
        let published = self.head()?;
        // This connection's data_version changes only for another SQL writer.
        // The Store's own appends to unprocessed D do not invalidate its prefix.
        let data_version = || {
            self.db
                .pragma_query_value(None, "data_version", |r| r.get::<_, i64>(0))
        };
        let version = data_version()?;
        if !slot.as_ref().is_some_and(|r| {
            r.verified_head == published
                && r.writer_data_version == Some(version)
                && Arc::ptr_eq(&self.objects.pins, &r.objects.pins)
        }) {
            *slot = None;
            let mut reader = self.reader()?;
            require(
                reader.verified_head == published && data_version()? == version,
                "publication changed during receipt verification",
            )?;
            // The same per-opening identity also binds the Store's commit capability.
            reader.objects.pins = Arc::clone(&self.objects.pins);
            reader.writer_data_version = Some(version);
            *slot = Some(reader);
        }
        slot.as_ref()
            .ok_or(crate::Error::Rejected("receipt reader unavailable"))
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
    // transactions are passed to callbacks. Public opens always use no-op hooks.
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
        counts::FULL_CHECKS.update(|n| n + 1);
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
