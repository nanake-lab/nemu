"""Read back configured expression/material values from a cached Blender scene.

Run through Blender -b --python under the pipeline lock, after asset:step.
This does not export or change the scene, and does not approve appearance.
"""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
import sys
from typing import Any

import bpy

BASE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(BASE))
from blender_common import armature, check_versions  # noqa: E402


def equal(actual: Any, expected: Any, field: str) -> None:
    if isinstance(expected, list):
        assert len(actual) == len(expected), field
        for index, value in enumerate(expected):
            equal(actual[index], value, f'{field}[{index}]')
    elif isinstance(expected, float):
        assert math.isclose(float(actual), expected, rel_tol=1e-6, abs_tol=1e-7), (field, actual, expected)
    else:
        assert actual == expected, (field, actual, expected)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--config', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    check_versions()
    cfg = json.loads(args.config.read_text(encoding='utf-8'))
    expressions = armature().data.vrm_addon_extension.vrm1.expressions
    customs = {item.custom_name: item for item in expressions.custom}
    for name, expected in cfg['expressions']['custom'].items():
        expression = customs[name]
        for source, target in [('isBinary', 'is_binary'), ('overrideMouth', 'override_mouth'),
                               ('overrideBlink', 'override_blink'), ('overrideLookAt', 'override_look_at')]:
            equal(getattr(expression, target), expected[source], f'{name}.{source}')
        equal(len(expression.morph_target_binds), len(expected['morphs']), name + '.morphCount')
        for bind, item in zip(expression.morph_target_binds, expected['morphs'], strict=True):
            equal(bind.node.mesh_object_name, item['object'], name + '.object')
            equal(bind.index, item['key'], name + '.key')
            equal(bind.weight, item['weight'], name + '.weight')
        equal(len(expression.material_color_binds), len(expected['colors']), name + '.colorCount')
        for bind, item in zip(expression.material_color_binds, expected['colors'], strict=True):
            equal(bind.material.name, item['material'], name + '.material')
            equal(bind.type, item['type'], name + '.colorType')
            equal(bind.target_value, item['target'], name + '.target')
        equal(len(expression.texture_transform_binds), len(expected['transforms']), name + '.transformCount')
        for bind, item in zip(expression.texture_transform_binds, expected['transforms'], strict=True):
            equal(bind.material.name, item['material'], name + '.material')
            equal(bind.scale, item['scale'], name + '.scale')
            equal(bind.offset, item['offset'], name + '.offset')
    for name, expected in cfg['expressions']['preset'].items():
        preset = getattr(expressions.preset, name)
        for source, target in [('isBinary', 'is_binary'), ('overrideMouth', 'override_mouth'),
                               ('overrideBlink', 'override_blink'), ('overrideLookAt', 'override_look_at')]:
            equal(getattr(preset, target), expected[source], f'{name}.{source}')
    used = {m.name for obj in bpy.context.scene.objects if obj.type == 'MESH' for m in obj.data.materials if m}
    for name, expected in cfg['materials'].items():
        assert name in used, f'Configured material is not used by the imported scene: {name}'
        mtoon = bpy.data.materials[name].vrm_addon_extension.mtoon1
        equal(mtoon.pbr_metallic_roughness.base_color_factor, expected['baseColor'], name + '.baseColor')
        for field, value in expected['mtoon'].items():
            equal(getattr(mtoon.extensions.vrmc_materials_mtoon, field), value, name + '.' + field)
    args.report.write_text(json.dumps(dict(
        status='PASS_TECHNICAL_PREFLIGHT_ONLY', blender=list(bpy.app.version),
        customExpressionsChecked=list(cfg['expressions']['custom']),
        presetRulesChecked=list(cfg['expressions']['preset']), materialsChecked=list(cfg['materials']),
        sourceAndFormalContentWritten=False, finalExportOrAssetLintAcceptance=False,
    ), indent=2) + '\n', encoding='utf-8')


if __name__ == '__main__':
    main()
