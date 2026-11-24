use super::*;

impl<L: LineageKind> ActionTransaction<'_, L> {
    fn check_authority(
        &self,
        request: &ActionRequestV2,
        authority: &Authority<'_>,
        now_ns: i64,
    ) -> Result<()> {
        authority.check(request, now_ns, self.limits)?;
        let action = self.action();
        ensure(
            self.event.kind == EventKind::MAINTENANCE
                && request.action_id == action.action_id
                && request.expires_at_ns.0 <= action.plan.deadline_ns.0
                && request.action_payload.0 == self.payload_bytes()?,
            "wrong action, payload or deadline",
        )?;
        let reader = &self.reader;
        let scope: ConnectorScope = reader.read(&action.plan.expected_scope)?;
        ensure(
            &scope == authority.scope && authority.consent == self.history.consent.as_ref(),
            "scope or authenticated consent changed",
        )?;
        if self.history.consent.is_some() {
            let approved: ApprovalRequestV2 = reader.read(&action.request)?;
            ensure(
                approved.tier == authority.tier,
                "authorized tier differs from the displayed request",
            )?;
        }
        let b = &self.origin;
        let frame = self.frame()?;
        ensure(
            b.protocol == *authority.protocol
                && &b.permission_snapshot == authority.permission
                && b.environment == frame.transaction.binding.environment
                && b.source_validity_version == frame.transaction.binding.source_validity_version
                && action
                    .plan
                    .dependencies
                    .iter()
                    .all(|r| frame.ticket.binding().read_set.contains(r)),
            "action resources or versions changed",
        )?;
        ensure(
            self.q.mode == CognitiveMode::COG_RUNNING,
            "local selection path is paused",
        )?;
        let scheduled = self
            .q
            .active_request
            .as_ref()
            .or(self.q.runnable_successor.as_ref())
            .ok_or(Error::Rejected("current Q has no bound input"))?;
        let current: CognitiveInput = reader.read(scheduled)?;
        let next = frame
            .next_input
            .as_ref()
            .ok_or(Error::Rejected("no current successor binding"))?;
        for live in [&current.binding, &next.binding] {
            ensure(
                live.branch_id == b.branch_id
                    && live.input_frontier == b.input_frontier
                    && live.snapshot_id == b.snapshot_id
                    && live.permission_snapshot == b.permission_snapshot
                    && live.protocol == b.protocol
                    && live.environment == b.environment
                    && live.source_validity_version == b.source_validity_version,
                "invalid source frontier, snapshot or runtime resource continuation",
            )?;
        }
        ensure(
            current.binding.q_revision == self.q.revision,
            "scheduled input is stale",
        )?;
        self.history.continuation(&frame.transaction)?;
        if let delivery::Content::Speech(s) = delivery::Content::decode(request, self.limits)? {
            ensure(
                s[0].snapshot_id == action.plan.snapshot_id,
                "speech uses another snapshot",
            )?;
        }
        Ok(())
    }

    pub fn authorize(
        mut self,
        request: &ActionRequestV2,
        authority: &Authority<'_>,
        now_ns: i64,
    ) -> Result<()> {
        ensure(
            self.action().status == ActionStatus::REQUESTED && self.history.registered,
            "not requested",
        )?;
        self.check_authority(request, authority, now_ns)?;
        ensure(
            self.event.stable_source_key == authorization_key(&request.grant, self.limits)?,
            "event is not bound to this issuance",
        )?;
        self.publish(ActionStatus::AUTHORIZED, None, vec![])
    }
}

impl<L: LineageKind> outbox::Outbox for ActionTransaction<'_, L> {
    type Error = Error;
    fn consume(
        &mut self,
        request: &ActionRequestV2,
        authority: &Authority<'_>,
        now_ns: i64,
    ) -> Result<()> {
        ensure(
            self.action().status == ActionStatus::AUTHORIZED,
            "no unspent authorized action",
        )?;
        self.check_authority(request, authority, now_ns)?;
        let maximum = authority
            .settings
            .max_dispatching
            .ok_or(Error::Rejected("concurrency default unset"))?;
        let active = self
            .q
            .inflight
            .iter()
            .filter(|a| {
                matches!(
                    a.status,
                    ActionStatus::DISPATCHING | ActionStatus::UNKNOWN_ACTION
                )
            })
            .count();
        ensure(active < maximum, "concurrency limit reached")?;
        ensure(
            self.history.authorization_key.as_ref()
                == Some(&authorization_key(&request.grant, self.limits)?),
            "wrong authorized issuance",
        )?;
        self.history.check_reservation(&request.grant)?;
        let consumption = PermissionResult {
            schema_version: 1,
            request_id: request.action_id.clone(),
            audit_record_id: self.event.id.clone(),
            dispatch_record_id: Some(request.grant.issuance_id.clone()),
            denied: None,
            management_commit: None,
        };
        let receipt = self.store.prepare(&consumption, "1")?;
        let reservation = self.store.prepare(authority.budget, "1")?;
        self.publish(ActionStatus::DISPATCHING, None, vec![receipt, reservation])
    }

    fn check_frontier(&self) -> Result<()> {
        ensure(self.store.next_raw()?.is_none(), "new ingress before send")
    }
}
