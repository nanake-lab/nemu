use core_types::{
    contract::Limits,
    generated::*,
    scalar::{DecimalU64, HexBytes},
};
use sha2::{Digest, Sha256};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};
use task_layer::{
    cause_projection as cp,
    task::{projection::*, protocol::*, reply::*, speech::*},
};

const LIMITS: Limits = Limits {
    max_bytes: 1 << 20,
    max_items: 10000,
    max_depth: 64,
};
fn object(name: &str) -> ObjectRef {
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(Sha256::digest(name.as_bytes()).to_vec()),
        object_type: "fixture".into(),
        resource_version: "1".into(),
        byte_length: DecimalU64(name.len() as u64),
    }
}
fn input() -> TaskInput {
    TaskInput {
        schema_version: 1,
        current_input: object("hey"),
        protocol_version: "fixture-p".into(),
        protocol_content: "p".into(),
        c_task: FactProjection {
            schema_version: 1,
            history_prefix: "h1".into(),
            query_version: "v".into(),
            index_version: "v".into(),
            status: QueryStatus::NO_MATCH,
            items: vec![],
            truncated: false,
        },
        snapshot_id: "s".into(),
        max_tokens: DecimalU64(24),
        max_duration_ns: DecimalU64(5_000_000_000),
    }
}
fn binding(task: &TaskInput) -> Binding {
    Binding {
        branch: "b".into(),
        snapshot: task.snapshot_id.clone(),
        frontier: vec!["o".into()],
        protocol_version: task.protocol_version.clone(),
        permission: object("grants"),
        q_revision: "q1".into(),
    }
}

