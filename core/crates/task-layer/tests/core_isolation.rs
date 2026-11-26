use core_runtime::{
    binding::digest,
    clock::{Bounds, Clock, Provider, Sample},
    pipeline::Input,
    stages::{Snapshot, StageSet},
};
use core_store::Experiment;
use core_types::{
    WireMessage,
    contract::Limits,
    generated::*,
    scalar::{DecimalU64, unhex},
};
use std::{fs, path::Path};
use task_layer::task::protocol::{GenerationBudget, Protocol, Source};

#[path = "../../core-runtime/tests/common/adapters.rs"]
mod adapters;
const LIMITS: Limits = Limits {
    max_bytes: 4 << 20,
    max_items: 4096,
    max_depth: 64,
};
fn fixture<T: WireMessage>() -> T {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../core-runtime/tests/fixtures")
        .join(format!("{}.hex", T::NAME));
    T::from_canonical(
        &unhex(fs::read_to_string(path).unwrap().trim()).unwrap(),
        LIMITS,
    )
    .unwrap()
}
struct FixedTime(Sample);
impl Provider for FixedTime {
    fn sample(&mut self) -> Sample {
        self.0.clone()
    }
}

#[test]
fn x01_sys06_protocol_and_task_facts_change_only_generation_inputs() {
    let tmp = tempfile::tempdir().unwrap();
    let folder = tmp.path().join("_candidates/core/protocol");
    fs::create_dir_all(&folder).unwrap();
    fs::write(
        folder.join("manifest.json"),
        r#"{"version":"p0","status":"candidate","files":["en.json"]}"#,
    )
    .unwrap();
    let mut traces = vec![];
    let mut inputs = vec![];
    for (p, history_text) in [
        ("protocol-a", "quoted-a"),
        ("protocol-b", "quoted-a"),
        ("protocol-a", "quoted-b"),
    ] {
        fs::write(
            folder.join("en.json"),
            serde_json::to_vec(
                &serde_json::json!({"facts":p,"boundaries":"b","format":"{acts}","language":"en"}),
            )
            .unwrap(),
        )
        .unwrap();
        let protocol = Protocol::load(tmp.path(), Source::Candidate, Lang::EN, &[]).unwrap();
        let event: EventEnvelope = fixture();
        let mut attributes: EventAttributesRecord = fixture();
        attributes.event_id = event.id.clone();
        attributes.exogenous = true;
        attributes.self_narrative = false;
        let mut base = Snapshot {
            x: fixture(),
            r: fixture(),
            theta: fixture(),
        };
        base.x.anchor_time_ns = DecimalU64(10);
        base.x.nature.connection = 0.1;
        base.x.learned.clear();
        base.r.tendencies.clear();
        base.theta.values.baseline.connection = 0.0;
        let mut b: ProposalBindingV2 = fixture();
        b.event_id = event.id.clone();
        b.raw_log_position = event.raw_log_position;
        b.anchor_time_ns = base.x.anchor_time_ns;
        b.process_time_ns = DecimalU64(30);
        b.x_revision = base.x.revision.clone();
        b.r_revision = base.r.revision.clone();
        b.theta_revision = base.theta.revision.clone();
        let model = b.environment.model.clone();
        b.environment.nature_appraiser = Some(model.clone());
        b.environment.situation_matcher = Some(model.clone());
        b.environment.direction_set = Some(model.clone());
        b.environment.law_version = Some(model.clone());
        let mut core_facts: FactProjection = fixture();
        core_facts.history_prefix = b.base_commit.clone();
        b.c_core_digest = digest(&core_facts, LIMITS).unwrap();
        let mut projected = base.x.clone();
        projected.anchor_time_ns = b.process_time_ns;
        b.projected_state_digest = digest(&projected, LIMITS).unwrap();
        b.accepted_z_digest = digest(&fixture::<SemanticGraph>(), LIMITS).unwrap();
        let mut task_facts = core_facts.clone();
        task_facts.items = vec![FactItem {
            schema_version: 1,
            content: model.clone(),
            source_id: history_text.into(),
            availability: QueryStatus::OK,
            correction_chain: vec![],
        }];
        let task = protocol.task_input(
            model,
            task_facts,
            "snapshot".into(),
            GenerationBudget {
                tokens: 128,
                duration_ns: 1000,
            },
        );
        assert!(!task.protocol_content.contains("quoted-"));
        inputs.push(task);
        let sample = |n| Sample {
            boot: "test".into(),
            epoch: "test".into(),
            monotonic_ns: n,
            wall_ns: 100 + n as i64,
            uncertainty_ns: Some(0),
        };
        let mut clock = Clock::start(
            sample(0),
            0,
            Bounds {
                max_interval_ns: 100,
                max_wall_skew_ns: 2,
                max_uncertainty_ns: 1,
            },
        )
        .unwrap();
        let time = clock.capture(&mut FixedTime(sample(30))).unwrap();
        let log = adapters::Log::default();
        let mut stages =
            StageSet::<Experiment>::complete(adapters::parts(&log), b.environment.clone(), LIMITS)
                .unwrap();
        let staged = stages
            .process_head(Input {
                event,
                attributes: &attributes,
                base,
                facts: &core_facts,
                binding: b,
                time,
                growth_paused: false,
            })
            .unwrap();
        assert_eq!(staged.snapshot().r.revision, "after-g");
        assert_eq!(staged.snapshot().x.nature.connection, 0.2);
        traces.push((
            staged.snapshot().x.canonical_bytes(LIMITS).unwrap(),
            staged.snapshot().r.canonical_bytes(LIMITS).unwrap(),
            staged.snapshot().theta.canonical_bytes(LIMITS).unwrap(),
            staged.ledger().to_vec(),
            staged.trace().to_vec(),
            staged.binding().clone(),
        ));
    }
    assert_ne!(inputs[0].protocol_version, inputs[1].protocol_version);
    assert_ne!(inputs[0].c_task, inputs[2].c_task);
    assert_eq!(traces[0], traces[1]);
    assert_eq!(traces[0], traces[2]);
}
