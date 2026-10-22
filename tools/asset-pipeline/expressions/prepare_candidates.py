"""Candidate-only glTF assembly; never writes source assets or production build.yaml.

All appearance values come from the candidate YAML. This preview compiler does
not replace Blender production acceptance after user selection.
"""
from __future__ import annotations

import copy
import hashlib
import json
import math
from pathlib import Path
import struct
from typing import Any

import jsonschema
import yaml  # type: ignore[import-untyped]

Json = dict[str, Any]
ROOT = Path(__file__).resolve().parents[3]
CANDIDATES = ROOT / 'content/_candidates/P2-06'


def load_glb(path: Path) -> tuple[Json, bytearray]:
    data = path.read_bytes()
    magic, version, total = struct.unpack_from('<III', data)
    if (magic, version, total) != (0x46546C67, 2, len(data)):
        raise ValueError(f'Invalid GLB: {path}')
    size, kind = struct.unpack_from('<II', data, 12)
    if kind != 0x4E4F534A:
        raise ValueError('GLB JSON chunk missing')
    binary_size, binary_kind = struct.unpack_from('<II', data, 20 + size)
    if binary_kind != 0x004E4942:
        raise ValueError('GLB BIN chunk missing')
    return json.loads(data[20:20 + size]), bytearray(data[28 + size:28 + size + binary_size])


def save_glb(path: Path, gltf: Json, binary: bytearray) -> None:
    gltf['buffers'][0]['byteLength'] = len(binary)
    encoded = json.dumps(gltf, ensure_ascii=False, allow_nan=False, separators=(',', ':')).encode()
    encoded += b' ' * (-len(encoded) % 4)
    binary += b'\0' * (-len(binary) % 4)
    path.write_bytes(struct.pack('<III', 0x46546C67, 2, 28 + len(encoded) + len(binary))
                     + struct.pack('<II', len(encoded), 0x4E4F534A) + encoded
                     + struct.pack('<II', len(binary), 0x004E4942) + binary)


def accessor(gltf: Json, binary: bytearray, rows: list[list[float]], kind: str, component: int = 5126) -> int:
    width = {'VEC3': 3, 'VEC4': 4}[kind]
    binary += b'\0' * (-len(binary) % 4)
    offset = len(binary)
    fmt = 'f' if component == 5126 else 'H'
    for row in rows:
        binary.extend(struct.pack('<' + fmt * width, *row))
    view = len(gltf['bufferViews'])
    gltf['bufferViews'].append(dict(buffer=0, byteOffset=offset, byteLength=len(binary) - offset, target=34962))
    result: Json = dict(bufferView=view, componentType=component, count=len(rows), type=kind)
    if kind == 'VEC3':
        result['min'] = [min(row[i] for row in rows) for i in range(width)]
        result['max'] = [max(row[i] for row in rows) for i in range(width)]
    index = len(gltf['accessors'])
    gltf['accessors'].append(result)
    return index


def glyph_triangles(cfg: Json) -> list[list[tuple[float, float]]]:
    """Return unit-space glyph triangles, all dimensions/colours supplied in YAML."""
    kind = cfg['kind']
    if kind in ('star', 'ellipse', 'tear'):
        count = 8 if kind == 'star' else 32
        points = []
        for i in range(count):
            angle = 2 * math.pi * i / count
            radius = .28 if kind == 'star' and i % 2 else 1.0
            x, y = radius * math.cos(angle), radius * math.sin(angle)
            if kind == 'tear':
                x *= (1 - y) * .5
            points.append((x, y))
        return [[(0.0, 0.0), points[i], points[(i + 1) % count]] for i in range(count)]
    segments = []
    if kind == 'cross':
        for sign in (-1, 1):
            for other in (-1, 1):
                segments += [((sign * .28, other), (sign * .28, other * .28)),
                             ((sign, other * .28), (sign * .28, other * .28))]
    elif kind == 'omega':
        points = [(x / 24, .5 * math.cos(2 * math.pi * x / 24)) for x in range(-24, 25)]
        segments = list(zip(points[:-1], points[1:]))
    else:
        raise ValueError(f'Unknown glyph kind: {kind}')
    triangles = []
    for (x1, y1), (x2, y2) in segments:
        length = math.hypot(x2 - x1, y2 - y1)
        nx, ny = -(y2 - y1) / length * .09, (x2 - x1) / length * .09
        a, b = (x1 + nx, y1 + ny), (x1 - nx, y1 - ny)
        c, d = (x2 + nx, y2 + ny), (x2 - nx, y2 - ny)
        triangles += [[a, b, c], [b, d, c]]
    return triangles


