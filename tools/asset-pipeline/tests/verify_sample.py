"""Acceptance evidence only: never used to relax the production build's lint exit."""
from __future__ import annotations

import argparse
import difflib
import hashlib
import json
from pathlib import Path
import sys
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from gltf import read_json


def inspect(directory: Path) -> dict[str, Any]:
    model = read_json(directory / "nemu.vrm")
    vrm = model["extensions"]["VRMC_vrm"]
    probe = vrm["expressions"]["custom"]["pipelineProbe"]
    for binding in ("morphTargetBinds", "materialColorBinds", "textureTransformBinds"):
        assert probe[binding], binding
    profile = json.loads((directory / "nemu.profile.json").read_text(encoding="utf-8"))
    spring = next(s for s in model["extensions"]["VRMC_springBone"]["springs"] if s["name"] == "PipelineTail")
    names = [model["nodes"][joint["node"]]["name"] for joint in spring["joints"]]
    assert len(names) >= 8 and names == profile["rig"]["tailBones"]
    tail = next(mesh for mesh in model["meshes"] if mesh["name"].startswith("PipelineTail"))
    for primitive in tail["primitives"]:
        assert {"POSITION", "TEXCOORD_0", "JOINTS_0", "WEIGHTS_0"} <= primitive["attributes"].keys()
        assert model["accessors"][primitive["attributes"]["POSITION"]]["count"] > 0
    lint = json.loads((directory / "lint.json").read_text(encoding="utf-8"))
    errors = [d for d in lint["diagnostics"] if d["severity"] == "error"]
    exceptions = {"A1.3-custom-expressions": "Sample has no Nemu-specific expression artwork; deferred to P2-06.", "A1.4-cat-ear-spring-bones": "Official sample has no cat ears; actual character chains are P2-05."}
    unexpected = [d for d in errors if d["rule"] not in exceptions]
    assert not unexpected, unexpected
    return {"rawErrors": lint["errors"], "rawWarnings": lint["warnings"], "excludedDiagnostics": [{**d, "reason": exceptions[d["rule"]]} for d in errors], "remainingErrors": len(unexpected), "tailBones": names, "heightM": profile["heightM"], "bindingKinds": list(probe), "sha256": hashlib.sha256((directory / "nemu.vrm").read_bytes()).hexdigest()}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("first", type=Path)
    parser.add_argument("second", type=Path)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    first, second = inspect(args.first), inspect(args.second)
    a = json.dumps(read_json(args.first / "nemu.vrm"), sort_keys=True, indent=2).splitlines(keepends=True)
    b = json.dumps(read_json(args.second / "nemu.vrm"), sort_keys=True, indent=2).splitlines(keepends=True)
    diff = "".join(difflib.unified_diff(a, b, fromfile="run-1", tofile="run-2"))
    args.evidence.mkdir(parents=True, exist_ok=True)
    (args.evidence / "gltf-json.diff").write_text(diff, encoding="utf-8")
    result = {"run1": first, "run2": second, "canonicalJsonIdentical": not diff, "ignoredFields": [], "diffLines": len(diff.splitlines())}
    (args.evidence / "sample-acceptance.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2, ensure_ascii=False))
    assert not diff, "glTF JSON differs; inspect gltf-json.diff"


if __name__ == "__main__":
    main()
