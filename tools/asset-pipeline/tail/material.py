"""Configure only the newly generated tail material, preserving source materials."""
from __future__ import annotations

from blender_common import Json, require_material


def configure_generated_tail(config: Json) -> None:
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
    settings = config["materials"].get(config["tail"]["materialName"], {}).get("mtoon", {})
    if "shade_color_factor" in settings:
        mtoon.extensions.vrmc_materials_mtoon.shade_color_factor = settings["shade_color_factor"]
    import bpy
    for polygon in bpy.data.objects[config["tail"]["objectName"]].data.polygons:
        polygon.use_smooth = True
