"""P6-17: dwell identity/duration and strict separation from frozen touch bytes."""

from __future__ import annotations

import struct
from typing import Any

from fixture_inputs import LIMITS, request


def legal_touch() -> list[dict[str, Any]]:
    # ADR 0025 §4.3: one contact, three reports; durations are fixture values,
    # not the P6-02 threshold/cadence. Zero and fractional milliseconds are valid.
    return [
        request(
            "TouchObservationV2",
            dict(
                schema_version=1,
                region="head",
                kind=kind,
                contact_id="contact-A",
                elapsed_ms=elapsed,
            ),
        )
        for kind, elapsed in ((7, 0.0), (8, 1000.25), (9, 3000.5))
    ] + [
        request(
            "TouchObservationV2",
            dict(
                schema_version=1,
                region="hand",
                kind=kind,
                speed=0.5,
            ),
        )
        for kind in range(1, 7)
    ]


def illegal_touch() -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for q in legal_touch()[:3]:
        value = q["value"]
        for field in ("contact_id", "elapsed_ms"):
            out.append(
                dict(
                    label=f"P6-17 dwell {value['kind']} missing {field}",
                    request=request(
                        "TouchObservationV2",
                        {k: v for k, v in value.items() if k != field},
                    ),
                )
            )
        for label, patch in (
            ("empty contact", dict(contact_id="")),
            ("negative duration", dict(elapsed_ms=-0.5)),
        ):
            out.append(
                dict(
                    label=f"P6-17 dwell {value['kind']} {label}",
                    request=request("TouchObservationV2", value | patch),
                )
            )
    # Fully populated protobuf: rejection is due to the nonfinite duration alone.
    prefix = b"\x08\x01\x12\x04head\x28\x08\x32\x09contact-A\x39"
    for bits in (0x7FF8000000000001, 0x7FF0000000000000, 0xFFF0000000000000):
        out.append(
            dict(
                label=f"P6-17 nonfinite elapsed_ms {bits:x}",
                request=dict(
                    name="TouchObservationV2",
                    op="protobuf",
                    hex=(prefix + struct.pack("<Q", bits)).hex(),
                    limits=LIMITS,
                ),
            )
        )
    # ADR 0007: old field 3 is reserved in V2; V2 field 5 is unknown in V1.
    old_pb = b"\x08\x01\x12\x04head\x18\x01"
    new_pb = b"\x08\x01\x12\x04head\x28\x01"
    for name, wire in (
        ("TouchObservationV2", old_pb),
        ("TouchObservation", new_pb),
        ("TouchObservationV2", new_pb + b"\x18\x01"),
    ):
        out.append(
            dict(
                label=f"P6-17 mixed touch protobuf as {name}",
                request=dict(
                    name=name,
                    op="protobuf",
                    hex=wire.hex(),
                    limits=LIMITS,
                ),
            )
        )
    # Independent minimal CanonicalV1 encodings (region=head, click, no speed).
    for name, tag, body in (
        ("TouchObservationV2", b"echo.touch", b"\x01\x00\x00\x00\x00"),
        ("TouchObservation", b"echo.touch-v2", b"\x00\x01\x00\x00\x00\x00\x00"),
    ):
        wire = (
            b"NCV1"
            + struct.pack("<II", 1, len(tag))
            + tag
            + b"\x04\x00\x00\x00head"
            + body
        )
        out.append(
            dict(
                label=f"P6-17 wrong touch canonical tag as {name}",
                request=dict(
                    name=name,
                    op="canonical",
                    hex=wire.hex(),
                    limits=LIMITS,
                ),
            )
        )
    # The old enum cannot acquire dwell; non-dwell V2 cannot silently gain identity.
    out.append(
        dict(
            label="P6-17 old touch cannot express dwell",
            request=request(
                "TouchObservation",
                dict(schema_version=1, region="head", kind=7),
            ),
        )
    )
    for kind in (4, 5):
        out.append(
            dict(
                label=f"P6-17 drag {kind} identity not authorized",
                request=request(
                    "TouchObservationV2",
                    dict(
                        schema_version=1,
                        region="hand",
                        kind=kind,
                        contact_id="contact-A",
                        elapsed_ms=0.0,
                    ),
                ),
            )
        )
    return out
