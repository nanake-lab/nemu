//! SYS17 physical-format subset. Does not convert state message generations.
use crate::{
    Fault, FaultPoint, LineageKind, Result, Store, StoreConfig, TemporaryRoot, error::require,
    objects::Objects, platform, schema,
};
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;

pub(crate) const AUDIT_SQL: &str = "CREATE TABLE ingress_rejections(position INTEGER PRIMARY KEY CHECK(position>=0),record BLOB NOT NULL);
CREATE TRIGGER immutable_ingress_rejections_UPDATE BEFORE UPDATE ON ingress_rejections BEGIN SELECT RAISE(ABORT,'append-only ingress audit'); END;
CREATE TRIGGER immutable_ingress_rejections_DELETE BEFORE DELETE ON ingress_rejections BEGIN SELECT RAISE(ABORT,'append-only ingress audit'); END;";

pub(crate) fn format(db: &Connection) -> Result<i32> {
    Ok(db.pragma_query_value(None, "user_version", |r| r.get(0))?)
}

/// Keeping the old rows in place makes rollback independent of backup freshness.
/// SQLite transactional DDL restores the complete old schema on precommit failure.
fn change_schema(db: &Connection, target: i32) -> Result<()> {
    require(target == 1 || target == 2, "unknown migration target")?;
    db.execute_batch("DROP TRIGGER immutable_metadata_UPDATE; DROP TRIGGER immutable_metadata_DELETE; ALTER TABLE metadata RENAME TO prior_metadata;")?;
    db.execute_batch(&format!("CREATE TABLE metadata(singleton INTEGER PRIMARY KEY CHECK(singleton=1),format INTEGER NOT NULL CHECK(format={target}),location TEXT NOT NULL,temporary INTEGER NOT NULL,header BLOB NOT NULL,branch TEXT NOT NULL);
INSERT INTO metadata SELECT singleton,{target},location,temporary,header,branch FROM prior_metadata;
DROP TABLE prior_metadata;
CREATE TRIGGER immutable_metadata_UPDATE BEFORE UPDATE ON metadata BEGIN SELECT RAISE(ABORT,'append-only authority'); END;
CREATE TRIGGER immutable_metadata_DELETE BEFORE DELETE ON metadata BEGIN SELECT RAISE(ABORT,'append-only authority'); END;
PRAGMA user_version={target};"))?;
    if target == 2 {
        db.execute_batch(AUDIT_SQL)?;
    } else {
        db.execute_batch("DROP TABLE ingress_rejections;")?;
    }
    Ok(())
}

impl<L: LineageKind> Store<L> {
    pub fn migrate_temporary(
        root: &TemporaryRoot,
        name: &str,
        config: StoreConfig,
        target: i32,
    ) -> Result<()> {
        Self::migrate_format(root.store_path(name)?, config, true, target, Fault::None)
    }
    /// Caller must arrange the pre-upgrade checkpoint required by ADR 0005.
    pub fn migrate_production(config: StoreConfig, target: i32) -> Result<()> {
        Self::migrate_format(
            crate::lineage::production_path::<L>()?,
            config,
            false,
            target,
            Fault::None,
        )
    }
    pub(crate) fn migrate_format(
        root: PathBuf,
        config: StoreConfig,
        temporary: bool,
        target: i32,
        fault: Fault,
    ) -> Result<()> {
        require(target == 1 || target == 2, "unknown migration target")?;
        config.validate()?;
        platform::no_links(&root)?;
        let root = root.canonicalize()?;
        platform::local_volume(&root, temporary)?;
        let _writer = platform::writer_lock(&root)?;
        // Never create/checkpoint a writable WAL before rejecting a V2 head.
        let ro = Connection::open_with_flags(
            root.join("store.sqlite"),
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        let (header, branch) = crate::store::check_header::<L>(&ro, &root, config, temporary)?;
        let objects = Objects::new(root.join("objects"), config);
        crate::store::require_writable_head(&ro, &objects)?;
        crate::recovery::verify(&ro, &objects, &header, &branch)?;
        crate::witness::reconcile(&ro, &root, &objects, false, &Fault::None)?;
        let old = format(&ro)?;
        require(old != target, "store already has requested format")?;
        if target == 1 {
            let n: i64 =
                ro.query_row("SELECT count(*) FROM ingress_rejections", [], |r| r.get(0))?;
            require(n == 0, "rollback would discard acknowledged ingress audit")?;
        }
        drop(ro);
        // Empty namespace is safe under either format and retained on rollback.
        let audit_dir = crate::witness::path(&root)?.join("reject");
        platform::no_links(&audit_dir)?;
        if !audit_dir.exists() {
            std::fs::create_dir(&audit_dir)?;
        }
        platform::sync_dir(&audit_dir)?;
        platform::sync_dir(
            audit_dir
                .parent()
                .ok_or(crate::Error::Rejected("missing receipt parent"))?,
        )?;
        let mut db = Connection::open_with_flags(
            root.join("store.sqlite"),
            OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;
        schema::configure(&db, config, true)?;
        let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        change_schema(&tx, target)?;
        fault.hit(FaultPoint::MigrationBeforeSql)?;
        tx.commit()?;
        fault.hit(FaultPoint::MigrationSqlCommitted)?;
        platform::sync_dir(&root)?;
        Ok(())
    }
}
