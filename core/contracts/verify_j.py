"""ADR 0036 resource binding and compatibility checks; real carrier/SYS17 not_run."""

from __future__ import annotations

import copy
from collections.abc import Callable
from typing import Any

from fixture_inputs import LIMITS
from fixture_j import bundle, reference


def rebind(b: dict[str, Any]) -> None:
    s, v, body, r, g = (
        b[k] for k in ("direction_set", "validation", "body", "receipt", "genesis")
    )
    for d, check, vector in zip(
        s["directions"], v["directions"], b["vectors"], strict=True
    ):
        d["vector"] = reference("JDirectionVector", vector)
        check["vector"] = d["vector"]
    sr = reference("JDirectionSet", s)
    v["direction_set"] = sr
    vr = reference("JDirectionValidation", v)
    br = reference("BodyReadoutParameters", body)
    g.update(
        direction_set=sr,
        direction_validation=vr,
        body_readout=br,
        carrier=s["hook"]["carrier"],
    )
    r.update(
        direction_set=sr,
        validation=vr,
        body_parameters=br,
        genesis=reference("GenesisManifestV3", g),
    )
    for use in r["directions"]:
        if use["status"] == 1:
            ds = [
                d
                for d in s["directions"]
                if d["kind"] == use["kind"]
                and d.get("tendency_id") == use.get("tendency_id")
            ]
            use.update(direction=ds[0]["vector"], validation=vr)



def resource_bundle(seed: int) -> dict[str, Any]:
    """Semantic test inputs; leave published codec-only fixtures and bytes intact."""
    b = bundle(seed)
    g = b["genesis"]
    g["law_version"]["parameters"].update(
        n0=1.0, w_teach=1.0, w_max=2.0, v_u=0.5, r_min=0.0, r_max=2.0,
        g_min=0.1, g_max=2.0, recovery_min_per_s=0.1, recovery_max_per_s=1.0,
    )
    g["initial_gain"].update(connection=1.0, empathy=1.0, curiosity=1.0, achievement=0.5)
    g.update(sigma0=1.0, tau0_s=1.0)
    rebind(b)
    return b


def genesis_cases() -> tuple[list[dict[str, Any]], list[str]]:
    # A non-fixture declaration is only a structural test, never real qualification.
    control = resource_bundle(0)
    control["validation"]["fixture"] = False
    rebind(control)
    cases = [dict(op="j_resources", value=control, formal=True, limits=LIMITS)]
    errors = [""]
    for path, replacement, error in (
        (("tau0_s",), 0.0, "tau0 must be positive"),
        (("sigma0",), 0.0, "sigma0 must be positive"),
        (("initial_gain", "achievement"), 2.0, "achievement must have the weakest initial gain"),
        (("law_version", "parameters", "n0"), 0.0, "n0 must be positive"),
        (("law_version", "parameters", "g_min"), 3.0, "g_min must not exceed g_max"),
    ):
        b = copy.deepcopy(control)
        target = b["genesis"]
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = replacement
        rebind(b)
        assert b["receipt"]["genesis"] == reference("GenesisManifestV3", b["genesis"])
        cases.append(dict(op="j_resources", value=b, formal=True, limits=LIMITS))
        errors.append(error)
    return cases, errors


