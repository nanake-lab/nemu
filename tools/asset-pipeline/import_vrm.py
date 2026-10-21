from __future__ import annotations

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse

import bpy

from blender_common import Json, run
from errors import fail


def execute(config: Json, args: argparse.Namespace) -> None:
    # Retain the enabled extension and its preferences; factory-reset disables it.
    bpy.data.batch_remove(list(bpy.data.objects))
    result = bpy.ops.import_scene.vrm(
        filepath=config["source"], use_addon_preferences=False,
        extract_textures_into_folder=False, enable_mtoon_outline_preview=False,
    )
    if result != {"FINISHED"}:
        fail("import", config["source"], "FINISHED", result)


if __name__ == "__main__":
    run("import", execute)
