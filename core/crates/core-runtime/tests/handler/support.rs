use crate::common::*;
use core_runtime::{binding::digest, handler::EventInputs, stages::Snapshot, store};
use core_store::{CommitCapability, Rehearsal, Store, StoreConfig, TemporaryRoot};
use core_types::{generated::*, scalar::DecimalU64};
use std::time::Duration;

pub fn config() -> StoreConfig {
    StoreConfig {
        encoding: limits(),
        busy_timeout: Duration::from_secs(1),
        max_pending_records: 32,
        max_pending_bytes: 1 << 20,
        max_object_bytes: 4 << 20,
        max_store_bytes: 64 << 20,
        max_sql_pages: 8192,
    }
}
pub fn initial(temp: &TemporaryRoot) -> (Store<Rehearsal>, CommitCapability<Rehearsal>) {
    let mut header: LineageHeaderV2 = fixture();
    header.lineage = Lineage::REHEARSAL;
    header.genesis_manifest_digest = None;
    header.world_id = None;
    header.world_seed = None;
    let mut writer =
        Store::<Rehearsal>::create_temporary(temp, "handler", header, "test", config()).unwrap();
    let cap = writer.take_commit_capability().unwrap();
    let resource = writer.prepare_raw(b"synthetic handler resource").unwrap();
    let mut state = Snapshot {
        x: fixture(),
        r: fixture(),
        theta: fixture(),
    };
    let law = writer
        .prepare(
            &with_refs::<LawVersion>(resource.reference()),
            &state.theta.parameters.law_id,
        )
        .unwrap();
    state.r.tendencies.clear();
    state.x.learned.clear();
    state.r.law_version = law.reference().clone();
    state.theta.law_version = law.reference().clone();
    let mut env: VersionedEnvironmentV2 = with_refs(resource.reference());
    env.implementations.clear();
    env.law_version = None;
    store::bootstrap_rehearsal(&mut writer, &cap, &env, &state).unwrap();
    (writer, cap)
}

pub fn inputs(
    store: &Store<Rehearsal>,
    event: &EventEnvelope,
) -> core_runtime::Result<EventInputs> {
    let roots = store.published_roots()?.unwrap();
    let q: InitialRuntimeRoot = store.verified_reader()?.read(&roots.q)?;
    let mut base = store::read_state(store)?;
    let mut binding: ProposalBindingV2 = with_refs(&q.environment.model);
    binding.branch_id = "test".into();
    binding.base_commit = store.base_commit()?;
    binding.event_id = event.id.clone();
    binding.raw_log_position = event.raw_log_position;
    binding.anchor_time_ns = base.x.anchor_time_ns;
    // Synthetic provider supplies an explicit, same-boot process time for each test event.
    let time = time(100 + event.raw_log_position.0);
    binding.process_time_ns = DecimalU64(time.ns());
    binding.x_revision = base.x.revision.clone();
    binding.r_revision = base.r.revision;
    binding.theta_revision = base.theta.revision;
    binding.environment = q.environment;
    binding.read_set = vec![
        binding.environment.model.clone(),
        roots.h,
        roots.x,
        roots.r,
        roots.theta,
        roots.l,
        roots.q,
        roots.u,
        event.payload_ref.clone(),
        event.provenance.clone(),
    ];
    binding.read_set.sort_by(|a, b| a.digest.0.cmp(&b.digest.0));
    binding.read_set.dedup();
    let mut facts: FactProjection = fixture();
    facts.items.clear();
    facts.history_prefix = binding.base_commit.clone();
    binding.c_core_digest = digest(&facts, limits())?;
    base.x.anchor_time_ns = binding.process_time_ns;
    binding.projected_state_digest = digest(&base.x, limits())?;
    let mut attributes: EventAttributesRecord = fixture();
    attributes.event_id = event.id.clone();
    Ok(EventInputs {
        binding,
        time,
        facts,
        attributes,
    })
}
