//! Proposal validation uses the complete frozen binding, not just base_commit.
use crate::{Result, check, same, validate};
use core_types::{WireMessage, contract::Limits, generated::ProposalBindingV2};

pub fn digest<T: WireMessage>(value: &T, limits: Limits) -> Result<core_types::scalar::HexBytes> {
    let schema = core_types::contract::Schema::builtin().map_err(crate::Error::Contract)?;
    let bytes = value
        .canonical_bytes(limits)
        .map_err(crate::Error::Contract)?;
    let decoded = schema
        .decode_canonical(T::NAME, &bytes, limits, 0)
        .map_err(crate::Error::Contract)?;
    let hex = schema
        .digest(T::NAME, &decoded, limits)
        .map_err(crate::Error::Contract)?;
    Ok(core_types::scalar::HexBytes(
        core_types::scalar::unhex(&hex).map_err(crate::Error::Contract)?,
    ))
}

/// A computation response. This wrapper is in-process only, never serialized.
pub struct Bound<T> {
    pub binding: ProposalBindingV2,
    pub value: T,
}
impl<T> Bound<T> {
    pub fn check(self, expected: &ProposalBindingV2, limits: Limits) -> Result<T> {
        check(
            same(&self.binding, expected, limits)?,
            "stale proposal binding",
        )?;
        Ok(self.value)
    }
}
pub fn validate_record<T: WireMessage>(
    value: &T,
    actual: &ProposalBindingV2,
    expected: &ProposalBindingV2,
    event_id: &str,
    limits: Limits,
) -> Result<()> {
    check(event_id == expected.event_id, "record event mismatch")?;
    check(same(actual, expected, limits)?, "stale record binding")?;
    validate(value, limits)
}
