"""ADR 0035 §9 P5-69(2,5,10): exact rational oracle; synthetic data only.

Run to regenerate d07-vectors.json. Uses Python standard library only. Fractions
prove the inequalities for each finite fixture; this is not a proof by sampling.
No production C/interval code is imported. All input floats are exact rationals.
"""
import json
import math
import struct
import sys
from fractions import Fraction as F
from pathlib import Path


def bits(x: float) -> str:
    return str(struct.unpack("<Q", struct.pack("<d", x))[0])


def outward(x: F, up: bool) -> float:
    y = float(x)
    if (up and F(y) < x) or (not up and F(y) > x):
        y = math.nextafter(y, math.inf if up else -math.inf)
    return y


def vectors() -> dict[str, object]:
    u, eta = F(1, 2**53), F(1, 2**1075)
    gamma = 3 * u / (1 - 3 * u)
    p, s, total = 1.0, 0.0, 0.0
    rows = []
    # §3.2 reachable witness, with no residual-square underflow in the C path.
    for n in range(1, 1075):
        a = (-0.5 if n % 2 else 0.5) if n <= 1072 else (1 - 0.5 * total if n == 1073 else -0.5)
        # Round each exact rational operation separately, nearest/even.
        s = float(F(float(F(s) / 2)) + 1)
        p = float(F(p) / 2)
        total = float(F(float(F(total) / 2)) + F(a))
        numerator = F(p) + F(total)  # b0 = 1
        denominator = F(p) + F(s)
        exact = numerator / denominator
        error = gamma * abs(exact) + (1 + gamma) * (u * F(p) + eta) + eta
        b = float(F(float(numerator)) / F(float(denominator)))
        assert abs(F(b) - exact) <= error
        rows.append([bits(x) for x in (a, p, s, total, b, outward(exact-error, False), outward(exact+error, True))])
    assert p == math.ulp(0.0) and total == 0 and b == 0

    # Real-arithmetic birth-share inequalities, independent of floating recurrence.
    birth: list[dict[str, object]] = []
    birth_steps = []
    for p0 in (F(1, 4), F(1), F(4), F(16)):
        real_p, real_s = p0, F(0)
        float_p, float_s = float(p0), 0.0
        hmin = F(1, 8)
        alpha_min = min(1 / p0, hmin)
        for n in range(1, 65):
            rho = (F(1, 2), F(3, 4), F(7, 8))[n % 3]
            real_p, real_s = rho * real_p, rho * real_s + 1
            beta = real_p / (real_p + real_s)
            assert beta <= p0 / (p0 + n)
            assert beta <= (1 - alpha_min)**n
            float_p = float(rho * F(float_p))
            float_s = float(F(float(rho * F(float_s))) + 1)
            birth_steps.append([bits(float(p0)), str(n), bits(float(rho)), bits(float_p), bits(float_s)])
        birth.append({"p0": str(p0), "steps": 64, "beta": str(beta)})
    return {"source": "ADR0035 sections 3.2, 3.5; exact Fraction oracle", "birth": birth, "birth_steps": birth_steps, "underflow": rows}


def check_trace() -> None:
    """§9(5): observed dynamic-rho states, exact stored-statistics oracle."""
    rows = json.load(sys.stdin)
    u, eta = F(1, 2**53), F(1, 2**1075)
    gamma = 3 * u / (1 - 3 * u)
    for row in rows:
        p, s, total, b0, left, right = [F(struct.unpack("<d", struct.pack("<Q", int(x)))[0]) for x in row]
        exact_left = (p * b0 + total) / (p + s)
        exact_right = (-p * b0 + total) / (p + s)
        left_error = gamma * abs(exact_left) + (1 + gamma) * (u * p + eta) + eta
        right_error = gamma * abs(exact_right) + (1 + gamma) * (u * p + eta) + eta
        assert abs(left - exact_left) <= left_error
        assert abs(right - exact_right) <= right_error
        assert abs((left - right) - p / (p + s) * 2 * b0) <= left_error + right_error
    print(f"dynamic same-H birth difference: {len(rows)} rational envelopes passed")


if __name__ == "__main__":
    target = Path(__file__).with_name("d07-vectors.json")
    result = vectors()
    if "--trace" in sys.argv:
        check_trace()
    elif "--check" in sys.argv:
        assert json.loads(target.read_text(encoding="utf-8")) == result
        print("exact rational birth bounds: 256 prefixes; rounding envelope: 1074 prefixes; passed")
    else:
        target.write_text(json.dumps(result, separators=(",", ":")) + "\n", encoding="utf-8")
