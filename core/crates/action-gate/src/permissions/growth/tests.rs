use super::*;
use core_types::{
    generated::{ObjectRef, ReviewVerdict},
    scalar::{DecimalU64, HexBytes},
};

fn limits() -> Limits {
    Limits {
        max_bytes: 8192,
        max_items: 64,
        max_depth: 16,
    }
}
fn user(entry: EntryPoint) -> CallerChecks {
    CallerChecks {
        entry,
        authenticated: true,
        is_current_core_user: true,
        panel_secondary_confirmation: true,
    }
}
fn command(kind: GrowthCommandKind) -> GrowthCommand {
    let review = kind == GrowthCommandKind::GROWTH_REVIEW_CONCLUSION;
    GrowthCommand {
        schema_version: 1,
        command_id: "test-command".into(),
        kind,
        verdict: review.then_some(ReviewVerdict::LAW_CORRECT),
        review_record: review.then_some(ObjectRef {
            schema_version: 1,
            digest: HexBytes(vec![7; 32]),
            object_type: "test-review".into(),
            resource_version: "test-v1".into(),
            byte_length: DecimalU64(1),
        }),
    }
}

#[test]
fn worker_helper_and_network_cannot_manage_growth_even_with_user_claims() {
    for entry in [EntryPoint::Worker, EntryPoint::Helper, EntryPoint::Network] {
        for kind in [
            GrowthCommandKind::GROWTH_PAUSE,
            GrowthCommandKind::GROWTH_RESUME,
            GrowthCommandKind::GROWTH_REVIEW_CONCLUSION,
        ] {
            assert_eq!(
                check_command(user(entry), &command(kind), limits()),
                Err(GrowthRejection::ForbiddenEntryPoint)
            );
        }
    }
}

#[test]
fn local_user_identity_and_panel_confirmation_are_required() {
    for entry in [EntryPoint::Panel, EntryPoint::Echoctl] {
        for kind in [
            GrowthCommandKind::GROWTH_PAUSE,
            GrowthCommandKind::GROWTH_RESUME,
        ] {
            let cmd = command(kind);
            let mut caller = user(entry);
            assert_eq!(check_command(caller, &cmd, limits()), Ok(()));
            caller.authenticated = false;
            assert_eq!(
                check_command(caller, &cmd, limits()),
                Err(GrowthRejection::Unauthenticated)
            );
            caller.authenticated = true;
            caller.is_current_core_user = false;
            assert_eq!(
                check_command(caller, &cmd, limits()),
                Err(GrowthRejection::NotCurrentCoreUser)
            );
            caller = user(entry);
            caller.panel_secondary_confirmation = false;
            let expected = if entry == EntryPoint::Panel {
                Err(GrowthRejection::PanelConfirmationMissing)
            } else {
                Ok(())
            };
            assert_eq!(check_command(caller, &cmd, limits()), expected);
        }
    }
}

#[test]
fn review_requires_echoctl_and_both_typed_fields() {
    let mut cmd = command(GrowthCommandKind::GROWTH_REVIEW_CONCLUSION);
    let caller = user(EntryPoint::Echoctl);
    for verdict in [ReviewVerdict::LAW_CORRECT, ReviewVerdict::LAW_INCORRECT] {
        cmd.verdict = Some(verdict);
        assert_eq!(check_command(caller, &cmd, limits()), Ok(()));
    }
    assert_eq!(
        check_command(user(EntryPoint::Panel), &cmd, limits()),
        Err(GrowthRejection::ReviewRequiresEchoctl)
    );
    cmd.review_record = None;
    assert_eq!(
        check_command(caller, &cmd, limits()),
        Err(GrowthRejection::InvalidCommand)
    );
    cmd = command(GrowthCommandKind::GROWTH_REVIEW_CONCLUSION);
    cmd.verdict = None;
    assert_eq!(
        check_command(caller, &cmd, limits()),
        Err(GrowthRejection::InvalidCommand)
    );
    assert!(ReviewVerdict::try_from(0).is_err());
    assert!(ReviewVerdict::try_from(3).is_err()); // No veto-result option.
}

#[test]
fn malformed_and_over_budget_commands_fail_closed() {
    let caller = user(EntryPoint::Echoctl);
    let valid = command(GrowthCommandKind::GROWTH_REVIEW_CONCLUSION);
    for invalid in [
        GrowthCommand {
            schema_version: 2,
            ..valid.clone()
        },
        GrowthCommand {
            command_id: String::new(),
            ..valid.clone()
        },
        GrowthCommand {
            kind: GrowthCommandKind::GROWTH_PAUSE,
            ..valid.clone()
        },
    ] {
        assert_eq!(
            check_command(caller, &invalid, limits()),
            Err(GrowthRejection::InvalidCommand)
        );
    }
    let mut invalid_ref = valid.clone();
    invalid_ref.review_record.as_mut().unwrap().digest = HexBytes(vec![0; 31]);
    assert_eq!(
        check_command(caller, &invalid_ref, limits()),
        Err(GrowthRejection::InvalidCommand)
    );
    assert_eq!(
        check_command(
            caller,
            &valid,
            Limits {
                max_bytes: 1,
                ..limits()
            }
        ),
        Err(GrowthRejection::InvalidCommand)
    );
}
