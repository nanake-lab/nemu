//! Stateless, in-process continuation. Frozen terminal IDL is not repurposed.
use crate::{Access, Budgets, Embedder, Error, Execution, FactQuery, FactView, Purpose, Result};
use core_store::LineageKind;
use core_types::{WireMessage, generated::*, scalar::hex};
use sha2::{Digest, Sha256};

/// Only this crate can issue a cursor. It binds the exact query, prefix, model,
/// authorization and budgets; it contains no authoritative or session state.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct PageCursor {
    binding: String,
    offset: usize,
}
fn binding(q: &FactQuery, access: &Access, index: &str) -> Result<String> {
    let mut plan = q.clone();
    plan.after = None;
    Ok(hex(&Sha256::digest(serde_json::to_vec(&(
        plan, access, index,
    ))?)))
}
pub(crate) fn start(q: &FactQuery, access: &Access, index: &str) -> Result<usize> {
    match &q.after {
        Some(cursor) if cursor.binding != binding(q, access, index)? => Err(Error::Invalid(
            "continuation query or authorization changed".into(),
        )),
        Some(cursor) => Ok(cursor.offset),
        None => Ok(0),
    }
}
pub(crate) fn finish(out: &mut Execution) -> Result<()> {
    out.trace.returned = out
        .projection
        .items
        .iter()
        .map(|i| i.source_id.clone())
        .collect();
    out.next_page = if out.projection.status == QueryStatus::BUDGET_EXHAUSTED
        && !out.projection.items.is_empty()
    {
        Some(PageCursor {
            binding: binding(
                &out.trace.query,
                &out.trace.access,
                &out.projection.index_version,
            )?,
            offset: out
                .trace
                .query
                .after
                .as_ref()
                .map_or(0, |c| c.offset)
                .checked_add(out.projection.items.len())
                .ok_or_else(|| Error::Invalid("continuation overflow".into()))?,
        })
    } else {
        None
    };
    out.trace.next_page = out.next_page.clone();
    Ok(())
}

impl<L: LineageKind> FactView<'_, L> {
    /// P5-22's generated first-page adapter; only QUERY_MEMORY is owned here.
    pub fn user_view(
        &self,
        request: &QueryV2,
        access: &Access,
        budgets: &Budgets,
        embedder: Option<&mut dyn Embedder>,
    ) -> Result<(QueryResultV2, Execution)> {
        self.user_view_page(request, access, budgets, None, embedder)
    }

    /// Trusted in-process caller retains/returns next_page separately from the
    /// unchanged QueryResultV2. Never encode a cursor into text/date/filter fields.
    pub fn user_view_page(
        &self,
        request: &QueryV2,
        access: &Access,
        budgets: &Budgets,
        after: Option<&PageCursor>,
        embedder: Option<&mut dyn Embedder>,
    ) -> Result<(QueryResultV2, Execution)> {
        request.canonical_bytes(crate::query::LIMITS)?;
        let mut q = FactQuery::new(&self.prefix, Purpose::UserView, "", budgets);
        q.budget.items = q
            .budget
            .items
            .min(usize::try_from(request.limit.0).unwrap_or(usize::MAX));
        q.after = after.cloned();
        if let Some(filter) = &request.filter {
            q.filter(filter)?;
        }
        let mut grant = access.clone();
        grant.allowed &= request.kind == QueryKindV2::QUERY_MEMORY;
        let mut execution = self.execute(&q, &grant, embedder)?;
        let items = execution
            .trace
            .memories
            .iter()
            .map(|m| QueryItem {
                schema_version: 1,
                r#ref: hex(&m.content.digest.0),
                at: m.at_ns.0.to_string(),
                kind: m.kind.clone(),
                summary: m.marked_text(),
                source: m.source.clone(),
                // MEMORY never exposes withdrawn records, including reactions.
                retracted: false,
            })
            .collect();
        let mut response = QueryResultV2 {
            schema_version: 1,
            query_id: request.query_id.clone(),
            status: execution.projection.status,
            items,
            growth_entries: vec![],
        };
        while response.canonical_bytes(crate::query::LIMITS)?.len() > q.budget.bytes
            && !response.items.is_empty()
        {
            response.items.pop();
            execution.projection.items.pop();
            execution.trace.memories.pop();
            response.status = QueryStatus::BUDGET_EXHAUSTED;
            execution.projection.status = QueryStatus::BUDGET_EXHAUSTED;
            execution.projection.truncated = true;
        }
        finish(&mut execution)?;
        Ok((response, execution))
    }
}
