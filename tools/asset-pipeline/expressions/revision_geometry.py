"""Candidate-only local morphs and smooth comic glyphs, parameterised by YAML."""
from __future__ import annotations

import math
from typing import Any

from prepare_candidates import accessor, add_glyph, glyph_triangles, read_positions

Json = dict[str, Any]


def smooth_lip_normals(gltf: Json, binary: bytearray, cfg: Json) -> int:
    """Smooth only the selected lower-lip surface normals; never move vertices."""
    face = next(n for n in gltf['nodes'] if n.get('name') == cfg['object'])
    changed = 0
    for primitive in gltf['meshes'][face['mesh']]['primitives']:
        if gltf['materials'][primitive['material']]['name'] != cfg['material']:
            continue
        positions = read_positions(gltf, binary, primitive['attributes']['POSITION'])
        normals = read_positions(gltf, binary, primitive['attributes']['NORMAL'])
        for i, (x, y, z) in enumerate(positions):
            if z < cfg['frontMinZ']:
                continue
            distance = max(abs(x) - cfg['halfWidth'], cfg['minY'] - y, y - cfg['maxY'], 0)
            t = max(0, 1 - distance / cfg['featherM'])
            weight = t * t * (3 - 2 * t) * cfg['strength']
            if not weight:
                continue
            vector = [a * (1 - weight) + b * weight for a, b in zip(normals[i], cfg['targetNormal'], strict=True)]
            length = math.sqrt(sum(v * v for v in vector))
            normals[i] = [v / length for v in vector]
            changed += 1
        primitive['attributes']['NORMAL'] = accessor(gltf, binary, normals, 'VEC3')
    if not changed:
        raise ValueError('Lower-lip normal region did not match any vertices')
    return changed


def mouth_morph(gltf: Json, binary: bytearray, cfg: Json) -> None:
    node = next(n for n in gltf['nodes'] if n.get('name') == cfg['object'])
    mesh = gltf['meshes'][node['mesh']]
    for primitive in mesh['primitives']:
        delta = []
        for x, y, z in read_positions(gltf, binary, primitive['attributes']['POSITION']):
            t = x / cfg['halfWidth']
            influence = math.exp(-(t ** 4 + ((y - cfg['centreY']) / cfg['radiusY']) ** 4)) if z > cfg['frontMinZ'] else 0
            if cfg['kind'] == 'cat':
                dy = cfg['amplitudeM'] * math.cos(2 * math.pi * t)
            elif cfg['kind'] == 'smug':
                dy = cfg['amplitudeM'] * math.tanh(t * cfg['cornerSlope'])
            elif cfg['kind'] == 'pout':
                dy = (y - cfg['centreY']) * (cfg['verticalScale'] - 1) + cfg['amplitudeM'] * math.cos(math.pi * t)
            else:
                raise ValueError('Unknown mouth morph kind ' + cfg['kind'])
            delta.append([x * (cfg['horizontalScale'] - 1) * influence, dy * influence, cfg['forwardM'] * influence])
        primitive['targets'].append(dict(POSITION=accessor(gltf, binary, delta, 'VEC3')))
    mesh['extras']['targetNames'].append(cfg['key'])
    mesh['weights'].append(0)


def bounded_brow_morph(gltf: Json, binary: bytearray, cfg: Json) -> None:
    face = next(n for n in gltf['nodes'] if n.get('name') == cfg['object'])
    mesh = gltf['meshes'][face['mesh']]
    source_index = mesh['extras']['targetNames'].index(cfg['sourceKey'])
    for primitive in mesh['primitives']:
        positions = read_positions(gltf, binary, primitive['attributes']['POSITION'])
        original = read_positions(gltf, binary, primitive['targets'][source_index]['POSITION'])
        delta = [[0.0, 0.0, 0.0] for _ in positions]
        for side in (-1, 1):
            selected = [i for i, p in enumerate(positions) if p[0] * side > 0 and p[1] > cfg['minY']
                        and abs(original[i][1]) > cfg['sourceDeltaMinM']]
            if not selected:
                continue
            mean = sum(original[i][1] for i in selected) / len(selected)
            maximum = max(abs(original[i][1] - mean) for i in selected)
            for i in selected:
                delta[i] = [0, (original[i][1] - mean) / maximum * cfg['maxRelativeTiltM'] + cfg['offsetYM'], cfg['forwardM']]
        primitive['targets'].append(dict(POSITION=accessor(gltf, binary, delta, 'VEC3')))
    mesh['extras']['targetNames'].append(cfg['key'])
    mesh['weights'].append(0)