struct Codec;
impl SpeechCodec for Codec {
    fn output_reserve(&self, tokens: u64) -> Result<u64, String> {
        Ok(tokens)
    }
    fn encode(
        &self,
        p: &cp::RequestPlan,
        records: &cp::RecordTable,
        instructions: &str,
        hints: &[String],
    ) -> Vec<u8> {
        let body =
            |id: &cp::RecordId| String::from_utf8(records.get(id).unwrap().body.clone()).unwrap();
        serde_json::to_vec(&serde_json::json!({"P":String::from_utf8_lossy(&p.protocol),
            "E":p.experience.iter().map(|e|body(&e.id)).collect::<Vec<_>>(),"o":body(&p.latest),"format":instructions,"hints":hints})).unwrap()
    }
    fn measure(&self, bytes: &[u8]) -> u64 {
        bytes.len() as u64
    }
}
fn material(task: &TaskInput) -> CloudMaterial<'static> {
    let latest = RecordVersion {
        schema_version: 1,
        branch_id: "b".into(),
        record_id: "o".into(),
        content: task.current_input.clone(),
    };
    CloudMaterial {
        prefix: "h1".into(),
        bound: binding(task),
        latest,
        settings: SpeechSettings::default(),
        samples: None,
        gaps: vec![],
        input: cp::AssemblyInput {
            protocol: vec![],
            protocol_cited: vec![],
            dialogue: vec![],
            latest: cp::RecordId(vec![]),
            required_facts: vec![],
            records: vec![cp::Record {
                id: cp::RecordId(b"o".to_vec()),
                content_version: task.current_input.digest.0.clone(),
                h_pos: cp::HPos(1),
                body: b"hey".to_vec(),
                eligibility: cp::Eligibility::Eligible,
            }],
            relations: vec![],
            frontier_complete: true,
            slots: vec![],
            decomposition_unknown: false,
            budget: cp::Budget {
                capacity: 8192,
                output_reserve: 24,
            },
        },
    }
}
struct Reviewer;
impl Governance for Reviewer {
    fn layers(&self, _: &str) -> Option<(&RecordVersion, &[QuoteLayer])> {
        None
    }
    fn self_id(&self) -> &str {
        "nemu"
    }
    fn supported(&self, p: &Point, _: &[&VisibleQuote<'_>]) -> bool {
        p.kind == Kind::Other && p.text == "Hello."
    }
}
struct Script {
    calls: Rc<Cell<usize>>,
    mode: SpeechMode,
    chunks: bool,
    action: Option<ActionPlan>,
}
impl Generator for Script {
    fn generate(
        &mut self,
        input: ModelInput<'_>,
        _: GenerationBudget,
    ) -> Result<Generated, FailureReason> {
        self.calls.set(self.calls.get() + 1);
        match input {
            ModelInput::Projected(bytes) => {
                let req: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                assert_eq!(req["o"], "hey");
            }
            ModelInput::Local { modulation, .. } => {
                assert!(matches!(modulation, Modulation::Zero(_)))
            }
        }
        let projected = self.mode == SpeechMode::B_REMOTE_PROJECTED;
        Ok(Generated {
            receipt: PlanAndCarrierReceipt {
                schema_version: 1,
                output_chunks: if self.chunks {
                    vec![OutputChunk {
                        schema_version: 1,
                        output_id: "out".into(),
                        action_id: "speak".into(),
                        chunk_index: DecimalU64(0),
                        text: "Hello.".into(),
                        lang: Lang::EN,
                        snapshot_id: "s".into(),
                        speech_mode: self.mode,
                        strict: false,
                    }]
                } else {
                    vec![]
                },
                action_plan: self.action.clone(),
                speech_mode: self.mode,
                strict: false,
                model_hash: object("script").digest,
                tokenizer_hash: object("script words").digest,
                carrier_receipt: (!projected).then(|| object("script applied")),
                projection_version: projected.then(|| "fixture".into()),
                intermediate_results: vec![],
                usage: object("one output token"),
            },
            used_output_tokens: 1,
            outline: projected
                .then(|| r#"{"points":[{"type":"other","text":"Hello.","cites":[]}]}"#.into()),
        })
    }
}
#[derive(Default)]
struct TestOutbox {
    calls: usize,
    chunks: usize,
}
impl Outbox for TestOutbox {
    fn register(
        &mut self,
        _: &Binding,
        chunks: &[OutputChunk],
        _: Option<&ActionPlan>,
    ) -> Result<(), String> {
        self.calls += 1;
        self.chunks += chunks.len();
        Ok(())
    }
}
fn setup(
    mode: SpeechMode,
    chunks: bool,
    action: Option<ActionPlan>,
) -> (Generators, Rc<Cell<usize>>) {
    let calls = Rc::new(Cell::new(0));
    let mut registry = Generators::default();
    registry.register(
        mode,
        Box::new(Script {
            calls: calls.clone(),
            mode,
            chunks,
            action,
        }),
    );
    (registry, calls)
}
fn generation<'a>(
    task: &'a TaskInput,
    b: &'a Binding,
    cloud: Option<&'a CloudRequest>,
) -> Generation<'a> {
    Generation {
        input: task,
        binding: b,
        projected_at_ns: 101,
        modulation: &DisabledModulation,
        cloud: cloud.map(|c| (c, &Reviewer as &dyn Governance)),
        limits: LIMITS,
    }
}

