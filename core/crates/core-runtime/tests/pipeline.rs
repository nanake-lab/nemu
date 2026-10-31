mod common;
use common::adapters::*;
use common::*;
use core_runtime::{
    Result,
    binding::Bound,
    pipeline::{ORDER, Step, verify_order},
    stages::*,
    status::Status,
    store::{CarrierUpdate, update_carrier},
};
use core_store::{Experiment, Life, Rehearsal};
use core_types::generated::*;

#[test]
fn x06_positive_nonidentity_g_c_fout_fixed_order_and_no_staged_j() {
    let case = Case::new();
    let log = Log::default();
    let mut stages =
        StageSet::<Experiment>::complete(parts(&log), case.binding.environment.clone(), limits())
            .unwrap();
    let staged = stages.process_head(case.input(false)).unwrap();
    assert_eq!(staged.trace(), &ORDER[..11]);
    assert_eq!(
        *log.borrow(),
        vec![
            Step::Phi,
            Step::Irl,
            Step::A,
            Step::M,
            Step::FIn,
            Step::G,
            Step::C,
            Step::FOut
        ]
    );
    assert_eq!(staged.snapshot().r.revision, "after-g");
    assert_eq!(staged.snapshot().theta.values.baseline.connection, 0.3);
    assert_eq!(staged.snapshot().x.nature.connection, 0.2);
    assert_eq!(staged.snapshot().x.learned[0].tendency_id, "new");
    assert_eq!(staged.snapshot().x.learned[0].value, 0.0);
    let mut wrong = ORDER;
    wrong.swap(7, 8);
    assert!(verify_order(&wrong).is_err());
    let mut repeated = ORDER.to_vec();
    repeated.insert(2, Step::Phi);
    assert!(verify_order(&repeated).is_err());
}
#[test]
fn rehearsal_missing_stages_is_explicit_and_all_other_lineages_require_every_stage() {
    let case = Case::new();
    assert!(
        StageSet::<Life>::complete(
            Components::default(),
            case.binding.environment.clone(),
            limits()
        )
        .is_err()
    );
    assert!(
        StageSet::<Experiment>::complete(
            Components::default(),
            case.binding.environment.clone(),
            limits()
        )
        .is_err()
    );
    for index in 0..7 {
        let mut p = parts(&Log::default());
        match index {
            0 => p.irl = None,
            1 => p.a = None,
            2 => p.m = None,
            3 => p.f = None,
            4 => p.g = None,
            5 => p.c = None,
            _ => p.j = None,
        };
        assert!(StageSet::<Life>::complete(p, case.binding.environment.clone(), limits()).is_err());
    }
    let mut stages = StageSet::<Rehearsal>::rehearsal(
        Components::default(),
        case.binding.environment.clone(),
        limits(),
    )
    .unwrap();
    let staged = stages.process_head(case.input(false)).unwrap();
    let status = staged.status().value();
    assert!(status.states.contains(&RunStatus::NO_GROWTH));
    assert!(status.states.contains(&RunStatus::IRL_DEGRADED));
    assert!(status.states.contains(&RunStatus::MODULATION_NOT_ENABLED));
    assert!(!staged.status().autonomy());
    assert_eq!(staged.snapshot().r, case.base.r);
    assert_eq!(staged.snapshot().theta, case.base.theta);
}
#[test]
fn paused_growth_still_records_a_m_and_maintenance_preserves_r_theta() {
    let case = Case::new();
    let log = Log::default();
    let mut stages =
        StageSet::<Experiment>::complete(parts(&log), case.binding.environment.clone(), limits())
            .unwrap();
    let staged = stages.process_head(case.input(true)).unwrap();
    assert!(log.borrow().contains(&Step::M));
    assert!(!log.borrow().contains(&Step::G));
    assert!(!log.borrow().contains(&Step::C));
    assert_eq!(staged.snapshot().r, case.base.r);
    assert_eq!(staged.snapshot().theta, case.base.theta);
    assert!(
        staged
            .status()
            .value()
            .states
            .contains(&RunStatus::GROWTH_PAUSED)
    );
    let mut maintenance_base = case.base.clone();
    maintenance_base.r.tendencies.push(fixture());
    maintenance_base.r.tendencies[0].evidence_a = -0.0;
    maintenance_base.theta.values.baseline.connection = -0.0;
    for t in [30, 100, 9000] {
        let view = stages.maintenance(&maintenance_base, &time(t)).unwrap();
        assert_eq!(
            core_runtime::binding::digest(&view.r, limits()).unwrap(),
            core_runtime::binding::digest(&maintenance_base.r, limits()).unwrap()
        );
        assert_eq!(view.theta, maintenance_base.theta);
        assert_eq!(theta_bits(&view.theta), theta_bits(&maintenance_base.theta));
        assert_eq!(r_bits(&view.r), r_bits(&maintenance_base.r));
    }
    assert_eq!(case.base.x.anchor_time_ns.0, 10);
}
#[test]
fn d11_old_occurrence_and_queue_delay_do_not_filter_pulse() {
    let mut case = Case::new();
    case.event.event_time_ns.0 = -1_000_000;
    case.event.ingest_time_ns.0 = 2;
    let log = Log::default();
    let mut stages =
        StageSet::<Rehearsal>::rehearsal(parts(&log), case.binding.environment.clone(), limits())
            .unwrap();
    let staged = stages.process_head(case.input(false)).unwrap();
    assert!(log.borrow().contains(&Step::FIn));
    assert_eq!(staged.times().occurred_ns, -1_000_000);
    assert_eq!(staged.times().received_ns, 2);
    assert_eq!(staged.times().process_ns, 30);
    assert_eq!(staged.snapshot().x.nature.connection, 0.2);
    assert!(!staged.status().autonomy());
}
#[test]
fn x02_same_head_different_time_interpreter_or_payload_binding_is_stale() {
    let case = Case::new();
    for index in 0..4 {
        let mut b = case.binding.clone();
        match index {
            0 => b.process_time_ns.0 += 1,
            1 => b.environment.irl.resource_version.push('x'),
            2 => b.proposal_payload_digest.0[0] ^= 1,
            _ => b.c_core_digest.0[0] ^= 1,
        };
        assert!(
            Bound {
                binding: b,
                value: ()
            }
            .check(&case.binding, limits())
            .is_err()
        );
    }
    let mut stale = Case::new();
    stale.binding.process_time_ns.0 += 1;
    let mut stages = StageSet::<Rehearsal>::rehearsal(
        Components::default(),
        stale.binding.environment.clone(),
        limits(),
    )
    .unwrap();
    assert!(stages.process_head(stale.input(false)).is_err());
}
#[test]
fn status_intersection_never_clears_storage_or_carrier_fault() {
    let mut status = Status::default();
    status.add(
        RunStatus::STORAGE_RECOVERY_FAULT,
        FailureCode::STRUCTURAL_FAULT,
    );
    status.add(RunStatus::IRL_DEGRADED, FailureCode::IRL_FAILED);
    status.add(
        RunStatus::CORE_UNMODULATED,
        FailureCode::CARRIER_UNAVAILABLE,
    );
    assert!(!status.autonomy());
    assert_eq!(status.value().states.len(), 3);
    assert!(
        status
            .terminal(Lineage::EXPERIMENT, SpeechMode::A_LOCAL_STRICT, false, None)
            .is_none()
    );
}
#[test]
fn adr0031_every_m_update_clears_installs_prefills_and_stops_at_failure() {
    struct Carrier {
        log: Vec<u8>,
        fail: Option<u8>,
    }
    impl Carrier {
        fn step(&mut self, n: u8) -> Result<()> {
            self.log.push(n);
            if self.fail == Some(n) {
                Err(core_runtime::Error::Rejected("fixture failure"))
            } else {
                Ok(())
            }
        }
    }
    impl CarrierUpdate for Carrier {
        fn clear_kv(&mut self) -> Result<()> {
            self.step(1)
        }
        fn install(&mut self, _: &ModulationEnvelopeV2) -> Result<()> {
            self.step(2)
        }
        fn prefill(&mut self) -> Result<()> {
            self.step(3)
        }
    }
    let envelope = fixture();
    let mut c = Carrier {
        log: vec![],
        fail: None,
    };
    update_carrier(&mut c, &envelope, limits()).unwrap();
    update_carrier(&mut c, &envelope, limits()).unwrap();
    assert_eq!(c.log, vec![1, 2, 3, 1, 2, 3]);
    for fail in 1..=3 {
        let mut c = Carrier {
            log: vec![],
            fail: Some(fail),
        };
        assert!(update_carrier(&mut c, &envelope, limits()).is_err());
        assert_eq!(c.log, (1..=fail).collect::<Vec<_>>());
    }
}
