//! Single-use dispatch. The W02 owner supplies a serialized consume-and-reserve
//! transaction. No executor callback is reachable until that transaction returns.
use crate::lifecycle::Settings;
use core_types::{
    contract::{Limits, Result, require},
    generated::*,
    permission::check_dispatch_binding,
};
pub mod delivery;
mod issuer;
pub use issuer::Issuer;

/// These values come independently from the authenticated connection, live
/// catalog and versioned permission resources, never from the submitted grant.
pub struct Authority<'a> {
    pub issuer: &'a Issuer,
    pub session: &'a str,
    pub protocol: &'a ObjectRef,
    pub permission: &'a ObjectRef,
    pub revocation_version: u64,
    pub scope: &'a ConnectorScope,
    pub budget: &'a ExecutionBudget,
    pub consent: Option<&'a ActionConsent>,
    pub trust: Option<&'a TrustMemory>,
    pub tier: ToolTier,
    pub settings: &'a Settings,
}

impl Authority<'_> {
    pub fn check(&self, request: &ActionRequestV2, now_ns: i64, limits: Limits) -> Result<()> {
        let g = self.issuer.registered(&request.grant.issuance_id)?;
        check_dispatch_binding(request, g, self.session, now_ns, limits)?;
        require(
            &g.protocol == self.protocol
                && g.permission_version == self.permission.resource_version
                && g.revocation_version.0 == self.revocation_version
                && &g.scope == self.scope
                && &g.budget == self.budget
                && g.consent.as_ref() == self.consent
                && g.trust.as_ref() == self.trust
                && g.tier == self.tier,
            "grant revoked or scope changed",
        )?;
        if let Some(policy) = &self.scope.desktop {
            require(
                policy.irreversible == self.settings.irreversible,
                "desktop settings changed",
            )?;
        }
        let ttl = self
            .settings
            .credential_lifetime_ns
            .ok_or("credential lifetime unset")?;
        require(
            ttl > 0
                && i128::from(g.expires_at_ns.0) - i128::from(g.issued_at_ns.0) <= i128::from(ttl),
            "credential exceeds configured lifetime",
        )
    }
}

pub trait Outbox {
    type Error: From<String>;
    /// Re-read Q and its committed prefix under the sole writer, compare the
    /// authorized payload, consume issuance_id, reserve budget and commit
    /// DISPATCHING. An ambiguous commit must return an error.
    fn consume(
        &mut self,
        request: &ActionRequestV2,
        authority: &Authority<'_>,
        now_ns: i64,
    ) -> std::result::Result<(), Self::Error>;
    /// Inspect durable ingress immediately before each external send.
    fn check_frontier(&self) -> std::result::Result<(), Self::Error>;
}

/// The executor receives only payload decoded from the checked immutable bytes.
/// Each numbered speech block is sent separately. Any error after consume leaves
/// DISPATCHING for reconciliation; no retry path reuses this credential.
pub fn dispatch<O: Outbox>(
    outbox: &mut O,
    request: &ActionRequestV2,
    authority: &Authority<'_>,
    limits: Limits,
    mut now: impl FnMut() -> Option<i64>,
    mut send: impl FnMut(delivery::Message<'_>) -> std::result::Result<(), O::Error>,
) -> std::result::Result<(), O::Error> {
    let mut at = now().ok_or_else(|| "issuance clock domain unavailable".to_string())?;
    authority.check(request, at, limits)?;
    let payload = delivery::Content::decode(request, limits)?;
    outbox.consume(request, authority, at)?;
    let mut emit = |message| {
        outbox.check_frontier()?;
        let current = now().ok_or_else(|| "issuance clock domain unavailable".to_string())?;
        require(current >= at, "dispatch clock moved backwards")?;
        at = current;
        authority.check(request, current, limits)?;
        send(message)
    };
    match &payload {
        delivery::Content::Body => emit(delivery::Message::Body(request)),
        delivery::Content::Speech(speech) => {
            for chunk in speech {
                emit(delivery::Message::Chunk(chunk))?;
            }
            Ok(())
        }
    }
}
