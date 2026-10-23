"""Read emitted GLBs back and check the user-requested structural boundaries."""
from __future__ import annotations

import json
import math
from pathlib import Path

import yaml  # type: ignore[import-untyped]

from prepare_candidates import load_glb, read_positions


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    folder = root / 'content/_candidates/P2-06/revision2'
    cfg = yaml.safe_load((folder / 'expressions.candidates.yaml').read_text(encoding='utf-8'))
    original, binary = load_glb((folder / cfg['baseModel']).resolve())
    original_face = next(n for n in original['nodes'] if n.get('name') == 'Face')
    evidence = []
    for variant in ('A', 'B', 'C'):
        model, data = load_glb(folder / 'models' / (variant + '.vrm'))
        assert data[:len(binary)] == binary
        assert model['skins'] == original['skins']
        assert model['extensions']['VRMC_springBone'] == original['extensions']['VRMC_springBone']
        face_node = next(n for n in model['nodes'] if n.get('name') == 'Face')
        face = model['meshes'][face_node['mesh']]
        names = face['extras']['targetNames']
        expressions = model['extensions']['VRMC_vrm']['expressions']['custom']
        assert set(expressions) == set(cfg['variants'][variant]['expressions']['custom'])
        close = [b for b in expressions['nemuSleepy']['morphTargetBinds'] if b['node'] == model['nodes'].index(face_node) and names[b['index']] == 'Fcl_EYE_Close']
        assert len(close) == 1 and close[0]['weight'] == 1
        assert expressions['nemuSleepy']['overrideBlink'] == 'block'
        assert all(names[b['index']] != 'Fcl_ALL_Angry' for b in expressions['nemuAngryMark']['morphTargetBinds'] if b['node'] == model['nodes'].index(face_node))
        brow_index = names.index('P206AngryBrows')
        brow_limit = cfg['variants'][variant]['browMorph']['maxRelativeTiltM'] + abs(cfg['variants'][variant]['browMorph']['offsetYM'])
        brow_maximum = max(abs(row[1]) for primitive in face['primitives'] for row in read_positions(model, data, primitive['targets'][brow_index]['POSITION']))
        assert brow_maximum <= brow_limit + 1e-7
        teary = next(n for n in model['nodes'] if n.get('name') == 'P206Teary')
        tear_primitive = model['meshes'][teary['mesh']]['primitives'][0]
        positions = read_positions(model, data, tear_primitive['attributes']['POSITION'])
        delta = read_positions(model, data, tear_primitive['targets'][0]['POSITION'])
        assert all(p[0] + d[0] < 0 for p, d in zip(positions, delta, strict=True)), 'Tears crossed to the other eye'
        changed_normals = 0
        for before, after in zip(original['meshes'][original_face['mesh']]['primitives'], face['primitives'], strict=True):
            assert before['attributes']['POSITION'] == after['attributes']['POSITION']
            assert before['attributes']['TEXCOORD_0'] == after['attributes']['TEXCOORD_0']
            old_normals = read_positions(original, binary, before['attributes']['NORMAL'])
            new_normals = read_positions(model, data, after['attributes']['NORMAL'])
            base_positions = read_positions(model, data, after['attributes']['POSITION'])
            for old, new, (x, y, z) in zip(old_normals, new_normals, base_positions, strict=True):
                if old != new:
                    patch = cfg['lowerLipNormals']
                    assert abs(x) < patch['halfWidth'] + patch['featherM']
                    assert patch['minY'] - patch['featherM'] < y < patch['maxY'] + patch['featherM']
                    assert z >= patch['frontMinZ']
                    assert math.isclose(sum(v * v for v in new), 1, abs_tol=1e-6)
                    changed_normals += 1
        assert changed_normals > 0
        evidence.append(dict(variant=variant, fullEyeClose=True, singleEyeTear=True,
            noWholeAngryStack=True, sourceAndSpringsPreserved=True, positionsAndUVPreserved=True,
            maximumAngryBrowVerticalDeltaM=brow_maximum,
            changedNormalsOnlyWithinConfiguredLipRegion=changed_normals))
    (root / 'reports/P2-06/revision2/structural-verification.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
    print('3 GLBs: closed eyes, single-eye tears, bounded local normals and source preservation verified')


if __name__ == '__main__':
    main()