#[test]
fn sys14_failed_explanation_retains_reason_and_late_success_never_generates_or_registers() {
    let t = input();
    let mode = SpeechMode::B_REMOTE_PROJECTED;
    let (mut adapters, calls) = setup(mode, true, None);
    let reason = FailureReason {
        schema_version: 1,
        code: FailureCode::IRL_FAILED,
        detail_ref: Some(object("failure audit")),
    };
    let mut turn = ReplyTurn::new(material(&t).latest, mode);
    let mut outbox = TestOutbox::default();
    let b = binding(&t);
    for progress in [
        Interpretation::Pending,
        Interpretation::Failed(reason.clone()),
        Interpretation::Committed,
    ] {
        let result = turn
            .advance(&material(&t).latest, progress, &mut adapters, || {
                panic!("failed turn prepared")
            })
            .unwrap();
        if let Some(plan) = result {
            plan.submit(&b, &[], &mut outbox).unwrap()
        }
    }
    assert_eq!(
        turn.state().fault_receipt().unwrap().reasons,
        vec![reason.clone()]
    );
    assert_eq!(turn.state(), &ReplyState::Failed(reason));
    assert_eq!((calls.get(), outbox.chunks), (0, 0));
}
#[test]
fn sys14_valid_no_effect_generates_after_real_wait_longer_than_generation_budget() {
    let mut t = input();
    t.max_duration_ns = DecimalU64(100_000_000);
    let cloud = prepare(&t, material(&t), &Codec, LIMITS).unwrap();
    let b = binding(&t);
    let mode = SpeechMode::B_REMOTE_PROJECTED;
    let (mut adapters, calls) = setup(mode, true, None);
    let mut turn = ReplyTurn::new(material(&t).latest, mode);
    let waiting = Instant::now();
    assert!(
        turn.advance(
            &material(&t).latest,
            Interpretation::Pending,
            &mut adapters,
            || panic!("waiting")
        )
        .unwrap()
        .is_none()
    );
    std::thread::sleep(Duration::from_millis(150));
    let waited = waiting.elapsed();
    assert!(waited > Duration::from_nanos(t.max_duration_ns.0));
    let started = Instant::now();
    let plan = turn
        .advance(
            &material(&t).latest,
            Interpretation::Committed,
            &mut adapters,
            || Ok(generation(&t, &b, Some(&cloud))),
        )
        .unwrap()
        .unwrap();
    let mut outbox = TestOutbox::default();
    plan.submit(&b, &[], &mut outbox).unwrap();
    println!(
        "SYS14 task path wait_us={} generation_and_registration_us={} generated={} registered_chunks={}",
        waited.as_micros(),
        started.elapsed().as_micros(),
        calls.get(),
        outbox.chunks
    );
    assert_eq!((calls.get(), outbox.chunks), (1, 1));
}
#[test]
fn sys14_permitted_zero_preserves_adapter_usage_and_reason() {
    let t = input();
    let b = binding(&t);
    let mode = SpeechMode::A_LOCAL_STRICT;
    let (mut adapters, _) = setup(mode, true, None);
    let mut turn = ReplyTurn::new(material(&t).latest, mode);
    let plan = turn
        .advance(
            &material(&t).latest,
            Interpretation::Committed,
            &mut adapters,
            || Ok(generation(&t, &b, None)),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        plan.zero_reason().unwrap().code,
        FailureCode::MODULATION_DISABLED
    );
    assert_eq!(plan.receipt().usage, object("one output token"));
    assert!(!plan.receipt().strict);
}
#[test]
fn declined_action_out_of_scope_and_new_observation_do_not_register() {
    let t = input();
    let b = binding(&t);
    let mode = SpeechMode::A_LOCAL_STRICT;
    let action = ActionPlan {
        schema_version: 1,
        plan_id: "plan".into(),
        snapshot_id: "s".into(),
        task_version: "1".into(),
        protocol_version: "fixture-p".into(),
        output_or_tool_parameters: object("move"),
        expected_scope: object("body"),
        dependencies: vec![],
        deadline_ns: core_types::scalar::DecimalI64(200),
    };
    for (proposed, fresh, permitted, want) in [
        (false, true, true, 0),
        (true, true, false, 0),
        (true, false, true, 0),
        (true, true, true, 1),
    ] {
        let (mut adapters, _) = setup(mode, false, proposed.then(|| action.clone()));
        let mut turn = ReplyTurn::new(material(&t).latest, mode);
        let plan = turn
            .advance(
                &material(&t).latest,
                Interpretation::Committed,
                &mut adapters,
                || Ok(generation(&t, &b, None)),
            )
            .unwrap()
            .unwrap();
        let mut current = b.clone();
        if !fresh {
            current.frontier.push("new observation".into())
        }
        let mut outbox = TestOutbox::default();
        let allowed = if permitted {
            vec![object("body")]
        } else {
            vec![]
        };
        assert_eq!(
            plan.submit(&current, &allowed, &mut outbox).is_ok(),
            fresh && (!proposed || permitted)
        );
        assert_eq!(outbox.calls, want);
    }
}
