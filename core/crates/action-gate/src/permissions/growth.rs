//! v2.0 §5.4/§10/§11, ADR 0010 decision 5, CONTRACTS §4.3.

use core_types::{
    WireMessage,
    contract::Limits,
    generated::{GrowthCommand, GrowthCommandKind},
};

/// Internal adapter classification, not a wire type and not proof of identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryPoint {
    Panel,
    Echoctl,
    Worker,
    Helper,
    Network,
}

/// Facts an authenticated local adapter must establish independently of payload.
/// These booleans carry no capability and cannot authorize a call.
#[derive(Clone, Copy, Debug)]
pub struct CallerChecks {
    pub entry: EntryPoint,
    pub authenticated: bool,
    pub is_current_core_user: bool,
    pub panel_secondary_confirmation: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrowthRejection {
    ForbiddenEntryPoint,
    Unauthenticated,
    NotCurrentCoreUser,
    PanelConfirmationMissing,
    ReviewRequiresEchoctl,
    InvalidCommand,
}

/// Check established caller facts and the generated command contract.
/// Success only means these constraints hold. It does not commit an event,
/// audit a command, or bypass the pending alarm/reference binding contract.
pub fn check_command(
    caller: CallerChecks,
    command: &GrowthCommand,
    limits: Limits,
) -> Result<(), GrowthRejection> {
    if matches!(
        caller.entry,
        EntryPoint::Worker | EntryPoint::Helper | EntryPoint::Network
    ) {
        return Err(GrowthRejection::ForbiddenEntryPoint);
    }
    if !caller.authenticated {
        return Err(GrowthRejection::Unauthenticated);
    }
    if !caller.is_current_core_user {
        return Err(GrowthRejection::NotCurrentCoreUser);
    }
    if caller.entry == EntryPoint::Panel {
        if command.kind == GrowthCommandKind::GROWTH_REVIEW_CONCLUSION {
            return Err(GrowthRejection::ReviewRequiresEchoctl);
        }
        if !caller.panel_secondary_confirmation {
            return Err(GrowthRejection::PanelConfirmationMissing);
        }
    }
    command
        .canonical_bytes(limits)
        .map_err(|_| GrowthRejection::InvalidCommand)?;
    Ok(())
}

#[cfg(test)]
mod tests;
