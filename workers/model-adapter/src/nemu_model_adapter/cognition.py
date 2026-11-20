"""Bind local cognition to its actual input, generated material and observed forward pass."""
from __future__ import annotations

from collections.abc import Callable
from hashlib import sha256

from echo.cognition import check_cognition_message
from echo.runtime import Limits
from echo.typed_api import to_value
from echo.types import CognitiveCompletion, CognitiveInput, CognitiveMaterial, CognitiveResultAndReceipt, ObjectRef

from .carrier import Observation
from .resources import Unmodulated, canonical_ref


def local_result(
    request: CognitiveInput, observed: Observation,
    product: CognitiveMaterial, resolve_product: Callable[[ObjectRef], bytes], usage: ObjectRef,
    record_probe: Callable[[ObjectRef, Observation], ObjectRef], limits: Limits,
) -> CognitiveResultAndReceipt:
    """P5-39 owns scheduling and adoption. Its trusted object writer must persist the
    actual probe and usage before a completed/truncated result can leave the worker.
    """
    value = to_value(request)
    check_cognition_message("CognitiveInput", value, limits)
    actual_input = canonical_ref("CognitiveInput", value, limits)
    binding = request.binding
    hook = observed.key.hook
    generated_text = resolve_product(product.content)
    if (observed.cognitive_input != actual_input or binding.snapshot_id != observed.snapshot_id
            or binding.environment.carrier != hook.carrier or binding.environment.model != hook.model
            or binding.environment.tokenizer != hook.tokenizer
            or binding.environment.direction_set != observed.key.direction_set
            or sha256(generated_text).digest() != observed.output_sha256
            or product.content.byte_length != len(generated_text)
            or observed.positions == 0 or observed.forward_batches == 0):
        raise Unmodulated("cognitive execution binding")
    # The material owner resolves content bytes. The receipt writer receives the actual
    # observation rather than an arbitrary carrier_receipt supplied in the request.
    receipt = record_probe(actual_input, observed)
    result = CognitiveResultAndReceipt(
        1, binding, actual_input, [product], None, [], None, observed.completion,
        "token budget" if observed.completion == CognitiveCompletion.COG_TRUNCATED else None,
        observed.model_hash, observed.tokenizer_hash, observed.key.envelope, receipt, usage)
    check_cognition_message("CognitiveResultAndReceipt", to_value(result), limits)
    return result
