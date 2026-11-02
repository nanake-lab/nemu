//! CONTRACTS §6: confirmation requirements for an already classified action.
//! The remembered grant is a current authoritative lookup, not a worker claim.
//! Social classification awaits the V2-89 successor contract.

use core_types::generated::{ApprovalChoice, ToolTier};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confirmation {
    NotRequired,
    InitialContextGrant,
    EveryAction,
}

/// Compute only the confirmation requirement. `NotRequired` is not permission
/// to execute: identity, scope, budgets, revocation and credentials still apply.
pub fn confirmation(tier: ToolTier, remembered_grant: bool) -> Confirmation {
    match (tier, remembered_grant) {
        (ToolTier::T0, _) | (ToolTier::T1 | ToolTier::T2, true) => Confirmation::NotRequired,
        (ToolTier::T1, false) => Confirmation::InitialContextGrant,
        (ToolTier::T2, false) | (ToolTier::T3, _) => Confirmation::EveryAction,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoiceError {
    Denied,
    T3CannotBeRemembered,
}

/// Validate a user's choice only; this does not bind it to a caller or payload,
/// persist a grant, or authorize execution. I26 validation is additionally needed.
pub fn check_choice(tier: ToolTier, choice: ApprovalChoice) -> Result<(), ChoiceError> {
    match (tier, choice) {
        (_, ApprovalChoice::DENY) => Err(ChoiceError::Denied),
        (ToolTier::T3, ApprovalChoice::ALLOW_ALWAYS) => Err(ChoiceError::T3CannotBeRemembered),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_matrix_and_revoked_grant() {
        for (tier, initial, trusted) in [
            (
                ToolTier::T0,
                Confirmation::NotRequired,
                Confirmation::NotRequired,
            ),
            (
                ToolTier::T1,
                Confirmation::InitialContextGrant,
                Confirmation::NotRequired,
            ),
            (
                ToolTier::T2,
                Confirmation::EveryAction,
                Confirmation::NotRequired,
            ),
            (
                ToolTier::T3,
                Confirmation::EveryAction,
                Confirmation::EveryAction,
            ),
        ] {
            assert_eq!(confirmation(tier, false), initial);
            assert_eq!(confirmation(tier, true), trusted);
            // A fresh lookup after revocation must not reuse the prior answer.
            assert_eq!(confirmation(tier, false), initial);
        }
    }

    #[test]
    fn denial_and_t3_trust_cannot_be_overridden() {
        for tier in [ToolTier::T0, ToolTier::T1, ToolTier::T2, ToolTier::T3] {
            assert_eq!(
                check_choice(tier, ApprovalChoice::DENY),
                Err(ChoiceError::Denied)
            );
            assert_eq!(check_choice(tier, ApprovalChoice::ALLOW_ONCE), Ok(()));
            let expected = if tier == ToolTier::T3 {
                Err(ChoiceError::T3CannotBeRemembered)
            } else {
                Ok(())
            };
            assert_eq!(check_choice(tier, ApprovalChoice::ALLOW_ALWAYS), expected);
        }
    }
}
