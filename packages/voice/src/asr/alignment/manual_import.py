"""Validate a human export, report blind audits, freeze a candidate; never runs the DUT."""
from __future__ import annotations
import json
from pathlib import Path
import sys
import wave
sys.path.insert(0,str(Path(__file__).resolve().parent.parent))
from cache_paths import configure_cache_environment, data_directory
from manual_reference import PLAN_PATH, REPO, primary, selection, validate_annotations
from reference_data import file_sha, freeze, inside, number, obj, objects, read_json, text, text_sha


def main() -> None:
    configure_cache_environment()
    if len(sys.argv)!=2:
        raise ValueError('Pass the final annotation JSON path inside the external evaluation directory')
    root=data_directory().resolve()
    source=Path(sys.argv[1]).resolve()
    if not source.is_relative_to(root) or not source.is_file():
        raise ValueError('Annotation export must be a file in the external evaluation directory')
    plan,stable=selection()
    inputs_path=root/'alignment-inputs-v3.json'
    if file_sha(inputs_path)!=plan['canonicalInputsSha256']:
        raise ValueError('Frozen inputs changed')
    inputs=objects(read_json(inputs_path))
    for row in inputs:
        wav=inside(root,text(row['path']))
        if file_sha(wav)!=row['sha256']:
            raise ValueError('Canonical audio changed')
        with wave.open(str(wav),'rb') as stream:
            if (stream.getframerate(),stream.getnchannels(),stream.getsampwidth())!=(16000,1,2):
                raise ValueError('Canonical PCM format changed')
            row['frameCount']=stream.getnframes()
        original=inside(root,text(row['sourcePath']))
        if file_sha(original)!=row['sourceSha256']:
            raise ValueError('Original audio changed')
        approved=original.with_suffix('.txt').read_text(encoding='utf-8-sig').strip()
        if text_sha(approved)!=row['transcriptSha256']:
            raise ValueError('Approved transcript changed')
    manual=validate_annotations(read_json(source),plan,inputs)
    human={text(r['id']):r for r in manual}
    auto={text(r['id']):r for r in stable}
    audits=[]
    if not isinstance(plan['auditIds'],list):
        raise ValueError('Missing audit IDs')
    for identifier in plan['auditIds']:
        key=text(identifier)
        points=human[key]
        a=primary(auto[key])
        human_start,human_end=number(points['startSample'])/16,number(points['endSample'])/16
        delta_start=human_start-number(a['speechStartMs'])
        delta_end=human_end-number(a['speechEndMs'])
        secondary=objects(auto[key]['attempts'])[1]
        audits.append({'id':key,'humanStartMs':human_start,'humanEndMs':human_end,
            'primaryStartMs':a['speechStartMs'],'primaryEndMs':a['speechEndMs'],
            'humanMinusPrimaryStartMs':delta_start,'humanMinusPrimaryEndMs':delta_end,
            'humanMinusSecondaryStartMs':human_start-number(secondary['speechStartMs']),
            'humanMinusSecondaryEndMs':human_end-number(secondary['speechEndMs']),
            'over80Ms':abs(delta_start)>80+1e-8 or abs(delta_end)>80+1e-8})
    candidate=[]
    for row in inputs:
        identifier=text(row['id'])
        clean={k:v for k,v in row.items() if k!='frameCount'}
        if identifier in auto:
            points=primary(auto[identifier])
            clean.update(speechStartMs=points['speechStartMs'],speechEndMs=points['speechEndMs'],
                         annotation='qwen3-stable-primary-v1')
        else:
            marked=human[identifier]
            clean.update(speechStartMs=number(marked['startSample'])/16,
                         speechEndMs=number(marked['endSample'])/16,annotation='human-audible-v1')
        candidate.append(clean)
    digest=file_sha(source)
    target=root/f'hybrid-reference-candidate-{digest[:16]}.json'
    freeze(target,candidate)
    report={'method':plan['method'],'planSha256':file_sha(PLAN_PATH),
        'annotationsSha256':digest,'candidatePath':str(target),'candidateSha256':file_sha(target),
        'automaticCount':39,'manualCount':21,'auditCount':5,
        'auditPassed':all(r['over80Ms'] is False for r in audits),'audits':audits,
        'formalMeasurementStarted':False,'publishedAsActiveReference':False}
    freeze(target.with_suffix('.meta.json'),report)
    freeze(REPO/f'reports/P4-01/manual-audit-{digest[:16]}.json',report)
    print(json.dumps(report,ensure_ascii=False,indent=2))
    print('Candidate only. Review every audit discrepancy before freezing a formal 60-sample measurement reference.')


if __name__=='__main__':
    main()
