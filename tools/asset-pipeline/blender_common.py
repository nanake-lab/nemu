"""Shared IO only. Each stage remains a separate Blender process and script."""
from __future__ import annotations

import argparse
import importlib
import json
import sys
import tomllib
from pathlib import Path
from typing import Any, Callable

import bpy

from errors import BuildError, fail

Json = dict[str, Any]


def armature() -> Any:
    values = sorted((o for o in bpy.data.objects if o.type == "ARMATURE"), key=lambda o: o.name)
    if len(values) != 1:
        fail("scene", "armature", "exactly one", [o.name for o in values])
    return values[0]


def resolve_bone(arm: Any, name: str, step: str) -> str:
    if name in arm.data.bones:
        return name
    bones = {getattr(key, "value", key): value for key, value in arm.data.vrm_addon_extension.vrm1.humanoid.human_bones.human_bone_name_to_human_bone().items()}
    if name in bones and bones[name].node.bone_name:
        return str(bones[name].node.bone_name)
    fail(step, name, "existing bone name or humanoid role", "missing")
    raise AssertionError("unreachable")


def require_material(name: str, step: str) -> Any:
    material = bpy.data.materials.get(name)
    if material is None:
        fail(step, name, "existing material", "missing")
    return material


def check_versions() -> None:
    base = Path(__file__).parent
    expected = (base / "blender-version.txt").read_text().strip()
    actual = ".".join(map(str, bpy.app.version))
    if expected != actual:
        fail("version", "Blender", expected, actual)
    enabled = sorted(k for k in bpy.context.preferences.addons.keys() if k == "io_scene_vrm" or k.endswith(".vrm"))
    if len(enabled) != 1:
        fail("version", "VRM Add-on", "one enabled VRM extension", enabled)
    module = importlib.import_module(enabled[0])
    if module.__file__ is None:
        fail("version", enabled[0], "extension module file", "missing")
    manifest = Path(module.__file__).with_name("blender_manifest.toml")
    actual = tomllib.loads(manifest.read_text(encoding="utf-8"))["version"] if manifest.is_file() else ".".join(map(str, module.bl_info["version"]))
    expected = (base / "vrm-addon-version.txt").read_text().strip()
    if actual != expected:
        fail("version", enabled[0], expected, actual)


def run(step: str, action: Callable[[Json, argparse.Namespace], None]) -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--output-blend", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    try:
        check_versions()
        config: Json = json.loads(args.config.read_text(encoding="utf-8"))
        action(config, args)
        bpy.context.preferences.filepaths.save_version = 0
        bpy.ops.wm.save_as_mainfile(filepath=str(args.output_blend), check_existing=False)
        print(json.dumps({"step": step, "status": "ok", "blend": str(args.output_blend)}))
    except Exception as error:
        detail = str(error) if isinstance(error, BuildError) else f"step={step} object={action.__module__} expected=valid configured Blender data actual={type(error).__name__}: {error}"
        print(json.dumps({"step": step, "status": "failed", "detail": detail}), file=sys.stderr)
        raise
