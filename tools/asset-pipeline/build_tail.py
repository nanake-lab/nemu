from __future__ import annotations

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse
import math

import bpy
from mathutils import Vector

from blender_common import Json, armature, require_material, resolve_bone, run
from errors import fail
from geometry import chain_points, radius_at, weights_at


def execute(config: Json, args: argparse.Namespace) -> None:
    cfg = config["tail"]
    if not cfg["enabled"]:
        return
    arm = armature()
    parent = resolve_bone(arm, cfg["parentBone"], "tail")
    names = [f'{cfg["bonePrefix"]}{index:02d}' for index in range(cfg["boneCount"])]
    if any(name in arm.data.bones for name in names) or cfg["objectName"] in bpy.data.objects:
        fail("tail", cfg["objectName"], "unused object and bone names", "collision")
    origin = arm.data.bones[parent].head_local + Vector(cfg["offsetM"])
    coords = [origin + Vector(p) for p in chain_points(cfg["curve"], cfg["lengthM"], cfg["boneCount"])]
    curve = bpy.data.curves.new(cfg["objectName"] + "Curve", "CURVE")
    curve.dimensions = "3D"
    curve.resolution_u = cfg["resolution"]
    curve.bevel_depth = 1  # radius is in metres on each configured control point
    curve.bevel_resolution = cfg["sectionResolution"]
    curve.use_fill_caps = cfg["capEnds"]
    spline = curve.splines.new("POLY")
    surface_coords = [origin + Vector(p) for p in chain_points(cfg["curve"], cfg["lengthM"], cfg["boneCount"] * cfg["resolution"])]
    spline.points.add(len(surface_coords) - 1)
    for index, (point, co) in enumerate(zip(spline.points, surface_coords)):
        point.co = (*co, 1)
        point.radius = radius_at(cfg["radiusCurve"], index / (len(surface_coords) - 1))
    if cfg["tipShape"] == "taper":
        spline.points[-1].radius = cfg["tipRadiusM"]
    obj = bpy.data.objects.new(cfg["objectName"], curve)
    bpy.context.collection.objects.link(obj)
    obj.parent = arm
    bpy.context.view_layer.update()
    mesh = bpy.data.meshes.new_from_object(obj.evaluated_get(bpy.context.evaluated_depsgraph_get()))
    bpy.data.objects.remove(obj, do_unlink=True)
    obj = bpy.data.objects.new(cfg["objectName"], mesh)
    bpy.context.collection.objects.link(obj)
    obj.parent = arm
    bpy.data.curves.remove(curve)
    uv = mesh.uv_layers.new(name="UVMap")
    # Parametric UV along the nearest centerline segment, seam at atan2's cut.
    vertex_t: list[float] = []
    vertex_u: list[float] = []
    for vertex in mesh.vertices:
        nearest: tuple[float, float, float] | None = None
        for index, (a, b) in enumerate(zip(coords, coords[1:])):
            tangent = b - a
            t = min(1.0, max(0.0, (vertex.co - a).dot(tangent) / tangent.length_squared))
            radial = vertex.co - (a + tangent * t)
            axis = tangent.normalized()
            ref = axis.orthogonal().normalized()
            u = math.atan2(radial.dot(axis.cross(ref)), radial.dot(ref)) / (2 * math.pi) + 0.5
            candidate = (radial.length_squared, (index + t) / cfg["boneCount"], u)
            if nearest is None or candidate[0] < nearest[0]:
                nearest = candidate
        assert nearest is not None
        vertex_t.append(nearest[1])
        vertex_u.append(nearest[2])
    for polygon in mesh.polygons:
        us = [vertex_u[mesh.loops[i].vertex_index] for i in polygon.loop_indices]
        seam = max(us) - min(us) > 0.5
        for i in polygon.loop_indices:
            vi = mesh.loops[i].vertex_index
            u = vertex_u[vi]
            uv.data[i].uv = (u + 1 if seam and u < 0.5 else u, vertex_t[vi])
    bpy.context.view_layer.objects.active = arm
    with bpy.context.temp_override(object=arm, active_object=arm, selected_objects=[arm], selected_editable_objects=[arm]):
        bpy.ops.object.mode_set(mode="EDIT")
        try:
            previous = arm.data.edit_bones[parent]
            for name, a, b in zip(names, coords, coords[1:]):
                bone = arm.data.edit_bones.new(name)
                bone.head, bone.tail = a, b
                bone.parent = previous
                bone.use_connect = previous.name in names
                previous = bone
        finally:
            bpy.ops.object.mode_set(mode="OBJECT")
    groups = [obj.vertex_groups.new(name=name) for name in names]
    for index, t in enumerate(vertex_t):
        for bone_index, weight in weights_at(t, len(names)):
            if weight > 0:
                groups[bone_index].add([index], weight, "REPLACE")
    modifier = obj.modifiers.new("TailSkin", "ARMATURE")
    modifier.object = arm
    material = require_material(cfg["hairMaterial"], "tail").copy()
    material.name = cfg["materialName"]
    material.vrm_addon_extension.mtoon1.pbr_metallic_roughness.base_color_factor = cfg["color"]
    mesh.materials.append(material)
    arm["nemu_tail_bones"] = names


if __name__ == "__main__":
    run("tail", execute)
