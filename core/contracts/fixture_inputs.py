"""Deterministic source fixtures. Expected bytes are frozen separately, never in verification."""

from __future__ import annotations
import json
from pathlib import Path
from typing import Any

SCHEMA = json.loads(
    (Path(__file__).parent / "generated/schema.json").read_text(encoding="utf-8")
)
LIMITS = dict(max_bytes=1048576, max_items=4096, max_depth=32)


def sample(name: str, seed: int, depth: int = 0) -> dict[str, Any]:
    message = SCHEMA["messages"][name]

    def atom(f: dict[str, Any], n: int) -> Any:
        t = f["type"]
        if t in SCHEMA["messages"]:
            return sample(t, n, depth + 1)
        if t in SCHEMA["enums"]:
            values = [v for v in SCHEMA["enums"][t].values() if v]
            return values[n % len(values)]
        if t == "string":
            return ["A", "中文🙂", "\ufeffzero\0"][n % 3] + str(n)
        if t == "bytes":
            return bytes([n % 256] * f.get("size", n % 3)).hex()
        if t == "bool":
            return n % 2 == 0
        if t == "uint64":
            return str([0, 2**64 - 1, 2**53 + 1][n % 3])
        if t == "sint64":
            return str([0, -(2**63), 2**63 - 1][n % 3])
        if t == "uint32":
            return max(int(f.get("min", 0)), [0, 2**32 - 1, 17][n % 3])
        return max(
            f.get("min", -1e300),
            min(
                f.get("max", 1e300),
                [0.0, f.get("min", -1.25), f.get("max", 2.5)][n % 3],
            ),
        )

    out: dict[str, Any] = {}
    for f in message["fields"]:
        k = f["name"]
        if k == "schema_version":
            out[k] = 1
        elif f["repeated"]:
            out[k] = [atom(f, seed + j) for j in range(seed % 3 if depth < 2 else 0)]
        elif f["required"] or seed % 3:
            out[k] = atom(f, seed)
    for key, count in message["rules"].get("lengths", {}).items():
        out[key] = [atom(next(f for f in message["fields"] if f["name"] == key), seed)]
        out[count] = len(out[key])
    cases = message["rules"].get("cases")
    if cases:
        discriminator = message["rules"].get("discriminator", "kind")
        # Choose among declared cases only: an enum may gain values that an older,
        # superseded message never accepted (e.g. Lineage EXPERIMENT for LineageHeader).
        # For every message whose cases cover its enum this is the previous choice.
        declared = sorted(int(k) for k in cases)
        out[discriminator] = declared[seed % len(declared)]
        case = cases[str(out[discriminator])]
        required = case.get("required", [])
        allowed = set(
            required
            + case.get("optional", [])
            + message["rules"].get("common", [])
            + ["schema_version", discriminator]
        )
        out = {k: v for k, v in out.items() if k in allowed}
        for k in required:
            if k not in out:
                out[k] = atom(
                    next(f for f in message["fields"] if f["name"] == k), seed
                )
        for k, values in case.get("values", {}).items():
            out[k] = values[seed % len(values)]
    return out


def requests() -> list[dict[str, Any]]:
    base = [
        dict(name=n, op="encode", value=sample(n, s), limits=LIMITS)
        for n in SCHEMA["messages"]
        for s in range(3)
    ]
    for name in ("Anchor", "GazeTarget", "BodyAction", "LedgerEntry"):
        base.extend(
            dict(name=name, op="encode", value=sample(name, seed), limits=LIMITS)
            for seed in range(3, len(SCHEMA["messages"][name]["rules"]["cases"]))
        )
    return base


def request(name: str, value: dict[str, Any]) -> dict[str, Any]:
    return dict(name=name, op="encode", value=value, limits=LIMITS)


def unit(c: float, e: float, q: float, a: float) -> dict[str, Any]:
    return dict(schema_version=1, connection=c, empathy=e, curiosity=q, achievement=a)


