use crate::{
    Access, Budgets, Embedder, Error, FactQuery, FactView, INDEX_VERSION, Purpose, QUERY_VERSION,
    Result, embedding::vector_bytes, source,
};
use core_store::LineageKind;
use core_types::{WireMessage, generated::*, scalar::hex};
use rusqlite::params;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Clone, Debug)]
pub struct RetrievalTrace {
    pub query: FactQuery,
    pub access: Access,
    pub model_version: Option<String>,
    pub query_vector: Option<Vec<f32>>,
    /// Accepted source IDs, in rank order, per entity/time/FTS/vector channel.
    pub channels: [Vec<String>; 4],
    pub returned: Vec<String>,
    pub steps_used: usize,
}
impl RetrievalTrace {
    /// Persist through the outer transaction as a generated audit object before
    /// using the projection. No private index state is an implicit replay input.
    pub fn audit_record(&self, projection: &FactProjection) -> Result<AuditExtension> {
        Ok(AuditExtension {
            schema_version: 1,
            audit_schema_tag: "echo.fact-query-trace".into(),
            audit_schema_version: 1,
            payload: core_types::scalar::HexBytes(serde_json::to_vec(&serde_json::json!({
                "version": QUERY_VERSION,"prefix":self.query.history_prefix,"purpose":format!("{:?}",self.query.purpose),
                "text":self.query.text,"entities":self.query.entities,"kinds":self.query.kinds,"payload_types":self.query.payload_types,"sources":self.query.sources,
                "from_ns":self.query.from_ns.map(|n|n.to_string()),"to_ns":self.query.to_ns.map(|n|n.to_string()),
                "budget":{"items":self.query.budget.items,"bytes":self.query.budget.bytes,"steps":self.query.budget.steps},
                "access":{"allowed":self.access.allowed,"audit":self.access.audit,"sources":self.access.sources},
                "model":self.model_version,"query_vector_bits":self.query_vector.as_ref().map(|v|v.iter().map(|f|f.to_bits()).collect::<Vec<_>>()),
                "channels":self.channels,"returned":self.returned,"steps_used":self.steps_used,"projection":projection
            }))?),
        })
    }
}
pub struct Execution {
    pub projection: FactProjection,
    pub trace: RetrievalTrace,
}

fn spend(steps: &AtomicUsize, n: usize) -> Result<()> {
    steps
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_sub(n))
        .map(|_| ())
        .map_err(|_| Error::Invalid("step budget exhausted".into()))
}

