from __future__ import annotations

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse

import bpy

from blender_common import Json, armature, run
from errors import fail
from gltf import read_json


def execute(config: Json, args: argparse.Namespace) -> None:
    arm = armature()
    meta = arm.data.vrm_addon_extension.vrm1.meta
    meta.vrm_name = config["meta"]["name"]
    meta.version = config["meta"]["version"]
    meta.authors.clear()
    for name in config["meta"]["authors"]:
        meta.authors.add().value = name
    # VRM 1.0 fixes licenseUrl to the VRM license. The content license belongs in otherLicenseUrl.
    meta.other_license_url = config["meta"]["contentLicenseUrl"]
    result = bpy.ops.export_scene.vrm(
        filepath=str(args.output_dir / "nemu.vrm"), check_existing=False,
        use_addon_preferences=False, armature_object_name=arm.name,
        export_invisibles=False, export_only_selections=False, export_all_influences=False,
        export_lights=False, export_gltf_animations=False, export_try_sparse_sk=False,
        ignore_warning=True,
    )
    if result != {"FINISHED"}:
        fail("export", arm.name, "FINISHED", result)
    actual_license = read_json(args.output_dir / "nemu.vrm")["extensions"]["VRMC_vrm"]["meta"]["licenseUrl"]
    if actual_license != config["meta"]["contentLicenseUrl"]:
        fail("export", "meta.licenseUrl", config["meta"]["contentLicenseUrl"], f"{actual_license}; VRM 1.0 extension fixes this URL. Resolve CONTRACTS 1.5 before exporting a different content license URL.")


if __name__ == "__main__":
    run("export", execute)
