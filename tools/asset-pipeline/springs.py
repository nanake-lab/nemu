from __future__ import annotations

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse
import hashlib

import bpy

from blender_common import Json, armature, resolve_bone, run


def execute(config: Json, args: argparse.Namespace) -> None:
    arm = armature()
    cfg = config["springs"]
    extension = arm.data.vrm_addon_extension.spring_bone1
    extension.enable_animation = False
    if cfg["mode"] == "replace":
        for collider in extension.colliders:
            if collider.bpy_object:
                for child in list(collider.bpy_object.children):
                    bpy.data.objects.remove(child, do_unlink=True)
                bpy.data.objects.remove(collider.bpy_object, do_unlink=True)
        extension.springs.clear()
        extension.collider_groups.clear()
        extension.colliders.clear()
    group = extension.collider_groups.add()
    group.uuid = hashlib.sha256(cfg["groupName"].encode()).hexdigest()[:32]
    group.vrm_name = cfg["groupName"]
    for index, item in enumerate(cfg["colliders"]):
        collider = extension.colliders.add()
        collider.uuid = hashlib.sha256(f'{cfg["groupName"]}:{index}'.encode()).hexdigest()[:32]
        collider.node.bone_name = resolve_bone(arm, item["bone"], "springs")
        collider.shape_type = item["shape"].title()
        collider.reset_bpy_object(bpy.context, arm)
        shape = getattr(collider.shape, item["shape"])
        shape.offset = item["offsetM"]
        shape.radius = item["radiusM"]
        if item["shape"] == "capsule":
            shape.tail = item["tailM"]
        group.colliders.add().collider_uuid = collider.uuid
    if cfg["attachToExisting"]:
        for spring in extension.springs:
            spring.collider_groups.add().collider_group_uuid = group.uuid
    for item in cfg["chains"]:
        spring = extension.springs.add()
        spring.vrm_name = item["name"]
        spring.center.bone_name = resolve_bone(arm, item["centerBone"], "springs")
        bones = list(arm.get("nemu_tail_bones", [])) if item["bones"] == ["$tail"] else item["bones"]
        for name in bones:
            joint = spring.joints.add()
            joint.node.bone_name = resolve_bone(arm, name, "springs")
            joint.hit_radius = item["hitRadiusM"]
            joint.stiffness = item["stiffness"]
            joint.gravity_power = item["gravityPower"]
            joint.gravity_dir = item["gravityDir"]
            joint.drag_force = item["dragForce"]
        spring.collider_groups.add().collider_group_uuid = group.uuid


if __name__ == "__main__":
    run("springs", execute)
