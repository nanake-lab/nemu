use core_types::{
    WireMessage,
    contract::{Limits, Result, require},
    generated::*,
};

pub enum Content {
    Body,
    Speech(Vec<OutputChunk>),
}

pub enum Message<'a> {
    Body(&'a ActionRequestV2),
    Chunk(&'a OutputChunk),
}

impl Content {
    pub fn decode(request: &ActionRequestV2, limits: Limits) -> Result<Self> {
        let bytes = &request.action_payload.0;
        if BodyAction::from_canonical(bytes, limits).is_ok() {
            return Ok(Self::Body);
        }
        let speech = PlanAndCarrierReceipt::from_canonical(bytes, limits)?;
        let first = speech.output_chunks.first().ok_or("no speech chunks")?;
        for (index, chunk) in speech.output_chunks.iter().enumerate() {
            require(
                chunk.action_id == request.action_id
                    && chunk.output_id == first.output_id
                    && chunk.chunk_index.0 == index as u64
                    && chunk.snapshot_id == first.snapshot_id
                    && chunk.speech_mode == speech.speech_mode
                    && chunk.strict == speech.strict,
                "speech block binding changed",
            )?;
        }
        Ok(Self::Speech(speech.output_chunks))
    }
}

pub fn body_result(action_id: &str, receipt: &BodyActionResult) -> Result<ActionStatus> {
    require(receipt.action_id == action_id, "wrong body action")?;
    match receipt.status {
        ActionStatus::SUCCEEDED
        | ActionStatus::FAILED
        | ActionStatus::CANCELLED_ACTION
        | ActionStatus::UNKNOWN_ACTION => Ok(receipt.status),
        _ => Err("body receipt is not an execution outcome".into()),
    }
}

pub fn speech_result(
    action_id: &str,
    speech: &PlanAndCarrierReceipt,
    receipt: &SpeechDelivery,
) -> Result<ActionStatus> {
    let first = speech.output_chunks.first().ok_or("no speech blocks")?;
    require(
        receipt.action_id == action_id && receipt.output_id == first.output_id,
        "wrong speech delivery",
    )?;
    let count = receipt.delivered_chunks.0;
    let chunks = speech.output_chunks.len() as u64;
    // The stage reports JS string lengths (UTF-16 units). A partial block can
    // have played characters without being a completely delivered block.
    let max_chars: usize = speech
        .output_chunks
        .iter()
        .map(|c| c.text.encode_utf16().count())
        .sum();
    require(
        count <= chunks && receipt.played_chars.0 <= max_chars as u64,
        "impossible delivery extent",
    )?;
    match receipt.status {
        DeliveryStatus::DELIVERED => {
            require(count == chunks, "incomplete delivery")?;
            Ok(ActionStatus::SUCCEEDED)
        }
        DeliveryStatus::PARTIAL => Ok(ActionStatus::CANCELLED_ACTION),
        DeliveryStatus::CANCELLED_DELIVERY => Ok(ActionStatus::CANCELLED_ACTION),
        DeliveryStatus::UNKNOWN_DELIVERY => Ok(ActionStatus::UNKNOWN_ACTION),
    }
}
