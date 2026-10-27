"""P5-34 targeted codec vectors; semantic contextual cases live in Rust tests."""

from __future__ import annotations

import struct
from typing import Any

from fixture_inputs import LIMITS, request, sample, unit


def allocation(q: float = 0.75) -> dict[str, Any]:
    return dict(schema_version=1, tendency_id="t", support=0.5, intensity=1.0,
                share=1.0, ordinal="0", mu=0.75, projection=0.25, actual_retention=q)


def tendency() -> dict[str, Any]:
    return dict(schema_version=1, tendency_id="t",
                concept=dict(schema_version=1, concept_id="c", registry_version="v"),
                predicate=[], direction=unit(1.0, 0.0, 0.0, 0.0),
                evidence_a=-1.0, evidence_b=5.0, evidence_absolute=1.0,
                h_hat=0.1, group_count="2", flip_count="0", last_sign=1,
                pi=-0.2, typical_magnitude=0.25, magnitude_sum=0.5, magnitude_weight=2.0)


def legal_retention() -> list[dict[str, Any]]:
    a, t = allocation(), tendency()
    teaching = {k: v for k, v in allocation(1.0).items() if k != "projection"}
    teaching["support"] = 0.0
    tiny = t | dict(evidence_a=2**-53, evidence_absolute=2**-53, evidence_b=4.0,
                    pi=2**-55, magnitude_sum=5e-324, magnitude_weight=0.0,
                    typical_magnitude=0.0)
    return [request("EvidenceAllocationV3", a),
            request("EvidenceAllocationV3", teaching),
            request("EvidenceAllocationV3", allocation(5e-324)),
            request("TendencyV3", t), request("TendencyV3", tiny),
            request("EvidenceGroupV3", sample("EvidenceGroupV3", 0) | dict(allocations=[a])),
            request("RSnapshotV3", sample("RSnapshotV3", 0) | dict(tendencies=[t]))]


def illegal_retention() -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []

    def bad(label: str, name: str, value: dict[str, Any]) -> None:
        out.append(dict(label=label, request=request(name, value)))

    a, t = allocation(), tendency()
    for q in (0.0, -0.1, 1.0000000000000002):
        bad(f"P5-34 q outside (0,1]: {q}", "EvidenceAllocationV3", a | dict(actual_retention=q))
    bad("P5-34 missing q", "EvidenceAllocationV3", {k: v for k, v in a.items() if k != "actual_retention"})
    for v in (-1.0, 0.9999999999999999):
        for signed in (-1.0, 1.0):
            bad(f"P5-34 V below |A|: {v}, {signed}", "TendencyV3", t | dict(evidence_absolute=v, evidence_a=signed))
    bad("P5-34 missing V", "TendencyV3", {k: v for k, v in t.items() if k != "evidence_absolute"})
    bad("P5-34 duplicate V3 allocations", "EvidenceGroupV3", sample("EvidenceGroupV3", 0) | dict(allocations=[a, a]))
    bad("P5-34 duplicate V3 tendencies", "RSnapshotV3", sample("RSnapshotV3", 0) | dict(tendencies=[t, t]))
    for container, field, old in (("EvidenceGroupV3", "allocations", "EvidenceAllocationV2"),
                                  ("RSnapshotV3", "tendencies", "TendencyV2")):
        bad("P5-34 superseded nested payload", container, sample(container, 0) | {field: [sample(old, 0)]})
    # Reserved container fields are rejected even when they carry empty nested bytes.
    for name, numbers in (("EvidenceGroupV3", (7, 8)), ("RSnapshotV3", (4, 5))):
        for number in numbers:
            out.append(dict(label=f"P5-34 reserved {name}.{number}", request=dict(
                name=name, op="protobuf", hex=(b"\x08\x01" + bytes([(number << 3) | 2, 0])).hex(), limits=LIMITS)))
    # Complete allocation message: only q is invalid, rather than missing another required field.
    prefix = (b"\x08\x01\x12\x01t\x19" + struct.pack("<d", 0.5) +
              b"\x21" + struct.pack("<d", 1.0) + b"\x29" + struct.pack("<d", 1.0) +
              b"\x30\x00\x39" + struct.pack("<d", 0.75) + b"\x41" + struct.pack("<d", 0.25))
    for bits in (0x7FF8000000000001, 0x7FF0000000000000, 0xFFF0000000000000):
        out.append(dict(label=f"P5-34 nonfinite q {bits:x}", request=dict(
            name="EvidenceAllocationV3", op="protobuf", hex=(prefix + b"\x49" + struct.pack("<Q", bits)).hex(), limits=LIMITS)))
    # Complete tendency with the targeted invalid V at the end.
    concept = b"\x08\x01\x12\x01c\x1a\x01v"
    direction = b"\x08\x01" + b"".join(bytes([tag]) + struct.pack("<d", value)
        for tag, value in ((0x11, 1.0), (0x19, 0.0), (0x21, 0.0), (0x29, 0.0)))
    prefix_v = (b"\x08\x01\x12\x01t\x1a" + bytes([len(concept)]) + concept +
                b"\x2a" + bytes([len(direction)]) + direction +
                b"\x31" + struct.pack("<d", -1.0) + b"\x39" + struct.pack("<d", 5.0) +
                b"\x41" + struct.pack("<d", 0.1) + b"\x48\x02\x50\x00\x58\x01" +
                b"\x61" + struct.pack("<d", -0.2) + b"\x69" + struct.pack("<d", 0.25) +
                b"\x81\x01" + struct.pack("<d", 0.5) + b"\x89\x01" + struct.pack("<d", 2.0))
    for bits in (0x7FF8000000000001, 0x7FF0000000000000, 0xFFF0000000000000):
        out.append(dict(label=f"P5-34 nonfinite V {bits:x}", request=dict(
            name="TendencyV3", op="protobuf", hex=(prefix_v + b"\x91\x01" + struct.pack("<Q", bits)).hex(), limits=LIMITS)))
    return out
