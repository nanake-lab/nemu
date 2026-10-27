use crate::{Result, StoreConfig, error::require};
use rusqlite::Connection;

pub(crate) fn configure(db: &Connection, config: StoreConfig, writable: bool) -> Result<()> {
    config.validate()?;
    require(
        rusqlite::version_number() >= 3_051_003,
        "SQLite lacks the WAL-reset corruption fix",
    )?;
    db.busy_timeout(config.busy_timeout)?;
    db.pragma_update(None, "foreign_keys", true)?;
    if writable {
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.pragma_update(None, "synchronous", "FULL")?;
        db.pragma_update(None, "max_page_count", config.max_sql_pages)?;
    }
    let wal: String = db.pragma_query_value(None, "journal_mode", |r| r.get(0))?;
    require(wal.eq_ignore_ascii_case("wal"), "WAL mode unavailable")?;
    require(
        db.pragma_query_value(None, "foreign_keys", |r| r.get::<_, i32>(0))? == 1,
        "foreign keys disabled",
    )?;
    if writable {
        require(
            db.pragma_query_value(None, "synchronous", |r| r.get::<_, i32>(0))? == 2,
            "FULL synchronization unavailable",
        )?;
    } else {
        db.pragma_update(None, "query_only", true)?;
    }
    Ok(())
}

// Physical indexes contain references only. There are no alternate R/theta/L state tables.
pub(crate) const INITIAL: &str = r#"
CREATE TABLE metadata(singleton INTEGER PRIMARY KEY CHECK(singleton=1), format INTEGER NOT NULL CHECK(format=2),
 location TEXT NOT NULL, temporary INTEGER NOT NULL, header BLOB NOT NULL, branch TEXT NOT NULL);
CREATE TABLE object_refs(digest TEXT PRIMARY KEY, reference BLOB NOT NULL);
CREATE TABLE visible_objects(digest TEXT PRIMARY KEY REFERENCES object_refs(digest));
CREATE TABLE raw_ingress(position INTEGER PRIMARY KEY CHECK(position>=0), event BLOB NOT NULL, size INTEGER NOT NULL);
CREATE TABLE commits(id TEXT PRIMARY KEY, parent TEXT REFERENCES commits(id), position INTEGER NOT NULL UNIQUE,
 anchor INTEGER NOT NULL CHECK(anchor>=0), snapshot BLOB, decision TEXT NOT NULL);
CREATE TABLE branch_head(singleton INTEGER PRIMARY KEY CHECK(singleton=1), commit_id TEXT REFERENCES commits(id),
 processed INTEGER NOT NULL CHECK(processed>=0), anchor INTEGER NOT NULL CHECK(anchor>=0), snapshot BLOB);
INSERT INTO branch_head VALUES(1,NULL,0,0,NULL);
CREATE TABLE ingress_decisions(position INTEGER PRIMARY KEY REFERENCES raw_ingress(position), commit_id TEXT NOT NULL REFERENCES commits(id),
 decision TEXT NOT NULL, device TEXT NOT NULL, stable_key TEXT NOT NULL, fingerprint BLOB NOT NULL);
CREATE UNIQUE INDEX ingress_accepted_key ON ingress_decisions(device,stable_key) WHERE decision='accepted';
CREATE TABLE facts(event_id TEXT PRIMARY KEY, position INTEGER NOT NULL UNIQUE REFERENCES raw_ingress(position), commit_id TEXT NOT NULL REFERENCES commits(id), reference BLOB NOT NULL);
CREATE TABLE fact_corrections(event_id TEXT NOT NULL REFERENCES facts(event_id), corrected_id TEXT NOT NULL, PRIMARY KEY(event_id,corrected_id));
CREATE TABLE snapshots(commit_id TEXT PRIMARY KEY REFERENCES commits(id), reference BLOB NOT NULL);
CREATE TABLE snapshot_objects(commit_id TEXT NOT NULL REFERENCES commits(id), role TEXT NOT NULL, object_key TEXT NOT NULL,
 reference BLOB NOT NULL, PRIMARY KEY(commit_id,role,object_key));
CREATE TABLE runtime_records(commit_id TEXT NOT NULL REFERENCES commits(id), reference BLOB NOT NULL);
CREATE TABLE outbox(commit_id TEXT NOT NULL REFERENCES commits(id), action_id TEXT NOT NULL, reference BLOB NOT NULL, PRIMARY KEY(commit_id,action_id));
CREATE TABLE accepted_results(commit_id TEXT NOT NULL REFERENCES commits(id), reference BLOB NOT NULL, PRIMARY KEY(commit_id,reference));
CREATE TABLE dependency_index(commit_id TEXT NOT NULL REFERENCES commits(id), owner TEXT NOT NULL, dependency TEXT NOT NULL,
 PRIMARY KEY(commit_id,owner,dependency));
CREATE TABLE scope_index(commit_id TEXT NOT NULL REFERENCES commits(id), source_digest TEXT NOT NULL, index_ref BLOB NOT NULL);
CREATE TABLE evidence_index(commit_id TEXT NOT NULL REFERENCES commits(id), tendency_id TEXT NOT NULL, group_id TEXT NOT NULL,
 reference BLOB NOT NULL, PRIMARY KEY(commit_id,tendency_id,group_id));
CREATE TABLE evidence_groups(commit_id TEXT NOT NULL REFERENCES commits(id), group_id TEXT NOT NULL, reference BLOB NOT NULL, PRIMARY KEY(commit_id,group_id));
CREATE TABLE ledger_index(entry_id TEXT PRIMARY KEY, event_id TEXT NOT NULL REFERENCES facts(event_id), commit_id TEXT NOT NULL REFERENCES commits(id), reference BLOB NOT NULL);
PRAGMA user_version=2;
"#;

pub(crate) fn create(db: &Connection) -> Result<()> {
    db.execute_batch(INITIAL)?;
    db.execute_batch(crate::migration::AUDIT_SQL)?;
    for table in [
        "metadata",
        "raw_ingress",
        "commits",
        "ingress_decisions",
        "facts",
        "fact_corrections",
        "snapshots",
        "snapshot_objects",
        "runtime_records",
        "outbox",
        "accepted_results",
        "ledger_index",
        "object_refs",
        "visible_objects",
        "evidence_groups",
        "evidence_index",
    ] {
        for operation in ["UPDATE", "DELETE"] {
            // Names are fixed literals, never caller-controlled SQL.
            db.execute_batch(&format!("CREATE TRIGGER immutable_{table}_{operation} BEFORE {operation} ON {table} BEGIN SELECT RAISE(ABORT,'append-only authority'); END;"))?;
        }
    }
    Ok(())
}

pub(crate) fn integer(n: u64) -> Result<i64> {
    i64::try_from(n).map_err(|_| crate::Error::Rejected("storage counter overflow"))
}
pub(crate) fn unsigned(n: i64) -> Result<u64> {
    u64::try_from(n).map_err(|_| crate::Error::Rejected("negative storage counter"))
}
