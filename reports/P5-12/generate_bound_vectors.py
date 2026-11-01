"""Independent exact-rational oracle for two-term nonnegative beta budgets.

Only synthetic binary64 inputs. No production imports, models or storage.
The expected upper bound is the least binary64 number >= the exact sum;
an unrepresentable finite upper bound is a fault, without a tolerance.
"""

import argparse
import json
import math
import random
import struct
from fractions import Fraction
from pathlib import Path


def value(bits: int) -> float:
    return float(struct.unpack(">d", bits.to_bytes(8, "big"))[0])


def bits(number: float) -> str:
    return struct.pack(">d", number).hex()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="compare without changing the fixture")
    args = parser.parse_args()
    randomizer = random.Random(51220261026)
    raw_pairs = [
        (0, 0), (1, 1), (0x3FE0000000000000, 0x3FE0000000000000),
        (0x3FF0000000000000, 1), (0x7FEFFFFFFFFFFFFF, 1),
        (0x7FEFFFFFFFFFFFFF, 0x7FEFFFFFFFFFFFFF),
        (0x3FF0000000000000, 0x3CA0000000000000),
        (0x0010000000000000, 0x000FFFFFFFFFFFFF),
    ]
    raw_pairs += [
        (randomizer.randrange(0x7FF0000000000000),
         randomizer.randrange(0x7FF0000000000000))
        for _ in range(300)
    ]
    vectors: list[dict[str, str | None]] = []
    for left, right in raw_pairs:
        exact = Fraction.from_float(value(left)) + Fraction.from_float(value(right))
        try:
            rounded = float(exact)
            if Fraction.from_float(rounded) < exact:
                rounded = math.nextafter(rounded, math.inf)
            expected = bits(rounded) if math.isfinite(rounded) else None
        except OverflowError:
            expected = None
        vectors.append({"left": f"{left:016x}", "right": f"{right:016x}", "upper": expected})
    root = Path(__file__).resolve().parents[2]
    target = root / "core/crates/core-kernel/src/modulation/tests/bound-vectors.json"
    content = json.dumps(vectors, indent=2) + "\n"
    if args.check:
        if target.read_text(encoding="utf-8") != content:
            raise ValueError("Exact-rational oracle does not match the committed fixture")
        print(f"Verified {len(vectors)} exact-rational vectors without changing {target}")
    else:
        target.write_text(content, encoding="utf-8")
        print(f"{len(vectors)} exact-rational vectors written to {target}")


if __name__ == "__main__":
    main()
