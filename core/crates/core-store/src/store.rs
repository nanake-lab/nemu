use crate::{
    Error, Experiment, Fault, Life, LineageKind, PreparedObject, Reader, Rehearsal, Result,
    StoreConfig, TemporaryRoot, error::require, lineage::production_path, objects::Objects,
    platform, schema,
};
use core_types::{
    WireMessage,
    generated::{CoreTransactionV2, GenesisManifestV2, Lineage, LineageHeaderV2, ObjectRef},
    scalar::hex,
};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use std::{
    fs::{self, File},
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub struct Store<L: LineageKind> {
    pub(crate) db: Connection,
    pub(crate) objects: Objects,
    pub(crate) root: PathBuf,
    pub(crate) branch: String,
    pub(crate) header: LineageHeaderV2,
    pub(crate) fault: Fault,
    pub(crate) capability_issued: bool,
    _writer: File,
    _lineage: PhantomData<L>,
}

impl<L: LineageKind> Store<L> {
    pub fn open_temporary(root: &TemporaryRoot, name: &str, config: StoreConfig) -> Result<Self> {
        Self::open(root.store_path(name)?, config, true)
    }
    pub fn open_production(config: StoreConfig) -> Result<Self> {
        Self::open(production_path::<L>()?, config, false)
    }
    pub(crate) fn create(
        root: PathBuf,
        header: LineageHeaderV2,
        branch: &str,
        config: StoreConfig,
        temporary: bool,
    ) -> Result<Self> {
        Self::create_with_fault(root, header, branch, config, temporary, Fault::None)
    }
    pub(crate) fn create_with_fault(
        root: PathBuf,
        header: LineageHeaderV2,
        branch: &str,
        config: StoreConfig,
        temporary: bool,
        fault: Fault,
    ) -> Result<Self> {
        config.validate()?;
        require(header.lineage == L::KIND, "lineage type mismatch")?;
        let encoded = header.canonical_bytes(config.encoding)?;
        require(!branch.is_empty(), "branch identity is required")?;
        require(
            !root.exists(),
            "initialization never replaces an existing directory",
        )?;
        let parent = root
            .parent()
            .ok_or(Error::Rejected("missing storage parent"))?;
        platform::no_links(parent)?;
        fs::create_dir_all(parent)?;
        platform::local_volume(parent, temporary)?;
        fs::create_dir(&root)?;
        fault.hit(crate::FaultPoint::InitRootCreated)?;
        platform::sync_dir(parent)?;
        fault.hit(crate::FaultPoint::InitRootSynced)?;
        let root = root.canonicalize()?;
        let writer = platform::writer_lock(&root)?;
        fs::create_dir(root.join("objects"))?;
        fault.hit(crate::FaultPoint::InitObjectsCreated)?;
        platform::sync_dir(&root)?;
        fault.hit(crate::FaultPoint::InitObjectsSynced)?;
        let mut db = Connection::open(root.join("store.sqlite"))?;
        schema::configure(&db, config, true)?;
        let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        schema::create(&tx)?;
        tx.execute(
            "INSERT INTO metadata VALUES(1,1,?1,?2,?3,?4)",
            params![root.to_string_lossy(), temporary, encoded, branch],
        )?;
        tx.commit()?;
        fault.hit(crate::FaultPoint::InitSqlCommitted)?;
        platform::sync_dir(&root)?;
        fault.hit(crate::FaultPoint::InitDbDirectorySynced)?;
        crate::witness::create(&root, &fault)?;
        Ok(Self {
            objects: Objects::new(root.join("objects"), config),
            root,
            db,
            branch: branch.into(),
            header,
            fault,
            capability_issued: false,
            _writer: writer,
            _lineage: PhantomData,
        })
    }
    pub(crate) fn open(root: PathBuf, config: StoreConfig, temporary: bool) -> Result<Self> {
        Self::open_with_fault(root, config, temporary, Fault::None)
    }
    pub(crate) fn open_with_fault(
        root: PathBuf,
        config: StoreConfig,
        temporary: bool,
        fault: Fault,
    ) -> Result<Self> {
        platform::no_links(&root)?;
        let root = root.canonicalize()?;
        platform::local_volume(&root, temporary)?;
        let writer = platform::writer_lock(&root)?;
        // No CREATE flag: missing/corrupt stores never silently become a new life.
        let db = Connection::open_with_flags(
            root.join("store.sqlite"),
            OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;
        let (header, branch) = check_header::<L>(&db, &root, config, temporary)?;
        schema::configure(&db, config, true)?;
        let store = Self {
            objects: Objects::new(root.join("objects"), config),
            root,
            db,
            branch,
            header,
            fault,
            capability_issued: false,
            _writer: writer,
            _lineage: PhantomData,
        };
        store.verify()?;
        crate::witness::reconcile(&store.db, &store.root, &store.objects, true, &store.fault)?;
        Ok(store)
    }
    pub fn path(&self) -> &Path {
        &self.root
    }
    pub fn header(&self) -> &LineageHeaderV2 {
        &self.header
    }
    pub fn set_fault(&mut self, fault: Fault) {
        self.fault = fault;
    }
    pub fn prepare<T: WireMessage>(&self, value: &T, version: &str) -> Result<PreparedObject<L>> {
        require(
            T::NAME != core_types::generated::RSnapshotV2::NAME
                && T::NAME != core_types::generated::RSnapshotV3::NAME,
            "R snapshots must use prepare_r",
        )?;
        self.objects.prepare(value, version, &self.fault)
    }
    pub fn prepare_r(
        &self,
        value: &core_types::generated::RSnapshotV3,
    ) -> Result<PreparedObject<L>> {
        self.objects.prepare_r(value, &self.fault)
    }
    pub fn reader(&self) -> Result<Reader<L>> {
        let temporary: bool = self
            .db
            .query_row("SELECT temporary FROM metadata", [], |r| r.get(0))?;
        Reader::open(&self.root, self.objects.config, temporary)
    }
    pub fn current(&self) -> Result<Option<CoreTransactionV2>> {
        current(&self.db, &self.objects)
    }
    pub fn head(&self) -> Result<(Option<String>, u64, u64)> {
        head(&self.db)
    }
    pub fn verify(&self) -> Result<()> {
        crate::recovery::verify(&self.db, &self.objects, &self.header, &self.branch)?;
        crate::witness::reconcile(&self.db, &self.root, &self.objects, false, &Fault::None)
    }
    /// Pins retained by PreparedObject protect the prepare-to-publish window.
    /// Published history and durable D objects are never garbage collected.
    pub fn collect_garbage(&mut self) -> Result<usize> {
        let pins = self
            .objects
            .pins
            .lock()
            .map_err(|_| Error::Rejected("retention lock poisoned"))?;
        let retained: std::collections::BTreeSet<_> = references(&self.db, self.objects.config)?
            .into_iter()
            .map(|r| hex(&r.digest.0))
            .collect();
        let mut removed = 0;
        for entry in fs::read_dir(&self.objects.root)? {
            let entry = entry?;
            let path = entry.path();
            platform::no_links(&path)?;
            require(
                entry.file_type()?.is_file(),
                "unexpected object directory entry",
            )?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let digest = name.strip_suffix(".obj");
            if digest.is_none_or(|d| !retained.contains(d) && !pins.contains_key(d)) {
                fs::remove_file(path)?;
                removed += 1;
                self.fault.hit(crate::FaultPoint::GcDeleted)?;
            }
        }
        self.fault.hit(crate::FaultPoint::GcDirectorySyncBefore)?;
        platform::sync_dir(&self.objects.root)?;
        self.fault.hit(crate::FaultPoint::GcDirectorySynced)?;
        Ok(removed)
    }
}
impl Store<Rehearsal> {
    pub fn create_temporary(
        root: &TemporaryRoot,
        name: &str,
        header: LineageHeaderV2,
        branch: &str,
        config: StoreConfig,
    ) -> Result<Self> {
        Self::create(root.store_path(name)?, header, branch, config, true)
    }
    /// Called by startup only when no LIFE exists. Does not open or inspect LIFE data.
    pub fn startup(header: LineageHeaderV2, branch: &str, config: StoreConfig) -> Result<Self> {
        require(
            !production_path::<Life>()?.exists(),
            "LIFE exists; explicit LIFE startup is required",
        )?;
        let path = production_path::<Rehearsal>()?;
        if path.exists() {
            Self::open(path, config, false)
        } else {
            Self::create(path, header, branch, config, false)
        }
    }
}
impl Store<Experiment> {
    pub fn create_temporary(
        root: &TemporaryRoot,
        name: &str,
        header: LineageHeaderV2,
        branch: &str,
        config: StoreConfig,
    ) -> Result<Self> {
        Self::create(root.store_path(name)?, header, branch, config, true)
    }
}
impl Store<Life> {
    /// W14 verifies the user's signature and complete runtime configuration before
    /// invoking this explicit genesis-only storage entry point. No ordinary init API.
    pub fn create_from_genesis(
        root: &TemporaryRoot,
        name: &str,
        header: LineageHeaderV2,
        genesis: &GenesisManifestV2,
        branch: &str,
        config: StoreConfig,
    ) -> Result<Self> {
        validate_genesis(&header, genesis, config)?;
        Self::create(root.store_path(name)?, header, branch, config, true)
    }
    pub fn birth(
        header: LineageHeaderV2,
        genesis: &GenesisManifestV2,
        branch: &str,
        config: StoreConfig,
    ) -> Result<Self> {
        validate_genesis(&header, genesis, config)?;
        Self::create(production_path::<Life>()?, header, branch, config, false)
    }
}
fn validate_genesis(
    header: &LineageHeaderV2,
    genesis: &GenesisManifestV2,
    config: StoreConfig,
) -> Result<()> {
    let digest = core_types::contract::Schema::builtin()?.digest(
        GenesisManifestV2::NAME,
        &serde_json::to_value(genesis)?,
        config.encoding,
    )?;
    require(
        header
            .genesis_manifest_digest
            .as_ref()
            .is_some_and(|d| hex(&d.0) == digest),
        "genesis manifest digest mismatch",
    )
}
pub(crate) fn check_header<L: LineageKind>(
    db: &Connection,
    root: &Path,
    config: StoreConfig,
    temporary: bool,
) -> Result<(LineageHeaderV2, String)> {
    require(
        db.pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))? == 1,
        "unknown storage format; explicit migration required",
    )?;
    let (format, location, is_temp, bytes, branch): (i32, String, bool, Vec<u8>, String) = db
        .query_row(
            "SELECT format,location,temporary,header,branch FROM metadata WHERE singleton=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )?;
    let header = LineageHeaderV2::from_canonical(&bytes, config.encoding)?;
    require(
        format == 1 && header.lineage == L::KIND,
        "lineage or storage format mismatch",
    )?;
    require(
        is_temp == temporary && (L::KIND != Lineage::EXPERIMENT || temporary),
        "storage environment mismatch",
    )?;
    require(
        location == root.to_string_lossy(),
        "copied or moved store requires explicit recovery; cannot resume a cloned branch",
    )?;
    Ok((header, branch))
}
pub(crate) fn head(db: &Connection) -> Result<(Option<String>, u64, u64)> {
    let (id, position, anchor): (Option<String>, i64, i64) = db.query_row(
        "SELECT commit_id,processed,anchor FROM branch_head WHERE singleton=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    Ok((id, schema::unsigned(position)?, schema::unsigned(anchor)?))
}
pub(crate) fn current(db: &Connection, objects: &Objects) -> Result<Option<CoreTransactionV2>> {
    let bytes: Option<Vec<u8>> = db.query_row(
        "SELECT snapshot FROM branch_head WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    bytes
        .map(|b| -> Result<_> {
            let r = ObjectRef::from_canonical(&b, objects.config.encoding)?;
            Ok(CoreTransactionV2::from_canonical(
                &objects.read(&r)?,
                objects.config.encoding,
            )?)
        })
        .transpose()
}
pub(crate) fn references(db: &Connection, config: StoreConfig) -> Result<Vec<ObjectRef>> {
    let mut stmt = db.prepare("SELECT reference FROM object_refs")?;
    stmt.query_map([], |r| r.get::<_, Vec<u8>>(0))?
        .map(|b| Ok(ObjectRef::from_canonical(&b?, config.encoding)?))
        .collect()
}
pub(crate) fn publish_refs(db: &Connection, refs: &[ObjectRef], config: StoreConfig) -> Result<()> {
    for r in refs {
        let bytes = r.canonical_bytes(config.encoding)?;
        let key = hex(&r.digest.0);
        let old: Option<Vec<u8>> = db
            .query_row(
                "SELECT reference FROM object_refs WHERE digest=?1",
                [&key],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(old) = old {
            require(old == bytes, "object reference metadata conflict")?;
        } else {
            db.execute("INSERT INTO object_refs VALUES(?1,?2)", params![key, bytes])?;
        }
    }
    Ok(())
}
