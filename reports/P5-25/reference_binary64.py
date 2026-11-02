"""V2-93 explicit binary64 reference. Never overwrites historical evidence.

Consumes the archived original inputs (no RNG, seed or event changes).
Each primitive is rounded separately; powers use x*x; no tolerance.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import sys


def finite(value: float) -> float:
    if not math.isfinite(value):
        raise ArithmeticError("nonfinite")
    return 0.0 if value == 0.0 else value


def mul(left: float, right: float) -> float:
    value = finite(left * right)
    if left != 0.0 and right != 0.0 and value == 0.0:
        raise ArithmeticError("underflow")
    return value


def add(left: float, right: float) -> float:
    return finite(left + right)


def div(left: float, right: float) -> float:
    value = finite(left / right)
    if left != 0.0 and value == 0.0:
        raise ArithmeticError("underflow")
    return value


def generate(out: Path) -> dict[str, object]:
    root = Path(__file__).resolve().parent
    source = b"".join((root / "evidence/tests" / f"theta1.bin.part-{part:03}").read_bytes()
                      for part in range(19))
    target = out / "theta1-mul.bin"
    if target.exists() or (out / "reference.json").exists():
        raise FileExistsError("Use a fresh output directory; evidence is immutable")
    output = bytearray()
    offset = rows = input_mismatch = 0
    for _ in range(500):
        nb, nv, rho, b0, count = struct.unpack_from("<ddddI", source, offset)
        output.extend(source[offset:offset+36])
        offset += 36
        mass = total = squares = 0.0
        baseline = b0
        prior_variance = mul(0.3, 0.3)
        variance = prior_variance
        for _ in range(count):
            reaction, exogenous, _, _, _ = struct.unpack_from("<dBddd", source, offset)
            if exogenous:
                residual = finite(reaction - baseline)
                mass = add(mul(rho, mass), 1.0)
                squares = add(mul(rho, squares), mul(residual, residual))
                total = add(mul(rho, total), reaction)
                baseline = div(add(mul(nb, b0), total), add(nb, mass))
                variance = div(add(mul(nv, prior_variance), squares), add(nv, mass))
            sigma = finite(math.sqrt(variance))
            gain = min(3.0, max(0.2, div(mul(1.0, 0.3), sigma)))
            row = struct.pack("<dBddd", reaction, exogenous, baseline, variance, gain)
            input_mismatch += row[:9] != source[offset:offset+9]
            output.extend(row)
            offset += 33
            rows += 1
    assert offset == len(source) and rows == 72822 and input_mismatch == 0
    out.mkdir(parents=True, exist_ok=True)
    target.write_bytes(output)
    report: dict[str, object] = {
        "reference": "P5-25-binary64-mul-v1", "python": sys.version,
        "events": rows, "streams": 500, "input_mismatches": input_mismatch,
        "seed": "original 20261019+10; inputs read from archived bytes, not regenerated",
        "original_sha256": hashlib.sha256(source).hexdigest(),
        "reference_sha256": hashlib.sha256(output).hexdigest(),
        "source_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "legacy_result": "failed (unchanged archived Python **2 comparison)",
    }
    (out / "reference.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", required=True, type=Path)
    print(json.dumps(generate(parser.parse_args().out_dir), indent=2))
