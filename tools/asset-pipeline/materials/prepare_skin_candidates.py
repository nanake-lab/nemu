"""Compile skin-only parameter fragments; preserve base colours and texture bytes."""
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
sys.path.insert(0, str(BASE / 'expressions'))
from config import UniqueLoader, finite_tree  # noqa: E402
from prepare_candidates import load_glb, save_glb  # noqa: E402

Json = dict[str, Any]
CANDIDATES = ROOT / 'content/_candidates/P2-06/skin'
PARAMETERS = {
    'shade_color_factor': 'shadeColorFactor',
    'shading_shift_factor': 'shadingShiftFactor',
    'shading_toony_factor': 'shadingToonyFactor',
    'parametric_rim_color_factor': 'parametricRimColorFactor',
    'parametric_rim_fresnel_power_factor': 'parametricRimFresnelPowerFactor',
    'parametric_rim_lift_factor': 'parametricRimLiftFactor',
}


def main() -> None:
    cfg: Json = yaml.load((CANDIDATES / 'skin-materials.candidates.yaml').read_text(encoding='utf-8'), Loader=UniqueLoader)
    finite_tree(cfg)
    source = (CANDIDATES / cfg['baseModel']).resolve()
    source_bytes = source.read_bytes()
    model, binary = load_glb(source)
    targets = set(cfg['targets'])
    found = {m['name'] for m in model['materials'] if m['name'] in targets}
    if found != targets or len(found) != 2:
        raise ValueError(f'Expected exactly named face/body skin materials: {targets}; found {found}')
    standard: Json = yaml.load((CANDIDATES.parent / 'materials.standard.yaml').read_text(encoding='utf-8'), Loader=UniqueLoader)['materials']
    schema = json.loads((BASE / 'build.schema.json').read_text(encoding='utf-8'))['properties']['materials']
    cache = BASE / '.cache/P2-06-skin-lint'
    cache.mkdir(parents=True, exist_ok=True)
    runtime, evidence, manifest = {}, [], []
    for name, preset in cfg['presets'].items():
        if set(preset['mtoon']) != set(PARAMETERS):
            raise ValueError(f'{name}: only explicit shade/boundary/rim overrides are allowed')
        modified = copy.deepcopy(model)
        fragment = {n: copy.deepcopy(standard[n]) for n in cfg['targets']}
        patch = {PARAMETERS[key]: value for key, value in preset['mtoon'].items()}
        for material in modified['materials']:
            if material['name'] in targets:
                material['extensions']['VRMC_materials_mtoon'].update(patch)
                fragment[material['name']]['mtoon'].update(preset['mtoon'])
        jsonschema.validate(fragment, schema)
        fragment_path = CANDIDATES / f'materials.skin-{name}.yaml'
        fragment_path.write_text(yaml.safe_dump(dict(materials=fragment),sort_keys=False,allow_unicode=True),encoding='utf-8')
        path = cache / f'skin-{name}.vrm'
        save_glb(path, modified, binary.copy())
        profile = json.loads((CANDIDATES / cfg['baseProfile']).read_text(encoding='utf-8'))
        profile['model'] = path.name
        (cache / f'skin-{name}.profile.json').write_text(json.dumps(profile,indent=2)+'\n',encoding='utf-8')
        for a, b in zip(model['materials'], modified['materials'], strict=True):
            assert a['pbrMetallicRoughness'] == b['pbrMetallicRoughness']
            if a['name'] not in targets:
                assert a == b
        decoded, output_binary = load_glb(path)
        assert binary == output_binary
        assert model['extensions'] == decoded['extensions']
        assert model['meshes'] == decoded['meshes']
        runtime[name] = patch
        evidence.append(dict(candidate=name,label=preset['label'],modelSha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                             targets=cfg['targets'],baseColorAndTexturesUnchanged=True,nonSkinMaterialsUnchanged=True,
                             expressionAndSpringExtensionsUnchanged=True,baseModelSha256=hashlib.sha256(source_bytes).hexdigest()))
        manifest.append(dict(path=path.name,kind='model',source='P2-06 standard preview plus skin-only YAML candidate '+name,
                             license='custom:Nemu private content; final terms pending P7',redistributable=False))
    (cache / 'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    (CANDIDATES / 'runtime-patches.json').write_text(json.dumps(runtime,indent=2)+'\n',encoding='utf-8')
    settings = yaml.load((CANDIDATES / 'comparison-settings.yaml').read_text(encoding='utf-8'), Loader=UniqueLoader)
    (CANDIDATES / 'comparison-settings.json').write_text(json.dumps(settings,indent=2)+'\n',encoding='utf-8')
    (ROOT / 'reports/P2-06/skin-audit/candidate-integrity.json').write_text(json.dumps(evidence,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    assert source.read_bytes() == source_bytes
    print(f'{len(evidence)} skin-only parameter candidates; source/base colours/textures preserved')


if __name__ == '__main__':
    main()
