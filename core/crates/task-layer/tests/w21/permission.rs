use super::*;

pub(super) fn request() -> ActionRequestV2 {
    let vectors: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../../contracts/golden/legal.json")).unwrap();
    let vector = vectors
        .iter()
        .map(|v| &v["request"])
        .find(|v| v["name"] == "ActionRequestV2")
        .unwrap();
    serde_json::from_value(vector["value"].clone()).unwrap()
}

struct CheckedOutbox<'a> {
    request: ActionRequestV2,
    registered: &'a ExecutionCredential,
    payload: &'a [u8],
    action_id: &'a str,
    accepted: usize,
}
impl Outbox for CheckedOutbox<'_> {
    fn register(
        &mut self,
        binding: &Binding,
        _: &[OutputChunk],
        action: Option<ScopedAction<'_, '_>>,
    ) -> Result<(), String> {
        let action = action.ok_or("expected a planned action")?;
        let registered = RegisteredDispatch {
            issued: self.registered,
            executor_session: &self.registered.executor_session_id,
            now_ns: self.registered.issued_at_ns.0,
            action_id: self.action_id,
            payload: self.payload,
        };
        binding
            .permission
            .check_dispatch(&self.request, action.scope, registered, LIMITS)?;
        self.accepted += 1;
        Ok(())
    }
}

#[test]
fn typed_action_matches_the_registered_grant_scope_and_live_versions() {
    let good = request();
    let g = &good.grant;
    let valid = PermissionBinding {
        policy: ObjectRef {
            resource_version: g.permission_version.clone(),
            ..object("policy")
        },
        protocol: g.protocol.clone(),
        revocation_version: g.revocation_version,
    };
    for case in [
        "valid",
        "revoked during generation",
        "stale grant",
        "other policy",
        "other protocol",
        "other scope",
        "substituted grant",
        "other payload",
        "other action",
    ] {
        let mut t = input();
        t.protocol_version.clone_from(&g.protocol.resource_version);
        let mut b = binding(&t);
        b.permission = valid.clone();
        let mut scope = g.scope.clone();
        let mut presented = good.clone();
        match case {
            "other policy" => b.permission.policy.resource_version.push('2'),
            "other protocol" => b.permission.protocol.digest.0[0] ^= 1,
            "stale grant" => b.permission.revocation_version.0 += 1,
            "other scope" => scope.resource_id.push('2'),
            "substituted grant" => presented.grant.issuance_id.push('2'),
            _ => (),
        }
        let allowed = AllowedAction::new("fixture", &scope, "catalog-v1", LIMITS).unwrap();
        let action = ActionPlan {
            schema_version: 1,
            plan_id: "plan".into(),
            snapshot_id: t.snapshot_id.clone(),
            task_version: "1".into(),
            protocol_version: t.protocol_version.clone(),
            output_or_tool_parameters: object("parameters"),
            expected_scope: allowed.reference().clone(),
            dependencies: vec![],
            deadline_ns: core_types::scalar::DecimalI64(200),
        };
        let mode = SpeechMode::A_LOCAL_STRICT;
        let (mut generators, _) = setup(mode, false, Some(action));
        let mut turn = ReplyTurn::new(material(&t).latest, mode);
        let plan = turn
            .advance(
                &material(&t).latest,
                Interpretation::Committed { input_position: 1 },
                &mut generators,
                || Ok(generation(&t, &b, None)),
            )
            .unwrap()
            .unwrap();
        let mut live = b.clone();
        if case == "revoked during generation" {
            live.permission.revocation_version.0 += 1;
        }
        let mut outbox = CheckedOutbox {
            request: presented,
            registered: g,
            payload: if case == "other payload" {
                b"changed"
            } else {
                &good.action_payload.0
            },
            action_id: if case == "other action" {
                "another-action"
            } else {
                &good.action_id
            },
            accepted: 0,
        };
        let result = plan.submit(&live, &[allowed], &mut outbox);
        assert_eq!(result.is_ok(), case == "valid", "{case}: {result:?}");
        assert_eq!(outbox.accepted, usize::from(case == "valid"), "{case}");
    }
}
