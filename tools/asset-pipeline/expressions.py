from __future__ import annotations

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse

import bpy

from blender_common import Json, armature, require_material, run
from errors import fail


def execute(config: Json, args: argparse.Namespace) -> None:
    expressions = armature().data.vrm_addon_extension.vrm1.expressions
    for name, rules in sorted(config["expressions"]["preset"].items()):
        if not hasattr(expressions.preset, name):
            fail("expressions", name, "existing VRM preset", "unknown")
        preset = getattr(expressions.preset, name)
        preset.override_mouth = rules["overrideMouth"]
        preset.override_blink = rules["overrideBlink"]
        preset.override_look_at = rules["overrideLookAt"]
        preset.is_binary = rules["isBinary"]
    for name, cfg in sorted(config["expressions"]["custom"].items()):
        existing = next((i for i, e in enumerate(expressions.custom) if e.custom_name == name), None)
        if existing is not None:
            expressions.custom.remove(existing)
        expression = expressions.custom.add()
        expression.custom_name = name
        expression.override_mouth = cfg["overrideMouth"]
        expression.override_blink = cfg["overrideBlink"]
        expression.override_look_at = cfg["overrideLookAt"]
        expression.is_binary = cfg["isBinary"]
        for item in cfg["morphs"]:
            obj = bpy.data.objects.get(item["object"])
            if obj is None or obj.type != "MESH" or obj.data.shape_keys is None or item["key"] not in obj.data.shape_keys.key_blocks:
                fail("expressions", f'{item["object"]}.{item["key"]}', "existing mesh shape key", "missing")
            bind = expression.morph_target_binds.add()
            bind.node.mesh_object_name = obj.name
            bind.index = item["key"]
            bind.weight = item["weight"]
        for item in cfg["colors"]:
            bind = expression.material_color_binds.add()
            bind.material = require_material(item["material"], "expressions")
            bind.type = item["type"]
            bind.target_value = item["target"]
        for item in cfg["transforms"]:
            bind = expression.texture_transform_binds.add()
            bind.material = require_material(item["material"], "expressions")
            bind.scale = item["scale"]
            bind.offset = item["offset"]


if __name__ == "__main__":
    run("expressions", execute)
