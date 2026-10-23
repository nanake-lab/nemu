use crate::{
    Embedder, Result,
    embedding::vector_bytes,
    source::{self, Record},
};
use core_store::{LineageKind, Reader};
use core_types::generated::CoreTransactionV2;
use rusqlite::{Connection, params};

/// Lineage is preserved in the Reader borrow; consumers receive projections only.
///
/// ```compile_fail
/// use core_store::{Reader, Life, Rehearsal};
/// use fact_view::FactView;
/// fn cross(reader: &Reader<Rehearsal>) {
///     let view: FactView<'_, Life> = FactView::rebuild(reader, "commit", 100, None).unwrap();
/// }
/// ```
pub struct FactView<'a, L: LineageKind> {
    pub(crate) reader: &'a Reader<L>,
    pub(crate) db: Connection,
    pub(crate) records: Vec<Record>,
    pub(crate) prefix: String,
    pub(crate) last: CoreTransactionV2,
    pub(crate) model: Option<String>,
}
impl<'a, L: LineageKind> FactView<'a, L> {
    /// Rebuild atomically into a fresh private in-memory derived database. No
    /// caller-provided cache path can alias or overwrite an authority database.
    pub fn rebuild(
        reader: &'a Reader<L>,
        prefix: &str,
        limit: usize,
        mut embedder: Option<&mut dyn Embedder>,
    ) -> Result<Self> {
        let transactions = reader.transactions_at(prefix, limit)?;
        let records = source::records(reader, &transactions)?;
        crate::sqlite_vec::register()?;
        let mut db = Connection::open_in_memory()?;
        db.execute_batch("CREATE TABLE facts(rowid INTEGER PRIMARY KEY, id TEXT UNIQUE, text TEXT NOT NULL, at INTEGER NOT NULL); CREATE VIRTUAL TABLE fulltext USING fts5(text, tokenize='trigram'); CREATE TABLE embeddings(rowid INTEGER PRIMARY KEY, vector BLOB NOT NULL);")?;
        let model = embedder.as_ref().map(|e| e.version().to_string());
        let vectors = if let Some(e) = embedder.as_mut() {
            e.passages(&records.iter().map(|r| r.text.clone()).collect::<Vec<_>>())?
        } else {
            vec![]
        };
        if model.is_some() && vectors.len() != records.len() {
            return Err(crate::Error::Invalid("embedding count mismatch".into()));
        }
        let tx = db.transaction()?;
        let mut dimension = None;
        for (i, r) in records.iter().enumerate() {
            tx.execute(
                "INSERT INTO facts VALUES(?1,?2,?3,?4)",
                params![i as i64, r.event.id, r.text, r.event.event_time_ns.0],
            )?;
            tx.execute(
                "INSERT INTO fulltext(rowid,text) VALUES(?1,?2)",
                params![i as i64, r.text],
            )?;
            if let Some(v) = vectors.get(i) {
                if dimension.is_some_and(|d| d != v.len()) {
                    return Err(crate::Error::Invalid("embedding dimensions differ".into()));
                }
                dimension = Some(v.len());
                tx.execute(
                    "INSERT INTO embeddings VALUES(?1,?2)",
                    params![i as i64, vector_bytes(v)?],
                )?;
            }
        }
        tx.commit()?;
        db.pragma_update(None, "query_only", true)?;
        let last = transactions
            .last()
            .ok_or(crate::Error::Invalid("empty prefix".into()))?
            .clone();
        Ok(Self {
            reader,
            db,
            records,
            prefix: prefix.into(),
            last,
            model,
        })
    }
}
