from __future__ import annotations

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse

import bpy

from blender_common import Json, armature, require_material, resolve_bone, run
from errors import fail
from gltf import read_json


def execute(config: Json, args: argparse.Namespace) -> None:
    arm = armature()
    if arm.data.vrm_addon_extension.spec_version != "1.0":
        fail("validate", arm.name, "VRM 1.0 (convert 0.x before this pipeline)", arm.data.vrm_addon_extension.spec_version)
    for name in config["sourceChecks"]["bones"]:
        resolve_bone(arm, name, "validate")
    expressions = arm.data.vrm_addon_extension.vrm1.expressions.preset
    source_expressions = read_json(Path(config["source"]))["extensions"]["VRMC_vrm"].get("expressions", {}).get("preset", {})
    for name in config["sourceChecks"]["expressions"]:
        if not hasattr(expressions, name) or name not in source_expressions:
            fail("validate", name, "preset expression", "missing")
    for name in config["sourceChecks"]["materials"]:
        require_material(name, "validate")
    if not any(o.type == "MESH" for o in bpy.data.objects):
        fail("validate", arm.name, "at least one mesh", "none")


if __name__ == "__main__":
    run("validate", execute)