def verify(
    commands: dict[str, list[str]],
    probe: Callable[[list[str], list[dict[str, Any]]], list[dict[str, Any]]],
    legal: list[dict[str, Any]],
) -> None:
    good = [
        dict(op="j_resources", value=resource_bundle(i), formal=False, limits=LIMITS)
        for i in range(3)
    ]
    receipt = resource_bundle(0)["receipt"]
    envelope = {
        k: receipt[k]
        for k in (
            "snapshot_id",
            "projected_at_ns",
            "x_digest",
            "r_digest",
            "theta_digest",
            "carrier",
            "model",
            "tokenizer",
            "direction_set",
            "total_bound",
        )
    }
    envelope.update(
        schema_version=1,
        effect_receipt_digest=reference("JEffectReceipt", receipt)["digest"],
        nature_coefficients=dict(
            schema_version=1,
            **{
                k: d["coefficient"]
                for k, d in zip(
                    ("connection", "empathy", "curiosity", "achievement"),
                    receipt["directions"][:4],
                    strict=True,
                )
            },
        ),
        learned_coefficients=[
            dict(
                schema_version=1,
                tendency_id=d["tendency_id"],
                coefficient=d["coefficient"],
            )
            for d in receipt["directions"][4:]
            if d["status"] == 1
        ],
    )
    envelope_good: dict[str, Any] = dict(
        op="j_envelope", value=dict(envelope=envelope, receipt=receipt), limits=LIMITS
    )
    envelope_bad = []
    for key, value in (
        ("snapshot_id", "wrong"),
        ("effect_receipt_digest", "00" * 32),
        ("total_bound", 0.0),
    ):
        q = copy.deepcopy(envelope_good)
        q["value"]["envelope"][key] = value
        envelope_bad.append(q)
    q = copy.deepcopy(envelope_good)
    q["value"]["envelope"]["nature_coefficients"]["connection"] = 0.0
    envelope_bad.append(q)
    # P5-12's outward rounding must reject a bound rounded below 1 + minsub.
    rounding = copy.deepcopy(receipt)
    for i, d in enumerate(rounding["directions"]):
        if d["status"] == 1:
            d.update(
                beta=1.0 if i == 0 else (5e-324 if i == 1 else 0.0), coefficient=0.0
            )
    rounding["body"].update(facing=0.5, tension=0.0, energy=0.5)
    rounding["total_bound"] = float.fromhex("0x1.0000000000001p0")
    numeric_good: dict[str, Any] = dict(
        op="encode", name="JEffectReceipt", value=rounding, limits=LIMITS
    )
    numeric_bad = copy.deepcopy(numeric_good)
    numeric_bad["value"]["total_bound"] = 1.0
    nonneutral = copy.deepcopy(numeric_good)
    nonneutral["value"]["body"]["facing"] = 0.75
    rehearsal_nonneutral = copy.deepcopy(nonneutral)
    rehearsal_nonneutral["value"]["mode"] = 2
    bad: list[dict[str, Any]] = []
    labels: list[str] = []

    def reject(
        label: str, edit: Callable[[dict[str, Any]], None], bind: bool = False
    ) -> None:
        b = copy.deepcopy(resource_bundle(0))
        edit(b)
        if bind:
            rebind(b)
        bad.append(dict(op="j_resources", value=b, formal=False, limits=LIMITS))
        labels.append(label)

    reject(
        "resource-version",
        lambda b: b["receipt"]["direction_set"].update(resource_version="wrong"),
    )
    reject(
        "resource-digest", lambda b: b["receipt"]["validation"].update(digest="00" * 32)
    )
    reject(
        "resource-length",
        lambda b: b["receipt"]["body_parameters"].update(byte_length="1"),
    )
    reject(
        "wrong-model", lambda b: b["receipt"]["model"].update(resource_version="wrong")
    )
    reject(
        "wrong-concept-registry",
        lambda b: b["direction_set"]["directions"][4]["concept"].update(
            registry_version="wrong"
        ),
    )
    reject("wrong-hook", lambda b: b["direction_set"]["hook"].update(layer_index=99))
    reject("wrong-mask", lambda b: b["direction_set"]["hook"].update(decode_mask=2))
    reject("wrong-shape", lambda b: b["direction_set"]["hook"].update(width=6), True)
    reject("wrong-norm", lambda b: b["vectors"][0].update(values=[2, 0, 0, 0, 0]), True)
    reject(
        "single-budget",
        lambda b: b["validation"]["directions"][0].update(max_beta=0.0625),
        True,
    )
    reject(
        "combination-budget",
        lambda b: b["validation"].update(combination_max_beta=0.5),
        True,
    )
    reject(
        "receipt-qualified-list",
        lambda b: b["receipt"]["directions"][4].update(tendency_id="unknown"),
    )
    reject("missing-vector", lambda b: b["vectors"].pop())
    reject("body-version", lambda b: b["receipt"]["body"].update(version="wrong"))
    fixture = dict(op="j_resources", value=resource_bundle(0), formal=True, limits=LIMITS)
    bad.append(fixture)
    labels.append("fixture-not-formal")
    restores, restored, cross = [], [], []
    for name, other in (
        ("GenesisManifestV2", "GenesisManifestV3"),
        ("GenesisManifestV3", "GenesisManifestV2"),
    ):
        rows = [x for x in legal if x["request"]["name"] == name][:3]
        assert len(rows) == 3
        for row in rows:
            for op in ("canonical", "protobuf"):
                restores.append(
                    dict(op=op, name=name, hex=row["expected"][op], limits=LIMITS)
                )
                restored.append(row["expected"])
                cross.append(
                    dict(op=op, name=other, hex=row["expected"][op], limits=LIMITS)
                )
    genesis, genesis_errors = genesis_cases()
    direct = [dict(op="j_genesis", name="GenesisManifestV3", value=x["value"]["genesis"], limits=LIMITS) for x in genesis]
    codec = [dict(x, op="encode") for x in direct]
    mask_bad = []
    for key, value in (("prefill_mask", 3), ("decode_mask", 2)):
        q = copy.deepcopy(genesis[0])
        q["value"]["direction_set"]["hook"][key] = value
        rebind(q["value"])
        mask_bad.append(q)
    for language, command in commands.items():
        # All six remain codec-valid. Rejection must be inherited semantics, not bytes/hash.
        assert all(x["ok"] for x in probe(command, codec)), (language, "genesis codec")
        for requests in (direct, genesis):
            results = probe(command, requests)
            assert results[0] == {"ok": True}, (language, "genesis control")
            for result, error in zip(results[1:], genesis_errors[1:], strict=True):
                assert not result["ok"] and error in result["error"], (language, result, error)
        assert all(not x["ok"] for x in probe(command, mask_bad)), (language, "formal masks")

        assert probe(command, [numeric_good])[0]["ok"], (language, "outward bound")
        assert not probe(command, [numeric_bad])[0]["ok"], (
            language,
            "rounded-down bound",
        )
        assert probe(command, [nonneutral])[0]["ok"], (
            language, "zero coefficients do not imply neutral body"
        )
        assert not probe(command, [rehearsal_nonneutral])[0]["ok"], (
            language, "rehearsal body"
        )
        assert probe(command, [envelope_good]) == [{"ok": True}], (
            language,
            "envelope binding",
        )
        assert all(not x["ok"] for x in probe(command, envelope_bad)), (
            language,
            "envelope mismatch",
        )
        assert all(x["ok"] for x in probe(command, good)), (
            language,
            "resolved resources",
        )
        for label, result in zip(labels, probe(command, bad), strict=True):
            assert not result["ok"], (language, label)
        assert probe(command, restores) == restored, (language, "restore")
        assert all(not x["ok"] for x in probe(command, cross)), (
            language,
            "cross-version",
        )
    print("P5-42 B1: genesis control + 5 rebound semantic refusals, direct/resource routes, codec preservation and formal mask controls PASS per language")
    print(
        f"P5-42: 3 resolved bundles, {len(bad)} resource refusals, 12 restores/12 reinterpretation refusals per language PASS; real carrier/SYS17 not_run"
    )
