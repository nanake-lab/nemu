from __future__ import annotations

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse
import copy
import json

from blender_common import Json, armature, resolve_bone, run
from gltf import measurements, read_json


def execute(config: Json, args: argparse.Namespace) -> None:
    arm = armature()
    height, head_offset = measurements(read_json(args.output_dir / "nemu.vrm"))
    result = copy.deepcopy(config["profile"])
    result.update(version=2, model="nemu.vrm", heightM=height, headTopOffsetM=head_offset)
    for side in ("earBonesL", "earBonesR"):
        result["rig"][side] = [resolve_bone(arm, name, "profile") for name in result["rig"][side]]
    result["rig"]["tailBones"] = list(arm.get("nemu_tail_bones", []))
    (args.output_dir / "nemu.profile.json").write_text(json.dumps(result, ensure_ascii=False, indent=2, allow_nan=False) + "\n", encoding="utf-8")


if __name__ == "__main__":
    run("profile", execute)
