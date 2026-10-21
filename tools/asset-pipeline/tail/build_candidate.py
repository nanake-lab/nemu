"""P2-05 candidate tail stage: copy hair shading, not its unrelated atlas UVs."""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from blender_common import Json, require_material, run
from build_tail import execute as build_tail


def execute(config: Json, args: argparse.Namespace) -> None:
    build_tail(config, args)
    if not config["tail"]["enabled"]:
        return
    # The generated tube has its own UVs. Reusing an atlas samples clothes/skin/alpha cuts.
    # Retain the copied MToon shading parameters; only this new tail material is affected.
    material = require_material(config["tail"]["materialName"], "tail")
    mtoon = material.vrm_addon_extension.mtoon1
    for texture in mtoon.all_textures(downgrade_to_mtoon0=False):
        texture.source = None
    # White factors depended on the source atlas. With no atlas they would glow white.
    # The tail uses its configured palette color for both base and shade, with no emission.
    mtoon.emissive_factor = (0, 0, 0)
    mtoon.extensions.vrmc_materials_mtoon.shade_color_factor = config["materials"][config["tail"]["materialName"]]["mtoon"]["shade_color_factor"]
    import bpy
    for polygon in bpy.data.objects[config["tail"]["objectName"]].data.polygons:
        polygon.use_smooth = True


if __name__ == "__main__":
    run("tail-candidate", execute)
