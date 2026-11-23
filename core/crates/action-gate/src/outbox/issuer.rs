//! The local permission service owns this transient issuance table. W02 stores
//! the commitment to the grant and its consumption; the bearer is never logged.
use super::*;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Issuer {
    grants: BTreeMap<String, ExecutionCredential>,
}

impl Issuer {
    /// Called by the trusted permission owner with its resolved scope, subject,
    /// budget and validity interval. An executor cannot enroll its wire request.
    pub fn issue(
        &mut self,
        grant: ExecutionCredential,
        payload: Vec<u8>,
        settings: &Settings,
        limits: Limits,
    ) -> Result<ActionRequestV2> {
        check_settings(&grant, settings)?;
        let request = ActionRequestV2 {
            schema_version: 1,
            action_id: grant.action_id.clone(),
            expires_at_ns: grant.expires_at_ns,
            grant: grant.clone(),
            action_payload: core_types::scalar::HexBytes(payload),
        };
        check_dispatch_binding(
            &request,
            &grant,
            &grant.executor_session_id,
            grant.issued_at_ns.0,
            limits,
        )?;
        let std::collections::btree_map::Entry::Vacant(slot) =
            self.grants.entry(grant.issuance_id.clone())
        else {
            return Err("issuance ID already registered".into());
        };
        slot.insert(grant);
        Ok(request)
    }

    pub fn registered(&self, id: &str) -> Result<&ExecutionCredential> {
        self.grants
            .get(id)
            .ok_or_else(|| "unknown or withdrawn issuance".into())
    }

    /// Removal only withdraws authority. A new process starts without any
    /// grants, including grants issued under a now-incomparable clock.
    pub fn withdraw(&mut self, id: &str) {
        self.grants.remove(id);
    }
}
