"""Reproduce MATH-U2U3-01 witnesses, with exact rational distance/error checks.

Python 3.10+, standard library only. Run from the repository root:
    python reports/MATH-U2U3-01/verify.py
    python reports/MATH-U2U3-01/verify.py --samples 20000 --seed 20261112
    python reports/MATH-U2U3-01/verify.py --json

This is a research attachment, not a replacement for Rust tests or product
acceptance. Operations mirror the pinned Rust order, with no fused operation.
All distances and defects use Fraction, never rounded float subtraction.
The libm ln(2) case is a source-level evaluation of its k=-1 branch; it does
not call the platform expm1 and does not claim to execute the Rust crate.
The tiny-input case uses the inspected libm 0.2.16 return-x branch.
Nothing is written to the repository by running this script.
"""

from __future__ import annotations

import argparse
import json
import math
import platform
import random
import struct
import sys
from fractions import Fraction as Q
from typing import Any

MAIN = "925fb9af05fd0912ca0b6f8006f62fc95550ae33"
PR171 = "3f03c34f9eabce2f89b043725622e7efc0b9ef18"
LIBM_URL = (
    "https://github.com/rust-lang/compiler-builtins/blob/"
    "libm-v0.2.16/libm/src/math/expm1.rs"
)
LIBM_BLOB = "3ce1d886bb1940ea09de06c89184e5e0da205707"


def normalize(value: float) -> float:
    assert math.isfinite(value)
    return 0.0 if value == 0.0 else value


def bits(value: float) -> str:
    return f"0x{struct.unpack('>Q', struct.pack('>d', value))[0]:016x}"


def datum(value: float) -> dict[str, str]:
    return {"hex": value.hex(), "bits": bits(value), "rational": str(Q(value))}


def operation(name: str, exact: Q, rounded: float) -> dict[str, Any]:
    return {"operation": name, "exact": str(exact), "rounded": datum(rounded),
            "error": str(Q(rounded) - exact)}


def box(value: Q) -> Q:
    return min(Q(1), max(Q(-1), value))


def a1(x: float, h: float, v: float) -> tuple[tuple[float, float], list[dict[str, Any]]]:
    d = normalize(v - h)
    s = normalize(x + d)
    trace = [operation("d=RN(v-h)", Q(v) - Q(h), d),
             operation("s=RN(x+d)", Q(x) + Q(d), s)]
    if -1.0 < s < 1.0:
        return (s, v), trace
    bound = 1.0 if s >= 1.0 else -1.0
    loss = normalize(bound - x)
    t = normalize(h + loss)
    hp = normalize(min(max(t, min(h, v)), max(h, v)))
    trace += [operation("loss=RN(bound-x)", Q(bound) - Q(x), loss),
              operation("t=RN(h+loss)", Q(h) + Q(loss), t)]
    return (bound, hp), trace


def ideal_a1(x: float, h: float, v: float) -> tuple[Q, Q]:
    xp = box(Q(x) + Q(v) - Q(h))
    return xp, Q(h) + xp - Q(x)


def phi(value: float, rest: float, pull: float, clip: bool) -> tuple[float, list[dict[str, Any]]]:
    diff = normalize(rest - value)
    product = normalize(pull * diff)
    total = normalize(value + product)
    result = normalize(min(1.0, max(-1.0, total))) if clip else total
    trace = [operation("diff=RN(rest-value)", Q(rest) - Q(value), diff),
             operation("product=RN(pull*diff)", Q(pull) * Q(diff), product),
             operation("total=RN(value+product)", Q(value) + Q(product), total)]
    return result, trace


def libm_ln2_case() -> dict[str, Any]:
    """Evaluate only the branch used by the specified input, from pinned source.

    Constants/formulas attributed to the MIT-licensed libm source above.
    This small case evaluation is not a general-purpose expm1 implementation.
    """
    rate = float.fromhex("0x1.62e42fefa39efp-1")
    z = -rate
    hi = z + float.fromhex("0x1.62e42fee00000p-1")
    lo = -float.fromhex("0x1.a39ef35793c76p-33")
    reduced = hi - lo
    correction = (hi - reduced) - lo
    # Polynomial constants are written as exact binary64 hex literals.
    half = 0.5 * reduced
    square_half = reduced * half
    q1, q2, q3, q4, q5 = [float.fromhex(value) for value in (
        "-0x1.11111111110f4p-5", "0x1.a01a019fe5585p-10",
        "-0x1.4ce199eaadbb7p-14", "0x1.0cfca86e65239p-18",
        "-0x1.afdb76e09c32dp-23")]
    r1 = 1.0 + square_half * (q1 + square_half * (q2 + square_half * (
        q3 + square_half * (q4 + square_half * q5))))
    assert r1 == 1.0
    t = 3.0 - r1 * half
    error = square_half * ((r1 - t) / (6.0 - reduced * t))
    error = reduced * (error - correction) - correction
    error = error - square_half
    result = 0.5 * (reduced - error) - 0.5
    assert bits(result) == "0xbfe0000000000000"
    assert correction == 0.0 and t == 3.0
    return {"rate": datum(rate), "delta_ns": 1_000_000_000,
            "reduced": datum(reduced), "correction": datum(correction),
            "expm1": datum(result), "pull": datum(-result),
            "method": "source branch evaluation, not Rust execution"}


