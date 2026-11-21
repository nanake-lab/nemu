"""Process-local P5-11 appraisal boundary. The caller owns source admission and commit."""

import asyncio
import math
from collections.abc import Awaitable, Callable
from copy import deepcopy
from dataclasses import dataclass
from enum import Enum
from typing import cast

from echo.runtime import (
    SCHEMA,
    Limits,
    canonical_scalar,
    digest,
    encode_canonical,
    scalar,
)
from echo.typed_api import from_value, to_value
from echo.types import (
    EventEnvelope,
    EventKind,
    FactProjection,
    NatureAppraisalRecord,
    NatureVector,
    ObjectRef,
    ProposalBindingV2,
    SemanticGraph,
    UserText,
    UserUtterance,
)


@dataclass(frozen=True, slots=True)
class AppraisalView:
    """EV6. Only these three fields reach the local evaluator."""

    z: SemanticGraph
    c_core: FactProjection
    persons: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class LocalAppraiser:
    """Loaded resource and callable supplied by the caller; no default evaluator."""

    resource: ObjectRef
    producer: str
    producer_version: str
    evaluate: Callable[[AppraisalView], Awaitable[dict[str, float | None]]]


class AppraisalFailure(Enum):
    UNSUPPORTED_SOURCE = "unsupported_source"
    INVALID_INPUT = "invalid_input"
    BINDING_MISMATCH = "binding_mismatch"
    UNAVAILABLE = "unavailable"
    TIMEOUT = "timeout"
    INVALID_OUTPUT = "invalid_output"


@dataclass(frozen=True, slots=True)
class IncompleteAppraisal:
    """U4. Explicit unknowns leave the other known coordinates available for inspection only."""

    known: dict[str, float]


type AppraisalResult = NatureAppraisalRecord | IncompleteAppraisal | AppraisalFailure

# U3, first batch. The gateway stores canonical type tags in EventEnvelope.payload_ref.
_SUPPORTED = {
    SCHEMA["messages"][cls.__name__]["tag"] for cls in (UserText, UserUtterance)
}
_CHANNELS = {
    f["name"]: f
    for f in SCHEMA["messages"]["NatureVector"]["fields"]
    if f["name"] != "schema_version"
}


async def appraise(
    event: EventEnvelope,
    binding: ProposalBindingV2,
    view: AppraisalView | None,
    local: LocalAppraiser | None,
    *,
    record_id: str,
    limits: Limits,
    deadline: float | None = None,
) -> AppraisalResult:
    """Appraise the current, already admitted event. Deadline is on the running loop's clock.

    The trusted caller supplies its governed z, fact projection, involved person IDs and
    frozen binding. The local callable sees a detached view and cannot supply record metadata.
    """
    loop = asyncio.get_running_loop()
    try:
        encode_canonical("EventEnvelope", to_value(event), limits)
        if (
            event.id != binding.event_id
            or event.raw_log_position != binding.raw_log_position
            or event.payload_digest != event.payload_ref.digest
        ):
            return AppraisalFailure.BINDING_MISMATCH
        if (
            event.kind != EventKind.OBSERVATION
            or event.payload_ref.object_type not in _SUPPORTED
        ):
            return AppraisalFailure.UNSUPPORTED_SOURCE
        if (
            view is None
            or not record_id
            or (deadline is not None and not math.isfinite(deadline))
        ):
            return AppraisalFailure.INVALID_INPUT
        if local is None:
            return AppraisalFailure.UNAVAILABLE

        # Freeze the entire envelope before yielding. Only the core can check its current head.
        binding = deepcopy(binding)
        encode_canonical("ProposalBindingV2", to_value(binding), limits)
        z, facts = to_value(view.z), to_value(view.c_core)
        if (
            binding.environment.nature_appraiser is None
            or binding.environment.nature_appraiser != local.resource
            or binding.c_core_digest
            != bytes.fromhex(digest("FactProjection", facts, limits))
            or binding.base_commit != view.c_core.history_prefix
            or binding.accepted_z_digest
            != bytes.fromhex(digest("SemanticGraph", z, limits))
        ):
            return AppraisalFailure.BINDING_MISMATCH
        persons = view.persons
        if (
            type(persons) is not tuple
            or len(persons) > limits.max_items
            or any(
                type(p) is not str or not p or len(p.encode()) > limits.max_bytes
                for p in persons
            )
            or len(set(persons)) != len(persons)
            or not local.producer
            or not local.producer_version
        ):
            return AppraisalFailure.INVALID_INPUT
        # P5-21 defines persons as a set. Present the same canonical order to every callback.
        persons = tuple(
            sorted(
                persons, key=lambda person: canonical_scalar("string", person, limits)
            )
        )
        # Materialize only declared fields, including nested DTOs; never forward a caller's subclass.
        isolated = AppraisalView(
            from_value("SemanticGraph", z, limits),
            from_value("FactProjection", facts, limits),
            persons,
        )
    except (AttributeError, ValueError, TypeError, OverflowError):
        return AppraisalFailure.INVALID_INPUT

    if deadline is not None and loop.time() >= deadline:
        return AppraisalFailure.TIMEOUT
    try:
        async with asyncio.timeout_at(deadline):
            reply = await local.evaluate(isolated)
    except TimeoutError:
        return AppraisalFailure.TIMEOUT
    except Exception:
        return AppraisalFailure.UNAVAILABLE
    if deadline is not None and loop.time() >= deadline:
        return AppraisalFailure.TIMEOUT

    # The local test seam uses None for an assessed unknown. Omitted/extra fields are malformed.
    try:
        if type(reply) is not dict or reply.keys() != _CHANNELS.keys():
            return AppraisalFailure.INVALID_OUTPUT
        known = {
            key: float(scalar(field, reply[key], limits, 0))
            for key, field in _CHANNELS.items()
            if reply[key] is not None
        }
        if len(known) != len(_CHANNELS):
            return IncompleteAppraisal(known)
        vector = cast(
            NatureVector,
            from_value("NatureVector", {"schema_version": 1, **known}, limits),
        )
        record = NatureAppraisalRecord(
            1,
            record_id,
            event.id,
            vector,
            list(persons),
            binding,
            local.producer,
            local.producer_version,
        )
        encode_canonical("NatureAppraisalRecord", to_value(record), limits)
        return record
    except (ValueError, TypeError, OverflowError):
        return AppraisalFailure.INVALID_OUTPUT