def smooth_glyph(gltf: Json, binary: bytearray, suffix: str, cfg: Json, face: Json) -> None:
    if cfg['kind'] not in ('rounded-cross', 'glossy-tear'):
        add_glyph(gltf, binary, suffix, cfg, face)
        return
    name = 'P206' + suffix
    triangles: list[tuple[list[tuple[float, float]], list[float]]] = []

    def disk(cx: float, cy: float, rx: float, ry: float, color: list[float]) -> None:
        n = cfg['segments']
        for i in range(n):
            a, b = 2 * math.pi * i / n, 2 * math.pi * (i + 1) / n
            triangles.append(([(cx, cy), (cx + rx * math.cos(a), cy + ry * math.sin(a)),
                               (cx + rx * math.cos(b), cy + ry * math.sin(b))], color))

    if cfg['kind'] == 'glossy-tear':
        for triangle in glyph_triangles({**cfg, 'kind': 'tear'}):
            triangles.append((triangle, cfg['color']))
        h = cfg['highlight']
        disk(h['centre'][0], h['centre'][1], h['radius'][0], h['radius'][1], h['color'])
    else:
        gap, bend = cfg['gapFraction'], cfg['cornerRadiusFraction']
        for width, color in [(cfg['borderHalfStrokeFraction'], cfg['borderColor']),
                             (cfg['halfStrokeFraction'], cfg['color'])]:
            for sx in (-1, 1):
                for sy in (-1, 1):
                    points: list[tuple[float, float]] = [(sx * gap, sy)]
                    for i in range(cfg['curveSegments'] + 1):
                        t = i / cfg['curveSegments']
                        points.append((sx * (gap + bend * t * t), sy * (gap + bend * (1 - t) ** 2)))
                    points.append((sx, sy * gap))
                    for (x1, y1), (x2, y2) in zip(points[:-1], points[1:], strict=True):
                        length = math.hypot(x2 - x1, y2 - y1)
                        nx, ny = -(y2 - y1) / length * width, (x2 - x1) / length * width
                        a, b = (x1 + nx, y1 + ny), (x1 - nx, y1 - ny)
                        c, d = (x2 + nx, y2 + ny), (x2 - nx, y2 - ny)
                        triangles.extend([([a, b, c], color), ([b, d, c], color)])
                    for x, y in points:
                        disk(x, y, width, width, color)
    base, delta, colors = [], [], []
    for centre in cfg['centres']:
        for triangle, color in triangles:
            for x, y in triangle:
                base.append(list(centre))
                delta.append([x * cfg['radius'][0], y * cfg['radius'][1], 0])
                colors.append(color)
    material = len(gltf['materials'])
    gltf['materials'].append(dict(name=name, doubleSided=True, alphaMode='OPAQUE',
        pbrMetallicRoughness=dict(baseColorFactor=[1, 1, 1, 1], metallicFactor=0, roughnessFactor=1),
        extensions={'KHR_materials_unlit': {}}))
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
    face_index = gltf['nodes'].index(face)
    parent = next((n for n in gltf['nodes'] if face_index in n.get('children', [])), None)
    if parent:
        parent['children'].append(node)
    else:
        gltf['scenes'][gltf.get('scene', 0)]['nodes'].append(node)
