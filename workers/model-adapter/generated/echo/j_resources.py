"""ADR 0036 boundary checks. No commit authority or real calibration is inferred."""

from __future__ import annotations

import math
from typing import Any

from .genesis import check_genesis
from .runtime import SCHEMA, Limits, digest, encode_canonical, fail, normalize

NAMES = {
    "JHookBinding",
    "JDirectionVector",
    "JDirection",
    "JDirectionSet",
    "JValidatedDirection",
    "JDirectionValidation",
    "BodyReadoutParameters",
    "JDirectionUse",
    "JEffectReceipt",
    "GenesisManifestV3",
}


def check_j_envelope(
    envelope: dict[str, Any], receipt: dict[str, Any], limits: Limits
) -> None:
    """Bind the existing envelope to the canonical receipt, without a reverse reference."""
    e = normalize("ModulationEnvelopeV2", envelope, limits)
    r = normalize("JEffectReceipt", receipt, limits)
    check_j_message("JEffectReceipt", r, limits)
    fail(
        e["effect_receipt_digest"].hex() == digest("JEffectReceipt", r, limits),
        "receipt digest",
    )
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
    ):
        fail(e[k] == r[k], "envelope binding " + k)
    for key, d in zip(
        ("connection", "empathy", "curiosity", "achievement"),
        r["directions"][:4],
        strict=True,
    ):
        fail(e["nature_coefficients"][key] == d["coefficient"], "nature coefficient")
    learned = [
        dict(
            schema_version=1, tendency_id=d["tendency_id"], coefficient=d["coefficient"]
        )
        for d in r["directions"][4:]
        if d["status"] == 1
    ]
    fail(e["learned_coefficients"] == learned, "learned coefficients")


def add_bound(a: float, b: float) -> float:
    hi, lo = max(a, b), min(a, b)
    total = hi + lo
    fail(math.isfinite(total), "beta overflow")
    if lo - (total - hi) > 0:
        total = math.nextafter(total, math.inf)
    fail(math.isfinite(total), "beta overflow")
    return total


def ordered(items: list[dict[str, Any]], limits: Limits) -> None:
    fail(len(items) >= 4, "four nature directions required")
    fail([d["kind"] for d in items[:4]] == [1, 2, 3, 4], "nature order")
    fail(all(d["kind"] == 5 for d in items[4:]), "learned kind")
    # Same key bytes as CanonicalV1 set=tendency_id, not Unicode lexical order.
    keys = [
        len(d["tendency_id"].encode()).to_bytes(4, "little") + d["tendency_id"].encode()
        for d in items[4:]
    ]
    fail(keys == sorted(set(keys)), "learned order/duplicate")


def check_j_message(name: str, value: dict[str, Any], limits: Limits) -> None:
    if name not in NAMES:
        return
    v = normalize(name, value, limits)

    def tag(key: str, expected: str) -> None:
        if key in v:
            fail(v[key]["object_type"] == expected, key + " type")

    if name == "JDirectionVector":
        fail(bool(v["values"]), "empty vector")
    elif name in ("JDirection", "JDirectionUse"):
        learned = v["kind"] == 5
        fail(("tendency_id" in v) == learned, "tendency identity")
        if name == "JDirection":
            fail(("concept" in v) == learned, "concept identity")
            tag("vector", "echo.j-direction-vector")
        else:
            included = v["status"] == 1
            fail(learned or included, "nature cannot be excluded")
            fail(
                all(
                    (k in v) == included
                    for k in ("direction", "validation", "beta", "coefficient")
                ),
                "use presence",
            )
            tag("direction", "echo.j-direction-vector")
            tag("validation", "echo.j-direction-validation")
            if included:
                fail(abs(v["coefficient"]) <= v["beta"], "coefficient budget")
    elif name == "JDirectionSet":
        ordered(v["directions"], limits)
        for d in v["directions"]:
            check_j_message("JDirection", d, limits)
    elif name == "JValidatedDirection":
        tag("vector", "echo.j-direction-vector")
        fail(abs(v["measured_norm"] - 1) <= v["norm_tolerance"], "unit norm")
    elif name == "JDirectionValidation":
        tag("direction_set", "echo.j-direction-set")
        fail(len(v["directions"]) >= 4, "validation list")
        for d in v["directions"]:
            check_j_message("JValidatedDirection", d, limits)
    elif name == "JEffectReceipt":
        fail(v["projected_at_ns"] >= v["anchor_time_ns"], "time before anchor")
        for k, t in (
            ("direction_set", "echo.j-direction-set"),
            ("validation", "echo.j-direction-validation"),
            ("body_parameters", "echo.body-readout-parameters"),
            ("genesis", "echo.genesis-v3"),
        ):
            tag(k, t)
        ordered(v["directions"], limits)
        total = 0.0
        for d in v["directions"]:
            check_j_message("JDirectionUse", d, limits)
            if d["status"] == 1:
                total = add_bound(total, d["beta"])
                if v["mode"] == 2:
                    fail(d["coefficient"] == 0, "rehearsal coefficient")
        fail(total == v["total_bound"], "total bound")
        if v["mode"] == 2:
            fail(
                [v["body"][k] for k in ("facing", "tension", "energy")]
                == [0.5, 0, 0.5],
                "rehearsal body",
            )
    elif name == "GenesisManifestV3":
        tag("direction_set", "echo.j-direction-set")
        tag("direction_validation", "echo.j-direction-validation")
        tag("body_readout", "echo.body-readout-parameters")


