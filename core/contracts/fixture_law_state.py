"""ADR 0035 §9 P5-68: synthetic encoding fixtures, not executed law trajectories."""

from __future__ import annotations
import copy
import struct
import sys
from pathlib import Path
from typing import Any
from fixture_theta import sample_theta

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "workers/irl/generated"))
from echo.bridge import from_json  # noqa: E402
from echo.runtime import Limits, digest, encode_canonical, encode_protobuf  # noqa: E402
from echo.theta import LAW_STATE_NUMERICS_ID  # noqa: E402

NAMES = {
    "ThetaEstimatorParametersV2",
    "ThetaSnapshotV3",
    "FastBasisIdentity",
    "LawLearnedCoordinate",
    "LawStateVector",
    "FastBasisSlot",
    "FastStateSnapshotV3",
    "FastEffectRecord",
    "FastStateMigrationRecord",
}


def ref(tag: str, seed: int) -> dict[str, Any]:
    return dict(
        schema_version=1,
        digest=f"{seed:02x}" * 32,
        object_type=tag,
        resource_version="synthetic-adr0035",
        byte_length="100",
    )


def vector(value: float, learned: bool = False) -> dict[str, Any]:
    return dict(
        schema_version=1,
        nature=dict(
            schema_version=1,
            connection=value,
            empathy=0.0,
            curiosity=0.0,
            achievement=0.0,
        ),
        learned=[dict(schema_version=1, tendency_id="k", value=value)]
        if learned
        else [],
    )


def sample_law_state(name: str, seed: int) -> dict[str, Any]:
    from fixture_inputs import LIMITS

    i = seed % 3
    if name == "ThetaEstimatorParametersV2":
        p = sample_theta("ThetaEstimatorParameters", i)
        p.update(law_id="synthetic-adr0035", numerics_id=LAW_STATE_NUMERICS_ID)
        return p
    if name == "ThetaSnapshotV3":
        # Saved fields reused as synthetic message data, not a migration/restore.
        v = sample_theta("ThetaSnapshotV2", 1)
        p = sample_law_state("ThetaEstimatorParametersV2", i)
        v.update(
            parameters=p,
            parameters_digest=digest(
                "ThetaEstimatorParametersV2",
                from_json("ThetaEstimatorParametersV2", p),
                Limits(**LIMITS),
            ),
            law_version=ref("echo.law-version", i + 1),
            prior_mass=[0.0, 5e-324, 1.0][i],
        )
        return v
    if name == "FastBasisIdentity":
        return (
            dict(schema_version=1, kind=2)
            if i == 2
            else dict(schema_version=1, kind=1, record_id=["event-A", "事件-B"][i])
        )
    if name == "LawLearnedCoordinate":
        return dict(
            schema_version=1,
            tendency_id=["k", "学得", "z"][i],
            value=[0.0, -2.0, 0.3][i],
        )
    if name == "LawStateVector":
        return vector([0.0, -2.0, 0.3][i], i > 0)
    if name == "FastBasisSlot":
        return dict(
            schema_version=1,
            basis=sample_law_state("FastBasisIdentity", i),
            current=vector([0.0, -1.0, 2.0][i]),
            target=vector([0.0, 0.0, 1.0][i]),
        )
    if name == "FastStateSnapshotV3":
        return dict(
            schema_version=1,
            revision=f"synthetic-{i}",
            representation_version="ADR0035-A1-v1",
            anchor_time_ns=str(2**53 + i),
            nature=vector(0.5)["nature"],
            learned=[],
            law_version=ref("echo.law-version", 1),
            slots=[sample_law_state("FastBasisSlot", j) for j in range(i)],
        )
    if name == "FastEffectRecord":
        return dict(
            schema_version=1,
            basis=sample_law_state("FastBasisIdentity", 0),
            role=i + 1,
            target=vector([0.0, -1.0, 1.0][i], i > 0),
            theta_before=ref("echo.theta-snapshot-v3", 1),
            effects_before=ref("echo.authorized-effects", 2),
            interpretation_refs=[ref("echo.semantic-record-v2", 3)],
            governance_refs=[ref("echo.basis-governance", 4)],
            law_version=ref("echo.law-version", 5),
        )
    if name == "FastStateMigrationRecord":
        return dict(
            schema_version=1,
            previous_snapshot=ref("echo.fast-state-v2", i + 1),
            migrated_snapshot=ref("echo.fast-state-v3", i + 4),
            prior_decomposition_unknown=True,
        )
    raise ValueError(name)


