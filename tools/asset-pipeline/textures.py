from __future__ import annotations

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse

import bpy

from blender_common import Json, run


def execute(config: Json, args: argparse.Namespace) -> None:
    limit = config["textures"]["maxSize"]
    for image in sorted(bpy.data.images, key=lambda i: i.name):
        if image.users == 0:
            bpy.data.images.remove(image)
            continue
        width, height = image.size
        if max(width, height) > limit:
            factor = limit / max(width, height)
            image.scale(max(1, round(width * factor)), max(1, round(height * factor)))
        if image.source != "VIEWER" and image.has_data:
            image.pack()


if __name__ == "__main__":
    run("textures", execute)
