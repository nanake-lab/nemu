//! A fixed v2.0 order, stopping at a non-forgeable STAGED capability.
//! The caller settles Gamma through `store::next` first. Nothing here writes or
//! runs a model/clock implicitly. Terminal integration and asynchronous worker
//! completion are not implemented by these synchronous trusted adapters.
use crate::{
    Error, Result,
    binding::{digest, validate_record},
    check,
    clock::ProcessTime,
    same,
    stages::*,
    status::Status,
    validate,
    validation::{
        failure, recoverable, stage_error, validate_inputs, validate_resize, validate_theta,
        validate_x,
    },
};
use core_store::LineageKind;
use core_types::generated::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Gamma,
    Phi,
    History,
    Irl,
    A,
    M,
    FIn,
    G,
    C,
    FOut,
    Ledger,
    Commit,
    J,
}
pub const ORDER: [Step; 13] = [
    Step::Gamma,
    Step::Phi,
    Step::History,
    Step::Irl,
    Step::A,
    Step::M,
    Step::FIn,
    Step::G,
    Step::C,
    Step::FOut,
    Step::Ledger,
    Step::Commit,
    Step::J,
];
pub fn verify_order(trace: &[Step]) -> Result<()> {
    check(
        trace.len() <= ORDER.len() && trace == &ORDER[..trace.len()],
        "stage order or repeated projection",
    )
}
/// Does not infer observation time from ingress or overwrite actual event time.
/// The distinct observation time/source requires ADR 0032's successor contract.
pub struct EventTimes {
    pub occurred_ns: i64,
    pub received_ns: i64,
    pub process_ns: u64,
}
pub struct Input<'a> {
    pub event: EventEnvelope,
    pub attributes: &'a EventAttributesRecord,
    pub base: Snapshot,
    pub facts: &'a FactProjection,
    pub binding: ProposalBindingV2,
    pub time: ProcessTime,
    /// A read-only input from authoritative Q, not a UI control or permission.
    pub growth_paused: bool,
}
/// An uncommitted result cannot cross lineages.
/// ```compile_fail
/// use core_runtime::{pipeline::StagedEvent, store::prepare_existing_head};
/// use core_store::{Store, ReadTicket, Experiment, Rehearsal};
/// fn cross(s: &Store<Rehearsal>, t: ReadTicket<Rehearsal>, e: StagedEvent<Experiment>, l: core_types::contract::Limits) {
///     let _ = prepare_existing_head(s, t, e, l);
/// }
/// ```
pub struct StagedEvent<L: LineageKind> {
    pub(crate) event: EventEnvelope,
    pub(crate) binding: ProposalBindingV2,
    pub(crate) state: Snapshot,
    pub(crate) base: Snapshot,
    pub(crate) semantic: Option<SemanticOutput>,
    pub(crate) appraisal: Option<NatureAppraisalRecord>,
    pub(crate) matching: Option<MatchOutput>,
    pub(crate) ledger: Vec<LedgerEntry>,
    pub(crate) status: Status,
    pub(crate) trace: Vec<Step>,
    pub(crate) time: ProcessTime,
    pub(crate) lineage: std::marker::PhantomData<L>,
}
impl<L: LineageKind> StagedEvent<L> {
    pub fn snapshot(&self) -> &Snapshot {
        &self.state
    }
    pub fn status(&self) -> &Status {
        &self.status
    }
    pub fn trace(&self) -> &[Step] {
        &self.trace
    }
    pub fn binding(&self) -> &ProposalBindingV2 {
        &self.binding
    }
    pub fn ledger(&self) -> &[LedgerEntry] {
        &self.ledger
    }
    pub fn times(&self) -> EventTimes {
        EventTimes {
            occurred_ns: self.event.event_time_ns.0,
            received_ns: self.event.ingest_time_ns.0,
            process_ns: self.time.ns(),
        }
    }
}

