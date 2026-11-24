use super::*;

/// Rebuilt from the complete immutable prefix. W02 fails on a truncated read.
/// No spent IDs, reservations, or consent are restored from process memory.
pub(super) struct History {
    pub registered: bool,
    pub consent: Option<ActionConsent>,
    pub authorization_key: Option<String>,
    reservations: Vec<(String, ExecutionBudget)>,
    origin: Option<CoreTransactionV2>,
    unchanged: bool,
    frontier: Option<u64>,
}

fn same_state(left: &CoreTransactionV2, right: &CoreTransactionV2) -> bool {
    left.staged_x == right.staged_x
        && left.staged_r == right.staged_r
        && left.staged_theta == right.staged_theta
        && left.staged_l == right.staged_l
        && left.staged_u == right.staged_u
        && left.binding.environment == right.binding.environment
        && left.binding.source_validity_version == right.binding.source_validity_version
}

impl History {
    pub fn read<L: LineageKind>(
        reader: &core_store::Reader<L>,
        head: &str,
        limit: usize,
        action: &CognitiveInflightAction,
    ) -> Result<Self> {
        let mut history = Self {
            registered: false,
            consent: None,
            authorization_key: None,
            reservations: vec![],
            origin: None,
            unchanged: true,
            frontier: None,
        };

        for tx in reader.transactions_at(head, limit)? {
            let mut lifecycle = None;
            let mut consent = None;
            let mut dispatch = None;
            let mut budget = None;
            for r in &tx.audit_refs {
                match r.object_type.as_str() {
                    "echo.action-lifecycle" => {
                        ensure(lifecycle.is_none(), "ambiguous lifecycle in action commit")?;
                        lifecycle = Some(reader.read::<ActionLifecycle>(r)?);
                    }
                    "echo.action-consent" => consent = Some(reader.read::<ActionConsent>(r)?),
                    "echo.permission-result" => {
                        let receipt: PermissionResult = reader.read(r)?;
                        if receipt.dispatch_record_id.is_some() {
                            dispatch = Some(receipt);
                        }
                    }
                    "echo.execution-budget" => budget = Some(reader.read::<ExecutionBudget>(r)?),
                    _ => {}
                }
            }
            if let Some(record) = &lifecycle
                && record.status == ActionStatus::DISPATCHING
            {
                let receipt = dispatch.ok_or(Error::Rejected("missing consumption receipt"))?;
                ensure(
                    receipt.request_id == record.action_id
                        && receipt.audit_record_id == tx.binding.event_id,
                    "misbound consumption receipt",
                )?;
                history.reservations.push((
                    receipt
                        .dispatch_record_id
                        .ok_or(Error::Rejected("no dispatch ID"))?,
                    budget.ok_or(Error::Rejected("missing reservation"))?,
                ));
            }
            if let Some(origin) = &history.origin {
                let own = lifecycle
                    .as_ref()
                    .is_some_and(|a| a.action_id == action.action_id && a.plan == action.plan);
                history.unchanged &= own
                    && same_state(origin, &tx)
                    && history.frontier == Some(tx.binding.raw_log_position.0);
                if own {
                    let event: EventEnvelope = reader.read(&tx.staged_h)?;
                    if event.kind == EventKind::ACTION_REQUEST {
                        history.registered = true;
                    }
                    if let Some(c) = consent {
                        ensure(
                            c.action_id == action.action_id && history.consent.is_none(),
                            "duplicate consent",
                        )?;
                        history.consent = Some(c);
                    }
                    if lifecycle
                        .as_ref()
                        .is_some_and(|a| a.status == ActionStatus::AUTHORIZED)
                    {
                        history.authorization_key = Some(event.stable_source_key);
                    }
                }
            } else if tx.staged_q.object_type == "echo.cognition-state" {
                let q: CognitionState = reader.read(&tx.staged_q)?;
                if let Some(a) = q.inflight.iter().find(|a| a.action_id == action.action_id) {
                    ensure(
                        a.status == ActionStatus::PLANNED
                            && a.origin_record == action.origin_record
                            && a.plan == action.plan
                            && a.request == action.request,
                        "ambiguous original plan",
                    )?;
                    history.origin = Some(tx.clone());
                }
            }
            history.frontier = Some(tx.processed_position.0);
        }
        ensure(history.origin.is_some(), "missing committed plan")?;
        Ok(history)
    }

    pub fn continuation(&self, tx: &CoreTransactionV2) -> Result<()> {
        let origin = self
            .origin
            .as_ref()
            .ok_or(Error::Rejected("unresolved origin"))?;
        ensure(
            self.unchanged
                && self.frontier == Some(tx.binding.raw_log_position.0)
                && same_state(origin, tx),
            "snapshot is not an action-only continuation",
        )
    }

    pub fn check_reservation(&self, g: &ExecutionCredential) -> Result<()> {
        let mut used = 0u64;
        for (issuance, budget) in &self.reservations {
            ensure(issuance != &g.issuance_id, "credential already consumed")?;
            if budget.budget_id == g.budget.budget_id && budget.version == g.budget.version {
                ensure(
                    budget.unit == g.budget.unit && budget.limit == g.budget.limit,
                    "budget version changed meaning",
                )?;
                used = used
                    .checked_add(budget.charge.0)
                    .ok_or(Error::Rejected("spent budget overflow"))?;
            }
        }
        ensure(
            used.checked_add(g.budget.charge.0)
                .is_some_and(|n| n <= g.budget.limit.0),
            "budget exhausted",
        )
    }
}
