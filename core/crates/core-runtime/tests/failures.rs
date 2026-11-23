mod common;
use common::{adapters::*, *};
use core_runtime::{binding::Bound, pipeline::Step, stages::*, store::validate_compilation};
use core_store::Experiment;
use core_types::{generated::*, scalar::DecimalU64};

struct Unavailable;
impl Interpret for Unavailable {
    fn interpret(&mut self, _: Interpretation<'_>) -> StageResult<Bound<SemanticOutput>> {
        Err(FailureCode::IRL_FAILED)
    }
}

struct InvalidA;
struct InvalidIrl;
impl Interpret for InvalidIrl {
    fn interpret(&mut self, i: Interpretation<'_>) -> StageResult<Bound<SemanticOutput>> {
        let mut graph: SemanticGraph = fixture();
        graph.schema_version = 99;
        Ok(Bound {
            binding: i.binding.clone(),
            value: SemanticOutput {
                graph,
                concepts: vec![],
            },
        })
    }
}
#[test]
fn invalid_semantic_output_degrades_with_no_drive() {
    let c = Case::new();
    let log = Log::default();
    let mut p = parts(&log);
    p.irl = Some(Box::new(InvalidIrl));
    let mut stages =
        StageSet::<Experiment>::complete(p, c.binding.environment.clone(), limits()).unwrap();
    let result = stages.process_head(c.input(false)).unwrap();
    assert!(
        result
            .status()
            .value()
            .states
            .contains(&RunStatus::IRL_DEGRADED)
    );
    assert_eq!(result.snapshot().x, c.x_with_anchor(30));
    assert_eq!(result.snapshot().r, c.base.r);
    assert_eq!(result.snapshot().theta, c.base.theta);
    assert!(!log.borrow().contains(&Step::FIn));
}
impl Appraise for InvalidA {
    fn appraise(&mut self, i: Appraisal<'_>) -> StageResult<NatureAppraisalRecord> {
        let mut a: NatureAppraisalRecord = fixture();
        a.binding = i.binding.clone();
        a.event_id = i.binding.event_id.clone();
        a.appraisal.connection = f64::NAN;
        Ok(a)
    }
}
struct InvalidM;
impl Match for InvalidM {
    fn match_event(&mut self, i: Matching<'_>) -> StageResult<MatchOutput> {
        let mut r: SituationMatchRecord = fixture();
        r.binding = i.binding.clone();
        r.event_id = i.binding.event_id.clone();
        r.matches = vec![TendencyMatch {
            schema_version: 1,
            tendency_id: "bad".into(),
            mu: 2.0,
        }];
        Ok(MatchOutput {
            record: r,
            candidates: vec![],
        })
    }
}
#[test]
fn invalid_appraisal_or_match_degrades_without_poisoning_facts() {
    for index in 0..2 {
        let c = Case::new();
        let mut p = parts(&Log::default());
        if index == 0 {
            p.a = Some(Box::new(InvalidA))
        } else {
            p.m = Some(Box::new(InvalidM))
        };
        let mut stages =
            StageSet::<Experiment>::complete(p, c.binding.environment.clone(), limits()).unwrap();
        let result = stages.process_head(c.input(false)).unwrap();
        assert_eq!(result.snapshot().r, c.base.r);
        let status = result.status().value();
        assert!(
            status
                .reasons
                .iter()
                .any(|r| r.code == FailureCode::STRUCTURAL_FAULT)
        );
        assert!(status.states.contains(&if index == 0 {
            RunStatus::A_DEGRADED
        } else {
            RunStatus::M_DEGRADED
        }));
    }
}
impl Appraise for Unavailable {
    fn appraise(&mut self, _: Appraisal<'_>) -> StageResult<NatureAppraisalRecord> {
        Err(FailureCode::DEPENDENCY_FAULT)
    }
}
impl Match for Unavailable {
    fn match_event(&mut self, _: Matching<'_>) -> StageResult<MatchOutput> {
        Err(FailureCode::RESOURCE_LIMIT)
    }
}
#[test]
fn sys14_irl_a_m_failures_have_distinct_drives_and_growth_paths() {
    for (index, state) in [
        (0, RunStatus::IRL_DEGRADED),
        (1, RunStatus::A_DEGRADED),
        (2, RunStatus::M_DEGRADED),
    ] {
        let c = Case::new();
        let log = Log::default();
        let mut p = parts(&log);
        match index {
            0 => p.irl = Some(Box::new(Unavailable)),
            1 => p.a = Some(Box::new(Unavailable)),
            _ => p.m = Some(Box::new(Unavailable)),
        };
        let mut stages =
            StageSet::<Experiment>::complete(p, c.binding.environment.clone(), limits()).unwrap();
        let staged = stages.process_head(c.input(false)).unwrap();
        let status = staged.status().value();
        assert!(status.states.contains(&state));
        assert_eq!(staged.snapshot().r, c.base.r);
        assert!(!log.borrow().contains(&Step::G));
        if index < 2 {
            assert_eq!(staged.snapshot().theta, c.base.theta);
            assert_eq!(staged.snapshot().x, c.x_with_anchor(30));
            assert!(!log.borrow().contains(&Step::C));
            assert!(!log.borrow().contains(&Step::FOut));
        } else {
            assert!(log.borrow().contains(&Step::C));
            assert_eq!(staged.snapshot().theta.values.baseline.connection, 0.3);
            assert_eq!(staged.snapshot().x.nature.connection, 0.2);
        }
        if index == 0 {
            assert!(!log.borrow().contains(&Step::FIn));
        }
    }
}
impl Case {
    fn x_with_anchor(&self, n: u64) -> FastStateSnapshotV2 {
        let mut x = self.base.x.clone();
        x.anchor_time_ns = DecimalU64(n);
        x
    }
}
#[test]
fn maintenance_events_and_self_narratives_are_not_theta_evidence() {
    for index in 0..3 {
        let mut c = Case::new();
        let log = Log::default();
        match index {
            0 => c.event.kind = EventKind::MAINTENANCE,
            1 => c.attributes.exogenous = false,
            _ => c.attributes.self_narrative = true,
        };
        let mut stages =
            StageSet::<Experiment>::complete(parts(&log), c.binding.environment.clone(), limits())
                .unwrap();
        let staged = stages.process_head(c.input(false)).unwrap();
        assert_eq!(staged.snapshot().theta, c.base.theta);
        assert!(!log.borrow().contains(&Step::C));
        if index == 0 {
            assert_eq!(staged.snapshot().r, c.base.r);
            assert!(!log.borrow().contains(&Step::FIn));
            assert!(!log.borrow().contains(&Step::G));
        }
    }
}
struct BadF {
    inner: F,
    second_clock: bool,
}
impl Fast for BadF {
    fn project(&self, v: Projection<'_>, t: u64) -> StageResult<FastStateSnapshotV2> {
        self.inner.project(v, t)
    }
    fn pulse(&self, p: Pulse<'_>) -> StageResult<FastStateSnapshotV2> {
        let mut x = self.inner.pulse(p)?;
        if self.second_clock {
            x.anchor_time_ns.0 += 1
        };
        Ok(x)
    }
    fn finish(
        &self,
        a: &[ActiveTendency],
        x: &FastStateSnapshotV2,
    ) -> StageResult<FastStateSnapshotV2> {
        let mut x = self.inner.finish(a, x)?;
        x.nature.connection += 0.1;
        Ok(x)
    }
}
#[test]
fn x06_rejects_second_clock_and_second_fout_pulse() {
    for second_clock in [true, false] {
        let c = Case::new();
        let log = Log::default();
        let mut p = parts(&log);
        p.f = Some(Box::new(BadF {
            inner: F(log),
            second_clock,
        }));
        let mut stages =
            StageSet::<Experiment>::complete(p, c.binding.environment.clone(), limits()).unwrap();
        assert!(stages.process_head(c.input(false)).is_err());
    }
}
#[test]
fn missing_versioned_stage_resource_refuses_startup() {
    for index in 0..4 {
        let c = Case::new();
        let mut e = c.binding.environment.clone();
        match index {
            0 => e.nature_appraiser = None,
            1 => e.situation_matcher = None,
            2 => e.direction_set = None,
            _ => e.law_version = None,
        };
        assert!(StageSet::<Experiment>::complete(parts(&Log::default()), e, limits()).is_err());
    }
}
#[test]
fn j_validation_binds_snapshot_time_state_and_resources() {
    let c = Case::new();
    let mut e: ModulationEnvelopeV2 = fixture();
    e.snapshot_id = "committed-fixture".into();
    e.projected_at_ns = c.base.x.anchor_time_ns;
    e.x_digest = core_runtime::binding::digest(&c.base.x, limits()).unwrap();
    e.r_digest = core_runtime::binding::digest(&c.base.r, limits()).unwrap();
    e.theta_digest = core_runtime::binding::digest(&c.base.theta, limits()).unwrap();
    let env = &c.binding.environment;
    e.carrier = env.carrier.clone();
    e.model = env.model.clone();
    e.tokenizer = env.tokenizer.clone();
    e.direction_set = env.direction_set.clone().unwrap();
    validate_compilation(&e, "committed-fixture", &c.base, 10, env, limits()).unwrap();
    for index in 0..4 {
        let mut bad = e.clone();
        match index {
            0 => bad.snapshot_id.push('x'),
            1 => bad.projected_at_ns.0 += 1,
            2 => bad.x_digest.0[0] ^= 1,
            _ => bad.direction_set.resource_version.push('x'),
        };
        assert!(
            validate_compilation(&bad, "committed-fixture", &c.base, 10, env, limits()).is_err()
        );
    }
}
