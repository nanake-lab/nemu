"""P5-36 regression: one caller budget across codecs and theta semantics.

Generate large inputs in memory, not frozen megabyte-sized golden files.
The real-state fixture is unchanged and every probe checks theta semantics.
"""

from __future__ import annotations

import copy
import json
from collections.abc import Callable
from pathlib import Path
from typing import Any

Query = dict[str, Any]
Probe = Callable[[list[str], list[Query]], list[Query]]


def verify(commands: dict[str, list[str]], probe: Probe) -> None:
    here = Path(__file__).resolve().parent
    original = json.loads((here / "fixtures/theta-states.json").read_text())[1]["value"]
    limits = dict(max_bytes=4_194_304, max_items=4096, max_depth=64)
    checks = 0
    for size in (1_048_576, 1_048_577):
        value = copy.deepcopy(original)
        value["estimator"]["last_event"]["event_id"] = "x" * size
        parameters = copy.deepcopy(value["parameters"])
        parameters["law_id"] = "l" * size
        for name, message in (
            ("ThetaLastEvent", value["estimator"]["last_event"]),
            ("ThetaSnapshotV2", value),
            ("ThetaEstimatorParameters", parameters),
        ):
            request = dict(name=name, op="encode", value=message, limits=limits)
            expected = probe(commands["Python"], [request])[0]
            assert expected["ok"], (name, size, expected.get("error"))
            assert all(len(expected[w]) // 2 < limits["max_bytes"] for w in ("canonical", "protobuf"))
            queries = [request] + [
                dict(name=name, op=w, hex=expected[w], limits=limits)
                for w in ("canonical", "protobuf")
            ]
            # Tight caller budgets must still fail, including semantic validation.
            denied = [dict(q, limits={**limits, "max_bytes": 1_048_576}) for q in queries]
            if name == "ThetaSnapshotV2":
                for budget in (dict(max_items=3), dict(max_depth=1)):
                    denied.extend(dict(q, limits={**limits, **budget}) for q in queries)
                bad = copy.deepcopy(request)
                bad["value"]["parameters_digest"] = "00" * 32
                denied.append(bad)
            for language, command in commands.items():
                answers = probe(command, queries + denied)
                for i, answer in enumerate(answers):
                    assert answer["ok"] == (i < len(queries)), (language, name, size, i, answer.get("error"))
                    if answer["ok"]:
                        assert answer == expected, (language, name, size, i)
                    checks += 1
            print(json.dumps(dict(theta_limits=name, event_or_law_id_bytes=size,
                                  canonical_bytes=len(expected["canonical"]) // 2,
                                  protobuf_bytes=len(expected["protobuf"]) // 2,
                                  result="passed")))
    print(json.dumps(dict(theta_limits_checks=checks, languages=list(commands), result="passed")))


if __name__ == "__main__":
    import os
    import sys
    from verify import probe

    root = Path(__file__).resolve().parents[2]
    target = Path(os.environ.get("CARGO_TARGET_DIR", str(root / "core/target")))
    verify({
        "Rust": [str(target /
                     "debug/examples" / ("contract_probe.exe" if os.name == "nt" else "contract_probe"))],
        "Python": [sys.executable, str(root / "core/contracts/python_probe.py")],
        "TypeScript": ["node", str(root / "packages/contracts/src/generated/.test-build/core/contracts/probe.js")],
    }, probe)
