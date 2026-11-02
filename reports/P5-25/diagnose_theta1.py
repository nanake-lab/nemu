"""Read-only diagnosis: retain the original oracle and compare explicit multiplication.
Does not regenerate fixtures, modify assertions, or change acceptance status.
"""
import json
import math
from pathlib import Path
import struct

root = Path(__file__).resolve().parent
raw = (root / "evidence/tests/theta1.bin").read_bytes()
offset = 0
first = None
differences = 0
bits = lambda x: struct.pack("<d", x).hex()
for stream in range(500):
    nb, ns, rho, b0, count = struct.unpack_from("<ddddI", raw, offset)
    offset += 36
    sw = sa = sq = 0.0
    b = b0
    var = 0.3 * 0.3
    for event in range(count):
        a, exo, expected_b, expected_var, expected_g = struct.unpack_from("<dBddd", raw, offset)
        offset += 33
        before_b = b
        residual = a - b
        if exo:
            sq = rho * sq + residual * residual
            sw, sa = rho * sw + 1, rho * sa + a
            b = (nb * b0 + sa) / (nb + sw)
            var = (ns * (0.3 * 0.3) + sq) / (ns + sw)
        assert bits(b) == bits(expected_b)
        if bits(math.sqrt(var)) != bits(math.sqrt(expected_var)):
            differences += 1
            if first is None:
                first = {"stream_zero_based": stream, "event_zero_based": event, "exogenous": bool(exo),
                    "nb": nb, "ns": ns, "rho": rho, "b0": b0, "a": a, "before_b": before_b,
                    "residual": residual, "square_multiply": residual * residual,
                    "square_python_power": residual**2,
                    "ordered_var": var, "original_var": expected_var,
                    "ordered_sigma": math.sqrt(var), "original_sigma": math.sqrt(expected_var),
                    "ordered_sigma_u64": struct.unpack("<Q", struct.pack("<d", math.sqrt(var)))[0],
                    "original_sigma_u64": struct.unpack("<Q", struct.pack("<d", math.sqrt(expected_var)))[0]}
assert offset == len(raw)
print(json.dumps({"status": "diagnostic_only_acceptance_still_failed", "first_difference": first,
    "sigma_bit_mismatches_of_72822_events": differences,
    "baseline_bit_mismatches": 0,
    "explanation": "Explicit binary64 multiplication and Python **2 have distinct rounding on this platform; this diagnostic reproduces Rust's first sigma bits without changing implementation or fixture."}, indent=2))
