//! The writer admits an explicitly dispatched request, then rechecks its head
//! immediately before a single H/x/Q/outbox publication. No scheduler or I/O to a model.
use crate::{
    CommitCapability, Error, FaultPoint, LineageKind, PreparedObject, Result, Store,
    error::require, objects::Objects, publication, schema::integer,
};
use core_types::{
    WireMessage,
    generated::*,
    scalar::hex,
    semantic::{check_cognition_message, check_speak_think_message, resolved_text_range},
};
use rusqlite::params;
use std::sync::Arc;
mod transition;
pub(crate) use transition::verify_transition;

pub struct ActivityTicket<L: LineageKind> {
    input: CognitiveInput,
    object: PreparedObject<L>,
    previous: Option<ObjectRef>,
}
impl<L: LineageKind> ActivityTicket<L> {
    pub fn input(&self) -> &CognitiveInput {
        &self.input
    }
    pub fn reference(&self) -> &ObjectRef {
        self.object.reference()
    }
    pub fn previous(&self) -> Option<&ObjectRef> {
        self.previous.as_ref()
    }
}
#[derive(Debug, PartialEq)]
pub enum ActivityOutcome {
    Published(ObjectRef),
    AuditOnly,
}

pub(crate) fn binding(record: &InternalActivityRecordV2) -> Result<&CognitionBinding> {
    match (&record.local, &record.remote) {
        (Some(local), None) => Ok(&local.binding),
        (None, Some(remote)) => Ok(&remote.binding),
        _ => Err(Error::Rejected("expected exactly one cognition result")),
    }
}
pub(crate) fn check_record(objects: &Objects, record: &InternalActivityRecordV2) -> Result<()> {
    check_speak_think_message(
        InternalActivityRecordV2::NAME,
        &serde_json::to_value(record)?,
        objects.config.encoding,
    )?;
    let Some(remote) = &record.remote else {
        return Ok(());
    };
    if let Some(response) = &remote.raw_response {
        require(
            response.object_type == crate::RAW_BYTES,
            "response requires an exact byte resource",
        )?;
    }
    if let Some(request) = &remote.sent_request {
        require(
            request.object_type == crate::RAW_BYTES
                && Some(&request.digest) == remote.request_sha256.as_ref(),
            "actual request identity mismatch",
        )?;
        crate::raw::request_body(&objects.read(request)?)?;
    }
    Ok(())
}
pub(crate) fn valid_product(objects: &Objects, record: &InternalActivityRecordV2) -> Result<bool> {
    let Some(remote) = &record.remote else {
        return Ok(true);
    };
    let Some(r) = &remote.returned_text else {
        return Ok(true);
    };
    if r.object_type != crate::RAW_BYTES {
        return Ok(false);
    }
    let body = objects.read(r)?;
    let Ok(text) = std::str::from_utf8(&body) else {
        return Ok(false);
    };
    if let (Some(range), Some(product)) = (&remote.accepted_range, &remote.product) {
        if product.content.object_type != crate::RAW_BYTES {
            return Ok(false);
        }
        let Ok(slice) = resolved_text_range(text, range) else {
            return Ok(false);
        };
        return Ok(slice.as_bytes() == objects.read(&product.content)?);
    }
    Ok(true)
}

impl<L: LineageKind> Store<L> {
    /// Resolve the registered input in Q. Its own registration may advance the
    /// commit once. Only base_commit changes at dispatch, never the read set,
    /// Q revision, permission, frontier, request identity, or actual material.
    pub fn begin_activity(&self) -> Result<ActivityTicket<L>> {
        require(
            crate::migration::format(&self.db)? == 3 && self.next_raw()?.is_none(),
            "activity requires a settled format 3 head",
        )?;
        let current = self
            .current_manifest()?
            .ok_or(Error::Rejected("missing activity base"))?;
        let head = self.base_commit()?;
        let parent = current
            .event_binding
            .as_ref()
            .map(|b| &b.base_commit)
            .or_else(|| {
                current
                    .activity_commit
                    .as_ref()
                    .map(|c| &c.binding.base_commit)
            })
            .ok_or(Error::Rejected("missing origin"))?;
        let state = CognitionState::from_canonical(
            &self.objects.read(&current.staged_q)?,
            self.objects.config.encoding,
        )?;
        let limits = self.objects.config.encoding;
        check_cognition_message(CognitionState::NAME, &serde_json::to_value(&state)?, limits)?;
        let request = state
            .active_request
            .as_ref()
            .or(state.runnable_successor.as_ref())
            .ok_or(Error::Rejected("cognition has no runnable request"))?;
        let mut input = CognitiveInput::from_canonical(&self.objects.read(request)?, limits)?;
        check_cognition_message(CognitiveInput::NAME, &serde_json::to_value(&input)?, limits)?;
        let b = &input.binding;
        require(
            b.branch_id == self.branch
                && b.q_revision == state.revision
                && (b.base_commit == head
                    || (&b.base_commit == parent && {
                        let snapshot = self.objects.reference(&current, "1")?;
                        hex(&snapshot.digest.0) == head
                    })),
            "scheduled input no longer describes this head",
        )?;
        let activity = state
            .activities
            .iter()
            .find(|a| a.activity_id == b.activity_id)
            .ok_or(Error::Rejected("missing active work state"))?;
        input.binding.base_commit = head;
        let object = self.prepare(&input, "1")?;
        Ok(ActivityTicket {
            input,
            object,
            previous: activity.latest_step.clone(),
        })
    }

