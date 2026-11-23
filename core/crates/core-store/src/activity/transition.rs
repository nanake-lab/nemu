use super::*;
pub(crate) fn verify_transition(
    objects: &Objects,
    old: &CoreTransactionV4,
    new: &CoreTransactionV4,
) -> Result<()> {
    let limits = objects.config.encoding;
    check_speak_think_message(CoreTransactionV4::NAME, &serde_json::to_value(new)?, limits)?;
    let c = new
        .activity_commit
        .as_ref()
        .ok_or(Error::Rejected("missing internal origin"))?;
    check_record(objects, &c.activity)?;
    require(
        super::valid_product(objects, &c.activity)?,
        "invalid accepted UTF-8 product",
    )?;
    require(
        objects.reference(&c.activity, "1")? == new.staged_h
            && new.event_binding.is_none()
            && new.staged_r == old.staged_r
            && new.staged_theta == old.staged_theta
            && new.staged_l == old.staged_l
            && new.staged_u == old.staged_u
            && new.staged_ledger.is_empty()
            && new.staged_concepts.is_empty()
            && new.accepted_results == c.interpretations,
        "internal publication has unauthorized state or results",
    )?;
    check_interpretations(objects, c)?;
    let x = FastStateSnapshotV2::from_canonical(&objects.read(&new.staged_x)?, limits)?;
    require(
        x.anchor_time_ns == new.anchor_time_ns,
        "internal x anchor mismatch",
    )?;
    let mut dispatched = old_input(objects, &old.staged_q)?;
    let origin = old
        .event_binding
        .as_ref()
        .map(|b| &b.base_commit)
        .or_else(|| old.activity_commit.as_ref().map(|p| &p.binding.base_commit))
        .ok_or(Error::Rejected("missing previous origin"))?;
    require(
        dispatched.binding.base_commit == c.binding.base_commit
            || &dispatched.binding.base_commit == origin,
        "prior registration does not bind this dispatch",
    )?;
    dispatched
        .binding
        .base_commit
        .clone_from(&c.binding.base_commit);
    let dispatched_ref = objects.reference(&dispatched, "1")?;
    require(
        dispatched.binding.canonical_bytes(limits)? == c.binding.canonical_bytes(limits)?
            && new.audit_refs.contains(&dispatched_ref)
            && c.activity
                .local
                .as_ref()
                .is_none_or(|r| r.input == dispatched_ref),
        "internal commit does not match the registered input",
    )?;
    let mut before = CognitionState::from_canonical(&objects.read(&old.staged_q)?, limits)?;
    let after = CognitionState::from_canonical(&objects.read(&new.staged_q)?, limits)?;
    for state in [&before, &after] {
        check_cognition_message(CognitionState::NAME, &serde_json::to_value(state)?, limits)?;
    }
    require(
        before.mode == CognitiveMode::COG_RUNNING
            && before.revision == c.binding.q_revision
            && after.revision != before.revision
            && after.active_request.is_none(),
        "invalid continuation revision or mode",
    )?;
    let next = after
        .runnable_successor
        .as_ref()
        .ok_or(Error::Rejected("missing registered continuation"))?;
    let next_input = CognitiveInput::from_canonical(&objects.read(next)?, limits)?;
    check_cognition_message(
        CognitiveInput::NAME,
        &serde_json::to_value(&next_input)?,
        limits,
    )?;
    require(
        next_input.binding.q_revision == after.revision
            && next_input.binding.base_commit == c.binding.base_commit
            && next_input.binding.branch_id == c.binding.branch_id
            && next_input.binding.activity_id == c.binding.activity_id
            && next_input.binding.request_id != c.binding.request_id
            && next_input.binding.step_id != c.binding.step_id
            && c.binding.round_index.0.checked_add(1) == Some(next_input.binding.round_index.0),
        "successor does not identify the next round",
    )?;
    let work = before
        .activities
        .iter_mut()
        .find(|a| a.activity_id == c.binding.activity_id)
        .ok_or(Error::Rejected("unknown activity"))?;
    require(
        c.activity.previous == work.latest_step,
        "broken activity previous link",
    )?;
    let successor = after
        .activities
        .iter()
        .find(|a| a.activity_id == c.binding.activity_id)
        .ok_or(Error::Rejected("lost activity"))?;
    let products: &[CognitiveMaterial] = match (&c.activity.local, &c.activity.remote) {
        (Some(local), None) => {
            require(
                matches!(
                    local.completion,
                    CognitiveCompletion::COG_COMPLETED | CognitiveCompletion::COG_TRUNCATED
                ),
                "failed local result cannot advance Q",
            )?;
            if let Some(focus) = &local.next_focus {
                work.focus = Some(focus.clone());
            }
            if let Some(plan) = &local.action_plan {
                let mut additions = after
                    .inflight
                    .iter()
                    .filter(|a| a.origin_record == new.staged_h);
                let action = additions
                    .next()
                    .ok_or(Error::Rejected("missing registered action"))?;
                require(
                    additions.next().is_none()
                        && action.plan == *plan
                        && action.origin_step_id == c.binding.step_id
                        && action.round_index == c.binding.round_index
                        && action.registered_at_ns == new.anchor_time_ns
                        && action.status == ActionStatus::PLANNED
                        && action.credential.is_none()
                        && action.actual_result.is_none(),
                    "action differs from local proposal",
                )?;
                before.inflight.push(action.clone());
            }
            &local.products
        }
        (None, Some(remote)) => {
            require(
                matches!(
                    remote.adoption,
                    TextAdoption::TEXT_ADOPTION_FULL | TextAdoption::TEXT_ADOPTION_RANGE
                ),
                "unadopted remote return cannot be a thought",
            )?;
            remote.product.as_slice()
        }
        _ => return Err(Error::Rejected("invalid result origin")),
    };
    work.latest_step = Some(new.staged_h.clone());
    work.revision = after.revision.clone();
    work.successor = Some(next.clone());
    work.materials = next_input
        .materials
        .iter()
        .map(|m| m.content.clone())
        .collect();
    require(
        work.focus == next_input.focus
            && products.iter().all(|p| next_input.materials.contains(p))
            && work.materials == successor.materials,
        "continuation loses product or changes focus",
    )?;
    // A continuation may retain a bounded subset of the prior materials. A
    // remote writer cannot inject queries, a new focus, a tool, or other input.
    if c.activity.remote.is_some() {
        let prior = &dispatched;
        require(
            next_input.binding.environment == c.binding.environment
                && next_input.binding.protocol == c.binding.protocol
                && next_input.binding.permission_snapshot == c.binding.permission_snapshot
                && next_input.binding.source_validity_version == c.binding.source_validity_version
                && next_input.binding.read_set == c.binding.read_set
                && next_input.binding.input_frontier == c.binding.input_frontier
                && next_input.budget == dispatched.budget,
            "cloud text changed the authorized execution context",
        )?;
        require(
            next_input.new_inputs.is_empty()
                && next_input.results.is_empty()
                && next_input.retrievals.is_empty()
                && next_input
                    .materials
                    .iter()
                    .all(|m| products.contains(m) || prior.materials.contains(m)),
            "remote return supplied controls or materials",
        )?;
    }
    before.revision = after.revision.clone();
    before.active_request = None;
    before.runnable_successor = Some(next.clone());
    require(
        before.canonical_bytes(limits)? == after.canonical_bytes(limits)?,
        "out-of-scope Q change",
    )
}
fn old_input(objects: &Objects, q: &ObjectRef) -> Result<CognitiveInput> {
    let state = CognitionState::from_canonical(&objects.read(q)?, objects.config.encoding)?;
    let r = state
        .active_request
        .or(state.runnable_successor)
        .ok_or(Error::Rejected("missing prior request"))?;
    Ok(CognitiveInput::from_canonical(
        &objects.read(&r)?,
        objects.config.encoding,
    )?)
}

