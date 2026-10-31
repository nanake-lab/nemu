use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Disposable acceleration state. Private fields prevent a caller from
/// replacing R/theta or the consumed input cursor. Reconstruct with `replay`;
/// never persist this cache as another authority beside H/R/theta/Q.
#[derive(Clone)]
pub struct Progress<E, K: Kernel<E>> {
    history: Vec<Commit<E>>,
    lineage: String,
    target: Scope,
    applied: Scope,
    state: K::State,
    inputs: Vec<K::Input>,
    paused: bool,
    dirty: bool,
    // V2-97(3): pending recomputation is not proof that a fault was repaired.
    unresolved_fault: Option<CheckRecord>,
    alarms: BTreeMap<String, Binding>,
    audits: Vec<Audit<K::State>>,
}

impl<E, K: Kernel<E>> Progress<E, K> {
    pub fn state(&self) -> &K::State {
        &self.state
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn target_scope(&self) -> &Scope {
        &self.target
    }
    pub fn applied_scope(&self) -> &Scope {
        &self.applied
    }
    pub fn audits(&self) -> &[Audit<K::State>] {
        &self.audits
    }
    pub fn needs_recompute(&self) -> bool {
        self.dirty
    }
    pub fn unresolved_fault(&self) -> Option<&CheckRecord> {
        self.unresolved_fault.as_ref()
    }
}

pub fn start<E, K: Kernel<E>>(kernel: &K, lineage: &str, scope: Scope) -> Result<Progress<E, K>> {
    scope.validate()?;
    if lineage.is_empty() {
        return Err(Fault::Binding("empty lineage"));
    }
    Ok(Progress {
        history: Vec::new(),
        lineage: lineage.into(),
        target: scope.clone(),
        applied: scope.clone(),
        state: kernel.genesis(&scope)?,
        inputs: Vec::new(),
        paused: false,
        dirty: false,
        unresolved_fault: None,
        alarms: BTreeMap::new(),
        audits: Vec::new(),
    })
}

/// Pure append of exactly one immutable commit. Err leaves `old` unchanged.
/// Calculation faults produce a rejection audit and preserve the *whole*
/// previously applied state until a bound repair explicitly rebuilds from H.
pub fn step<E: Clone, K: Kernel<E>>(
    kernel: &K,
    old: &Progress<E, K>,
    history: &[Commit<E>],
) -> Result<Progress<E, K>> {
    step_mode(kernel, old, history, false)
}

fn step_mode<E: Clone, K: Kernel<E>>(
    kernel: &K,
    old: &Progress<E, K>,
    history: &[Commit<E>],
    full: bool,
) -> Result<Progress<E, K>> {
    if history.len() != old.history.len() + 1
        || !old
            .history
            .iter()
            .zip(history)
            .all(|(a, b)| same_commit(kernel, a, b))
    {
        return Err(Fault::History("not an immutable one-commit extension"));
    }
    validate(history)?;
    let entry = history.last().expect("nonempty extension");
    let mut target = old.target.clone();
    let mut paused = old.paused;
    let mut force = old.dirty || full;
    let mut calculate = false;
    let mut repairing = false;
    let mut kind = match &entry.command {
        Command::Event(_) => {
            calculate = !paused;
            AuditKind::Advance
        }
        Command::Pause => {
            paused = true;
            AuditKind::Pause
        }
        Command::Resume => {
            paused = false;
            calculate = true;
            force = true;
            AuditKind::Resume
        }
        Command::Rebuild { scope, reason } => {
            scope.validate()?;
            validate_change(&target, scope, reason)?;
            if let RebuildReason::Repair { alarm_id, binding } = reason {
                // V2-97(3), V2-101 and AC2-18: only the accepted repair of the
                // current fault may release it; manual pause still prevents G/C.
                if paused
                    || !old.unresolved_fault.as_ref().is_some_and(|fault| {
                        &fault.alarm_id == alarm_id && &fault.binding == binding
                    })
                {
                    return Err(Fault::Binding(
                        "repair requires current fault and resumed growth",
                    ));
                }
                repairing = true;
            }
            target = scope.clone();
            force = true;
            calculate = !paused;
            AuditKind::Rebuild(reason.clone())
        }
        Command::Review {
            alarm_id,
            binding,
            verdict,
        } => {
            if old.alarms.get(alarm_id) != Some(binding) {
                return Err(Fault::Binding("review does not name its recorded alarm"));
            }
            AuditKind::Review {
                alarm_id: alarm_id.clone(),
                verdict: *verdict,
            }
        }
    };
    let binding = Binding {
        lineage: old.lineage.clone(),
        prefix: entry.prefix.clone(),
        scope: target.clone(),
    };
    if let Some(check) = &entry.check {
        if !calculate
            || check.binding != binding
            || check.alarm_id.is_empty()
            || old.alarms.contains_key(&check.alarm_id)
            || check.triggers.is_empty()
        {
            return Err(Fault::Binding("stale, duplicate or non-calculation check"));
        }
        for trigger in &check.triggers {
            if let Trigger::SlowDrift { direction, .. } = trigger
                && ![-1, 1].contains(direction)
            {
                return Err(Fault::InvalidInput("invalid drift trigger".into()));
            }
        }
    }
    let mut state = old.state.clone();
    let mut inputs = old.inputs.clone();
    let mut applied = old.applied.clone();
    let mut dirty = old.dirty;
    let mut unresolved_fault = old.unresolved_fault.clone();
    let rejection = entry.check.as_ref().and_then(|check| match &check.outcome {
        outcome @ (CheckOutcome::Inconsistent | CheckOutcome::ComputationFault(_)) => {
            Some(outcome.clone())
        }
        _ => None,
    });
    if calculate && unresolved_fault.is_some() && !repairing {
        // R1 / V2-97(3): keep collecting H, but neither a normal event, Resume,
        // resource change nor a fresh check can silently repair prior failure.
        kind = AuditKind::FaultHeld;
    } else if calculate {
        match candidate(kernel, old, history, &target, force) {
            Ok((value, consumed)) if rejection.is_none() => {
                state = value;
                inputs = consumed;
                applied = target.clone();
                dirty = false;
                unresolved_fault = None;
            }
            Ok(_) => {}
            Err(Fault::Computation(reason)) => {
                if !matches!(entry.check.as_ref().map(|c| &c.outcome),
                    Some(CheckOutcome::ComputationFault(recorded)) if *recorded == reason)
                {
                    // The caller must first record the fault, bound to this
                    // prefix, before a rejection can become replayable H.
                    return Err(Fault::Computation(reason));
                }
            }
            Err(error) => return Err(error),
        }
    } else if matches!(entry.command, Command::Event(_) | Command::Rebuild { .. }) {
        dirty = true;
        kind = AuditKind::Deferred;
    }
    if let Some(outcome) = rejection {
        kind = AuditKind::Rejected(outcome);
        dirty = true;
        unresolved_fault = entry.check.clone();
    }
    let notify = entry.check.is_some() || matches!(kind, AuditKind::Rejected(_));
    let audit = Audit {
        binding: binding.clone(),
        kind,
        before_scope: old.applied.clone(),
        after_scope: applied.clone(),
        before: old.state.clone(),
        after: state.clone(),
        changed: !kernel.same_bits(&old.state, &state),
        check: entry.check.clone(),
        unresolved_fault: unresolved_fault.clone(),
        notify_developer_and_caretaker: notify,
    };
    let mut alarms = old.alarms.clone();
    if let Some(check) = &entry.check {
        alarms.insert(check.alarm_id.clone(), binding);
    }
    let mut audits = old.audits.clone();
    audits.push(audit);
    Ok(Progress {
        history: history.to_vec(),
        lineage: old.lineage.clone(),
        target,
        applied,
        state,
        inputs,
        paused,
        dirty,
        unresolved_fault,
        alarms,
        audits,
    })
}

fn candidate<E: Clone, K: Kernel<E>>(
    kernel: &K,
    old: &Progress<E, K>,
    history: &[Commit<E>],
    scope: &Scope,
    force: bool,
) -> Result<(K::State, Vec<K::Input>)> {
    let inputs = kernel.project(history, scope)?;
    let incremental = !force
        && old.applied == *scope
        && inputs.len() >= old.inputs.len()
        && old
            .inputs
            .iter()
            .zip(&inputs)
            .all(|(a, b)| kernel.same_input(a, b));
    let mut state = if incremental {
        old.state.clone()
    } else {
        kernel.genesis(scope)?
    };
    let begin = if incremental { old.inputs.len() } else { 0 };
    for input in &inputs[begin..] {
        state = kernel.advance(&state, input, scope)?;
    }
    Ok((state, inputs))
}

/// Structural H replay. Uses the normative recurrence and historical checks;
/// it establishes I23, not numerical independence from that recurrence.
pub fn replay<E: Clone, K: Kernel<E>>(
    kernel: &K,
    lineage: &str,
    initial: Scope,
    history: &[Commit<E>],
) -> Result<Progress<E, K>> {
    let mut result = start(kernel, lineage, initial)?;
    for end in 1..=history.len() {
        result = step_mode(kernel, &result, &history[..end], true)?;
    }
    Ok(result)
}

fn validate<E>(history: &[Commit<E>]) -> Result<()> {
    let mut ids = BTreeSet::new();
    let mut prefixes = BTreeSet::new();
    let mut last = None;
    for entry in history {
        if entry.id.is_empty()
            || entry.prefix.is_empty()
            || !ids.insert(&entry.id)
            || !prefixes.insert(&entry.prefix)
            || last.is_some_and(|p| p >= entry.position)
        {
            return Err(Fault::History(
                "empty/duplicate identity or unordered position",
            ));
        }
        last = Some(entry.position);
    }
    Ok(())
}

fn same_commit<E, K: Kernel<E>>(kernel: &K, a: &Commit<E>, b: &Commit<E>) -> bool {
    if a.id != b.id || a.position != b.position || a.prefix != b.prefix || a.check != b.check {
        return false;
    }
    match (&a.command, &b.command) {
        (Command::Event(x), Command::Event(y)) => kernel.same_event(x, y),
        (Command::Pause, Command::Pause) | (Command::Resume, Command::Resume) => true,
        (
            Command::Rebuild {
                scope: x,
                reason: r,
            },
            Command::Rebuild {
                scope: y,
                reason: s,
            },
        ) => x == y && r == s,
        (
            Command::Review {
                alarm_id: x,
                binding: bx,
                verdict: vx,
            },
            Command::Review {
                alarm_id: y,
                binding: by,
                verdict: vy,
            },
        ) => x == y && bx == by && vx == vy,
        _ => false,
    }
}

fn validate_change(old: &Scope, new: &Scope, reason: &RebuildReason) -> Result<()> {
    let valid = match reason {
        RebuildReason::SourceCorrection => {
            old.law == new.law
                && old.interpretation == new.interpretation
                && old.sources != new.sources
        }
        RebuildReason::Reinterpretation => {
            old.law == new.law
                && old.sources == new.sources
                && old.interpretation != new.interpretation
        }
        RebuildReason::LawVersion => old.law != new.law,
        RebuildReason::Repair { .. } => old == new,
    };
    if valid {
        Ok(())
    } else {
        Err(Fault::Binding("rebuild reason/resource mismatch"))
    }
}
