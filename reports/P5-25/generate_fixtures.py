"""Fixed-seed, original THETA-1/2 inputs/oracles; standard library only.
Run from any directory. Does not edit the original checker or acceptance thresholds.
Binary schema: THETA-1 = 500*(4 little-endian f64, u32 N, N*(f64 a,u8 exogenous,3 f64 b,var,g)).
THETA-2 = 6000 little-endian f64 Gaussian samples (before multiplication by 0.2).
"""
import dataclasses
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import random
import struct
import sys

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).resolve().parent / "evidence" / "tests"
source = ROOT / "reports/echo-core-v2-draft/check_law.py"
spec = importlib.util.spec_from_file_location("original_check_law", source)
module = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = module
spec.loader.exec_module(module)
OUT.mkdir(parents=True, exist_ok=True)
results = [dataclasses.asdict(module.check_theta()), dataclasses.asdict(module.check_feedback_loop())]
assert all(r["ok"] for r in results)
rng = random.Random(module.SEED + 10)
rows = 0
with (OUT / "theta1.bin").open("wb") as f:
    for _ in range(500):
        nb, ns, rho = rng.uniform(1, 20), rng.uniform(1, 20), rng.uniform(0.9, 1.0)
        b0 = rng.uniform(-1, 1)
        count = rng.randint(1, 300)
        f.write(struct.pack("<ddddI", nb, ns, rho, b0, count))
        sw = sa = sq = 0.0
        b, var = b0, 0.3**2
        for _ in range(count):
            a = rng.uniform(-1, 1)
            exo = rng.random() < 0.6
            if exo:
                sq = rho * sq + (a - b)**2
                sw, sa = rho * sw + 1, rho * sa + a
                b = (nb * b0 + sa) / (nb + sw)
                var = (ns * 0.3**2 + sq) / (ns + sw)
            g = min(3.0, max(0.2, 0.3 / math.sqrt(var)))
            f.write(struct.pack("<dBddd", a, exo, b, var, g))
            rows += 1
# Lossless repository transport: the large original remains the logical oracle.
# Rust reads all 19 parts in order; no sample, assertion or float is transformed.
theta1_bytes = (OUT / "theta1.bin").read_bytes()
for part, start in enumerate(range(0, len(theta1_bytes), 131072)):
    (OUT / f"theta1.bin.part-{part:03}").write_bytes(theta1_bytes[start:start + 131072])
rng = random.Random(module.SEED + 11)
with (OUT / "theta2-noise.bin").open("wb") as f:
    for _ in range(6000):
        f.write(struct.pack("<d", rng.gauss(0, 1)))
manifest = {
    "original_source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
    "seed": module.SEED, "theta1_events": rows, "python": sys.version,
    "original_results": results,
    "fixtures": {p.name: {"bytes": p.stat().st_size, "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
                 for p in sorted(OUT.glob("*.bin"))},
}
(OUT / "original-and-fixtures.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(manifest, ensure_ascii=False, indent=2))
