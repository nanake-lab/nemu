from __future__ import annotations

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse

from blender_common import Json, require_material, run
from errors import fail


def execute(config: Json, args: argparse.Namespace) -> None:
    for name, settings in sorted(config["materials"].items()):
        material = require_material(name, "materials")
        mtoon = material.vrm_addon_extension.mtoon1
        if not mtoon.enabled:
            fail("materials", name, "MToon 1.0 enabled", "disabled")
        mtoon.pbr_metallic_roughness.base_color_factor = settings["baseColor"]
        extension = mtoon.extensions.vrmc_materials_mtoon
        for field, value in sorted(settings["mtoon"].items()):
            if field not in extension.bl_rna.properties:
                fail("materials", f"{name}.{field}", "supported MToon RNA property", "unknown")
            setattr(extension, field, value)


if __name__ == "__main__":
    run("materials", execute)