def legal_law_state() -> list[dict[str, Any]]:
    from fixture_inputs import request

    rows: list[dict[str, Any]] = []

    def add(name: str, v: dict[str, Any]) -> None:
        rows.append(request(name, v))

    # §4.6 R4: targets (1,-1,1), then second target -> 0 leaves h2=-1.
    s = sample_law_state("FastStateSnapshotV3", 0)
    s["nature"] = vector(1)["nature"]
    s["slots"] = []
    for identity, h, target in [("first", 1, 1), ("second", -1, 0), ("third", 1, 1)]:
        v = sample_law_state("FastBasisSlot", 0)
        v.update(
            basis=dict(schema_version=1, kind=1, record_id=identity),
            current=vector(h),
            target=vector(target),
        )
        s["slots"].append(v)
    add("FastStateSnapshotV3", copy.deepcopy(s))
    # §4.6 B6': first+third merge => h=2, v=1; visible x remains 1.
    s["slots"][0]["current"] = vector(2)
    s["slots"].pop()
    add("FastStateSnapshotV3", copy.deepcopy(s))
    s["slots"] = [sample_law_state("FastBasisSlot", 2)]
    add("FastStateSnapshotV3", copy.deepcopy(s))  # singleton internal branch
    s["slots"] = [sample_law_state("FastBasisSlot", 0)]
    add("FastStateSnapshotV3", copy.deepcopy(s))  # all-zero slot
    # Binary64 merge witness (not a current-target amplitude bound).
    s["slots"][0].update(current=vector(0.30000000000000004), target=vector(0.1))
    add("FastStateSnapshotV3", copy.deepcopy(s))
    # Same learned IDs, signed values outside X are valid for h and target.
    s["learned"] = [dict(schema_version=1, tendency_id="k", value=0.5)]
    s["slots"][0].update(current=vector(-2, True), target=vector(0, True))
    add("FastStateSnapshotV3", s)
    r = sample_law_state("FastEffectRecord", 0)
    r.update(basis=sample_law_state("FastBasisIdentity", 2), role=4)
    add("FastEffectRecord", r)
    # §5.2: this application supplements a record whose fixed role is internal.
    r = copy.deepcopy(r)
    r["role"] = 2
    add("FastEffectRecord", r)
    # §7 P0: finite and >0, including subnormal and values beyond exact integers.
    for weight in (5e-324, float(2**53), float.fromhex("0x1.fffffffffffffp+1023")):
        p = sample_law_state("ThetaEstimatorParametersV2", 0)
        p["baseline_weight"] = weight
        add("ThetaEstimatorParametersV2", p)
    return rows


def illegal_law_state() -> list[dict[str, Any]]:
    from fixture_inputs import request, LIMITS

    rows: list[dict[str, Any]] = []

    def bad(name: str, path: str, value: Any, seed: int = 1) -> None:
        v = sample_law_state(name, seed)
        at = v
        keys = path.split(".")
        for k in keys[:-1]:
            at = at[int(k)] if isinstance(at, list) else at[k]
        if value is None:
            del at[keys[-1]]
        else:
            at[keys[-1]] = value
        rows.append(dict(label="P5-68-" + path, request=request(name, v)))

    # §4.2–4.3: an internal basis cannot reuse a recorded external target.
    # Encode structurally to test the semantic boundary on both wire inputs.
    name = "FastEffectRecord"
    v = sample_law_state(name, 2)
    v["basis"] = sample_law_state("FastBasisIdentity", 2)
    rows.append(dict(label="P5-68-internal-revisit", request=request(name, v)))
    for op, encode in (("canonical", encode_canonical), ("protobuf", encode_protobuf)):
        wire = encode(name, from_json(name, v), Limits(**LIMITS))
        rows.append(
            dict(
                label="P5-68-internal-revisit",
                request=dict(
                    name=name,
                    op=op,
                    hex=wire.hex(),
                    limits=LIMITS,
                ),
            )
        )

    for name, path, val in [
        ("ThetaEstimatorParametersV2", "baseline_weight", 0),
        ("ThetaEstimatorParametersV2", "numerics_id", "P5-25-v2-93-98-libm-0.2.16"),
        ("ThetaSnapshotV3", "prior_mass", -5e-324),
        ("ThetaSnapshotV3", "prior_mass", None),
        ("ThetaSnapshotV3", "parameters_digest", "00" * 32),
        ("ThetaSnapshotV3", "law_version.resource_version", "old-law"),
        ("ThetaSnapshotV3", "estimator.reaction_sum", [0, 0, 0]),
        ("FastBasisIdentity", "record_id", ""),
        ("FastBasisIdentity", "record_id", None),
        ("FastBasisIdentity", "kind", 2),  # branch cannot carry a record ID
        (
            "FastBasisSlot",
            "current.learned",
            [sample_law_state("LawLearnedCoordinate", 0)],
        ),
        ("FastStateSnapshotV3", "nature.connection", 1.0000000000000002),
        ("FastStateSnapshotV3", "slots.0.basis.record_id", ""),
        (
            "FastStateSnapshotV3",
            "learned",
            [dict(schema_version=1, tendency_id="k", value=0)],
        ),
        ("FastEffectRecord", "role", None),
        ("FastEffectRecord", "role", 0),
        ("FastEffectRecord", "role", 4),
        ("FastEffectRecord", "interpretation_refs", []),
        ("FastEffectRecord", "governance_refs", []),
        ("FastEffectRecord", "theta_before.object_type", "echo.theta-snapshot-v2"),
        ("FastStateMigrationRecord", "prior_decomposition_unknown", False),
        (
            "FastStateMigrationRecord",
            "previous_snapshot.object_type",
            "echo.fast-state-v3",
        ),
        (
            "FastStateMigrationRecord",
            "migrated_snapshot.object_type",
            "echo.fast-state-v2",
        ),
    ]:
        bad(name, path, val)
    for seed in (0, 2):
        item = sample_law_state("FastBasisSlot", seed)
        bad("FastStateSnapshotV3", "slots", [item, copy.deepcopy(item)])
    # Change exactly one double in an otherwise legal, complete wire message.
    # This rules out rejection solely because another required field was missing.
    for bits in (0x7FF0000000000000, 0xFFF0000000000000, 0x7FF8000000000001):
        for name, field in [
            ("ThetaSnapshotV3", "prior_mass"),
            ("FastBasisSlot", "current"),
            ("FastBasisSlot", "target"),
        ]:
            v = sample_law_state(name, 1)
            if field == "prior_mass":
                v[field] = 123.456
            else:
                v[field]["nature"]["connection"] = 123.456
            wire = encode_protobuf(name, from_json(name, v), Limits(**LIMITS))
            marker = struct.pack("<d", 123.456)
            assert wire.count(marker) == 1
            wire = wire.replace(marker, struct.pack("<Q", bits))
            rows.append(
                dict(
                    label="P5-68-nonfinite",
                    request=dict(
                        name=name, op="protobuf", hex=wire.hex(), limits=LIMITS
                    ),
                )
            )
    return rows
