use super::*;

/// Stage P in the owner's new Q, while preserving the activity that selected
/// it. The caller publishes this pin with the originating outer transaction.
pub fn stage_local_plan<L: LineageKind>(
    store: &Store<L>,
    mut q: CognitionState,
    origin: lifecycle::Origin<'_>,
    action_id: String,
    request: ObjectRef,
    registered_at_ns: u64,
    limits: Limits,
) -> Result<PreparedObject<L>> {
    let local = lifecycle::local_plan(origin)?;
    let source = match origin {
        lifecycle::Origin::Local(record) => {
            validate_semantics(record, limits)?;
            store.prepare(record, "1")?
        }
        lifecycle::Origin::Receipt(record) => {
            validate_semantics(record, limits)?;
            store.prepare(record, "1")?
        }
    };
    ensure(
        q.activities.iter().any(|a| {
            a.activity_id == local.binding.activity_id
                && a.latest_step.as_ref() == Some(source.reference())
        }),
        "plan origin is not this activity's accepted step",
    )?;
    ensure(
        !q.inflight
            .iter()
            .any(|a| a.action_id == action_id || &a.origin_record == source.reference()),
        "action ID or planned step already registered",
    )?;
    q.inflight.push(lifecycle::planned(
        origin,
        source.reference().clone(),
        action_id,
        request,
        registered_at_ns,
    )?);
    validate_semantics(&q, limits)?;
    store.prepare(&q, "1")
}

pub(super) fn validate_semantics<T: WireMessage>(value: &T, limits: Limits) -> Result<()> {
    let schema = core_types::contract::Schema::builtin()?;
    let v = schema.decode_canonical(T::NAME, &value.canonical_bytes(limits)?, limits, 0)?;
    core_types::semantic::check_cognition_message(T::NAME, &v, limits)?;
    core_types::semantic::check_speak_think_message(T::NAME, &v, limits)?;
    Ok(())
}

pub(super) fn read_local<L: LineageKind>(
    reader: &core_store::Reader<L>,
    reference: &ObjectRef,
    limits: Limits,
) -> Result<CognitiveResultAndReceipt> {
    match reference.object_type.as_str() {
        "echo.internal-activity-record" => {
            let record: InternalActivityRecord = reader.read(reference)?;
            validate_semantics(&record, limits)?;
            lifecycle::local_plan(lifecycle::Origin::Local(&record))?;
            Ok(record.result)
        }
        "echo.internal-activity-record-v2" => {
            let record: InternalActivityRecordV2 = reader.read(reference)?;
            lifecycle::local_plan(lifecycle::Origin::Receipt(&record))?;
            validate_semantics(&record, limits)?;
            record.local.ok_or(Error::Rejected("missing local result"))
        }
        _ => Err(Error::Rejected("unrecognized action origin")),
    }
}