def event_pair(name: str, x: float, xt: float, h: float, ht: float, v: float) -> dict[str, Any]:
    left, lt = a1(x, h, v)
    right, rt = a1(xt, ht, v)
    li = ideal_a1(x, h, v)
    ri = ideal_a1(xt, ht, v)
    le = [Q(left[j]) - li[j] for j in range(2)]
    re = [Q(right[j]) - ri[j] for j in range(2)]
    before = abs(Q(x) - Q(xt)) + abs(Q(h) - Q(ht))
    after = sum(abs(Q(left[j]) - Q(right[j])) for j in range(2))
    contracted = abs((Q(x) - Q(h)) - (Q(xt) - Q(ht)))
    error_difference = sum(abs(le[j] - re[j]) for j in range(2))
    assert after <= contracted + error_difference
    return {"name": name, "before": str(before), "after": str(after),
            "excess": str(after - before), "ratio": str(after / before),
            "sharp_envelope": str(contracted + error_difference),
            "left": {"x": datum(x), "h": datum(h), "v": datum(v),
                     "steps": lt, "out": [datum(a) for a in left],
                     "defect": [str(a) for a in le]},
            "right": {"x": datum(xt), "h": datum(ht), "v": datum(v),
                      "steps": rt, "out": [datum(a) for a in right],
                      "defect": [str(a) for a in re]}}


def witnesses() -> list[dict[str, Any]]:
    unit = 2.0 ** -54
    cases = [
        event_pair("A1-small", 0.25 + unit, 0.25 + 2 * unit, 0.0, 0.0, 0.25),
        event_pair("A1-relative", unit, math.nextafter(unit, math.inf), 0.0, 0.0, 0.5),
        event_pair("A1-large-slot", 1.0 - 2.0 ** -53, 1.0 - 2.0 ** -52,
                   float(2 ** 54 + 4), float(2 ** 54 + 4), 0.0),
    ]
    for case in cases:
        assert Q(case["after"]) > Q(case["before"])
        assert Q(case["after"]) == Q(case["sharp_envelope"])
    x, xt = 0.5 - 3 * unit, 0.5 - 4 * unit
    left, lt = phi(x, 1.0, 0.5, True)
    right, rt = phi(xt, 1.0, 0.5, True)
    before, after = abs(Q(x) - Q(xt)), abs(Q(left) - Q(right))
    assert before == Q(1, 2 ** 54) and after == Q(1, 2 ** 53)
    cases.append({"name": "Phi-half-pull", "before": str(before), "after": str(after),
                  "left": {"input": datum(x), "steps": lt, "out": datum(left)},
                  "right": {"input": datum(xt), "steps": rt, "out": datum(right)},
                  "pull_source": libm_ln2_case()})
    # libm 0.2.16 returns its argument for |z|<2^-54, so this pull is exact.
    p = 2.0 ** -55
    stuck_x, tx = phi(1.0, 0.0, p, True)
    stuck_h, th = phi(1.0, 0.0, p, False)
    assert stuck_x == stuck_h == 1.0
    # For B, [h,v]=[1,1] forces q=1 even if r is zero.
    for r in (0.0, 0.25, 1.0):
        w = r * 1.0
        q = min(max(w, min(stuck_h, 1.0)), max(stuck_h, 1.0))
        assert q == stuck_h == 1.0
    cases.append({"name": "Phi-B-stagnation", "rate_and_pull": datum(p),
                  "delta_ns": 1_000_000_000, "x_steps": tx, "h_steps": th,
                  "out_x": datum(stuck_x), "out_h": datum(stuck_h)})
    x = h = 1.0
    for _ in range(32):
        x, _ = phi(x, 0.0, 0.5, True)
        h, _ = phi(h, 0.0, 0.5, False)
        assert x == h == 0.5
        (x, h), _ = a1(x, h, 1.0)
        assert x == h == 1.0
    cases.append({"name": "U3-A1-periodic", "cycles": 32,
                  "pre_event": "1/2", "post_event": "1",
                  "claim": "finite-prefix reproduction; unlimited repetition proved in REPORT"})
    return cases


def search(samples: int, seed: int) -> dict[str, Any]:
    rng = random.Random(seed)
    violations = 0
    first: dict[str, Any] | None = None
    for index in range(samples):
        x = rng.uniform(-0.9, 0.9)
        xt = math.nextafter(x, math.inf)
        h = rng.uniform(-1.0, 1.0)
        v = rng.uniform(-1.0, 1.0)
        case = event_pair("adjacent-x-search", x, xt, h, h, v)
        if Q(case["after"]) > Q(case["before"]):
            violations += 1
            if first is None:
                first = {"sample": index, "case": case}
    return {"samples": samples, "seed": seed, "violations": violations,
            "first": first, "purpose": "directed search, not a proof or prevalence estimate"}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--samples", type=int, default=0)
    parser.add_argument("--seed", type=int, default=20261112)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    assert sys.float_info.radix == 2 and sys.float_info.mant_dig == 53
    assert args.samples >= 0
    cases = witnesses()
    result: dict[str, Any] = {"main": MAIN, "pr171": PR171, "python": platform.python_version(),
              "libm_source": LIBM_URL, "libm_blob": LIBM_BLOB,
              "distance": "exact Fraction", "rust_crate_executed": False,
              "witnesses": cases, "search": search(args.samples, args.seed)}
    if args.json:
        print(json.dumps(result, indent=2, ensure_ascii=False))
    else:
        for case in cases:
            print(case["name"], "PASS", case.get("before", ""), "->", case.get("after", ""))
        print("search", result["search"]["samples"], "samples;",
              result["search"]["violations"], "counterexamples")
        print("All distances and local-envelope checks used exact rational arithmetic.")
        print("Rust crate, Windows, and end-to-end product acceptance: not_run.")


if __name__ == "__main__":
    main()
