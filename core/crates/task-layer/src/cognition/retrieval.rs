use super::{MaterialBudget, fitting_prefix};
use core_types::generated::{FactProjection, ObjectRef};

/// Data from W19's task-purpose memory page. Keep the original event and the
/// recording time of an interpretation distinct. A marker belongs to this
/// particular content reference, never to the fact it interprets.
pub struct RecalledMaterial {
    pub content: ObjectRef,
    pub event_id: String,
    pub source: String,
    pub event_time_ns: i64,
    pub recorded_at_ns: i64,
    pub original: String,
    /// Exact W19 MemoryEntry::marked_text() rendering supplied by the query.
    pub marked_text: String,
    pub marker: Option<String>,
}
pub struct RecalledPage {
    pub projection: FactProjection,
    pub memories: Vec<RecalledMaterial>,
}
impl RecalledPage {
    /// Apply the local sender's item, rendered-byte and tokenizer budgets to
    /// the same prefix of the projection and its source-preserving W19 data.
    pub fn bounded<E>(
        &mut self,
        budget: MaterialBudget,
        measure: impl FnMut(&[RecalledMaterial]) -> Result<(usize, usize), E>,
    ) -> Result<usize, E> {
        let kept = fitting_prefix(&self.memories, budget, measure)?;
        let omitted = self.memories.len() - kept;
        self.memories.truncate(kept);
        self.projection.items.truncate(kept);
        if omitted > 0 {
            self.projection.truncated = true;
            self.projection.status = core_types::generated::QueryStatus::BUDGET_EXHAUSTED;
        }
        Ok(omitted)
    }

    pub fn check(&self, prefix: &str) -> Result<(), &'static str> {
        if self.projection.history_prefix != prefix
            || self.projection.items.len() != self.memories.len()
        {
            return Err("incomplete or stale task memory page");
        }
        for (item, memory) in self.projection.items.iter().zip(&self.memories) {
            if item.content != memory.content || item.source_id != memory.event_id {
                return Err("task material and its presentation refer to different sources");
            }
        }
        Ok(())
    }
}
