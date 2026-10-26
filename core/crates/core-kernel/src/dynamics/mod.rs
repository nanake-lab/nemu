//! Pure, atomic staging of Echo Core v2.0 §8 C. No x, R, model, clock or storage.
//! The estimator resource is temporary and has no authority to serialize a new IDL type.
mod bounds;
mod detector;
mod duration;
mod interval;
mod numeric;
mod time;
mod types;
mod update;
pub use bounds::StepBounds;
pub use time::{Downtime, RunningTime};
pub use types::{
    DynamicsFuses, DynamicsLaw, DynamicsState, ReactionEvent, ReactionSource, ReviewSignal,
    StagedDynamics,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    InvalidLaw,
    InvalidFuses,
    InvalidEvent,
    LawMismatch,
    EventOrder,
    NonFinite,
    Underflow,
    ZeroDenominator,
    CounterExhausted,
    TimePrecision,
    OutOfRange,
    PrecisionExhausted,
    InvalidRunningTime,
    ComputationBound { component: usize },
}
pub type Result<T> = std::result::Result<T, Fault>;

/// No event and excluded origins are exact identities. The supplied event is a
/// trusted projection from committed H and its stored unmodulated interpretation.
pub fn stage(
    previous: &DynamicsState,
    event: Option<&ReactionEvent>,
    law: &DynamicsLaw,
    fuses: &DynamicsFuses,
) -> Result<StagedDynamics> {
    if &previous.law != law {
        return Err(Fault::LawMismatch);
    }
    fuses.validate()?;
    let unchanged = || StagedDynamics {
        state: previous.clone(),
        change: None,
        signals: Vec::new(),
        bounds: None,
    };
    let Some(event) = event else {
        return Ok(unchanged());
    };
    if !event.source.learns() {
        return Ok(unchanged());
    }
    if event.id.is_empty() || !event.evaluation_unmodulated || event.reaction.schema_version != 1 {
        return Err(Fault::InvalidEvent);
    }
    if let Some((ordinal, time, id)) = &previous.last_event
        && (event.ordinal <= *ordinal || event.time_ns < *time || event.id == *id)
    {
        return Err(Fault::EventOrder);
    }
    event
        .running_time
        .validate_after(previous.last_running_time.as_ref(), event.time_ns)?;
    let mut a = core_types::semantic::nature(&event.reaction);
    for v in &mut a {
        *v = numeric::finite(*v)?;
        if !(-1.0..=1.0).contains(v) {
            return Err(Fault::OutOfRange);
        }
    }
    let detector = detector::observe(&previous.detector, a, law)?;
    let count = numeric::increment(previous.event_count)?;
    let numerator = numeric::add(
        numeric::mul(law.change_prior_weight, law.change_prior)?,
        numeric::count(detector.changes)?,
    )?;
    let denominator = numeric::add(law.change_prior_weight, numeric::count(count)?)?;
    let h = numeric::div(numerator, denominator)?.max(law.change_min);
    let mut next = update::statistics(previous, a, event.running_time.running_ns(), h)?;
    next.detector = detector;
    next.event_count = count;
    next.last_event = Some((event.ordinal, event.time_ns, event.id.clone()));
    next.last_running_time = Some(event.running_time.clone());
    update::finish(previous, next, event, fuses)
}

#[cfg(test)]
mod tests;
