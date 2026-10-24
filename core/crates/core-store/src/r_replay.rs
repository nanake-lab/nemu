//! Storage boundary for a trusted kernel's full effective-H replay. This module
//! does not implement G or invent q/V from a legacy statistical snapshot.
use crate::{Error, LineageKind, PreparedTransaction, ReadTicket, Result, Store, error::require};
use core_types::{
    WireMessage,
    generated::{
        CoreTransactionV2, EventEnvelope, EvidenceGroupV3, GenesisManifestV2, LawVersion,
        ObjectRef, ProposalBindingV2, RSnapshotV3, VersionedEnvironmentV2,
    },
    scalar::hex,
};
use std::collections::{BTreeMap, BTreeSet};

/// One accepted H event, in durable order. Correction/reinterpretation events
/// remain intact: only the governing kernel may determine the effective history.
pub struct RReplayEvent {
    pub event: EventEnvelope,
    pub binding: ProposalBindingV2,
    pub accepted_results: Vec<ObjectRef>,
    pub staged_concepts: Vec<ObjectRef>,
}

/// Constructed only by Store, from a verified full committed prefix plus the
/// pending event. No truncated Reader::facts result or old R statistics enter it.
pub struct RReplayInput {
    pub genesis: GenesisManifestV2,
    pub law: LawVersion,
    pub law_reference: ObjectRef,
    pub events: Vec<RReplayEvent>,
    resources: BTreeMap<String, (ObjectRef, Vec<u8>)>,
}
impl RReplayInput {
    /// Hash-verified source/resource bytes. R summaries and evidence allocations
    /// of every version are deliberately unavailable through this view.
    pub fn read(&self, reference: &ObjectRef) -> Result<&[u8]> {
        let (stored, bytes) =
            self.resources
                .get(&hex(&reference.digest.0))
                .ok_or(Error::Rejected(
                    "resource absent from verified replay input",
                ))?;
        require(stored == reference, "replay resource reference mismatch")?;
        Ok(bytes)
    }
}

/// Entire rebuilt R and updated evidence groups, including empty successors for
/// removed allocations. Already empty groups need not be republished.
/// This is a kernel result, not a wire schema.
pub struct RebuiltR {
    pub snapshot: RSnapshotV3,
    pub groups: Vec<EvidenceGroupV3>,
}