impl<L: LineageKind> StageSet<L> {
    pub fn process_head(&mut self, input: Input<'_>) -> Result<StagedEvent<L>> {
        let Input {
            event,
            attributes,
            base,
            facts,
            binding,
            time,
            growth_paused,
        } = input;
        let limits = self.limits;
        validate_inputs(&event, attributes, &base, facts, &binding, &time, limits)?;
        check(
            same(&binding.environment, &self.environment, limits)?,
            "stage environment mismatch",
        )?;
        let mut status = Status::default();
        if L::KIND == Lineage::REHEARSAL {
            status.add(RunStatus::NO_GROWTH, FailureCode::GROWTH_OFF_REHEARSAL);
            status.add(
                RunStatus::CORE_UNMODULATED,
                FailureCode::MODULATION_DISABLED,
            );
        } else if growth_paused {
            // No invented GROWTH_PAUSED FailureCode: the frozen contract has only RunStatus.
            status.add_state(RunStatus::GROWTH_PAUSED);
        }
        let mut trace = vec![Step::Gamma];
        let active_before = active(&base.r);
        let projected = if let Some(f) = &self.parts.f {
            f.project(
                Projection {
                    x: &base.x,
                    theta: &base.theta.values,
                    active: &active_before,
                },
                time.ns(),
            )
            .map_err(stage_error)?
        } else {
            status.add(
                RunStatus::MODULATION_NOT_ENABLED,
                FailureCode::MODULATION_DISABLED,
            );
            let mut x = base.x.clone();
            x.anchor_time_ns.0 = time.ns();
            x
        };
        validate_x(&projected, time.ns(), limits)?;
        check(
            digest(&projected, limits)? == binding.projected_state_digest,
            "projected state digest mismatch",
        )?;
        trace.extend([Step::Phi, Step::History, Step::Irl]);
        let semantic = self
            .parts
            .irl
            .as_mut()
            .map(|stage| {
                stage.interpret(Interpretation {
                    event: &event,
                    facts,
                    binding: &binding,
                })
            })
            .unwrap_or(Err(FailureCode::IRL_FAILED));
        let semantic = match semantic {
            Ok(output) => {
                let output = output.check(&binding, limits)?;
                let validated = (|| {
                    validate(&output.graph, limits)?;
                    check(
                        digest(&output.graph, limits)? == binding.accepted_z_digest,
                        "accepted z digest mismatch",
                    )?;
                    for concept in &output.concepts {
                        validate(concept, limits)?;
                        check(
                            same(&concept.binding, &binding, limits)?,
                            "concept binding mismatch",
                        )?;
                    }
                    Ok(())
                })();
                if recoverable(validated, &mut status, RunStatus::IRL_DEGRADED)? {
                    Some(output)
                } else {
                    None
                }
            }
            Err(reason) => {
                failure(&mut status, RunStatus::IRL_DEGRADED, reason)?;
                None
            }
        };
        trace.push(Step::A);
        let appraisal = if let Some(z) = &semantic {
            let response = self
                .parts
                .a
                .as_mut()
                .map(|stage| {
                    stage.appraise(Appraisal {
                        graph: &z.graph,
                        binding: &binding,
                    })
                })
                .unwrap_or(Err(FailureCode::DEPENDENCY_FAULT));
            match response {
                Ok(value) => {
                    if recoverable(
                        validate_record(&value, &value.binding, &binding, &value.event_id, limits),
                        &mut status,
                        RunStatus::A_DEGRADED,
                    )? {
                        Some(value)
                    } else {
                        None
                    }
                }
                Err(reason) => {
                    failure(&mut status, RunStatus::A_DEGRADED, reason)?;
                    None
                }
            }
        } else {
            None
        };
        trace.push(Step::M);
        let matching = if let Some(z) = &semantic
            && L::KIND != Lineage::REHEARSAL
        {
            let concepts: Vec<_> = base
                .r
                .tendencies
                .iter()
                .map(|t| (t.tendency_id.clone(), t.concept.clone()))
                .collect();
            let response = self
                .parts
                .m
                .as_mut()
                .map(|stage| {
                    stage.match_event(Matching {
                        graph: &z.graph,
                        concepts: &concepts,
                        binding: &binding,
                    })
                })
                .unwrap_or(Err(FailureCode::DEPENDENCY_FAULT));
            match response {
                Ok(value) => {
                    let validated = (|| {
                        validate_record(
                            &value.record,
                            &value.record.binding,
                            &binding,
                            &value.record.event_id,
                            limits,
                        )?;
                        for candidate in &value.candidates {
                            validate_record(
                                candidate,
                                &candidate.binding,
                                &binding,
                                &candidate.event_id,
                                limits,
                            )?;
                        }
                        Ok(())
                    })();
                    if recoverable(validated, &mut status, RunStatus::M_DEGRADED)? {
                        Some(value)
                    } else {
                        None
                    }
                }
                Err(reason) => {
                    failure(&mut status, RunStatus::M_DEGRADED, reason)?;
                    None
                }
            }
        } else {
            None
        };
        trace.push(Step::FIn);
        // Semantic failure and system maintenance never invent a drive.
        let x = if semantic.is_some() && event.kind != EventKind::MAINTENANCE {
            self.parts
                .f
                .as_ref()
                .map(|stage| {
                    stage.pulse(Pulse {
                        theta: &base.theta.values,
                        active: &active_before,
                        projected: &projected,
                        graph: semantic.as_ref().map(|z| &z.graph),
                        appraisal: appraisal.as_ref(),
                        matching: matching.as_ref().map(|m| &m.record),
                    })
                })
                .transpose()
                .map_err(stage_error)?
                .unwrap_or(projected)
        } else {
            projected
        };
        validate_x(&x, time.ns(), limits)?;
        trace.push(Step::G);
        let can_grow =
            L::KIND != Lineage::REHEARSAL && !growth_paused && event.kind != EventKind::MAINTENANCE;
        let mut ledger = vec![];
        let mut r = base.r.clone();
        if can_grow && let (Some(z), Some(a), Some(m)) = (&semantic, &appraisal, &matching) {
            let response = self
                .parts
                .g
                .as_mut()
                .ok_or(Error::Rejected("missing G"))?
                .learn(Learning {
                    r: &base.r,
                    baseline: &base.theta.values.baseline,
                    sigma: &base.theta.sigma,
                    event: &event,
                    attributes,
                    graph: &z.graph,
                    appraisal: a,
                    matching: m,
                    binding: &binding,
                })
                .map_err(stage_error)?
                .check(&binding, limits)?;
            validate(&response.value, limits)?;
            check(
                same(&response.value, &base.r, limits)? || !response.ledger.is_empty(),
                "G changed R without ledger evidence",
            )?;
            r = response.value;
            ledger.extend(response.ledger);
        }
        trace.push(Step::C);
        let mut theta = base.theta.clone();
        if can_grow
            && attributes.exogenous
            && !attributes.self_narrative
            && let Some(a) = &appraisal
        {
            let response = self
                .parts
                .c
                .as_mut()
                .ok_or(Error::Rejected("missing C"))?
                .update(DynamicsInput {
                    theta: &base.theta,
                    event: &event,
                    appraisal: a,
                    binding: &binding,
                })
                .map_err(stage_error)?
                .check(&binding, limits)?;
            validate_theta(&response.value, limits)?;
            check(
                same(&response.value, &base.theta, limits)? || !response.ledger.is_empty(),
                "C changed theta without ledger evidence",
            )?;
            theta = response.value;
            ledger.extend(response.ledger);
        }
        trace.push(Step::FOut);
        // With unchanged coordinates F_out is identity. In particular a failed
        // interpretation cannot acquire an extra pulse through a finish adapter.
        let changed = !same(&theta, &base.theta, limits)? || !same(&r, &base.r, limits)?;
        let finished = if changed {
            let result = self
                .parts
                .f
                .as_ref()
                .ok_or(Error::Rejected("missing F_out"))?
                .finish(&active(&r), &x)
                .map_err(stage_error)?;
            validate_resize(&x, &result, &r, limits)?;
            result
        } else {
            x
        };
        validate_x(&finished, time.ns(), limits)?;
        trace.push(Step::Ledger);
        for entry in &ledger {
            validate(entry, limits)?;
            check(entry.event_id == event.id, "ledger event mismatch")?;
        }
        verify_order(&trace)?;
        // E2 default m=0 applies even when F is connected. Compute/record x anyway.
        if L::KIND == Lineage::REHEARSAL {
            status.pause_autonomy();
        }
        Ok(StagedEvent {
            event,
            binding,
            state: Snapshot {
                x: finished,
                r,
                theta,
            },
            base,
            semantic,
            appraisal,
            matching,
            ledger,
            status,
            trace,
            time,
            lineage: std::marker::PhantomData,
        })
    }
    /// Read-only maintenance. No H append, evidence, G/C, or clock-anchor write.
    pub fn maintenance(&self, base: &Snapshot, time: &ProcessTime) -> Result<Snapshot> {
        validate_theta(&base.theta, self.limits)?;
        check(
            time.ns() >= base.x.anchor_time_ns.0,
            "maintenance before anchor",
        )?;
        let mut view = base.clone();
        if let Some(f) = &self.parts.f {
            view.x = f
                .project(
                    Projection {
                        x: &base.x,
                        theta: &base.theta.values,
                        active: &active(&base.r),
                    },
                    time.ns(),
                )
                .map_err(stage_error)?;
            validate_x(&view.x, time.ns(), self.limits)?;
        }
        Ok(view)
    }
}
