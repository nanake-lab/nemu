"""S1 synthetic-H views. CanonicalV1 AuditExtension is only an evidence envelope.

This is not a production ThetaSnapshot decoder. The theta object type is the
F contract's resolver key; the payload records precisely F's authorized view.
"""

import hashlib
import json
import struct
from copy import deepcopy
from typing import Any

from arithmetic import bits
from c_oracle import first_difference


def identity(body: dict[str, Any], tag: str) -> dict[str, Any]:
    def blob(v: bytes) -> bytes:
        return struct.pack("<I", len(v)) + v

    payload = json.dumps(body, sort_keys=True, separators=(",", ":")).encode()
    schema = b"echo.audit-extension"
    raw = (
        b"NCV1"
        + struct.pack("<I", 1)
        + blob(schema)
        + blob(tag.encode())
        + struct.pack("<I", 1)
        + blob(payload)
    )
    digest = hashlib.sha256(b"NEMU-CANONICALV1\0" + blob(schema) + raw).hexdigest()
    return dict(
        schema_version=1,
        digest=digest,
        byte_length=str(len(raw)),
        object_type=tag,
        resource_version="P5-71-v2",
    )


def objects(theta: dict[str, Any], event: dict[str, Any]) -> dict[str, Any]:
    return dict(
        theta=dict(
            b=[bits(v) for v in theta["b"]],
            g=[bits(v) for v in theta["g"]],
            lambda_bits=[bits(v) for v in theta["lambda"]],
        ),
        effects=dict(effects=[]),
        interpretation=dict(
            id=event["id"], a=[bits(v) for v in event["target"]], w="3ff0000000000000"
        ),
        governance=dict(
            id=event["id"], root=event["root"], role=event["op"], kind=event["kind"]
        ),
    )


def references(obj: dict[str, Any]) -> dict[str, Any]:
    return dict(
        theta_before=identity(obj["theta"], "echo.theta-snapshot-v3"),
        effects_before=identity(obj["effects"], "p571.effects-view"),
        interpretation_refs=[identity(obj["interpretation"], "p571.interpretation")],
        governance_refs=[identity(obj["governance"], "p571.governance")],
    )


def check(
    expected: dict[str, Any], audit: dict[str, Any]
) -> tuple[str, Any, Any] | None:
    record = audit.get("record") or audit["receipts"][0]
    refs = references(expected["objects"])
    return first_difference(
        expected, audit.get("synthetic_h"), "C02-H-objects"
    ) or first_difference(refs, {k: record[k] for k in refs}, "C02-H-refs")


def wrong_reference_checks(
    expected: dict[str, Any], audit: dict[str, Any]
) -> list[str]:
    assert check(expected, audit) is None, (
        "positive reference must pass before corruption"
    )
    killed = []
    for field in ("theta_before", "effects_before", "interpretation_refs"):
        altered = deepcopy(audit)
        record = altered.get("record") or altered["receipts"][0]
        ref = record[field][0] if isinstance(record[field], list) else record[field]
        ref["digest"] = "00" * 32
        assert check(expected, altered) is not None, field
        killed.append(field)
    return killed
