use crate::{
    LineageKind, Result, StoreConfig, TemporaryRoot,
    error::require,
    objects::Objects,
    platform, schema,
    store::{check_header, current},
};
use core_types::{
    WireMessage,
    generated::{CoreTransactionV2, ObjectRef},
};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use std::{marker::PhantomData, path::Path};
pub struct Reader<L: LineageKind> {
    pub(crate) db: Connection,
    pub(crate) objects: Objects,
    _lineage: PhantomData<L>,
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
        db.execute_batch("ROLLBACK")?;
        // The open check is a coherent snapshot; subsequent queries remain live.
        // The sole writer only appends immutable, fully verified publications.
        Ok(Self {
            db,
            objects,
            _lineage: PhantomData,
        })
    }
    pub fn activity_attempts(
        &self,
        request: &str,
        limit: usize,
    ) -> Result<Vec<(ObjectRef, ObjectRef)>> {
        require(
            limit < self.objects.config.encoding.max_items,
            "attempt query budget exceeded",
        )?;
        if crate::migration::format(&self.db)? < 3 {
            return Ok(Vec::new());
        }
        let mut stmt = self.db.prepare("SELECT activity, record FROM cognitive_attempts WHERE request_id=?1 ORDER BY position LIMIT ?2")?;
        let mut rows = stmt.query(rusqlite::params![
            request,
            crate::schema::integer(limit as u64 + 1)?
        ])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            require(result.len() < limit, "incomplete attempt audit")?;
            let decode =
                |bytes: Vec<u8>| ObjectRef::from_canonical(&bytes, self.objects.config.encoding);
            result.push((decode(row.get(0)?)?, decode(row.get(1)?)?));
        }
        Ok(result)
    }
    pub fn snapshot_at(&self, commit: &str) -> Result<Option<ObjectRef>> {
        let bytes: Option<Vec<u8>> =
            self.db
                .query_row("SELECT snapshot FROM commits WHERE id=?1", [commit], |r| {
                    r.get(0)
                })?;
        bytes
            .map(|b| {
                ObjectRef::from_canonical(&b, self.objects.config.encoding).map_err(Into::into)
            })
            .transpose()
    }
    pub fn current_snapshot(&self) -> Result<Option<ObjectRef>> {
        let bytes: Option<Vec<u8>> = self.db.query_row(
            "SELECT snapshot FROM branch_head WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        bytes
            .map(|b| {
                ObjectRef::from_canonical(&b, self.objects.config.encoding).map_err(Into::into)
            })
            .transpose()
    }
    pub fn current(&self) -> Result<Option<CoreTransactionV2>> {
        current(&self.db, &self.objects)
    }
    /// The event fields after decoding the exact snapshot tag. Activity and unknown
    /// tags cannot acknowledge terminal ingress.
    pub fn event_snapshot(&self, r: &ObjectRef) -> Result<CoreTransactionV2> {
        match r.object_type.as_str() {
            "echo.transaction-v2" => self.read(r),
            "echo.transaction-v4" => crate::manifest::event_view(self.read(r)?),
            _ => Err(crate::Error::Rejected("unsupported event snapshot")),
        }
    }
    pub fn read<T: WireMessage>(&self, r: &ObjectRef) -> Result<T> {
        require(
            r.object_type != crate::RAW_BYTES,
            "raw resource is not a CanonicalV1 message",
        )?;
        Ok(T::from_canonical(
            &self.visible_bytes(r)?,
            self.objects.config.encoding,
        )?)
    }
    pub fn read_raw(&self, r: &ObjectRef) -> Result<Vec<u8>> {
        require(
            r.object_type == crate::RAW_BYTES,
            "expected raw byte resource",
        )?;
        self.visible_bytes(r)
    }
    fn visible_bytes(&self, r: &ObjectRef) -> Result<Vec<u8>> {
        let stored: Option<Vec<u8>> = self.db.query_row("SELECT reference FROM object_refs JOIN visible_objects USING(digest) WHERE digest=?1", [crate::raw::storage_key(r)], |r| r.get(0)).optional()?;
        require(
            stored.as_deref() == Some(r.canonical_bytes(self.objects.config.encoding)?.as_slice()),
            "object is not visible in this lineage",
        )?;
        self.objects.read(r)
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
