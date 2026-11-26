use core_types::{WireMessage, contract::Limits, generated::*};
use time::{Date, OffsetDateTime, format_description::well_known::Rfc3339};

/// The caller supplies the authorized W19 user_view reader, the Q actions
/// (including refused rows), and the mode/status history at this same prefix.
/// No local copy of the user's history is maintained by the task layer.
#[derive(Clone, Copy)]
pub struct Journal<'a> {
    pub actions: Result<&'a [QueryItem], QueryStatus>,
    pub run_history: Result<&'a [QueryItem], QueryStatus>,
}

/// Adapter used by the trusted gateway host. The same authority grant and
/// source restriction apply to the Q/status snapshots and to the W19 reader.
pub struct UserView<'a, 'r, L: core_store::LineageKind> {
    pub facts: &'a fact_view::FactView<'r, L>,
    pub access: &'a fact_view::Access,
    pub budgets: &'a fact_view::Budgets,
    pub journal: Journal<'a>,
    pub limits: Limits,
}
impl<L: core_store::LineageKind> UserView<'_, '_, L> {
    pub fn handle(
        &self,
        request: &QueryV2,
        audit: impl FnOnce(&QueryV2, &QueryResultV2) -> Result<(), String>,
    ) -> Result<QueryResultV2, String> {
        query(
            request,
            self.journal,
            self.access,
            self.budgets.user_view,
            self.limits,
            |q| {
                self.facts
                    .user_view(q, self.access, self.budgets, None)
                    .map(|(r, _)| r)
                    .map_err(|e| e.to_string())
            },
            audit,
        )
    }
}

fn instant(text: &str, end: bool) -> Result<i128, String> {
    if let Ok(n) = text.parse::<i64>() {
        return Ok(i128::from(n));
    }
    if text.len() == 10 {
        let fmt =
            time::format_description::parse("[year]-[month]-[day]").map_err(|e| e.to_string())?;
        let date = Date::parse(text, &fmt).map_err(|e| e.to_string())?;
        return Ok(date.midnight().assume_utc().unix_timestamp_nanos()
            + if end { 86_400_000_000_000 - 1 } else { 0 });
    }
    Ok(OffsetDateTime::parse(text, &Rfc3339)
        .map_err(|e| e.to_string())?
        .unix_timestamp_nanos())
}

fn select(
    request: &QueryV2,
    rows: Result<&[QueryItem], QueryStatus>,
    access: &fact_view::Access,
    budget: fact_view::Budget,
    limits: Limits,
) -> Result<QueryResultV2, String> {
    let mut result = QueryResultV2 {
        schema_version: 1,
        query_id: request.query_id.clone(),
        status: QueryStatus::NO_MATCH,
        items: vec![],
        growth_entries: vec![],
    };
    if !access.allowed {
        result.status = QueryStatus::DENIED;
        return Ok(result);
    }
    let rows = match rows {
        Ok(rows) => rows,
        Err(status) => {
            if status == QueryStatus::OK {
                return Err("missing journal rows".into());
            }
            result.status = status;
            return Ok(result);
        }
    };
    let filter = request.filter.as_ref();
    let from = filter
        .and_then(|f| f.from.as_deref())
        .map(|s| instant(s, false))
        .transpose()?;
    let to = filter
        .and_then(|f| f.to.as_deref())
        .map(|s| instant(s, true))
        .transpose()?;
    if from.zip(to).is_some_and(|(a, b)| a > b) {
        return Err("inverted time interval".into());
    }
    for (i, row) in rows.iter().enumerate() {
        if i >= budget.steps {
            result.status = QueryStatus::BUDGET_EXHAUSTED;
            break;
        }
        if !access.sources.is_empty() && !access.sources.contains(&row.source) {
            continue;
        }
        if from.is_some() || to.is_some() {
            let at = instant(&row.at, false)?;
            if from.is_some_and(|t| at < t) || to.is_some_and(|t| at > t) {
                continue;
            }
        }
        if filter.is_some_and(|f| {
            (!f.kinds.is_empty() && !f.kinds.contains(&row.kind))
                || f.text.as_ref().is_some_and(|s| !row.summary.contains(s))
        }) {
            continue;
        }
        if row.retracted && !filter.and_then(|f| f.include_retracted).unwrap_or(false) {
            continue;
        }
        if result.items.len() as u64 >= request.limit.0 || result.items.len() >= budget.items {
            result.status = QueryStatus::BUDGET_EXHAUSTED;
            break;
        }
        result.status = QueryStatus::OK;
        result.items.push(row.clone());
        if result.canonical_bytes(limits)?.len() > budget.bytes {
            result.items.pop();
            result.status = QueryStatus::BUDGET_EXHAUSTED;
            break;
        }
    }
    Ok(result)
}

/// Persist a user_view audit through the host before releasing any result.
/// The generated query and result are immutable, including failure statuses.
pub fn query(
    request: &QueryV2,
    journal: Journal<'_>,
    access: &fact_view::Access,
    budget: fact_view::Budget,
    limits: Limits,
    memory: impl FnOnce(&QueryV2) -> Result<QueryResultV2, String>,
    audit: impl FnOnce(&QueryV2, &QueryResultV2) -> Result<(), String>,
) -> Result<QueryResultV2, String> {
    request.canonical_bytes(limits)?;
    let result = match request.kind {
        QueryKindV2::QUERY_MEMORY if !access.allowed => QueryResultV2 {
            schema_version: 1,
            query_id: request.query_id.clone(),
            status: QueryStatus::DENIED,
            items: vec![],
            growth_entries: vec![],
        },
        QueryKindV2::QUERY_MEMORY => memory(request)?,
        QueryKindV2::QUERY_ACTIONS => select(request, journal.actions, access, budget, limits)?,
        QueryKindV2::QUERY_RUN_HISTORY => {
            select(request, journal.run_history, access, budget, limits)?
        }
        QueryKindV2::QUERY_GROWTH => return Err("route to growth reader".into()),
    };
    if result.query_id != request.query_id
        || !result.growth_entries.is_empty()
        || result.items.len() as u64 > request.limit.0
    {
        return Err("user_view result does not match request".into());
    }
    if result.canonical_bytes(limits)?.len() > budget.bytes {
        return Err("user_view response exceeds budget".into());
    }
    audit(request, &result)?;
    Ok(result)
}
