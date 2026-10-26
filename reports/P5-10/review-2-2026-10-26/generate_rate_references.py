"""Exact-rational oracle for synthetic P5-10 mixed-scale rate vectors.

Run with Python's standard library only. Inputs are binary64 bit patterns;
Fraction evaluates the reference quotient exactly, and float rounds once.
This is a test-data generator, not an alternative production implementation.
"""

import itertools
import json
import math
from fractions import Fraction
from pathlib import Path
import struct


def bits(value: float) -> str:
    return f"{struct.unpack('>Q', struct.pack('>d', value))[0]:016x}"


def from_bits(value: int) -> float:
    return struct.unpack('>d', struct.pack('>Q', value))[0]


def reference(direction: list[float], rates: list[float]) -> float:
    weights = [Fraction.from_float(abs(value)) for value in direction]
    numerator = sum(
        (weight * Fraction.from_float(rate) for weight, rate in zip(weights, rates)),
        Fraction(0),
    )
    return float(numerator / sum(weights, Fraction(0)))


def main() -> None:
    cases: list[dict[str, object]] = []

    def add(name: str, direction: list[float], rates: list[float]) -> None:
        assert abs(math.sqrt(sum(value * value for value in direction)) - 1.0) <= 1e-12
        expected = reference(direction, rates)
        assert math.isfinite(expected) and min(rates) <= expected <= max(rates)
        cases.append({
            "name": name,
            "direction_bits": [bits(value) for value in direction],
            "lambda_bits": [bits(value) for value in rates],
            "expected_bits": bits(expected),
        })

    smallest = from_bits(1)
    largest = float.fromhex('0x1.fffffffffffffp+1023')
    add('review-counterexample', [0.8, 0.6, smallest, 0.0],
        [1e-30, 1e-30, 1e308, 1e-30])

    # All channel orders; signs must not affect the absolute-weight mean.
    pairs = [(0.8, 1e-30), (-0.6, 1e-30), (smallest, 1e308), (-0.0, 1e-30)]
    for index, order in enumerate(itertools.permutations(pairs)):
        add(f'review-permutation-{index}', [pair[0] for pair in order],
            [pair[1] for pair in order])

    for raw in [1, 2, 3, 17, (1 << 52) - 1]:
        tiny = from_bits(raw)
        for shape, direction in enumerate([[0.8, -0.6, tiny, 0.0],
                                           [0.5, -0.5, math.sqrt(0.5), -tiny]]):
            for rate_index, large_rate in enumerate([1e308, largest]):
                rates = [1e-30] * 4
                rates[2 if shape == 0 else 3] = large_rate
                for offset in range(4):
                    add(f'mixed-{raw}-{shape}-{rate_index}-{offset}',
                        direction[offset:] + direction[:offset],
                        rates[offset:] + rates[:offset])

    for offset in range(4):
        for name, direction, rates in [
            ('overflowing-direct-sum', [0.8, 0.6, smallest, 0.0],
             [largest, largest / 2.0, largest, 1.0]),
            ('subnormal-output', [0.8, 0.6, smallest, 0.0],
             [smallest, 2.0 * smallest, 3.0 * smallest, smallest]),
            ('unit-tolerance', [math.nextafter(1.0, math.inf), smallest, 0.0, 0.0],
             [1e-30, largest, 1e-30, 1e-30]),
        ]:
            add(f'{name}-{offset}', direction[offset:] + direction[:offset],
                rates[offset:] + rates[:offset])
        for index, rate in enumerate([smallest, from_bits(1 << 52), 1e308, largest]):
            direction = [0.8, 0.6, smallest, 0.0]
            add(f'common-rate-{index}-{offset}', direction[offset:] + direction[:offset],
                [rate] * 4)

    output = Path(__file__).resolve().parents[3] / 'profiles' / 'structural_fixture' / 'fast' / 'rate-reference-vectors.json'
    payload = {
        "purpose": "Synthetic structural tests only; no behavior or life-lineage evidence",
        "oracle": "Exact Fraction(binary64 inputs), sum(abs(d_i)*lambda_i)/sum(abs(d_i)), one final binary64 rounding",
        "generator": "reports/P5-10/review-2-2026-10-26/generate_rate_references.py",
        "cases": cases,
    }
    output.write_text(json.dumps(payload, indent=2) + '\n', encoding='utf-8')
    print(f'{len(cases)} exact-rational reference vectors written to {output}')


if __name__ == '__main__':
    main()
