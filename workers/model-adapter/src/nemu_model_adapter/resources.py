"""Consume a committed J publication using P5-42's generated CanonicalV1 types."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from hashlib import sha256
from math import isfinite
from typing import Any

from echo.j_resources import check_j_envelope, check_j_resources
from echo.runtime import SCHEMA, Limits, decode_canonical, digest, encode_canonical
from echo.typed_api import from_value, to_value
from echo.types import (
    JDirectionStatus, JHookBinding, Lineage, ModulationEnvelopeV2, ObjectRef, RunStatus,
)


class Unmodulated(ValueError):
    status = RunStatus.CORE_UNMODULATED


@dataclass(frozen=True)
class Publication:
    """Trusted publisher input, separate from the received bytes and object reader.

    P5-19 supplies the committed envelope and genesis. The current direction set and
    validation must already have been authorized against calibration/migration evidence.
    """
    envelope: ObjectRef
    genesis: ObjectRef
    direction_set: ObjectRef
    validation: ObjectRef
    lineage: Lineage


@dataclass(frozen=True)
class CacheKey:
    hook: JHookBinding  # includes model, tokenizer, carrier, layer, dtype and both masks
    direction_set: ObjectRef
    envelope: ObjectRef
    prefix_sha256: bytes
    numeric_config: bytes


@dataclass(frozen=True)
class Modulation:
    key: CacheKey
    snapshot_id: str
    delta: tuple[float, ...]


def canonical_ref(name: str, value: dict[str, Any], limits: Limits) -> ObjectRef:
    return ObjectRef(1, bytes.fromhex(digest(name, value, limits)),
                     SCHEMA["messages"][name]["tag"], value.get("version", "1"),
                     len(encode_canonical(name, value, limits)))


def receive(
    envelope_bytes: bytes, receipt_bytes: bytes, publication: Publication,
    hook: JHookBinding, read: Callable[[ObjectRef], bytes], prefix: bytes,
    numeric_config: bytes, limits: Limits,
) -> Modulation:
    """Resolve, validate and multiply only included columns, in canonical use order."""
    def resolve(name: str, ref: ObjectRef) -> dict[str, Any]:
        raw = read(ref)
        value = decode_canonical(name, raw, limits)
        if canonical_ref(name, value, limits) != ref:
            raise Unmodulated(f"{name} resource binding")
        return value

    try:
        value = decode_canonical("ModulationEnvelopeV2", envelope_bytes, limits)
        envelope: ModulationEnvelopeV2 = from_value("ModulationEnvelopeV2", value, limits)
        if canonical_ref("ModulationEnvelopeV2", value, limits) != publication.envelope:
            raise Unmodulated("uncommitted or stale envelope")
        receipt = decode_canonical("JEffectReceipt", receipt_bytes, limits)
        check_j_envelope(value, receipt, limits)
        if (envelope.direction_set != publication.direction_set
                or receipt["validation"] != to_value(publication.validation)
                or receipt["genesis"] != to_value(publication.genesis)):
            raise Unmodulated("publication resource binding")
        directions = resolve("JDirectionSet", publication.direction_set)
        if directions["hook"] != to_value(hook):
            raise Unmodulated("loaded carrier hook mismatch")
        validation = resolve("JDirectionValidation", publication.validation)
        genesis = resolve("GenesisManifestV3", publication.genesis)
        body = resolve("BodyReadoutParameters",
                       from_value("ObjectRef", receipt["body_parameters"], limits))
        vectors = [resolve("JDirectionVector", from_value("ObjectRef", d["vector"], limits))
                   for d in directions["directions"]]
        check_j_resources(directions, validation, body, receipt, genesis, vectors,
                          publication.lineage == Lineage.LIFE, limits)
        # ADR 0031: nonzero coefficients only in the experiment lineage until P5-31
        # and the user's activation decision. A cancellation in U·a is still nonzero m.
        used = [d for d in receipt["directions"] if d["status"] == JDirectionStatus.J_INCLUDED]
        if publication.lineage != Lineage.EXPERIMENT and any(d["coefficient"] for d in used):
            raise Unmodulated("nonzero modulation outside experiment")
        if validation["fixture"] and publication.lineage == Lineage.LIFE:
            raise Unmodulated("fixture in life lineage")
        columns = {(d["kind"], d.get("tendency_id")): v["values"]
                   for d, v in zip(directions["directions"], vectors, strict=True)}
        delta = [0.0] * hook.width
        for use in used:
            column = columns[use["kind"], use.get("tendency_id")]
            for i, component in enumerate(column):
                delta[i] += use["coefficient"] * component
        if not all(map(isfinite, delta)):
            raise Unmodulated("nonfinite U·a")
        key = CacheKey(hook, publication.direction_set, publication.envelope,
                       sha256(prefix).digest(), numeric_config)
        return Modulation(key, envelope.snapshot_id, tuple(delta))
    except (ValueError, OSError, KeyError, OverflowError) as exc:
        raise Unmodulated(str(exc)) from exc