    /// An old callback can arrive after the store has reopened. Reconstruct its
    /// original dispatch from that immutable head, never bind it to today's Q.
    pub fn audit_late_activity(
        &mut self,
        cap: &CommitCapability<L>,
        record: &InternalActivityRecordV2,
    ) -> Result<()> {
        require(
            Arc::ptr_eq(&cap.owner, &self.objects.pins),
            "foreign audit capability",
        )?;
        check_record(&self.objects, record)?;
        let b = binding(record)?;
        require(
            b.branch_id == self.branch
                && (b.base_commit != self.base_commit()? || self.next_raw()?.is_some()),
            "current return belongs to normal publication",
        )?;
        let reader = self.verified_reader()?;
        let r = reader
            .snapshot_at(&b.base_commit)?
            .ok_or(Error::Rejected("unknown dispatch base"))?;
        let history = crate::manifest::decode(&self.objects, &r)?;
        let q: CognitionState = reader.read(&history.staged_q)?;
        let authorized = q
            .active_request
            .as_ref()
            .or(q.runnable_successor.as_ref())
            .ok_or(Error::Rejected("no historical dispatch"))?;
        let mut input: CognitiveInput = reader.read(authorized)?;
        let old_base = history
            .event_binding
            .as_ref()
            .map(|v| &v.base_commit)
            .or_else(|| {
                history
                    .activity_commit
                    .as_ref()
                    .map(|c| &c.binding.base_commit)
            });
        require(
            input.binding.base_commit == b.base_commit
                || (hex(&r.digest.0) == b.base_commit
                    && Some(&input.binding.base_commit) == old_base),
            "historical input registration is stale",
        )?;
        input.binding.base_commit.clone_from(&b.base_commit);
        require(
            input
                .binding
                .canonical_bytes(self.objects.config.encoding)?
                == b.canonical_bytes(self.objects.config.encoding)?
                && q.revision == b.q_revision
                && q.activities
                    .iter()
                    .any(|a| a.activity_id == b.activity_id && a.latest_step == record.previous)
                && record.local.as_ref().is_none_or(|l| {
                    self.objects
                        .reference(&input, "1")
                        .is_ok_and(|r| r == l.input)
                }),
            "unbound historical result",
        )?;
        drop(reader);
        let h = self.prepare(record, "1")?;
        crate::activity_audit::record(self, &h, &b.request_id, "stale")
    }

