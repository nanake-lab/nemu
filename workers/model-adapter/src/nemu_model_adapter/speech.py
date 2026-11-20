"""Assemble the existing A/C result after the final local stream was observed."""
from __future__ import annotations

from collections.abc import Callable
from hashlib import sha256

from echo.runtime import Limits, normalize
from echo.typed_api import to_value
from echo.types import ObjectRef, OutputChunk, PlanAndCarrierReceipt, SpeechMode

from .carrier import Observation
from .resources import Unmodulated


def speech_result(mode: SpeechMode, chunks: list[OutputChunk], observed: Observation,
                  usage: ObjectRef,
                  record_probe: Callable[[Observation], ObjectRef], limits: Limits) -> PlanAndCarrierReceipt:
    if mode not in (SpeechMode.A_LOCAL_STRICT, SpeechMode.C_DRAFT_REWRITE):
        raise Unmodulated("not a local speech mode")
    drafts = [observed.draft] if observed.draft is not None else []
    if bool(drafts) != (mode == SpeechMode.C_DRAFT_REWRITE):
        raise Unmodulated("draft binding")
    output = sha256()
    for index, chunk in enumerate(chunks):
        if (chunk.snapshot_id != observed.snapshot_id or chunk.speech_mode != mode
                or not chunk.strict or chunk.chunk_index != index
                or (chunk.output_id, chunk.action_id) != (chunks[0].output_id, chunks[0].action_id)):
            raise Unmodulated("speech chunk binding")
        output.update(chunk.text.encode())
    if (output.digest() != observed.output_sha256 or observed.cognitive_input is not None
            or not observed.positions or not observed.forward_batches):
        raise Unmodulated("unobserved speech")
    result = PlanAndCarrierReceipt(1, chunks, None, mode, True, observed.model_hash,
                                   observed.tokenizer_hash, record_probe(observed), None, drafts, usage)
    normalize("PlanAndCarrierReceipt", to_value(result), limits)
    return result