def targeted_legal() -> list[dict[str, Any]]:
    """P5-21 vectors for values that seeded samples do not reach (new enum codes) and
    semantically realistic v2 records. Additional vectors only; never replace frozen ones."""
    e = SCHEMA["enums"]
    rs, fc = e["RunStatus"], e["FailureCode"]
    reason = dict(schema_version=1, code=fc["GROWTH_OFF_REHEARSAL"])
    tendency = sample("Tendency", 0) | dict(
        tendency_id="caretaker-closeness",
        direction=unit(1.0, 0.0, 0.0, 0.0),
        evidence_a=1.0,
        evidence_b=4.0,
        h_hat=0.1,
        group_count="3",
        flip_count="0",
        last_sign=e["EvidenceSign"]["SIGN_POSITIVE"],
        pi=0.875,
        typical_magnitude=0.25,
        relation=dict(
            schema_version=1, person_id="u*", kind=e["RelationKind"]["CLOSENESS"]
        ),
        attachment_prior=20.0,
    )
    appraisal = sample("NatureAppraisalRecord", 1) | dict(
        appraisal=unit(0.5, -0.25, 0.75, -1.0), persons=["person-a", "u*"]
    )
    match = sample("SituationMatchRecord", 0) | dict(
        matches=[
            dict(schema_version=1, tendency_id="t-1", mu=0.5),
            dict(schema_version=1, tendency_id="t-2", mu=1.0),
        ]
    )
    return [
        request("FailureReason", reason),
        request(
            "DegradedStatus",
            dict(schema_version=1, states=[rs["NO_GROWTH"]], reasons=[reason]),
        ),
        request(
            "DegradedStatus",
            dict(
                schema_version=1,
                states=[
                    rs["GROWTH_PAUSED"],
                    rs["GROWTH_REVIEW_PENDING"],
                    rs["A_DEGRADED"],
                    rs["M_DEGRADED"],
                ],
                reasons=[],
            ),
        ),
        request("Tendency", tendency),
        request("NatureAppraisalRecord", appraisal),
        request("SituationMatchRecord", match),
    ] + targeted_legal_p5_32(tendency)


def p5_32_allocation(tendency_id: str, ordinal: str, projection: float | None) -> dict[str, Any]:
    out: dict[str, Any] = dict(
        schema_version=1,
        tendency_id=tendency_id,
        support=-0.5,
        intensity=1.25,
        share=0.6,
        ordinal=ordinal,
        mu=0.75,
    )
    if projection is not None:
        out["projection"] = projection
    return out


def targeted_legal_p5_32(tendency: dict[str, Any]) -> list[dict[str, Any]]:
    """P5-32 (ADR 0016): realistic successors with the replay fields set."""
    v2 = tendency | dict(magnitude_sum=-0.125, magnitude_weight=2.5)
    experience = p5_32_allocation("t-1", "4", -0.25)
    teaching = p5_32_allocation("t-2", "0", None) | dict(support=1.0, mu=1.0)
    group = sample("EvidenceGroupV2", 0) | dict(allocations=[experience, teaching])
    snapshot = sample("RSnapshotV2", 0) | dict(tendencies=[v2])
    return [
        request("TendencyV2", v2),
        request("EvidenceAllocationV2", experience),
        request("EvidenceAllocationV2", teaching),
        request("EvidenceGroupV2", group),
        request("RSnapshotV2", snapshot),
    ]


