"""Lightweight input/IDL audit, independent of Rust's fixture freezer. No stores."""
from __future__ import annotations

import json
import struct
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "core/contracts"))
sys.path.insert(0, str(ROOT / "workers/irl/generated"))
from fixture_retention import illegal_retention, legal_retention  # noqa: E402
from echo import runtime  # noqa: E402
from echo.bridge import from_json  # noqa: E402


def main() -> None:
    limits = runtime.Limits(1048576, 4096, 32)
    legal = legal_retention()
    illegal = illegal_retention()
    for q in legal:
        value = runtime.normalize(q["name"], from_json(q["name"], q["value"]), limits)
        for encode, decode in ((runtime.encode_canonical, runtime.decode_canonical),
                               (runtime.encode_protobuf, runtime.decode_protobuf)):
            assert decode(q["name"], encode(q["name"], value, limits), limits) == value
    for vector in illegal:
        q = vector["request"]
        try:
            if q["op"] == "encode":
                runtime.normalize(q["name"], from_json(q["name"], q["value"]), limits)
            else:
                raw = bytes.fromhex(q["hex"])
                # Replace only the nonfinite target in our complete hand-built messages;
                # a valid control proves another missing field did not cause rejection.
                if "nonfinite" in vector["label"]:
                    control = raw[:-8] + struct.pack("<d", 1.0)
                    runtime.decode_protobuf(q["name"], control, limits)
                runtime.decode_protobuf(q["name"], raw, limits)
        except ValueError as error:
            if "nonfinite" in vector["label"]:
                assert "nonfinite" in str(error), (vector["label"], str(error))
            if "reserved" in vector["label"]:
                assert "unknown" in str(error), (vector["label"], str(error))
        else:
            raise AssertionError(vector["label"])
    baseline = ROOT / "core/contracts/baselines/p5-32"
    for p in baseline.glob("*.proto"):
        published = subprocess.check_output(["git", "show", "91b31ed:core/contracts/" + p.name], cwd=ROOT)
        assert p.read_bytes() == published, p.name
    print(json.dumps(dict(targeted_legal=len(legal), targeted_illegal=len(illegal),
                          hand_built_nonfinite_controls=6, p5_32_proto_files=7, status="passed")))


if __name__ == "__main__":
    main()
