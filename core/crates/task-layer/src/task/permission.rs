//! W21 retains the versioned Q binding. An execution credential is transient
//! and comes from the local issuer after an action has been registered.
use core_types::{
    WireMessage,
    contract::{Limits, Result, Schema, require},
    generated::{ActionPlan, ActionRequestV2, ConnectorScope, ExecutionCredential, ObjectRef},
    permission::check_dispatch_binding,
    scalar::{DecimalU64, HexBytes, unhex},
};

#[derive(Clone, Debug, PartialEq)]
pub struct PermissionBinding {
    pub protocol: ObjectRef,
    pub policy: ObjectRef,
    pub revocation_version: DecimalU64,
}

/// Independent issuer lookup, authenticated executor connection and the action
/// ID / exact payload the owner resolved from its registered action.
pub struct RegisteredDispatch<'a> {
    pub issued: &'a ExecutionCredential,
    pub executor_session: &'a str,
    pub now_ns: i64,
    pub action_id: &'a str,
    pub payload: &'a [u8],
}

impl PermissionBinding {
    /// Compare against the live Q binding and catalog scope, for speech or a
    /// tool. P5-14 still owns atomic consumption, budget reservation and send.
    pub fn check_dispatch(
        &self,
        request: &ActionRequestV2,
        scope: &ConnectorScope,
        registered: RegisteredDispatch<'_>,
        limits: Limits,
    ) -> Result<()> {
        let grant = registered.issued;
        check_dispatch_binding(
            request,
            grant,
            registered.executor_session,
            registered.now_ns,
            limits,
        )?;
        require(
            grant.protocol == self.protocol
                && grant.permission_version == self.policy.resource_version
                && grant.revocation_version == self.revocation_version
                && &grant.scope == scope
                && request.action_id == registered.action_id
                && request.action_payload.0 == registered.payload,
            "registered action or live permission changed",
        )
    }
}

/// Resolved from the trusted catalog. The generated plan can name only the
/// canonical object reference of this exact scope and resource version.
pub struct AllowedAction<'a> {
    payload_type: &'a str,
    scope_ref: ObjectRef,
    scope: &'a ConnectorScope,
}

impl<'a> AllowedAction<'a> {
    pub fn new(
        payload_type: &'a str,
        scope: &'a ConnectorScope,
        version: &str,
        limits: Limits,
    ) -> Result<Self> {
        let schema = Schema::builtin()?;
        let value = serde_json::to_value(scope).map_err(|e| e.to_string())?;
        core_types::semantic::check_permission_message(ConnectorScope::NAME, &value, limits)?;
        let scope_ref = ObjectRef {
            schema_version: 1,
            object_type: schema.message(ConnectorScope::NAME)?.tag.clone(),
            resource_version: version.to_owned(),
            digest: HexBytes(unhex(&schema.digest(
                ConnectorScope::NAME,
                &value,
                limits,
            )?)?),
            byte_length: DecimalU64(scope.canonical_bytes(limits)?.len() as u64),
        };
        scope_ref.canonical_bytes(limits)?;
        Ok(Self {
            payload_type,
            scope_ref,
            scope,
        })
    }

    pub fn reference(&self) -> &ObjectRef {
        &self.scope_ref
    }

    pub(super) fn resolve<'p>(&self, plan: &'p ActionPlan) -> Option<ScopedAction<'p, 'a>> {
        (plan.expected_scope == self.scope_ref
            && plan.output_or_tool_parameters.object_type == self.payload_type)
            .then_some(ScopedAction {
                plan,
                scope: self.scope,
            })
    }
}

/// The outbox receives the proposed action together with its resolved scope.
pub struct ScopedAction<'p, 'a> {
    pub plan: &'p ActionPlan,
    pub scope: &'a ConnectorScope,
}
