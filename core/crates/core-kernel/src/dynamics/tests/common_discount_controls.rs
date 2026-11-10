//! ADR 0035 §9 P5-69(7): real C through the existing pure control engine.
//! Synthetic committed projections only; no production H/L transaction or SYS17.
use super::*;
use crate::recompute::{
    self as r, Audit, AuditKind, Command, Commit, Kernel, RebuildReason, Scope,
};

#[derive(Clone, Debug, PartialEq)]
struct Recorded {
    reaction: ReactionEvent,
    excluded_by_correction: bool,
}

struct C {
    old: DynamicsLaw,
    new: DynamicsLaw,
}
impl C {
    fn fixture() -> Self {
        let mut old = law();
        old.version = "synthetic-legacy-controls".into();
        old.baseline_prior = [0.25; 4];
        old.baseline_weight = 4.0;
        let mut new = old.clone();
        new.version = "synthetic-D07-A-controls".into();
        Self { old, new }
    }
    fn scope(&self, new: bool) -> Scope {
        Scope {
            law: if new { &self.new } else { &self.old }.version.clone(),
            interpretation: "recorded-unmodulated-a".into(),
            sources: "original".into(),
        }
    }
}
impl Kernel<Recorded> for C {
    type State = DynamicsState;
    type Input = ReactionEvent;
    fn same_event(&self, a: &Recorded, b: &Recorded) -> bool {
        a == b && format!("{a:?}") == format!("{b:?}")
    }
    fn same_input(&self, a: &ReactionEvent, b: &ReactionEvent) -> bool {
        a == b && format!("{a:?}") == format!("{b:?}")
    }
    fn genesis(&self, scope: &Scope) -> r::Result<DynamicsState> {
        let state = if scope.law == self.old.version {
            DynamicsState::genesis(self.old.clone())
        } else if scope.law == self.new.version {
            DynamicsState::genesis_common_discount(self.new.clone())
        } else {
            return Err(r::Fault::MissingInput(scope.law.clone()));
        };
        state.map_err(|e| r::Fault::Computation(format!("{e:?}")))
    }
    fn project(
        &self,
        history: &[Commit<Recorded>],
        scope: &Scope,
    ) -> r::Result<Vec<ReactionEvent>> {
        if scope.interpretation != "recorded-unmodulated-a"
            || !["original", "corrected"].contains(&scope.sources.as_str())
        {
            return Err(r::Fault::MissingInput(
                "synthetic projection resource".into(),
            ));
        }
        Ok(history
            .iter()
            .filter_map(|c| match &c.command {
                Command::Event(e)
                    if !(scope.sources == "corrected" && e.excluded_by_correction) =>
                {
                    Some(e.reaction.clone())
                }
                _ => None,
            })
            .collect())
    }
    fn advance(
        &self,
        state: &DynamicsState,
        input: &ReactionEvent,
        _: &Scope,
    ) -> r::Result<DynamicsState> {
        stage(state, Some(input), &state.law, &fuses())
            .map(StagedDynamics::into_state)
            .map_err(|e| r::Fault::Computation(format!("{e:?}")))
    }
    fn same_bits(&self, a: &DynamicsState, b: &DynamicsState) -> bool {
        // All fields participate in derived equality; finite round-trip Debug
        // distinguishes signed zeros too. Explicit P bits guard the new state.
        a.prior_mass.map(f64::to_bits) == b.prior_mass.map(f64::to_bits)
            && a == b
            && format!("{a:?}") == format!("{b:?}")
    }
}

fn commit(position: u64, command: Command<Recorded>) -> Commit<Recorded> {
    Commit {
        id: format!("synthetic-commit-{position}"),
        position,
        prefix: format!("synthetic-prefix-{position}"),
        command,
        check: None,
    }
}
fn history() -> Vec<Commit<Recorded>> {
    (1..=5)
        .map(|i| {
            commit(
                i * 2,
                Command::Event(Recorded {
                    reaction: event(i * 2, if i <= 2 { 0.75 } else { -0.5 }),
                    excluded_by_correction: i == 1,
                }),
            )
        })
        .collect()
}
fn direct(kernel: &C, scope: &Scope, h: &[Commit<Recorded>]) -> DynamicsState {
    // Independent of the control engine and its projection/cache path.
    h.iter()
        .fold(kernel.genesis(scope).unwrap(), |s, c| match &c.command {
            Command::Event(e) => advance(&s, &e.reaction),
            _ => s,
        })
}
fn audit_eq(a: &Audit<DynamicsState>, b: &Audit<DynamicsState>) {
    // Full finite state + all audit metadata, including future added fields.
    identical(&a.before, &b.before);
    identical(&a.after, &b.after);
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
}
fn path(kernel: &C, initial: Scope, h: &[Commit<Recorded>]) -> r::Progress<Recorded, C> {
    let mut incremental = r::start(kernel, "synthetic-D07-lineage", initial.clone()).unwrap();
    for end in 1..=h.len() {
        incremental = r::step(kernel, &incremental, &h[..end]).unwrap();
        let whole = r::replay(kernel, "synthetic-D07-lineage", initial.clone(), &h[..end]).unwrap();
        identical(incremental.state(), whole.state());
        assert_eq!(incremental.paused(), whole.paused());
        assert_eq!(incremental.needs_recompute(), whole.needs_recompute());
        assert_eq!(incremental.target_scope(), whole.target_scope());
        assert_eq!(incremental.applied_scope(), whole.applied_scope());
        assert_eq!(incremental.unresolved_fault(), whole.unresolved_fault());
        assert_eq!(incremental.audits().len(), end);
        assert_eq!(whole.audits().len(), end);
        for (a, b) in incremental.audits().iter().zip(whole.audits()) {
            audit_eq(a, b);
        }
    }
    incremental
}
fn audit(
    a: &Audit<DynamicsState>,
    entry: &Commit<Recorded>,
    kind: AuditKind,
    scopes: (&Scope, &Scope),
    states: (&DynamicsState, &DynamicsState),
    changed: bool,
) {
    assert_eq!(a.kind, kind);
    assert_eq!(a.binding.lineage, "synthetic-D07-lineage");
    assert_eq!(a.binding.prefix, entry.prefix);
    assert_eq!(&a.binding.scope, scopes.1);
    assert_eq!(&a.before_scope, scopes.0);
    assert_eq!(&a.after_scope, scopes.1);
    identical(&a.before, states.0);
    identical(&a.after, states.1);
    assert_eq!(a.changed, changed);
    assert!(a.check.is_none() && a.unresolved_fault.is_none());
    assert!(!a.notify_developer_and_caretaker);
    println!("D07-control-audit {a:?}");
}