def check_j_resources(
    direction_set: dict[str, Any],
    validation: dict[str, Any],
    body: dict[str, Any],
    receipt: dict[str, Any],
    genesis: dict[str, Any],
    vectors: list[dict[str, Any]],
    formal: bool,
    limits: Limits,
) -> None:
    """Caller resolves immutable objects; caller still verifies committed state/evidence.

    All vectors are supplied in direction-set order. Exclusion truth and R+ completeness
    require the trusted snapshot; this interface does not invent it.
    """
    pairs = [
        ("JDirectionSet", direction_set),
        ("JDirectionValidation", validation),
        ("BodyReadoutParameters", body),
        ("JEffectReceipt", receipt),
        ("GenesisManifestV3", genesis),
    ]
    for name, value in pairs:
        check_j_message(name, value, limits)
    s, v, b, r, g = [normalize(n, x, limits) for n, x in pairs]
    check_genesis("GenesisManifestV3", g, limits)

    def bound(ref: dict[str, Any], name: str, value: dict[str, Any]) -> None:
        fail(ref["object_type"] == SCHEMA["messages"][name]["tag"], "resource type")
        fail(ref["digest"].hex() == digest(name, value, limits), "resource digest")
        fail(
            ref["byte_length"] == len(encode_canonical(name, value, limits)),
            "resource length",
        )
        if "version" in value:
            fail(ref["resource_version"] == value["version"], "resource version")
        else:
            fail(ref["resource_version"] == "1", "resource schema version")

    for ref in (v["direction_set"], r["direction_set"]):
        bound(ref, "JDirectionSet", s)
    for ref in (r["validation"],):
        bound(ref, "JDirectionValidation", v)
    for ref in (r["body_parameters"], g["body_readout"]):
        bound(ref, "BodyReadoutParameters", b)
    bound(r["genesis"], "GenesisManifestV3", g)
    for key in ("carrier", "model", "tokenizer"):
        fail(r[key] == s["hook"][key], "carrier binding")
    fail(r["body"]["version"] == b["version"], "body version")
    if formal:
        fail(not v["fixture"] and r["mode"] == 1, "fixture/rehearsal not formal")
        fail(
            s["hook"]["prefill_mask"] != 3 and s["hook"]["decode_mask"] == 1,
            "formal prediction positions",
        )
    fail(len(s["directions"]) == len(v["directions"]) == len(vectors), "resource lists")
    for d, checked, vector in zip(
        s["directions"], v["directions"], vectors, strict=True
    ):
        check_j_message("JDirectionVector", vector, limits)
        vector = normalize("JDirectionVector", vector, limits)
        fail(d["vector"] == checked["vector"], "validation direction order")
        bound(d["vector"], "JDirectionVector", vector)
        fail(len(vector["values"]) == s["hook"]["width"], "vector width")
        total = 0.0
        for x in vector["values"]:
            total += x * x
            fail(math.isfinite(total), "norm overflow")
        fail(math.sqrt(total) == checked["measured_norm"], "measured norm")
        fail(d["beta"] <= checked["max_beta"], "single beta budget")
    fail(r["total_bound"] <= v["combination_max_beta"], "combination budget")
    for use in r["directions"]:
        if use["status"] != 1:
            continue
        matches = [
            d
            for d in s["directions"]
            if d["kind"] == use["kind"]
            and d.get("tendency_id") == use.get("tendency_id")
        ]
        fail(len(matches) == 1, "included identity")
        d = matches[0]
        fail(
            use["direction"] == d["vector"] and use["beta"] == d["beta"],
            "included binding",
        )
        fail(use["validation"] == r["validation"], "included validation")
