use super::fixture;
use core_runtime::{binding::Bound, pipeline::Step, stages::*};
use core_types::{generated::*, scalar::DecimalU64};
use std::{cell::RefCell, rc::Rc};
pub type Log = Rc<RefCell<Vec<Step>>>;
pub struct Irl(pub Log);
impl Interpret for Irl {
    fn interpret(&mut self, i: Interpretation<'_>) -> StageResult<Bound<SemanticOutput>> {
        self.0.borrow_mut().push(Step::Irl);
        Ok(Bound {
            binding: i.binding.clone(),
            value: SemanticOutput {
                graph: fixture(),
                concepts: vec![],
            },
        })
    }
}
pub struct A(pub Log);
impl Appraise for A {
    fn appraise(&mut self, i: Appraisal<'_>) -> StageResult<NatureAppraisalRecord> {
        self.0.borrow_mut().push(Step::A);
        let mut a: NatureAppraisalRecord = fixture();
        a.binding = i.binding.clone();
        a.event_id = i.binding.event_id.clone();
        Ok(a)
    }
}
pub struct M(pub Log);
impl Match for M {
    fn match_event(&mut self, i: Matching<'_>) -> StageResult<MatchOutput> {
        self.0.borrow_mut().push(Step::M);
        assert!(i.concepts.is_empty());
        let mut record: SituationMatchRecord = fixture();
        record.binding = i.binding.clone();
        record.event_id = i.binding.event_id.clone();
        Ok(MatchOutput {
            record,
            candidates: vec![],
        })
    }
}
pub struct F(pub Log);
impl Fast for F {
    fn project(&self, s: Projection<'_>, t: u64) -> StageResult<FastStateSnapshotV2> {
        self.0.borrow_mut().push(Step::Phi);
        let mut x = s.x.clone();
        x.anchor_time_ns = DecimalU64(t);
        Ok(x)
    }
    fn pulse(&self, i: Pulse<'_>) -> StageResult<FastStateSnapshotV2> {
        self.0.borrow_mut().push(Step::FIn);
        let mut x = i.projected.clone();
        if i.appraisal.is_some() {
            x.nature.connection += 0.1;
        }
        Ok(x)
    }
    fn finish(
        &self,
        active: &[ActiveTendency],
        x: &FastStateSnapshotV2,
    ) -> StageResult<FastStateSnapshotV2> {
        self.0.borrow_mut().push(Step::FOut);
        let mut next = x.clone();
        next.learned = active
            .iter()
            .map(|t| LearnedComponent {
                schema_version: 1,
                tendency_id: t.tendency_id.clone(),
                value: x
                    .learned
                    .iter()
                    .find(|v| v.tendency_id == t.tendency_id)
                    .map(|v| v.value)
                    .unwrap_or(0.0),
            })
            .collect();
        Ok(next)
    }
}
pub struct G(pub Log);
impl Learn for G {
    fn learn(&mut self, i: Learning<'_>) -> StageResult<Bound<Change<RSnapshotV3>>> {
        self.0.borrow_mut().push(Step::G);
        assert_eq!(i.baseline.connection, 0.0);
        let mut r = i.r.clone();
        r.revision = "after-g".into();
        let mut t: TendencyV3 = fixture();
        t.tendency_id = "new".into();
        t.pi = 0.75;
        r.tendencies.push(t);
        let mut entry: LedgerEntry = fixture();
        entry.entry_id = "g-change".into();
        entry.event_id = i.event.id.clone();
        entry.tendency_change.as_mut().unwrap().tendency_id = "new".into();
        Ok(Bound {
            binding: i.binding.clone(),
            value: Change {
                value: r,
                ledger: vec![entry],
            },
        })
    }
}
pub struct C(pub Log);
impl Dynamics for C {
    fn update(&mut self, i: DynamicsInput<'_>) -> StageResult<Bound<Change<ThetaSnapshot>>> {
        self.0.borrow_mut().push(Step::C);
        let mut theta = i.theta.clone();
        theta.values.baseline.connection = 0.3;
        let mut entry: LedgerEntry = fixture();
        entry.entry_id = "c-change".into();
        entry.event_id = i.event.id.clone();
        entry.kind = LedgerKind::LEDGER_THETA_CHANGED;
        entry.tendency_change = None;
        entry.theta_change = Some(ThetaChange {
            schema_version: 1,
            before: i.theta.values.clone(),
            after: theta.values.clone(),
            events: vec![i.event.id.clone()],
        });
        Ok(Bound {
            binding: i.binding.clone(),
            value: Change {
                value: theta,
                ledger: vec![entry],
            },
        })
    }
}
pub struct J;
impl Compile for J {
    fn compile(&self, _: JInput<'_>) -> StageResult<ModulationEnvelopeV2> {
        panic!("J cannot see staged state")
    }
}
pub fn parts(log: &Log) -> Components {
    Components {
        irl: Some(Box::new(Irl(log.clone()))),
        a: Some(Box::new(A(log.clone()))),
        m: Some(Box::new(M(log.clone()))),
        f: Some(Box::new(F(log.clone()))),
        g: Some(Box::new(G(log.clone()))),
        c: Some(Box::new(C(log.clone()))),
        j: Some(Box::new(J)),
    }
}
