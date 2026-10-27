"""P5-36 actual-state exchange. Run after test:core-contracts builds both probes.

All IO is synthetic test data. Rust exports every prefix, Python and TypeScript
decode/re-encode each wire format independently, then Rust restores those returned
bytes and verifies every in-memory bit plus continuation (not just DTO equality).
Usage: python core/contracts/verify_theta_restore.py --output <temporary-directory>
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from typing import Any

from fixture_inputs import LIMITS
from verify import ROOT, probe


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    exported, returned = output / "actual-states.json", output / "returned-states.json"
    env = os.environ.copy()
    env.pop("P536_RESTORE", None)
    env["P536_EXPORT"] = str(exported)
    command = [
        "cargo",
        "test",
        "--manifest-path",
        "core/Cargo.toml",
        "-p",
        "core-kernel",
        "dynamics::storage::tests::actual_states_all_prefixes_roundtrip_and_continuation",
        "--",
        "--exact",
        "--nocapture",
    ]
    subprocess.run(command, cwd=ROOT, env=env, check=True)
    rows: list[dict[str, Any]] = json.loads(exported.read_text(encoding="utf-8"))
    assert len(rows) == 65 and [r["prefix"] for r in rows] == list(range(65))
    commands = {
        "python": [sys.executable, str(ROOT / "core/contracts/python_probe.py")],
        "typescript": [
            "node",
            str(
                ROOT
                / "packages/contracts/src/generated/.test-build/core/contracts/probe.js"
            ),
        ],
    }
    restored = []
    for language, adapter in commands.items():
        for encoding in ("canonical", "protobuf"):
            queries = [
                dict(
                    name="ThetaSnapshotV2", op=encoding, hex=r[encoding], limits=LIMITS
                )
                for r in rows
            ]
            answers = probe(adapter, queries)
            for original, answer in zip(rows, answers, strict=True):
                assert answer["ok"], (language, encoding, original["prefix"], answer)
                assert answer["value"] == original["value"]
                for wire in ("canonical", "protobuf"):
                    assert answer[wire] == original[wire], (
                        language,
                        wire,
                        original["prefix"],
                    )
                restored.append(
                    dict(
                        language=language,
                        encoding=encoding,
                        prefix=original["prefix"],
                        hex=answer[encoding],
                    )
                )
    returned.write_text(json.dumps(restored, indent=2) + "\n", encoding="utf-8")
    env.pop("P536_EXPORT")
    env["P536_RESTORE"] = str(returned)
    subprocess.run(command, cwd=ROOT, env=env, check=True)
    result = dict(
        result="passed",
        actual_prefixes=len(rows),
        returned_states=len(restored),
        languages=list(commands),
        encodings=["canonical", "protobuf"],
        state_oracle="exhaustive fields, f64::to_bits",
        continuation="same remaining events, changes, signals, bounds",
    )
    (output / "result.json").write_text(
        json.dumps(result, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(result))


if __name__ == "__main__":
    main()