def read_positions(gltf: Json, binary: bytearray, index: int) -> list[list[float]]:
    item = gltf['accessors'][index]
    if item['componentType'] != 5126 or item['type'] != 'VEC3' or 'sparse' in item:
        raise ValueError('Expected dense float32 VEC3 positions')
    view = gltf['bufferViews'][item['bufferView']]
    start = view.get('byteOffset', 0) + item.get('byteOffset', 0)
    stride = view.get('byteStride', 12)
    return [list(struct.unpack_from('<fff', binary, start + i * stride)) for i in range(item['count'])]


def cat_mouth_morph(gltf: Json, binary: bytearray, cfg: Json) -> None:
    node = next(n for n in gltf['nodes'] if n.get('name') == cfg['object'])
    mesh = gltf['meshes'][node['mesh']]
    for primitive in mesh['primitives']:
        positions = read_positions(gltf, binary, primitive['attributes']['POSITION'])
        deltas = []
        for x, y, z in positions:
            influence = math.exp(-((x / cfg['halfWidth']) ** 4 + ((y - cfg['centreY']) / cfg['radiusY']) ** 4)) if z > cfg['frontMinZ'] else 0.0
            dy = cfg['amplitudeM'] * math.cos(2 * math.pi * x / cfg['halfWidth']) * influence
            dx = x * (cfg['horizontalScale'] - 1) * influence
            deltas.append([dx, dy, 0.0])
        primitive['targets'].append(dict(POSITION=accessor(gltf, binary, deltas, 'VEC3')))
    mesh['extras']['targetNames'].append(cfg['key'])
    mesh['weights'].append(0)


def add_glyph(gltf: Json, binary: bytearray, suffix: str, cfg: Json, face: Json) -> None:
    name = 'P206' + suffix
    material = len(gltf['materials'])
    gltf['materials'].append(dict(name=name, doubleSided=True,
        alphaMode='BLEND' if cfg['kind'] == 'ellipse' else 'OPAQUE', pbrMetallicRoughness=dict(baseColorFactor=cfg['color'], metallicFactor=0, roughnessFactor=1),
        extensions={'KHR_materials_unlit': {}}))
    if 'KHR_materials_unlit' not in gltf['extensionsUsed']:
        gltf['extensionsUsed'].append('KHR_materials_unlit')
    base, delta, colors = [], [], []
    for centre in cfg['centres']:
        for triangle in glyph_triangles(cfg):
            for x, y in triangle:
                base.append(list(centre))
                colors.append([1.0, 1.0, 1.0, 1.0 if cfg['kind'] != 'ellipse' or (x == 0 and y == 0) else 0.0])
                delta.append([x * cfg['radius'][0], y * cfg['radius'][1], 0.0])
    head = gltf['extensions']['VRMC_vrm']['humanoid']['humanBones']['head']['node']
    joint = gltf['skins'][face['skin']]['joints'].index(head)
    attributes = dict(POSITION=accessor(gltf, binary, base, 'VEC3'),
        NORMAL=accessor(gltf, binary, [[0, 0, 1]] * len(base), 'VEC3'),
        COLOR_0=accessor(gltf, binary, colors, 'VEC4'),
        JOINTS_0=accessor(gltf, binary, [[joint, 0, 0, 0]] * len(base), 'VEC4', 5123),
        WEIGHTS_0=accessor(gltf, binary, [[1, 0, 0, 0]] * len(base), 'VEC4'))
    mesh = len(gltf['meshes'])
    gltf['meshes'].append(dict(name=name, weights=[0], extras=dict(targetNames=[name]),
        primitives=[dict(attributes=attributes, material=material, mode=4,
                         targets=[dict(POSITION=accessor(gltf, binary, delta, 'VEC3'))])]))
    node = len(gltf['nodes'])
    gltf['nodes'].append(dict(name=name, mesh=mesh, skin=face['skin']))
    # Match Face's parent to retain the same bind-space interpretation.
    parent = next((n for n in gltf['nodes'] if gltf['nodes'].index(face) in n.get('children', [])), None)
    if parent is not None:
        parent['children'].append(node)
    else:
        gltf['scenes'][gltf.get('scene', 0)]['nodes'].append(node)


