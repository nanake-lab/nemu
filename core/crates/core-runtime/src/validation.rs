use crate::{
    Error, Result, binding::digest, check, clock::ProcessTime, same, stages::Snapshot,
    status::Status, validate,
};
use core_types::{contract::Limits, generated::*, semantic::check_theta};

/// Operators §5.3 / v2.0 §§6, 8: encoding alone permits zero time divisors.
/// This is the law-independent check, not check_theta_within: resolving and
/// binding the authoritative law is tracked in P5-08-stage-inputs-and-law.md.
pub(crate) fn validate_theta(theta: &ThetaSnapshot, limits: Limits) -> Result<()> {
    validate(theta, limits)?;
    check_theta(theta).map_err(Error::Contract)
}

pub(crate) fn validate_inputs(
    event: &EventEnvelope,
    attributes: &EventAttributesRecord,
    base: &Snapshot,
    facts: &FactProjection,
    binding: &ProposalBindingV2,
    time: &ProcessTime,
    limits: Limits,
) -> Result<()> {
    validate(event, limits)?;
    validate(attributes, limits)?;
    validate(facts, limits)?;
    validate(binding, limits)?;
    validate(&base.x, limits)?;
    validate(&base.r, limits)?;
    validate_theta(&base.theta, limits)?;
    check(
        attributes.event_id == event.id
            && binding.event_id == event.id
            && binding.raw_log_position == event.raw_log_position,
        "head event mismatch",
    )?;
    check(
        binding.process_time_ns.0 == time.ns()
            && binding.anchor_time_ns == base.x.anchor_time_ns
            && time.ns() >= binding.anchor_time_ns.0,
        "process time or anchor mismatch",
    )?;
    check(
        binding.x_revision == base.x.revision
            && binding.r_revision == base.r.revision
            && binding.theta_revision == base.theta.revision,
        "state revision mismatch",
    )?;
    check(
        facts.history_prefix == binding.base_commit
            && digest(facts, limits)? == binding.c_core_digest,
        "fact prefix or digest mismatch",
    )
}
pub(crate) fn validate_x(x: &FastStateSnapshotV2, ns: u64, limits: Limits) -> Result<()> {
    validate(x, limits)?;
    check(x.anchor_time_ns.0 == ns, "F advanced time more than once")
}
pub(crate) fn stage_error(reason: FailureCode) -> Error {
    Error::Contract(format!("stage failed: {reason:?}"))
}
pub(crate) fn failure(status: &mut Status, stage: RunStatus, reason: FailureCode) -> Result<()> {
    check(!reason.is_superseded(), "superseded failure reason")?;
    status.add(stage, reason);
    Ok(())
}
/// Invalid worker data may degrade the event; stale identity is never rebased.
pub(crate) fn recoverable(
    result: Result<()>,
    status: &mut Status,
    stage: RunStatus,
) -> Result<bool> {
    match result {
        Ok(()) => Ok(true),
        Err(Error::Contract(_)) => {
            failure(status, stage, FailureCode::STRUCTURAL_FAULT)?;
            Ok(false)
        }
        Err(error) => Err(error),
    }
}
pub(crate) fn validate_resize(
    before: &FastStateSnapshotV2,
    after: &FastStateSnapshotV2,
    r: &RSnapshotV3,
    limits: Limits,
) -> Result<()> {
    validate(after, limits)?;
    check(
        same(&before.nature, &after.nature, limits)?,
        "F_out applied a second nature pulse",
    )?;
    let active: std::collections::BTreeSet<_> = r
        .tendencies
        .iter()
        .filter(|t| t.pi > 0.5)
        .map(|t| &t.tendency_id)
        .collect();
    let ids: std::collections::BTreeSet<_> = after.learned.iter().map(|v| &v.tendency_id).collect();
    check(
        ids == active && ids.len() == after.learned.len(),
        "F_out coordinates do not match R+",
    )?;
    for v in &after.learned {
        let expected = before
            .learned
            .iter()
            .find(|old| old.tendency_id == v.tendency_id)
            .map(|old| old.value)
            .unwrap_or(0.0);
        check(
            v.value.to_bits() == expected.to_bits(),
            "F_out changed a surviving coordinate or initialized a nonzero coordinate",
        )?;
    }
    Ok(())
}