#[test]
fn ac2_08_18_d07_pause_records_history_resume_catches_up_with_bound_audits() {
    // ADR 0035 §9(7), I23: controls preserve all state while H keeps growing.
    let k = C::fixture();
    let scope = k.scope(true);
    let plain = history();
    let frozen = direct(&k, &scope, &plain[..2]);
    let expected = direct(&k, &scope, &plain);
    let mut h = plain.clone();
    h.insert(2, commit(5, Command::Pause));
    let paused = path(&k, scope.clone(), &h);
    assert!(paused.paused() && paused.needs_recompute());
    identical(paused.state(), &frozen);
    audit(
        &paused.audits()[2],
        &h[2],
        AuditKind::Pause,
        (&scope, &scope),
        (&frozen, &frozen),
        false,
    );
    for (a, c) in paused.audits()[3..].iter().zip(&h[3..]) {
        audit(
            a,
            c,
            AuditKind::Deferred,
            (&scope, &scope),
            (&frozen, &frozen),
            false,
        );
    }
    h.push(commit(11, Command::Resume));
    let resumed = path(&k, scope.clone(), &h);
    assert!(!resumed.paused() && !resumed.needs_recompute());
    identical(resumed.state(), &expected);
    assert_eq!(expected.event_count, 5);
    assert!(!k.same_bits(&frozen, &expected));
    audit(
        resumed.audits().last().unwrap(),
        h.last().unwrap(),
        AuditKind::Resume,
        (&scope, &scope),
        (&frozen, &expected),
        true,
    );
}

#[test]
fn ac2_19_d07_source_correction_rebuilds_from_history_with_bound_audit() {
    // ADR 0035 §9(7): raw H is immutable; a versioned source projection excludes e1.
    let k = C::fixture();
    let original = k.scope(true);
    let mut corrected = original.clone();
    corrected.sources = "corrected".into();
    let plain = history();
    let before = direct(&k, &original, &plain);
    let absent = direct(&k, &original, &plain[1..]);
    let mut h = plain.clone();
    h.push(commit(
        11,
        Command::Rebuild {
            scope: corrected.clone(),
            reason: RebuildReason::SourceCorrection,
        },
    ));
    let result = path(&k, original.clone(), &h);
    assert_eq!(&h[..plain.len()], plain);
    identical(result.state(), &absent);
    assert_eq!(result.state().event_count, 4);
    assert!(!k.same_bits(&before, &absent));
    audit(
        result.audits().last().unwrap(),
        h.last().unwrap(),
        AuditKind::Rebuild(RebuildReason::SourceCorrection),
        (&original, &corrected),
        (&before, &absent),
        true,
    );
}

#[test]
fn ac2_20_d07_law_rebuild_uses_new_genesis_complete_history_and_audit() {
    // ADR 0035 §9(7): real legacy -> D07-A, not a new-law fold relabelled as migration.
    let k = C::fixture();
    let old = k.scope(false);
    let new = k.scope(true);
    let plain = history();
    let before = direct(&k, &old, &plain);
    let expected = direct(&k, &new, &plain);
    let mut h = plain.clone();
    h.push(commit(
        11,
        Command::Rebuild {
            scope: new.clone(),
            reason: RebuildReason::LawVersion,
        },
    ));
    let result = path(&k, old.clone(), &h);
    assert_eq!(&h[..plain.len()], plain);
    identical(result.state(), &expected);
    assert!(before.prior_mass.is_none());
    assert!(
        expected
            .prior_mass
            .is_some_and(|p| p > 0.0 && p < k.new.baseline_weight)
    );
    assert_eq!(expected.event_count, 5);
    assert_ne!(before.values.baseline, expected.values.baseline);
    // Wrong old genesis, omitted history and omitted/mutated P cannot pass the oracle.
    assert!(!k.same_bits(&before, &expected));
    assert!(!k.same_bits(&direct(&k, &new, &plain[1..]), &expected));
    let mut missing_p = expected.clone();
    missing_p.prior_mass = None;
    assert!(!k.same_bits(&missing_p, &expected));
    audit(
        result.audits().last().unwrap(),
        h.last().unwrap(),
        AuditKind::Rebuild(RebuildReason::LawVersion),
        (&old, &new),
        (&before, &expected),
        true,
    );
}