def bind_expression(gltf: Json, cfg: Json) -> Json:
    expression = {key: cfg[key] for key in ('overrideMouth', 'overrideBlink', 'overrideLookAt', 'isBinary')}
    binds = []
    for item in cfg['morphs']:
        node, mesh = next((i, gltf['meshes'][n['mesh']]) for i, n in enumerate(gltf['nodes'])
                          if n.get('name') == item['object'] and 'mesh' in n)
        index = mesh['extras']['targetNames'].index(item['key'])
        binds.append(dict(node=node, index=index, weight=item['weight']))
    expression['morphTargetBinds'] = binds
    if cfg['colors'] or cfg['transforms']:
        raise ValueError('Preview compiler only supports explicit morphs; use Blender for colour/texture binds')
    return expression


def main() -> None:
    cfg = yaml.safe_load((CANDIDATES / 'expressions.candidates.yaml').read_text(encoding='utf-8'))
    styles = yaml.safe_load((CANDIDATES / 'materials.candidates.yaml').read_text(encoding='utf-8'))['presets']
    source = (CANDIDATES / cfg['baseModel']).resolve()
    before = hashlib.sha256(source.read_bytes()).hexdigest()
    gltf, binary = load_glb(source)
    original = copy.deepcopy(gltf)
    face = next(n for n in gltf['nodes'] if n.get('name') == 'Face')
    cat_mouth_morph(gltf, binary, cfg['proceduralMorphs']['catMouth'])
    for suffix, glyph in cfg['glyphs'].items():
        add_glyph(gltf, binary, suffix, glyph, face)
    expressions = gltf['extensions']['VRMC_vrm']['expressions']
    for name, rules in cfg['expressions']['preset'].items():
        expressions['preset'][name].update(rules)
    jsonschema.validate(cfg['expressions'], json.loads((ROOT / 'tools/asset-pipeline/build.schema.json').read_text())['properties']['expressions'])
    expressions['custom'] = {name: bind_expression(gltf, expr) for name, expr in cfg['expressions']['custom'].items()}
    output = CANDIDATES / 'models'
    output.mkdir(exist_ok=True)
    parameter_map = dict(shading_shift_factor='shadingShiftFactor', shading_toony_factor='shadingToonyFactor',
        gi_equalization_factor='giEqualizationFactor', rim_lighting_mix_factor='rimLightingMixFactor',
        parametric_rim_fresnel_power_factor='parametricRimFresnelPowerFactor', parametric_rim_lift_factor='parametricRimLiftFactor',
        outline_width_factor='outlineWidthFactor', outline_lighting_mix_factor='outlineLightingMixFactor',
        shade_color_factor='shadeColorFactor', parametric_rim_color_factor='parametricRimColorFactor',
        outline_color_factor='outlineColorFactor', outline_width_mode='outlineWidthMode')
    for style, settings in styles.items():
        model = copy.deepcopy(gltf)
        fragment = {}
        for material in model['materials']:
            mtoon = material.get('extensions', {}).get('VRMC_materials_mtoon')
            if mtoon is None:
                continue
            applied = copy.deepcopy(settings)
            # Preserve source base colours; presets control shading, rim and outline.
            applied['baseColor'] = material.get('pbrMetallicRoughness', {}).get('baseColorFactor', [1, 1, 1, 1])
            if mtoon.get('outlineWidthMode') == 'none':
                applied['mtoon']['outline_width_mode'] = 'none'
                applied['mtoon']['outline_width_factor'] = 0
            for field, value in applied['mtoon'].items():
                mtoon[parameter_map[field]] = value
            fragment[material['name']] = applied
        save_glb(output / f'{style}.vrm', model, binary.copy())
        style_profile = json.loads((CANDIDATES / cfg['baseProfile']).read_text())
        style_profile['model'] = f'{style}.vrm'
        style_profile['emotionMap'] = {emo: variants['A'] for emo, variants in cfg['emotionMaps'].items()}
        (output / f'{style}.profile.json').write_text(json.dumps(style_profile,indent=2)+'\n')
        schema = json.loads((ROOT / 'tools/asset-pipeline/build.schema.json').read_text())
        jsonschema.validate(fragment, schema['properties']['materials'])
        (CANDIDATES / f'materials.{style}.yaml').write_text(yaml.safe_dump(dict(materials=fragment),sort_keys=False),encoding='utf-8')
    profile = json.loads((CANDIDATES / cfg['baseProfile']).read_text())
    profile['model'] = 'standard.vrm'
    profile['emotionMap'] = {emo: variants['A'] for emo, variants in cfg['emotionMaps'].items()}
    (output / 'nemu.profile.json').write_text(json.dumps(profile,indent=2)+'\n')
    (CANDIDATES / 'expressions.build-fragment.yaml').write_text(yaml.safe_dump(dict(expressions=cfg['expressions']),sort_keys=False),encoding='utf-8')
    (CANDIDATES / 'emotionMap.candidates.json').write_text(json.dumps(cfg['emotionMaps'],indent=2)+'\n')
    checks = dict(source=str(source),sourceSha256=before,sourceUnchanged=before == hashlib.sha256(source.read_bytes()).hexdigest(),
        springsUnchanged=original['extensions']['VRMC_springBone'] == gltf['extensions']['VRMC_springBone'],
        originalNonFaceMeshesUnchanged=all(original['meshes'][i] == gltf['meshes'][i] for i in range(len(original['meshes'])) if i != face['mesh']),
        faceBasePositionsUnchanged=all(a['attributes'] == b['attributes'] for a,b in zip(original['meshes'][face['mesh']]['primitives'],gltf['meshes'][face['mesh']]['primitives'],strict=True)),
        customExpressions=len(expressions['custom']), customPreviews=len(expressions['custom'])*len(cfg['customStrengths']),
        emotionCombinations=sum(len(v) for v in cfg['emotionMaps'].values()),materialPresets=len(styles),
        productionBuildRun=False,status='UNAPPROVED')
    manifest = [dict(path=str(path.relative_to(CANDIDATES)).replace('\\', '/'), kind='model' if path.suffix == '.vrm' else 'other', source='P2-05 short-thick-standard candidate plus P2-06 procedural expressions and MToon YAML; source sha256 ' + before, license='custom:Nemu private content; final terms pending P7', redistributable=False, author='nanake; Codex procedural candidate preparation', notes='UNAPPROVED; candidate preview, not production acceptance') for path in sorted(CANDIDATES.rglob('*')) if path.is_file() and path.name != 'manifest.json']
    (CANDIDATES / 'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    (ROOT / 'reports/P2-06/assembly-checks.json').write_text(json.dumps(checks,indent=2)+'\n')
    print(json.dumps(checks,indent=2))


if __name__ == '__main__':
    main()

