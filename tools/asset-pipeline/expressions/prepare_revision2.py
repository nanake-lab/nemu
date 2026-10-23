"""Assemble the second, unapproved P2-06 expression revision without touching r1."""
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import sys
from typing import Any

import jsonschema
import yaml  # type: ignore[import-untyped]

BASE = Path(__file__).resolve().parents[1]
ROOT = BASE.parents[1]
sys.path.insert(0, str(BASE))
from config import UniqueLoader, finite_tree  # noqa: E402
from prepare_candidates import bind_expression, load_glb, save_glb  # noqa: E402
from revision_geometry import bounded_brow_morph, mouth_morph, smooth_glyph, smooth_lip_normals  # noqa: E402

Json = dict[str, Any]
CANDIDATES = ROOT / 'content/_candidates/P2-06/revision2'
FIELDS = {
    'shading_shift_factor': 'shadingShiftFactor', 'shading_toony_factor': 'shadingToonyFactor',
    'gi_equalization_factor': 'giEqualizationFactor', 'rim_lighting_mix_factor': 'rimLightingMixFactor',
    'parametric_rim_fresnel_power_factor': 'parametricRimFresnelPowerFactor',
    'parametric_rim_lift_factor': 'parametricRimLiftFactor', 'outline_width_factor': 'outlineWidthFactor',
    'outline_lighting_mix_factor': 'outlineLightingMixFactor', 'shade_color_factor': 'shadeColorFactor',
    'parametric_rim_color_factor': 'parametricRimColorFactor', 'outline_color_factor': 'outlineColorFactor',
    'outline_width_mode': 'outlineWidthMode',
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    cfg: Json = yaml.load((CANDIDATES / 'expressions.candidates.yaml').read_text(encoding='utf-8'), Loader=UniqueLoader)
    finite_tree(cfg)
    source = (CANDIDATES / cfg['baseModel']).resolve()
    original, source_binary = load_glb(source)
    source_sha = sha(source)
    styles = yaml.safe_load((CANDIDATES / 'materials.candidates.yaml').read_text(encoding='utf-8'))['presets']
    skin_control = yaml.safe_load((CANDIDATES.parent / 'materials.standard.yaml').read_text(encoding='utf-8'))['materials']
    schema = json.loads((BASE / 'build.schema.json').read_text(encoding='utf-8'))
    folder = CANDIDATES / 'models'
    folder.mkdir(exist_ok=True)
    evidence = []
    for variant, params in cfg['variants'].items():
        gltf, binary = copy.deepcopy(original), source_binary.copy()
        face = next(n for n in gltf['nodes'] if n.get('name') == 'Face')
        normal_count = smooth_lip_normals(gltf, binary, cfg['lowerLipNormals']) if 'lowerLipNormals' in cfg else 0
        for morph in params['proceduralMorphs']:
            mouth_morph(gltf, binary, morph)
        bounded_brow_morph(gltf, binary, params['browMorph'])
        for name, glyph in params['glyphs'].items():
            smooth_glyph(gltf, binary, name, glyph, face)
        jsonschema.validate(params['expressions'], schema['properties']['expressions'])
        expressions = gltf['extensions']['VRMC_vrm']['expressions']
        for name, rules in params['expressions']['preset'].items():
            expressions['preset'][name].update(rules)
        expressions['custom'] = {name: bind_expression(gltf, expr) for name, expr in params['expressions']['custom'].items()}
        fragment = {}
        for material in gltf['materials']:
            mtoon = material.get('extensions', {}).get('VRMC_materials_mtoon')
            if mtoon is None:
                continue
            settings = copy.deepcopy(skin_control[material['name']] if material['name'] in cfg['skinControlMaterials'] else styles['contrast'])
            settings['baseColor'] = material.get('pbrMetallicRoughness', {}).get('baseColorFactor', [1, 1, 1, 1])
            if mtoon.get('outlineWidthMode') == 'none':
                settings['mtoon']['outline_width_mode'] = 'none'
                settings['mtoon']['outline_width_factor'] = 0
            # A diagnosed mouth-contour adjustment, if approved, is explicit in YAML.
            override = cfg.get('materialOverrides', {}).get(material['name'], {})
            settings['mtoon'].update(override)
            for key, value in settings['mtoon'].items():
                mtoon[FIELDS[key]] = value
            fragment[material['name']] = settings
        jsonschema.validate(fragment, schema['properties']['materials'])
        profile = json.loads((CANDIDATES / cfg['baseProfile']).read_text(encoding='utf-8'))
        profile['model'] = variant + '.vrm'
        profile['emotionMap'] = {name: choices[variant if variant in ('A', 'B') else 'A'] for name, choices in cfg['emotionMaps'].items()}
        path = folder / (variant + '.vrm')
        save_glb(path, gltf, binary)
        (folder / (variant + '.profile.json')).write_text(json.dumps(profile, indent=2) + '\n', encoding='utf-8')
        (CANDIDATES / ('expressions.' + variant + '.yaml')).write_text(yaml.safe_dump(dict(expressions=params['expressions']), sort_keys=False), encoding='utf-8')
        (CANDIDATES / ('materials.' + variant + '.yaml')).write_text(yaml.safe_dump(dict(materials=fragment), sort_keys=False), encoding='utf-8')
        assert gltf['extensions']['VRMC_springBone'] == original['extensions']['VRMC_springBone']
        assert gltf['skins'] == original['skins']
        assert binary[:len(source_binary)] == source_binary
        assert gltf['extensions']['VRMC_vrm']['meta'] == original['extensions']['VRMC_vrm']['meta']
        for i, mesh in enumerate(original['meshes']):
            if i != face['mesh']:
                assert gltf['meshes'][i] == mesh
            else:
                for a, b in zip(mesh['primitives'], gltf['meshes'][i]['primitives'], strict=True):
                    assert {k: v for k, v in a['attributes'].items() if k != 'NORMAL'} == {k: v for k, v in b['attributes'].items() if k != 'NORMAL'}
        evidence.append(dict(variant=variant, modelSha256=sha(path), sourceSha256=source_sha,
            sourceBytesAndBaseGeometryPreserved=True, skinsAndSpringsPreserved=True, metadataPreserved=True,
            lowerLipNormalVertices=normal_count,
            singleTearCentres=params['glyphs']['Teary']['centres'],
            sleepyEyeCloseWeight=next(m['weight'] for m in params['expressions']['custom']['nemuSleepy']['morphs'] if m['key'] == 'Fcl_EYE_Close')))
    assert sha(source) == source_sha
    (CANDIDATES / 'emotionMap.candidates.json').write_text(json.dumps(cfg['emotionMaps'], indent=2) + '\n', encoding='utf-8')
    render_sha = sha(Path(__file__).with_name('preview_revision2.mjs'))
    fingerprint = hashlib.sha256((json.dumps(cfg, sort_keys=True) + json.dumps(styles, sort_keys=True) +
                                 json.dumps(evidence, sort_keys=True) + source_sha + render_sha).encode()).hexdigest()
    (CANDIDATES / 'preview-catalog.json').write_text(json.dumps(dict(revision=cfg['revision'], fingerprint=fingerprint, models=evidence, intent=cfg['intent']), indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    manifest = [dict(path=p.relative_to(CANDIDATES).as_posix(), kind='model' if p.suffix == '.vrm' else 'other',
        source='P2-06 revision 2 procedural candidate; approved P2-05 model sha256 ' + source_sha,
        license='custom:Nemu private content; final terms pending P7', redistributable=False,
        notes='UNAPPROVED; expression revision only, skin control retained; no source write')
        for p in sorted(CANDIDATES.rglob('*')) if p.is_file() and p.name != 'manifest.json']
    (CANDIDATES / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    (ROOT / 'reports/P2-06/revision2/integrity.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
    print('3 revision-2 candidates assembled; sources/springs/skin base colours preserved')


if __name__ == '__main__':
    main()
