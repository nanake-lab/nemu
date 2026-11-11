"""ADR 0036 synthetic fixtures. No model/layer/mask/beta is chosen for deployment."""

from __future__ import annotations

import copy
import sys
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "workers/irl/generated"))

from echo.bridge import from_json  # noqa: E402
from echo.j_resources import NAMES as NAMES  # noqa: E402
from echo.runtime import SCHEMA, Limits, digest, encode_canonical  # noqa: E402

from fixture_law_state import ref  # noqa: E402

LIMITS = Limits(max_bytes=1048576, max_items=4096, max_depth=32)


def reference(name: str, value: dict[str, Any]) -> dict[str, Any]:
    v = from_json(name, value)
    return dict(
        schema_version=1,
        object_type=SCHEMA["messages"][name]["tag"],
        resource_version=value.get("version", "1"),
        digest=digest(name, v, LIMITS),
        byte_length=str(len(encode_canonical(name, v, LIMITS))),
    )


def bundle(seed: int) -> dict[str, Any]:
    from fixture_inputs import sample

    i = seed % 3
    vectors = [
        dict(
            schema_version=1,
            version=f"fixture-{i}-{j}",
            values=[float(k == j) for k in range(5)],
        )
        for j in range(5)
    ]
    ds = [
        dict(
            schema_version=1,
            kind=j + 1,
            vector=reference("JDirectionVector", x),
            beta=0.125,
        )
        for j, x in enumerate(vectors)
    ]
    ds[4].update(
        tendency_id="learned",
        concept=dict(
            schema_version=1,
            concept_id="concept",
            registry_version="fixture-registry-1",
        ),
    )
    hook = dict(
        schema_version=1,
        carrier=ref("fixture.carrier", 1),
        model=ref("fixture.model", 2),
        tokenizer=ref("fixture.tokenizer", 3),
        layer_index=i,
        residual_stage=i % 2 + 1,
        layer_kind=i + 1,
        width=5,
        dtype=i + 1,
        prefill_mask=i + 1,
        decode_mask=1,
    )
    s = dict(schema_version=1, version=f"fixture-set-{i}", hook=hook, directions=ds)
    v = dict(
        schema_version=1,
        version=f"fixture-validation-{i}",
        direction_set=reference("JDirectionSet", s),
        fixture=True,
        directions=[
            dict(
                schema_version=1,
                vector=d["vector"],
                max_beta=0.25,
                measured_norm=1.0,
                norm_tolerance=0.0,
                zh_evidence=ref("fixture.zh", 4),
                en_evidence=ref("fixture.en", 5),
                ja_evidence=ref("fixture.ja", 6),
            )
            for d in ds
        ],
        combination_max_beta=1.0,
        combination_evidence=ref("fixture.combination", 7),
    )
    row = dict(
        schema_version=1,
        connection=0.25,
        empathy=-0.5,
        curiosity=0.0,
        achievement=0.125,
    )
    b = dict(
        schema_version=1,
        version=f"fixture-body-{i}",
        facing=row,
        tension=row,
        energy=row,
    )
    g = sample("GenesisManifestV2", i)
    del g["proactivity_readout"]
    g.update(
        carrier=hook["carrier"],
        direction_set=reference("JDirectionSet", s),
        direction_validation=reference("JDirectionValidation", v),
        body_readout=reference("BodyReadoutParameters", b),
    )
    uses = [
        dict(
            schema_version=1,
            kind=d["kind"],
            status=1,
            direction=d["vector"],
            validation=reference("JDirectionValidation", v),
            beta=d["beta"],
            coefficient=(-1) ** j * 0.0625,
        )
        for j, d in enumerate(ds)
    ]
    uses[4]["tendency_id"] = "learned"
    # Keep the complete excluded list, including missing and mismatched qualifications.
    uses += [
        dict(schema_version=1, kind=5, tendency_id=t, status=status)
        for t, status in (
            ("unverified", 2),
            ("missing-data", 3),
            ("wrong-validation", 4),
        )
    ]
    uses[4:] = sorted(
        uses[4:],
        key=lambda d: (
            len(d["tendency_id"].encode()).to_bytes(4, "little")
            + d["tendency_id"].encode()
        ),
    )
    r = dict(
        schema_version=1,
        snapshot_id=f"synthetic-{i}",
        anchor_time_ns=str(i),
        projected_at_ns=str(2**53 + i),
        x_digest="11" * 32,
        r_digest="22" * 32,
        theta_digest="33" * 32,
        carrier=hook["carrier"],
        model=hook["model"],
        tokenizer=hook["tokenizer"],
        direction_set=reference("JDirectionSet", s),
        validation=reference("JDirectionValidation", v),
        body_parameters=reference("BodyReadoutParameters", b),
        genesis=reference("GenesisManifestV3", g),
        mode=1,
        directions=uses,
        total_bound=0.625,
        body=dict(
            schema_version=1,
            version=b["version"],
            facing=0.625,
            tension=0.125,
            energy=0.625,
        ),
    )
    return dict(
        direction_set=s, validation=v, body=b, receipt=r, genesis=g, vectors=vectors
    )


def sample_j(name: str, seed: int) -> dict[str, Any]:
    b = bundle(seed)
    values: dict[str, dict[str, Any]] = {
        "JHookBinding": b["direction_set"]["hook"],
        "JDirectionVector": b["vectors"][seed % 3],
        "JDirection": b["direction_set"]["directions"][seed % 3],
        "JDirectionSet": b["direction_set"],
        "JValidatedDirection": b["validation"]["directions"][seed % 3],
        "JDirectionValidation": b["validation"],
        "BodyReadoutParameters": b["body"],
        "JDirectionUse": b["receipt"]["directions"][seed % 3],
        "JEffectReceipt": b["receipt"],
        "GenesisManifestV3": b["genesis"],
    }
    return values[name]


def illegal_j() -> list[dict[str, Any]]:
    from fixture_inputs import LIMITS as bounds

    out = []

    def bad(name: str, label: str, edit: Any) -> None:
        v = copy.deepcopy(sample_j(name, 0))
        edit(v)
        out.append(
            dict(
                label=label,
                request=dict(name=name, op="encode", value=v, limits=bounds),
            )
        )

    bad("JDirection", "nature-has-tendency", lambda v: v.update(tendency_id="x"))
    bad(
        "JDirection",
        "wrong-vector-type",
        lambda v: v["vector"].update(object_type="private-protocol"),
    )
    bad("JDirectionSet", "missing-nature", lambda v: v["directions"].pop(0))
    bad(
        "JDirectionSet",
        "duplicate-learned",
        lambda v: v["directions"].append(v["directions"][-1]),
    )
    bad("JValidatedDirection", "non-unit", lambda v: v.update(measured_norm=0.5))
    bad(
        "JEffectReceipt",
        "total-understates-budget",
        lambda v: v.update(total_bound=0.5),
    )
    bad(
        "JEffectReceipt",
        "coefficient-exceeds-beta",
        lambda v: v["directions"][0].update(coefficient=0.25),
    )
    bad("JEffectReceipt", "rehearsal-not-zero", lambda v: v.update(mode=2))
    bad(
        "JEffectReceipt",
        "excluded-carries-coefficient",
        lambda v: v["directions"][-1].update(coefficient=0),
    )
    bad(
        "JEffectReceipt",
        "time-before-anchor",
        lambda v: v.update(anchor_time_ns="2", projected_at_ns="1"),
    )
    bad("JDirectionVector", "empty-direction", lambda v: v.update(values=[]))
    return out
