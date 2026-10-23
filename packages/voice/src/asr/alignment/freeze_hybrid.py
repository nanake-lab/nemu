"""Freeze the original manual export plus one explicitly confirmed human override."""
from __future__ import annotations
from pathlib import Path
import json
import sys
import wave
sys.path.insert(0,str(Path(__file__).resolve().parent.parent))
from cache_paths import configure_cache_environment, data_directory
from manual_reference import PLAN_PATH, REPO, selection, validate_annotations
from reference_data import file_sha, freeze, inside, number, obj, objects, read_json, text, text_sha


def main() -> None:
    configure_cache_environment()
    root=data_directory().resolve()
    plan,_=selection()
    source=root/'manual-labels/annotations-final-2026-10-22T09-11-45-820Z.json'
    if file_sha(source)!='c4c77c8a2ecc0f76e869c72561eaf750ec86f6ab6faa9528c8f33085d7cdc558':
        raise ValueError('Expected original human export unchanged')
    inputs=root/'alignment-inputs-v3.json'
    if file_sha(inputs)!=plan['canonicalInputsSha256']:
        raise ValueError('Frozen canonical input manifest changed')
    identities=objects(read_json(inputs))
    for row in identities:
        wav=inside(root,text(row['path']))
        if file_sha(wav)!=row['sha256']:
            raise ValueError('Canonical audio changed')
        with wave.open(str(wav),'rb') as stream:
            row['frameCount']=stream.getnframes()
        original=inside(root,text(row['sourcePath']))
        if file_sha(original)!=row['sourceSha256']:
            raise ValueError('Original audio changed')
        if text_sha(original.with_suffix('.txt').read_text(encoding='utf-8-sig').strip())!=row['transcriptSha256']:
            raise ValueError('Registered transcript changed')
    annotations=validate_annotations(read_json(source),plan,identities)
    human={text(row['id']):row for row in annotations}
    candidate=root/'hybrid-reference-candidate-c4c77c8a2ecc0f76.json'
    if file_sha(candidate)!='f4805f20085aa2605e7d4c60d02e7e5deb00f7a45e35688c2a4616991d9bda05':
        raise ValueError('Preserve original mixed candidate')
    rows=objects(read_json(candidate))
    for row in rows:
        if row['id']=='user-ja-013':
            row.update(speechStartMs=number(human['user-ja-013']['startSample'])/16,
                       speechEndMs=number(human['user-ja-013']['endSample'])/16,
                       annotation='human-audible-v1')
    manual_ids=[text(row['id']) for row in rows if row['annotation']=='human-audible-v1']
    if len(rows)!=60 or len(manual_ids)!=22:
        raise ValueError('Expected 38 automatic / 22 human references')
    target=root/'timing-references-hybrid-v2.json'
    freeze(target,rows)
    registration={'method':'P4-01-hybrid-audible-reference-v2','count':60,
        'referencesPath':target.relative_to(root).as_posix(),'referencesSha256':file_sha(target),
        'annotationsPath':source.relative_to(root).as_posix(),'annotationsSha256':file_sha(source),
        'inputsPath':inputs.relative_to(root).as_posix(),'inputsSha256':file_sha(inputs),
        'manualIds':manual_ids,'automaticCount':38,'manualCount':22,
        'originalAuditPassed':False,'auditResolution':'user-confirmed human override user-ja-013; old audit failure retained',
        'sourceFiles':[
            {'path':'reports/P4-01/manual-reference-plan.json','sha256':file_sha(PLAN_PATH)},
            {'path':'reports/P4-01/MANUAL-REFERENCE-PREREGISTRATION.md','sha256':file_sha(REPO/'reports/P4-01/MANUAL-REFERENCE-PREREGISTRATION.md')},
            {'path':'reports/P4-01/HYBRID-REFERENCE-V2.md','sha256':file_sha(REPO/'reports/P4-01/HYBRID-REFERENCE-V2.md')},
            {'path':text(plan['stableEvidencePath']),'sha256':text(plan['stableEvidenceSha256'])},
            {'path':'reports/P4-01/manual-audit-c4c77c8a2ecc0f76.json','sha256':file_sha(REPO/'reports/P4-01/manual-audit-c4c77c8a2ecc0f76.json')}],
        'referenceUncertainty':'38 model-derived points are context-consistent, not individually human calibrated; 22 human intervals; original 5-way audit 4/5 within80ms and one user-confirmed correction'}
    freeze(target.with_suffix('.meta.json'),registration)
    freeze(REPO/'reports/P4-01/hybrid-reference-registration-v2.json',registration)
    print(json.dumps({'count':60,'automatic':38,'manual':22,'referencesSha256':registration['referencesSha256']},indent=2))


if __name__=='__main__':
    main()