impl<L: LineageKind> Store<L> {
    /// Invoke the trusted governing-law kernel from genesis over complete H and
    /// stage its V3 result in the normal outer transaction. There is intentionally
    /// no default adapter. The adapter must resolve EV validity/reinterpretation,
    /// recompute pre-event facts and all q/V; structural checks cannot prove G.
    /// A failure leaves the old head and all historical bytes intact.
    pub fn stage_r_replay(
        &self,
        ticket: ReadTicket<L>,
        mut value: CoreTransactionV2,
        genesis: &ObjectRef,
        limit: usize,
        rebuild: impl FnOnce(&RReplayInput) -> Result<RebuiltR>,
    ) -> Result<PreparedTransaction<L>> {
        self.check_binding(ticket.binding())?;
        require(
            value.binding == *ticket.binding(),
            "replay binding mismatch",
        )?;
        let pending = self
            .next_raw()?
            .ok_or(Error::Rejected("no durable replay event"))?;
        require(
            value.staged_h == self.objects.reference(&pending, "1")?,
            "replay H differs from pending event",
        )?;
        require(
            value.binding.read_set.contains(genesis),
            "genesis absent from read set",
        )?;
        self.verify()?;
        let limits = self.objects.config.encoding;
        require(
            limit > 0 && limit <= limits.max_items,
            "replay event budget exceeded",
        )?;
        let count: i64 = self
            .db
            .query_row("SELECT count(*) FROM facts", [], |r| r.get(0))?;
        require(
            usize::try_from(count)
                .ok()
                .and_then(|n| n.checked_add(1))
                .is_some_and(|n| n <= limit),
            "incomplete H: replay event budget exceeded",
        )?;
        let manifest = GenesisManifestV2::from_canonical(&self.objects.read(genesis)?, limits)?;
        core_types::semantic::check_genesis(&manifest)?;
        if let Some(digest) = &self.header.genesis_manifest_digest {
            require(
                digest == &genesis.digest,
                "replay genesis differs from lineage",
            )?;
        }
        let law_reference = value
            .binding
            .environment
            .law_version
            .clone()
            .ok_or(Error::Rejected("replay requires a bound governing law"))?;
        let law = LawVersion::from_canonical(&self.objects.read(&law_reference)?, limits)?;
        core_types::semantic::check_law_parameters(&law.parameters)?;
        let summary_tags = r_summary_tags()?;
        let mut events = Vec::new();
        let mut statement = self.db.prepare("SELECT reference FROM snapshots JOIN commits ON commits.id=snapshots.commit_id ORDER BY commits.position")?;
        let rows = statement.query_map([], |row| row.get::<_, Vec<u8>>(0))?;
        for row in rows {
            let reference = ObjectRef::from_canonical(&row?, limits)?;
            let historical =
                CoreTransactionV2::from_canonical(&self.objects.read(&reference)?, limits)?;
            events.push(self.replay_event(&historical, &summary_tags)?);
        }
        require(
            events.len()
                == usize::try_from(count).map_err(|_| Error::Rejected("H count overflow"))?,
            "incomplete H replay input",
        )?;
        // The pending H identity is checked again by ordinary staging.
        require(
            !value
                .accepted_results
                .iter()
                .any(|r| summary_tags.contains(&r.object_type)),
            "replay takes original interpretation results, not caller-built R evidence",
        )?;
        events.push(self.replay_event(&value, &summary_tags)?);
        let mut input = RReplayInput {
            genesis: manifest,
            law,
            law_reference,
            events,
            resources: BTreeMap::new(),
        };
        let mut roots = vec![genesis.clone(), input.law_reference.clone()];
        let schema = core_types::contract::Schema::builtin()?;
        for item in &input.events {
            roots.push(item.event.payload_ref.clone());
            roots.push(item.binding.validation_snapshot.clone());
            roots.extend(item.accepted_results.iter().cloned());
            roots.extend(item.staged_concepts.iter().cloned());
            crate::objects::collect_refs(
                &schema,
                VersionedEnvironmentV2::NAME,
                &serde_json::to_value(&item.binding.environment)?,
                &mut roots,
            )?;
        }
        let mut bytes_used = 0u64;
        for root in roots {
            for reference in self.objects.closure(&root)? {
                if summary_tags.contains(&reference.object_type) {
                    continue;
                }
                let key = hex(&reference.digest.0);
                if let Some((old, _)) = input.resources.get(&key) {
                    require(old == &reference, "ambiguous replay resource version")?;
                    continue;
                }
                let bytes = self.objects.read(&reference)?;
                bytes_used = bytes_used
                    .checked_add(bytes.len() as u64)
                    .ok_or(Error::Rejected("replay byte budget overflow"))?;
                require(
                    bytes_used <= self.objects.config.max_store_bytes
                        && input.resources.len() < limits.max_items,
                    "replay resource budget exceeded",
                )?;
                input.resources.insert(key, (reference, bytes));
            }
        }
        let rebuilt = rebuild(&input)?;
        require(
            rebuilt.snapshot.law_version == input.law_reference,
            "rebuilt R law mismatch",
        )?;
        let ids: BTreeSet<_> = rebuilt.groups.iter().map(|g| g.group_id.as_str()).collect();
        require(ids.len() == rebuilt.groups.len(), "duplicate replay group")?;
        crate::r_validation::visit_latest_groups(&self.db, &self.objects, None, |group| {
            require(
                group.tendencies().is_empty() || ids.contains(group.id()),
                "replay omitted an allocated group; an empty successor is required",
            )
        })?;
        let mut pins = Vec::new();
        for group in &rebuilt.groups {
            let pin = self.prepare(group, "1")?;
            value.accepted_results.push(pin.reference().clone());
            pins.push(pin);
        }
        let r = self.prepare_r(&rebuilt.snapshot)?;
        value.staged_r = r.reference().clone();
        // stage pins the whole manifest closure before these temporary pins drop.
        self.stage_with_replay(ticket, value, true)
    }

    fn replay_event(
        &self,
        value: &CoreTransactionV2,
        summary_tags: &BTreeSet<String>,
    ) -> Result<RReplayEvent> {
        Ok(RReplayEvent {
            event: EventEnvelope::from_canonical(
                &self.objects.read(&value.staged_h)?,
                self.objects.config.encoding,
            )?,
            binding: value.binding.clone(),
            staged_concepts: value.staged_concepts.clone(),
            accepted_results: value
                .accepted_results
                .iter()
                .filter(|r| !summary_tags.contains(&r.object_type))
                .cloned()
                .collect(),
        })
    }
}

fn r_summary_tags() -> Result<BTreeSet<String>> {
    use core_types::generated::*;
    let schema = core_types::contract::Schema::builtin()?;
    [
        RSnapshot::NAME,
        RSnapshotV2::NAME,
        RSnapshotV3::NAME,
        Tendency::NAME,
        TendencyV2::NAME,
        TendencyV3::NAME,
        EvidenceGroup::NAME,
        EvidenceGroupV2::NAME,
        EvidenceGroupV3::NAME,
        EvidenceAllocation::NAME,
        EvidenceAllocationV2::NAME,
        EvidenceAllocationV3::NAME,
    ]
    .into_iter()
    .map(|name| Ok(schema.message(name)?.tag.clone()))
    .collect()
}
