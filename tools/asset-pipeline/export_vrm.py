from __future__ import annotations

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse

from typing import Any

from errors import fail
from gltf import read_json

Json = dict[str, Any]


def validate_export_metadata(metadata: Json, content_license: str) -> None:
    """ADR 0015: VRM public license and content license occupy distinct fields."""
    if metadata.get("licenseUrl") != "https://vrm.dev/licenses/1.0/":
        fail("export", "meta.licenseUrl", "https://vrm.dev/licenses/1.0/", metadata.get("licenseUrl"))
    if not content_license.strip() or metadata.get("otherLicenseUrl") != content_license:
        fail("export", "meta.otherLicenseUrl", content_license or "nonempty content license", metadata.get("otherLicenseUrl"))


def execute(config: Json, args: argparse.Namespace) -> None:
    import bpy
    from blender_common import armature

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
    metadata = read_json(args.output_dir / "nemu.vrm")["extensions"]["VRMC_vrm"]["meta"]
    validate_export_metadata(metadata, config["meta"]["contentLicenseUrl"])


if __name__ == "__main__":
    from blender_common import run

    run("export", execute)