/// The existing EV6 interpretation types identify their subject separately
/// from their computation binding. Check both before publish and on replay.
fn check_interpretations(objects: &Objects, commit: &InternalActivityCommitV2) -> Result<()> {
    let schema = core_types::contract::Schema::builtin()?;
    let kinds = [
        NatureAppraisalRecord::NAME,
        SituationMatchRecord::NAME,
        TendencyCandidateRecord::NAME,
        TeachingAnnotation::NAME,
        LieAnnotation::NAME,
        SameMatterAnnotation::NAME,
    ];
    for r in &commit.interpretations {
        let name = kinds
            .iter()
            .find(|n| schema.message(n).is_ok_and(|m| m.tag == r.object_type))
            .ok_or(Error::Rejected("unexpected internal interpretation type"))?;
        let value = schema.decode_canonical(name, &objects.read(r)?, objects.config.encoding, 0)?;
        let binding: ProposalBindingV2 = serde_json::from_value(value["binding"].clone())?;
        require(
            value["event_id"].as_str() == Some(&commit.activity.record_id)
                && binding.branch_id == commit.binding.branch_id
                && binding.base_commit == commit.binding.base_commit
                && binding.environment == commit.binding.environment
                && binding.source_validity_version == commit.binding.source_validity_version,
            "interpretation belongs to a different activity or computation",
        )?;
    }
    Ok(())
}
