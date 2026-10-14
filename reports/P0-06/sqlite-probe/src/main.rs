#![forbid(unsafe_code)]

use rusqlite::Connection;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // A build fingerprint only, never the user's rehearsal or life storage.
    let directory = std::env::temp_dir().join(format!(
        "nemu-w00-sqlite-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir(&directory)?;
    let result = probe(&directory.join("probe.sqlite3"));
    // Connection is dropped before removing only this newly-created directory.
    std::fs::remove_dir_all(&directory)?;
    assert!(!directory.exists());
    println!("temporary_storage_removed=true");
    result
}

fn probe(path: &std::path::Path) -> Result<(), Box<dyn Error>> {
    let connection = Connection::open(path)?;
    let version: String = connection.query_row("SELECT sqlite_version()", [], |row| row.get(0))?;
    let source_id: String =
        connection.query_row("SELECT sqlite_source_id()", [], |row| row.get(0))?;
    println!("sqlite_version={version}");
    println!("sqlite_source_id={source_id}");
    let journal: String = connection.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
    assert_eq!(journal, "wal");
    connection.execute_batch("PRAGMA synchronous=FULL; CREATE TABLE probe (value INTEGER); BEGIN; INSERT INTO probe VALUES (1); ROLLBACK;")?;
    let synchronous: i64 = connection.query_row("PRAGMA synchronous", [], |row| row.get(0))?;
    let count: i64 = connection.query_row("SELECT count(*) FROM probe", [], |row| row.get(0))?;
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    assert_eq!(synchronous, 2);
    assert_eq!(count, 0);
    assert_eq!(integrity, "ok");
    println!(
        "journal_mode={journal}; synchronous={synchronous}; rollback_count={count}; integrity={integrity}"
    );
    let mut options = connection.prepare("PRAGMA compile_options")?;
    for option in options.query_map([], |row| row.get::<_, String>(0))? {
        println!("compile_option={}", option?);
    }
    Ok(())
}