impl<L: LineageKind> FactView<'_, L> {
    pub fn execute(
        &self,
        query: &FactQuery,
        access: &Access,
        embedder: Option<&mut dyn Embedder>,
    ) -> Result<Execution> {
        if query.version != QUERY_VERSION
            || query.history_prefix.is_empty()
            || query.from_ns.zip(query.to_ns).is_some_and(|(f, t)| f > t)
        {
            return Err(Error::Invalid(
                "invalid query version, prefix or time interval".into(),
            ));
        }
        let mut result = Execution {
            projection: FactProjection {
                schema_version: 1,
                history_prefix: query.history_prefix.clone(),
                query_version: QUERY_VERSION.into(),
                status: QueryStatus::NO_MATCH,
                items: vec![],
                index_version: format!(
                    "{};{}",
                    INDEX_VERSION,
                    self.model.as_deref().unwrap_or("no-vector")
                ),
                truncated: false,
            },
            trace: RetrievalTrace {
                query: query.clone(),
                access: access.clone(),
                model_version: self.model.clone(),
                query_vector: None,
                channels: Default::default(),
                returned: vec![],
                steps_used: 0,
            },
        };
        if !access.allowed
            || (query.purpose == Purpose::Audit && !access.audit)
            || (!query.sources.is_empty()
                && !access.sources.is_empty()
                && !query.sources.is_subset(&access.sources))
        {
            result.projection.status = QueryStatus::DENIED;
            return Ok(result);
        }
        if query.history_prefix != self.prefix
            || (query.purpose != Purpose::Audit
                && self.reader.current()?.as_ref() != Some(&self.last))
        {
            result.projection.status = QueryStatus::INDEX_LAGGING;
            return Ok(result);
        }
        let steps = Arc::new(AtomicUsize::new(query.budget.steps));
        let hook = Arc::clone(&steps);
        self.db
            .progress_handler(1, Some(move || spend(&hook, 1).is_err()))?;
        let status = self.retrieve(query, access, embedder, &steps, &mut result);
        self.db.progress_handler(0, None::<fn() -> bool>)?;
        result.trace.steps_used = query.budget.steps - steps.load(Ordering::Relaxed);
        match status {
            Ok(()) => {}
            Err(Error::Sql(rusqlite::Error::SqliteFailure(ref e, _)))
                if e.code == rusqlite::ErrorCode::OperationInterrupted =>
            {
                result.projection.status = QueryStatus::BUDGET_EXHAUSTED;
                result.projection.truncated = true;
            }
            Err(Error::Invalid(ref s)) if s == "step budget exhausted" => {
                result.projection.status = QueryStatus::BUDGET_EXHAUSTED;
                result.projection.truncated = true;
            }
            Err(e) => return Err(e),
        }
        result.trace.returned = result
            .projection
            .items
            .iter()
            .map(|i| i.source_id.clone())
            .collect();
        Ok(result)
    }

    fn retrieve(
        &self,
        q: &FactQuery,
        access: &Access,
        mut embedder: Option<&mut dyn Embedder>,
        steps: &AtomicUsize,
        out: &mut Execution,
    ) -> Result<()> {
        let mut eligible = BTreeSet::new();
        for (i, r) in self.records.iter().enumerate() {
            spend(steps, 1)?;
            let e = &r.event;
            if (!access.sources.is_empty() && !access.sources.contains(&e.device))
                || (!q.sources.is_empty() && !q.sources.contains(&e.device))
                || ((!q.kinds.is_empty() || !q.payload_types.is_empty())
                    && !q.kinds.contains(&e.kind)
                    && !q.payload_types.contains(&e.payload_ref.object_type))
                || q.from_ns.is_some_and(|t| e.event_time_ns.0 < t)
                || q.to_ns.is_some_and(|t| e.event_time_ns.0 > t)
            {
                continue;
            }
            if q.purpose != Purpose::Audit
                && (!r.corrected.is_empty()
                    || (q.purpose != Purpose::Core && !r.retracted.is_empty()))
            {
                continue;
            }
            eligible.insert(i);
        }
        let mut channels: [Vec<usize>; 4] = Default::default();
        let mut entities = q.entities.clone();
        for i in &eligible {
            for (identity, mention) in &self.records[*i].mentions {
                spend(steps, mention.len() + 1)?;
                if q.text.contains(mention) {
                    entities.insert(identity.clone());
                }
            }
        }
        for i in &eligible {
            let r = &self.records[*i];
            spend(steps, r.entities.len() + 1)?;
            if r.entities
                .iter()
                .any(|e| entities.contains(e) || q.text.contains(e))
            {
                channels[0].push(*i);
            }
        }
        // Time is an independent recall route only when the observation carries
        // an explicit time constraint, or when listing without text.
        if q.text.is_empty() || q.from_ns.is_some() || q.to_ns.is_some() {
            channels[1] = eligible.iter().copied().collect();
        }
        let by_recent = |a: &usize, b: &usize| {
            self.records[*b]
                .event
                .event_time_ns
                .0
                .cmp(&self.records[*a].event.event_time_ns.0)
                .then_with(|| self.records[*a].event.id.cmp(&self.records[*b].event.id))
        };
        for channel in &mut channels[..2] {
            spend(steps, sort_cost(channel.len()))?;
            channel.sort_by(by_recent);
        }
        if !q.text.is_empty() {
            let ids = if q.text.chars().count() >= 3 {
                let mut stmt=self.db.prepare("SELECT rowid FROM fulltext WHERE fulltext MATCH ?1 ORDER BY bm25(fulltext),rowid")?;
                let literal = format!("\"{}\"", q.text.replace('"', "\"\""));
                stmt.query_map([literal], |r| r.get::<_, i64>(0))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            } else {
                // FTS5 trigram has no tokens for <3 Unicode characters. A
                // bounded literal scan preserves short Chinese/Japanese terms.
                let mut ids = Vec::new();
                for i in &eligible {
                    spend(steps, self.records[*i].text.len() + 1)?;
                    if self.records[*i].text.contains(&q.text) {
                        ids.push(*i as i64);
                    }
                }
                ids
            };
            channels[2] = ids
                .into_iter()
                .filter_map(|i| usize::try_from(i).ok())
                .filter(|i| eligible.contains(i))
                .collect();
            if let Some(version) = &self.model {
                let e = embedder.as_mut().ok_or(Error::Invalid(
                    "vector index requires its pinned embedder".into(),
                ))?;
                if e.version() != version {
                    return Err(Error::Invalid("vector model version mismatch".into()));
                }
                spend(steps, q.text.len() + 1)?;
                let vector = e.query(&q.text)?;
                let bytes = vector_bytes(&vector)?;
                out.trace.query_vector = Some(vector);
                // Compute distances ONLY for authorized, effective rows. A
                // private high-scoring document never consumes a public top-k.
                let mut distances = Vec::new();
                let mut stmt = self.db.prepare(
                    "SELECT vec_distance_cosine(vector,?1) FROM embeddings WHERE rowid=?2",
                )?;
                for i in &eligible {
                    spend(steps, 1)?;
                    let d: f64 = stmt.query_row(params![bytes, *i as i64], |r| r.get(0))?;
                    if !d.is_finite() {
                        return Err(Error::Invalid("nonfinite vector distance".into()));
                    }
                    distances.push((*i, d));
                }
                spend(steps, sort_cost(distances.len()))?;
                distances.sort_by(|(a, da), (b, db)| {
                    da.total_cmp(db)
                        .then_with(|| self.records[*a].event.id.cmp(&self.records[*b].event.id))
                });
                channels[3] = distances.into_iter().map(|(i, _)| i).collect();
            }
        }
        let mut scores: BTreeMap<usize, f64> = BTreeMap::new();
        // Fixed RRF k=60, top 64 per channel. Order and summation are fixed.
        for (j, channel) in channels.iter_mut().enumerate() {
            channel.truncate(64);
            out.trace.channels[j] = channel
                .iter()
                .map(|i| self.records[*i].event.id.clone())
                .collect();
            for (rank, i) in channel.iter().enumerate() {
                spend(steps, 1)?;
                *scores.entry(*i).or_default() += 1.0 / (60.0 + rank as f64 + 1.0);
            }
        }
        let mut ranked = scores.into_iter().collect::<Vec<_>>();
        spend(steps, sort_cost(ranked.len()))?;
        ranked.sort_by(|(a, sa), (b, sb)| {
            sb.total_cmp(sa)
                .then_with(|| self.records[*a].event.id.cmp(&self.records[*b].event.id))
        });
        for (i, _) in ranked {
            spend(steps, 1)?;
            if out.projection.items.len() >= q.budget.items {
                out.projection.status = QueryStatus::BUDGET_EXHAUSTED;
                out.projection.truncated = true;
                return Ok(());
            }
            let r = &self.records[i];
            let item = FactItem {
                schema_version: 1,
                content: r.event.payload_ref.clone(),
                source_id: r.event.id.clone(),
                availability: QueryStatus::OK,
                correction_chain: r.corrected.iter().chain(&r.retracted).cloned().collect(),
            };
            // Verify original authority bytes, type, digest, lineage and text.
            // Corruption never becomes NO_MATCH, nor a successful empty set.
            let original = self.reader.read::<EventEnvelope>(&r.event_ref);
            if original.as_ref().ok() != Some(&r.event) {
                out.projection.status = QueryStatus::OBJECT_MISSING;
                out.projection.items.clear();
                return Ok(());
            }
            match source::text(self.reader, &item.content) {
                Ok(Some(text)) if text == r.text => {}
                _ => {
                    out.projection.status = QueryStatus::OBJECT_MISSING;
                    out.projection.items.clear();
                    return Ok(());
                }
            }
            out.projection.items.push(item);
            out.projection.status = QueryStatus::OK;
            let encoded = out.projection.canonical_bytes(crate::query::LIMITS)?;
            let payload_bytes = out.projection.items.iter().try_fold(0usize, |sum, i| {
                sum.checked_add(usize::try_from(i.content.byte_length.0).ok()?)
            });
            if payload_bytes
                .and_then(|b| b.checked_add(encoded.len()))
                .is_none_or(|b| b > q.budget.bytes)
            {
                out.projection.items.pop();
                out.projection.status = QueryStatus::BUDGET_EXHAUSTED;
                out.projection.truncated = true;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Existing main IDL adapter. P5-22's successor will be aligned after merge.
    /// Retractions are always hidden from MEMORY, regardless of include_retracted.
    pub fn user_view(
        &self,
        request: &Query,
        access: &Access,
        budgets: &Budgets,
        embedder: Option<&mut dyn Embedder>,
    ) -> Result<(QueryResult, Execution)> {
        request.canonical_bytes(crate::query::LIMITS)?;
        let mut q = FactQuery::new(&self.prefix, Purpose::UserView, "", budgets);
        q.budget.items = q
            .budget
            .items
            .min(usize::try_from(request.limit.0).unwrap_or(usize::MAX));
        if let Some(filter) = &request.filter {
            q.filter(filter)?;
        }
        let mut grant = access.clone();
        grant.allowed &= request.kind == QueryKind::MEMORY;
        let mut execution = self.execute(&q, &grant, embedder)?;
        let items = execution
            .projection
            .items
            .iter()
            .map(|i| {
                let r = self
                    .records
                    .iter()
                    .find(|r| r.event.id == i.source_id)
                    .expect("projected record");
                QueryItem {
                    schema_version: 1,
                    r#ref: hex(&i.content.digest.0),
                    at: r.event.event_time_ns.0.to_string(),
                    kind: source::payload_name(&r.event.payload_ref)
                        .expect("allowlisted fact type"),
                    summary: r.text.clone(),
                    source: r.event.device.clone(),
                    retracted: !r.retracted.is_empty(),
                }
            })
            .collect();
        let mut response = QueryResult {
            schema_version: 1,
            query_id: request.query_id.clone(),
            status: execution.projection.status,
            items,
        };
        while response.canonical_bytes(crate::query::LIMITS)?.len() > q.budget.bytes
            && !response.items.is_empty()
        {
            response.items.pop();
            execution.projection.items.pop();
            response.status = QueryStatus::BUDGET_EXHAUSTED;
            execution.projection.status = QueryStatus::BUDGET_EXHAUSTED;
            execution.projection.truncated = true;
        }
        execution.trace.returned = execution
            .projection
            .items
            .iter()
            .map(|i| i.source_id.clone())
            .collect();
        Ok((response, execution))
    }
}
fn sort_cost(n: usize) -> usize {
    n.saturating_mul((usize::BITS - n.leading_zeros()) as usize)
}
