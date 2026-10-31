//! Review R1 / v2.0 §6: test the actual F input path, not a second R projection.
mod common;
use common::{adapters::*, *};
use core_runtime::stages::*;
use core_store::Experiment;
use core_types::{generated::*, semantic::nature};
use std::{cell::RefCell, rc::Rc};

struct FullMatch;
impl Match for FullMatch {
    fn match_event(&mut self, input: Matching<'_>) -> StageResult<MatchOutput> {
        let mut record: SituationMatchRecord = fixture();
        record.event_id = input.binding.event_id.clone();
        record.binding = input.binding.clone();
        record.matches = input
            .concepts
            .iter()
            .map(|(id, _)| TendencyMatch {
                schema_version: 1,
                tendency_id: id.clone(),
                mu: 1.0,
            })
            .collect();
        Ok(MatchOutput {
            record,
            candidates: vec![],
        })
    }
}

// Only the learned tendency contribution, not a production F implementation.
// No hidden R copy: every numerical input comes through Pulse.
struct MagnitudeF {
    inner: F,
    drives: Rc<RefCell<Vec<[f64; 4]>>>,
}
impl Fast for MagnitudeF {
    fn project(&self, input: Projection<'_>, ns: u64) -> StageResult<FastStateSnapshotV2> {
        self.inner.project(input, ns)
    }
    fn pulse(&self, input: Pulse<'_>) -> StageResult<FastStateSnapshotV2> {
        assert_eq!(input.active.len(), 1);
        let tendency = &input.active[0];
        let matches = &input.matching.unwrap().matches;
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].tendency_id, tendency.tendency_id);
        let amplitude = tendency.strength * matches[0].mu * tendency.typical_magnitude;
        let drive = nature(&tendency.direction).map(|d| amplitude * d);
        self.drives.borrow_mut().push(drive);
        let mut x = input.projected.clone();
        x.nature.connection += drive[0];
        Ok(x)
    }
    fn finish(
        &self,
        active: &[ActiveTendency],
        x: &FastStateSnapshotV2,
    ) -> StageResult<FastStateSnapshotV2> {
        self.inner.finish(active, x)
    }
}

/// Same pi=.75, mu=1 and direction: only r changes, producing .05 then .4.
#[test]
fn review_r1_f_receives_magnitude_and_distinguishes_the_review_counterexample() {
    let mut case = Case::new();
    let mut tendency: TendencyV3 = fixture();
    tendency.pi = 0.75;
    tendency.direction.connection = 1.0;
    tendency.direction.empathy = 0.0;
    tendency.direction.curiosity = 0.0;
    tendency.direction.achievement = 0.0;
    case.base.r.tendencies = vec![tendency];
    let drives = Rc::new(RefCell::new(Vec::new()));
    let log = Log::default();
    let mut components = parts(&log);
    components.m = Some(Box::new(FullMatch));
    components.f = Some(Box::new(MagnitudeF {
        inner: F(log),
        drives: drives.clone(),
    }));
    let mut stages =
        StageSet::<Experiment>::complete(components, case.binding.environment.clone(), limits())
            .unwrap();
    for (magnitude, expected) in [(0.1, 0.05), (0.8, 0.4)] {
        case.base.r.tendencies[0].typical_magnitude = magnitude;
        let staged = stages.process_head(case.input(true)).unwrap();
        assert_eq!(staged.snapshot().x.nature.connection, 0.1 + expected);
        assert_eq!(staged.snapshot().r, case.base.r);
        assert_eq!(staged.snapshot().theta, case.base.theta);
    }
    assert_eq!(
        *drives.borrow(),
        vec![[0.05, 0.0, 0.0, 0.0], [0.4, 0.0, 0.0, 0.0]]
    );
}
