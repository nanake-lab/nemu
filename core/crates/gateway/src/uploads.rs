//! Connection-owned bounded reservations and pins; core-store alone writes files.
use crate::{Error, Result, hooks::Decision};
use core_store::{LineageKind, PreparedObject, Store};
use core_types::{WireMessage, contract::Limits, generated::*, terminal_ingress::*};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

include!(concat!(env!("OUT_DIR"), "/upload_dispatch.rs"));
struct Receiving {
    object: ObjectRef,
    bytes: Vec<u8>,
    started: Instant,
}
struct Complete<L: LineageKind> {
    upload_id: String,
    bytes: Vec<u8>,
    pin: PreparedObject<L>,
    started: Instant,
}
pub(crate) struct Uploads<L: LineageKind> {
    active: BTreeMap<String, Receiving>,
    complete: Vec<Complete<L>>,
}
impl<L: LineageKind> Default for Uploads<L> {
    fn default() -> Self {
        Self {
            active: BTreeMap::new(),
            complete: vec![],
        }
    }
}
impl<L: LineageKind> Uploads<L> {
    pub fn expire(&mut self, lifetime: Duration) {
        self.active.retain(|_, u| u.started.elapsed() < lifetime);
        self.complete.retain(|u| u.started.elapsed() < lifetime);
    }
    pub fn release(&mut self, event: &TerminalEvent) {
        self.complete.retain(|u| {
            u.pin.reference() != &event.payload_ref && u.pin.reference() != &event.provenance
        });
    }
    pub fn read<T: WireMessage>(
        &self,
        store: &Store<L>,
        reference: &ObjectRef,
        limits: Limits,
    ) -> Result<T> {
        if let Some(u) = self
            .complete
            .iter()
            .find(|u| u.pin.reference() == reference)
        {
            return T::from_canonical(&u.bytes, limits).map_err(|_| Error::Contract);
        }
        Ok(store.reader()?.read(reference)?)
    }
    pub fn payload(
        &self,
        store: &Store<L>,
        reference: &ObjectRef,
        limits: Limits,
    ) -> Result<crate::TerminalPayload> {
        macro_rules! one { ($($ty:ty => $v:ident),+ $(,)?) => {
            match reference.object_type.as_str() {
                $(tag if tag == core_types::contract::Schema::builtin().map_err(|_| Error::Contract)?.message(<$ty>::NAME).map_err(|_| Error::Contract)?.tag =>
                    Ok(crate::TerminalPayload::$v(self.read::<$ty>(store, reference, limits)?)),)+
                _ => Err(Error::Contract)
            }
        } }
        one!(UserUtterance => Utterance, UserText => Text, UserRetraction => Retraction,
            TouchObservation => Touch, DesktopObservation => Desktop, CodexSessionObservation => Codex,
            SpeechDelivery => Speech, BodyActionResult => Body, TerminalMaintenance => Maintenance)
    }
    /// Reserve declared object bytes BEFORE taking a first chunk. Completed,
    /// unconsumed objects retain reservations/pins until use, expiry or disconnect.
    /// max_uploads counts simultaneous incomplete uploads, not completed objects.
    pub fn chunk(
        &mut self,
        store: &Store<L>,
        chunk: &ObjectUploadChunk,
        budget: &TerminalBudget,
        limits: Limits,
    ) -> Result<Decision<ObjectUploadReceipt>> {
        if chunk.data.0.len() > budget.max_chunk_bytes as usize
            || chunk.object.byte_length.0 > u64::from(budget.max_object_bytes)
        {
            return Ok(Err(IngressReason::UPLOAD_BUDGET_FULL));
        }
        if check_chunk(chunk, budget, limits).is_err() {
            return Ok(Err(IngressReason::OBJECT_MISMATCH));
        }
        if let Some(done) = self
            .complete
            .iter()
            .find(|u| u.upload_id == chunk.upload_id)
        {
            // A lost final reply may be retried within this live reservation.
            // Reusing its ID for another declaration cannot replace the object.
            if done.pin.reference() != &chunk.object
                || !chunk.final_chunk
                || done.bytes.get(chunk.offset.0 as usize..) != Some(chunk.data.0.as_slice())
            {
                return Ok(Err(IngressReason::OBJECT_MISMATCH));
            }
            return Ok(Ok(ObjectUploadReceipt {
                schema_version: 1,
                upload_id: chunk.upload_id.clone(),
                object: chunk.object.clone(),
                next_offset: core_types::scalar::DecimalU64(done.bytes.len() as u64),
                status: UploadStatus::UPLOAD_COMPLETE,
            }));
        }
        let new = !self.active.contains_key(&chunk.upload_id);
        let active = self.active.len() + usize::from(new);
        let used: u64 = self
            .active
            .values()
            .map(|u| u.object.byte_length.0)
            .chain(
                self.complete
                    .iter()
                    .map(|u| u.pin.reference().byte_length.0),
            )
            .sum();
        let reserved = used.checked_add(if new { chunk.object.byte_length.0 } else { 0 });
        // Deployment parser budget also bounds metadata/counts, independently
        // from object reservations (so tiny objects/huge IDs cannot grow RAM).
        let metadata: usize = self
            .active
            .iter()
            .map(|(id, u)| id.len() + u.object.resource_version.len() + 256)
            .chain(
                self.complete
                    .iter()
                    .map(|u| u.upload_id.len() + u.pin.reference().resource_version.len() + 256),
            )
            .sum();
        if active > budget.max_uploads as usize
            || reserved.is_none_or(|v| v > u64::from(budget.max_upload_bytes))
            || (new
                && (self.complete.len() + self.active.len() >= limits.max_items
                    || metadata
                        .saturating_add(chunk.upload_id.len())
                        .saturating_add(chunk.object.resource_version.len())
                        .saturating_add(256)
                        > limits.max_bytes))
        {
            return Ok(Err(IngressReason::UPLOAD_BUDGET_FULL));
        }
        let window = UploadWindow {
            upload_id: chunk.upload_id.clone(),
            object: self
                .active
                .get(&chunk.upload_id)
                .map(|u| u.object.clone())
                .unwrap_or_else(|| chunk.object.clone()),
            next_offset: self
                .active
                .get(&chunk.upload_id)
                .map_or(0, |u| u.bytes.len() as u64),
            active_uploads: active as u32,
            reserved_bytes: reserved.unwrap(),
        };
        if check_upload_window(chunk, budget, &window, limits).is_err() {
            return Ok(Err(IngressReason::OBJECT_MISMATCH));
        }
        let u = self
            .active
            .entry(chunk.upload_id.clone())
            .or_insert_with(|| Receiving {
                object: chunk.object.clone(),
                bytes: Vec::with_capacity(chunk.object.byte_length.0 as usize),
                started: Instant::now(),
            });
        u.bytes.extend(&chunk.data.0);
        let status = if chunk.final_chunk {
            let u = self.active.remove(&chunk.upload_id).unwrap();
            if check_uploaded_object(&u.object, &u.bytes, budget, limits).is_err() {
                return Ok(Err(IngressReason::OBJECT_MISMATCH));
            }
            let name = upload_type(&u.object.object_type).map_err(|_| Error::Contract)?;
            let pin = match prepare(store, &name, &u.bytes, &u.object.resource_version, limits) {
                Ok(pin) => pin,
                Err(
                    Error::Contract
                    | Error::Store(core_store::Error::Contract(_) | core_store::Error::Rejected(_)),
                ) => return Ok(Err(IngressReason::OBJECT_MISMATCH)),
                Err(e) => return Err(e),
            };
            if pin.reference() != &u.object {
                return Ok(Err(IngressReason::OBJECT_MISMATCH));
            }
            self.complete.push(Complete {
                upload_id: chunk.upload_id.clone(),
                bytes: u.bytes,
                pin,
                started: u.started,
            });
            UploadStatus::UPLOAD_COMPLETE
        } else {
            UploadStatus::UPLOAD_CONTINUE
        };
        Ok(Ok(ObjectUploadReceipt {
            schema_version: 1,
            upload_id: chunk.upload_id.clone(),
            object: chunk.object.clone(),
            next_offset: core_types::scalar::DecimalU64(chunk.offset.0 + chunk.data.0.len() as u64),
            status,
        }))
    }
}
