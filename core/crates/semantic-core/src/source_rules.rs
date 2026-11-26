//! Checks the already-approved external/self-narrative partition (V2-71, EV3).
//! This is NOT the complete versioned property producer. In particular, no rule
//! here guesses EV9 verifiability or authenticates an account/person association.
use crate::{Fault, Result, aliases::Aliases, wire};
use core_types::{contract::Limits, generated::EventAttributesRecord};

pub const PARTITION_VERSION: &str = "p5-06-source-partition-preflight-v1";

/// Selected by a trusted ingress/payload adapter, never by the model or free text.
/// These are in-process inputs, not new event kinds or persisted contracts.
#[derive(Clone, Copy, Debug)]
pub enum Origin {
    DirectedUserInteraction,
    OtherPersonsReply,
    DesktopAboutUser,
    OwnNarrative,
    OwnAction,
    TriggeredProgramResult,
    SystemMaintenance,
    ImportedRehearsalDiary,
}

/// Checks only known invariants. A successful result must not be used as proof
/// of verifiability, provenance, source validity, or evidence eligibility.
pub fn preflight(
    record: &EventAttributesRecord,
    event_id: &str,
    origin: Origin,
    authenticated_account: &str,
    persons: &Aliases,
    limits: Limits,
) -> Result<()> {
    wire(record, limits)?;
    let (external, narrative) = match origin {
        Origin::DirectedUserInteraction | Origin::OtherPersonsReply => (true, false),
        Origin::DesktopAboutUser
        | Origin::OwnAction
        | Origin::TriggeredProgramResult
        | Origin::SystemMaintenance => (false, false),
        Origin::OwnNarrative | Origin::ImportedRehearsalDiary => (false, true),
    };
    if record.event_id != event_id
        || record.rule_table_version != PARTITION_VERSION
        || record.exogenous != external
        || record.self_narrative != narrative
        || record.source_key != persons.representative(authenticated_account)?
    {
        return Err(Fault::Source);
    }
    Ok(())
}
