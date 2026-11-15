"""Only permitted arithmetic primitives, prereg v2 §3.2; no imported oracle flow.

Sources pinned at 8839dcae73c98437f61222f49d08cafd9ba8f4e6:
dynamics/tests/d07_oracle.py (bits, outward, Fraction operations) and
fast/contribution/tests/generate_oracle.py (one rounded Fraction conversion).
Hand expectations below are literal binary spacing/parity calculations.
"""

import math
import struct
from fractions import Fraction as Q

__all__ = ["Q", "bits", "rounded", "outward", "check_hand_vectors"]


def bits(x: float | Q | int) -> str:
    return f"{struct.unpack('<Q', struct.pack('<d', float(x)))[0]:016x}"


def rounded(x: Q) -> Q:
    return Q(float(x))


def outward(x: Q, up: bool) -> float:
    y = float(x)
    if (up and Q(y) < x) or (not up and Q(y) > x):
        y = math.nextafter(y, math.inf if up else -math.inf)
    return y


def check_hand_vectors() -> dict[str, int | str]:
    q, m = Q(1, 2**1074), Q(1, 2**1022)
    cases = [
        (Q(0), "0000000000000000"),
        (q, "0000000000000001"),
        (m - q, "000fffffffffffff"),
        (m, "0010000000000000"),
        (Q(1), "3ff0000000000000"),
        (Q(2), "4000000000000000"),
        (q / 2, "0000000000000000"),
        (3 * q / 2, "0000000000000002"),
        (5 * q / 2, "0000000000000002"),
        (m - q / 2, "0010000000000000"),
        (1 + Q(1, 2**53), "3ff0000000000000"),
        (1 + Q(3, 2**53), "3ff0000000000002"),
        (2 - Q(1, 2**53), "4000000000000000"),
    ]
    for x, expected in cases:
        assert bits(float(x)) == expected, (str(x), expected)
        assert bits(rounded(x)) == expected
    assert bits(float(-q / 2)) == "8000000000000000"
    assert bits(rounded(-q / 2)) == "0000000000000000"
    directions = [
        (1 + Q(1, 2**54), "3ff0000000000000", "3ff0000000000001"),
        (-1 - Q(1, 2**54), "bff0000000000001", "bff0000000000000"),
        (q / 2, "0000000000000000", "0000000000000001"),
        (m, "0010000000000000", "0010000000000000"),
    ]
    for x, lo, hi in directions:
        assert bits(outward(x, False)) == lo
        assert bits(outward(x, True)) == hi
    return {
        "nearest_and_roundtrip": len(cases),
        "signed_zero": 2,
        "directed": 2 * len(directions),
        "status": "passed",
    }


if __name__ == "__main__":
    import json

    print(json.dumps(check_hand_vectors()))