def targeted_illegal() -> list[dict[str, Any]]:
    """P5-21 frozen rejections: out-of-range a(e), μ, π; lineage and ledger variants;
    duplicate set keys; removed fields. Each is labelled with the rule it breaks."""
    e = SCHEMA["enums"]
    lineage = e["Lineage"]
    digest = "11" * 32
    cases: list[tuple[str, str, dict[str, Any]]] = []

    def bad(label: str, name: str, value: dict[str, Any]) -> None:
        cases.append((label, name, value))

    bad("a(e) above 1", "NatureVector", unit(1.5, 0.0, 0.0, 0.0))
    appraisal = sample("NatureAppraisalRecord", 0)
    bad(
        "a(e) below -1",
        "NatureAppraisalRecord",
        appraisal | dict(appraisal=unit(0.0, -1.0000001, 0.0, 0.0)),
    )
    bad("mu above 1", "TendencyMatch", dict(schema_version=1, tendency_id="t", mu=1.5))
    bad("mu below 0", "TendencyMatch", dict(schema_version=1, tendency_id="t", mu=-0.1))
    bad("pi above 1", "Tendency", sample("Tendency", 0) | dict(pi=1.5))
    bad(
        "pi below -1",
        "TendencyChange",
        sample("TendencyChange", 0) | dict(pi_after=-2.0),
    )
    header = dict(schema_version=1, created_at_ns="0")
    bad(
        "EXPERIMENT carries genesis digest",
        "LineageHeaderV2",
        header
        | dict(
            lineage=lineage["EXPERIMENT"],
            world_id="w",
            world_seed="1",
            genesis_manifest_digest=digest,
        ),
    )
    bad(
        "EXPERIMENT without seed",
        "LineageHeaderV2",
        header | dict(lineage=lineage["EXPERIMENT"], world_id="w"),
    )
    bad(
        "LIFE without genesis",
        "LineageHeaderV2",
        header | dict(lineage=lineage["LIFE"]),
    )
    bad(
        "REHEARSAL carries world",
        "LineageHeaderV2",
        header | dict(lineage=lineage["REHEARSAL"], world_id="w", world_seed="1"),
    )
    bad(
        "superseded header cannot express EXPERIMENT",
        "LineageHeader",
        header | dict(lineage=lineage["EXPERIMENT"]),
    )
    entry = sample("LedgerEntry", 0)
    bad(
        "ledger variant field mismatch",
        "LedgerEntry",
        {k: v for k, v in entry.items() if k != "tendency_change"}
        | dict(theta_change=sample("ThetaChange", 0)),
    )
    command = sample("GrowthCommand", 2)
    bad(
        "review conclusion without record",
        "GrowthCommand",
        {k: v for k, v in command.items() if k != "review_record"},
    )
    bad(
        "pause carries verdict",
        "GrowthCommand",
        sample("GrowthCommand", 0) | dict(verdict=e["ReviewVerdict"]["LAW_CORRECT"]),
    )
    t = sample("Tendency", 0)
    bad(
        "duplicate tendency in R",
        "RSnapshot",
        sample("RSnapshot", 0) | dict(tendencies=[t, t]),
    )
    learned = dict(schema_version=1, tendency_id="t", value=0.5)
    bad(
        "duplicate learned component",
        "FastStateSnapshotV2",
        sample("FastStateSnapshotV2", 0) | dict(learned=[learned, learned]),
    )
    bad(
        "unknown RunStatus code",
        "DegradedStatus",
        dict(schema_version=1, states=[14], reasons=[]),
    )
    bad(
        "removed field hooks",
        "ModulationEnvelopeV2",
        sample("ModulationEnvelopeV2", 0) | dict(hooks=[]),
    )
    bad(
        "removed field scope_interpreter",
        "VersionedEnvironmentV2",
        sample("VersionedEnvironmentV2", 0)
        | dict(scope_interpreter=sample("ObjectRef", 0)),
    )
    allocation = p5_32_allocation("t", "0", 0.5)
    bad("allocation mu above 1", "EvidenceAllocationV2", allocation | dict(mu=1.5))
    bad("allocation mu below 0", "EvidenceAllocationV2", allocation | dict(mu=-0.1))
    bad(
        "allocation without ordinal",
        "EvidenceAllocationV2",
        {k: v for k, v in allocation.items() if k != "ordinal"},
    )
    bad("ordinal above u64", "EvidenceAllocationV2", allocation | dict(ordinal=str(2**64)))
    v2 = sample("TendencyV2", 0)
    bad("magnitude weight below 0", "TendencyV2", v2 | dict(magnitude_weight=-1.0))
    bad(
        "tendency v2 without magnitude sum",
        "TendencyV2",
        {k: v for k, v in v2.items() if k != "magnitude_sum"},
    )
    bad(
        "duplicate allocation in group v2",
        "EvidenceGroupV2",
        sample("EvidenceGroupV2", 0) | dict(allocations=[allocation, allocation]),
    )
    bad(
        "duplicate tendency in R v2",
        "RSnapshotV2",
        sample("RSnapshotV2", 0) | dict(tendencies=[v2, v2]),
    )
    bad(
        "superseded allocation in group v2",
        "EvidenceGroupV2",
        sample("EvidenceGroupV2", 0) | dict(allocations=[sample("EvidenceAllocation", 0)]),
    )
    return [dict(label=label, request=request(n, v)) for label, n, v in cases]
