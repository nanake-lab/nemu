use crate::{LineageKind, Reader, Result, error::require};
use core_types::{
    WireMessage,
    generated::{EvidenceGroupV2, LedgerEntry, ObjectRef},
};
use rusqlite::params;

impl<L: LineageKind> Reader<L> {
    /// Complete ledger prefix or an explicit budget error; never an empty/truncated
    /// result masquerading as evidence that no prior cost was recorded.
    pub fn ledger_at(&self, commit: &str, limit: usize) -> Result<Vec<LedgerEntry>> {
        let position = self.query_prefix(commit, limit)?;
        let mut stmt = self.db.prepare("SELECT l.reference FROM ledger_index l JOIN commits c ON c.id=l.commit_id WHERE c.position<=?1 ORDER BY c.position,l.entry_id LIMIT ?2")?;
        let refs = stmt
            .query_map(params![position, (limit + 1) as i64], |r| {
                r.get::<_, Vec<u8>>(0)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        self.decode_complete(refs, limit)
    }
    /// Latest accepted version of each evidence group at an explicit commit. A
    /// group update that removes an allocation also removes the old association.
    pub fn evidence_for(
        &self,
        tendency: &str,
        commit: &str,
        limit: usize,
    ) -> Result<Vec<EvidenceGroupV2>> {
        let position = self.query_prefix(commit, limit)?;
        let mut stmt = self.db.prepare("WITH latest AS (SELECT g.commit_id,g.group_id,g.reference,row_number() OVER (PARTITION BY g.group_id ORDER BY c.position DESC) AS rank FROM evidence_groups g JOIN commits c ON c.id=g.commit_id WHERE c.position<=?1) SELECT l.reference FROM latest l JOIN evidence_index i ON i.commit_id=l.commit_id AND i.group_id=l.group_id WHERE l.rank=1 AND i.tendency_id=?2 ORDER BY l.group_id LIMIT ?3")?;
        let refs = stmt
            .query_map(params![position, tendency, (limit + 1) as i64], |r| {
                r.get::<_, Vec<u8>>(0)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        self.decode_complete(refs, limit)
    }
    fn query_prefix(&self, commit: &str, limit: usize) -> Result<i64> {
        require(
            limit < self.objects.config.encoding.max_items && limit < i64::MAX as usize,
            "query budget exceeded",
        )?;
        Ok(self
            .db
            .query_row("SELECT position FROM commits WHERE id=?1", [commit], |r| {
                r.get(0)
            })?)
    }
    fn decode_complete<T: WireMessage>(&self, refs: Vec<Vec<u8>>, limit: usize) -> Result<Vec<T>> {
        require(
            refs.len() <= limit,
            "incomplete query: result exceeds budget",
        )?;
        refs.into_iter()
            .map(|bytes| {
                self.read(&ObjectRef::from_canonical(
                    &bytes,
                    self.objects.config.encoding,
                )?)
            })
            .collect()
    }
}