    /// The trusted D8 reviewer independently checks the archived review against
    /// the exact candidate, sources and returned text. Only its positive decision
    /// can enter the builder. Replays read the committed verdict and never rerun it.
    pub fn finish_activity(
        &mut self,
        cap: &CommitCapability<L>,
        ticket: &ActivityTicket<L>,
        record: InternalActivityRecordV2,
        review_d8: impl FnOnce(&RemoteCognitiveReceipt, &[u8], &str) -> Result<bool>,
        build: impl FnOnce(
            &Self,
            &ObjectRef,
            &CoreTransactionV4,
        ) -> Result<(CoreTransactionV4, Vec<PreparedObject<L>>)>,
    ) -> Result<ActivityOutcome> {
        require(
            Arc::ptr_eq(&cap.owner, &self.objects.pins)
                && Arc::ptr_eq(&ticket.object.pins, &self.objects.pins),
            "foreign cognition capability",
        )?;
        check_record(&self.objects, &record)?;
        let b = binding(&record)?;
        let limits = self.objects.config.encoding;
        require(
            b.canonical_bytes(limits)? == ticket.input.binding.canonical_bytes(limits)?
                && record.previous == ticket.previous
                && record
                    .local
                    .as_ref()
                    .is_none_or(|r| &r.input == ticket.reference()),
            "return does not match dispatch",
        )?;
        let h = self.prepare(&record, "1")?;
        let already_audited: bool = self.db.query_row("SELECT EXISTS(SELECT 1 FROM cognitive_attempts WHERE request_id=?1 UNION ALL SELECT 1 FROM activity_keys WHERE request_id=?1)", [&b.request_id], |r| r.get(0))?;
        let reason = if !valid_product(&self.objects, &record)? {
            Some("invalid-content")
        } else if b.base_commit != self.base_commit()?
            || self.next_raw()?.is_some()
            || already_audited
        {
            Some("stale")
        } else if let Some(r) = &record.remote {
            match r.adoption {
                TextAdoption::TEXT_ADOPTION_STALE => Some("stale"),
                TextAdoption::TEXT_ADOPTION_REJECTED => Some("rejected"),
                TextAdoption::TEXT_ADOPTION_NO_TEXT => Some("no-text"),
                _ => {
                    let review = r
                        .review
                        .as_ref()
                        .ok_or(Error::Rejected("missing D8 review"))?;
                    let product = r
                        .product
                        .as_ref()
                        .ok_or(Error::Rejected("missing accepted product"))?;
                    let text = self.objects.read(&product.content)?;
                    let text = std::str::from_utf8(&text)
                        .map_err(|_| Error::Rejected("invalid product UTF-8"))?;
                    match review_d8(r, &self.objects.read(review)?, text) {
                        Ok(true) => None,
                        Ok(false) => Some("rejected"),
                        Err(error) => {
                            crate::activity_audit::record(
                                self,
                                &h,
                                &b.request_id,
                                "review-unavailable",
                            )?;
                            return Err(error);
                        }
                    }
                }
            }
        } else if record.local.as_ref().is_some_and(|l| {
            matches!(
                l.completion,
                CognitiveCompletion::COG_FAILED | CognitiveCompletion::COG_CANCELLED
            )
        }) {
            Some("incomplete")
        } else {
            None
        };
        if let Some(why) = reason {
            crate::activity_audit::record(self, &h, &b.request_id, why)?;
            return Ok(ActivityOutcome::AuditOnly);
        }
        // The deployed m=0 interpreter and F stage do not yet issue a trusted
        // internal transition. A synthetic state builder is only admissible
        // in an explicitly temporary database (ADR 0010, AT12).
        let temporary: bool = self.db.query_row(
            "SELECT temporary FROM metadata WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        require(
            temporary,
            "internal interpretation and F have not been qualified for persistent lineages",
        )?;
        let old = self
            .current_manifest()?
            .ok_or(Error::Rejected("missing published base"))?;
        let (value, _pins) = build(self, h.reference(), &old)?;
        let c = value
            .activity_commit
            .as_ref()
            .ok_or(Error::Rejected("missing activity commit"))?;
        require(
            value.staged_h == *h.reference()
                && c.activity == record
                && c.binding == *b
                && value.processed_position.0 == self.head()?.1
                && value.anchor_time_ns.0 >= self.head()?.2,
            "activity transaction does not match return or frontier",
        )?;
        verify_transition(&self.objects, &old, &value)?;
        let manifest = self.prepare(&value, "1")?;
        let root = manifest.reference();
        let id = hex(&root.digest.0);
        let refs = self.objects.closure(root)?;
        let bytes = root.canonical_bytes(limits)?;
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let position = crate::store::next_commit_position(&tx)?;
        publication::visible(&tx, &self.objects, &refs)?;
        tx.execute(
            "INSERT INTO commits VALUES(?1,?2,?3,?4,?5,'activity')",
            params![
                id,
                b.base_commit,
                position,
                integer(value.anchor_time_ns.0)?,
                bytes
            ],
        )?;
        tx.execute("INSERT INTO snapshots VALUES(?1,?2)", params![id, bytes])?;
        publication::index(&tx, &self.objects, &id, &value, &refs)?;
        require(tx.execute("UPDATE branch_head SET commit_id=?1,anchor=?2,snapshot=?3 WHERE singleton=1 AND commit_id=?4 AND processed=?5",params![id,integer(value.anchor_time_ns.0)?,bytes,b.base_commit,integer(value.processed_position.0)?])? == 1, "activity lost its base")?;
        self.fault.hit(FaultPoint::BeforePublish)?;
        tx.commit()?;
        self.fault.hit(FaultPoint::SqlCommitted)?;
        crate::witness::record(
            &self.root,
            "commits",
            position as u64,
            root,
            &self.objects,
            &self.fault,
        )?;
        self.fault.hit(FaultPoint::AfterPublish)?;
        Ok(ActivityOutcome::Published(root.clone()))
    }
}
