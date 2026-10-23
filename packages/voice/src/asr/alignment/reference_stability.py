"""Preregistered independent context stability QA; never publishes timing references."""
from __future__ import annotations
from datetime import datetime, timezone
import importlib
import importlib.metadata
import os
from pathlib import Path
import subprocess
from typing import cast
from context_references import pcm, write_pcm, REPO, HERE
from cache_paths import configure_cache_environment, data_directory, evaluation_directory
from align_user import Factory, LANGUAGES, REVISION, Torch, verify_models
from import_continuous import CoarseAligner
from segmented_tokens import FixedSegments
from reference_data import boundaries_v2, contextual_pcm, file_sha, freeze, inside, number, obj, objects, read_json, recordings, text

PLAN=REPO/'reports/P4-01/reference-stability-plan.json'
DOC=REPO/'reports/P4-01/REFERENCE-STABILITY-PLAN.md'


def compare_contexts(first: tuple[float,float] | None, second: tuple[float,float] | None) -> dict[str,object]:
    if first is None or second is None:
        return {'stable':False,'reason':'invalid-context'}
    differences=[abs(number(a)-number(b)) for a,b in zip(first,second,strict=True)]
    return {'stable':all(d<=80+1e-8 for d in differences),
            'startDifferenceMs':differences[0],'endDifferenceMs':differences[1]}


def main() -> None:
    configure_cache_environment()
    if os.environ.get('CI'):
        raise ValueError('Local evaluation only')
    files=[PLAN,DOC,Path(__file__),HERE/'context_references.py',HERE/'reference_data.py',
           HERE/'segmented_tokens.py',HERE/'align_user.py',HERE/'manifest.json',HERE/'uv.lock']
    paths=[str(p.relative_to(REPO)) for p in files]
    subprocess.run(['git','ls-files','--error-unmatch',*paths],cwd=REPO,capture_output=True,check=True)
    subprocess.run(['git','diff','--exit-code','HEAD','--',*paths],cwd=REPO,capture_output=True,check=True)
    provenance={str(p.relative_to(REPO)):file_sha(p) for p in files}
    commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip()
    root=data_directory().resolve()
    plan=obj(read_json(PLAN))
    selected=recordings(root,plan)
    inputs=root/'alignment-inputs-v3.json'
    if file_sha(inputs)!=plan['inputsSha256']:
        raise ValueError('Frozen canonical input manifest changed')
    frozen={text(r['id']):r for r in objects(read_json(inputs))}
    canonical=[]
    for r in selected:
        row=frozen[r.id]
        wav=inside(root,text(row['path']))
        if row['sourceSha256']!=r.source_sha256 or row['transcriptSha256']!=r.transcript_sha256 or file_sha(wav)!=row['sha256']:
            raise ValueError('Canonical audio or approved text changed')
        canonical.append((r,wav))
    stamp=datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')
    raw=root/('reference-stability-'+stamp)
    raw.mkdir()
    anchors: dict[str,tuple[bytes,str,str]]={}
    public={text(r['id']):r for r in objects(read_json(REPO/'packages/voice/src/asr/public-data.lock.json'))}
    for a in objects(plan['anchors']):
        anchor_id=text(a['id'])
        row=public[anchor_id]
        source=inside(root,text(row['path']))
        if file_sha(source)!=a['sha256'] or row['sha256']!=a['sha256']:
            raise ValueError('Anchor SHA mismatch')
        normalized=raw/(anchor_id+'.wav')
        subprocess.run(['ffmpeg','-v','error','-i',str(source),'-ar','16000','-ac','1','-c:a','pcm_s16le',str(normalized)],check=True,capture_output=True)
        anchors[anchor_id]=(pcm(normalized),text(row['text']),file_sha(normalized))
    cache=evaluation_directory()/'qwen3-forced-aligner'/REVISION
    verify_models(cache,False)
    os.environ.update(CUDA_VISIBLE_DEVICES='',HF_HUB_OFFLINE='1',TRANSFORMERS_OFFLINE='1')
    for package,version in {'qwen-asr':'0.0.6','torch':'2.10.0+cpu','transformers':'4.57.6'}.items():
        if importlib.metadata.version(package)!=version:
            raise ValueError('Unexpected evaluation package version')
    torch=cast(Torch,importlib.import_module('torch'))
    torch.set_num_threads(2)
    torch.set_num_interop_threads(1)
    torch.manual_seed(0)
    torch.use_deterministic_algorithms(True)
    factory=cast(Factory,getattr(importlib.import_module('qwen_asr'),'Qwen3ForcedAligner'))
    aligner=cast(CoarseAligner,factory.from_pretrained(str(cache),dtype=torch.float32,
        device_map='cpu',attn_implementation='eager',local_files_only=True))
    results: list[dict[str,object]]=[]
    for recording,wav in canonical:
        target_pcm=pcm(wav)
        estimates: list[tuple[float,float] | None]=[]
        attempts: list[dict[str,object]]=[]
        for suffix in ('001','002'):
            anchor_id=f'public-{recording.lang}-{suffix}'
            anchor_pcm,anchor_text,anchor_sha=anchors[anchor_id]
            joined,prefix=contextual_pcm(anchor_pcm,target_pcm)
            name=f'{recording.id}.{suffix}'
            context=raw/(name+'.wav')
            write_pcm(context,joined)
            full_text=anchor_text+'。\n'+recording.transcript
            original=aligner.aligner_processor
            segments=[original.encode_timestamp(anchor_text,LANGUAGES[recording.lang])[0],
                      original.encode_timestamp(recording.transcript,LANGUAGES[recording.lang])[0]]
            aligner.aligner_processor=FixedSegments(original,full_text,segments)
            try:
                batch=aligner.align(audio=str(context),text=full_text,language='Japanese')
            finally:
                aligner.aligner_processor=original
            items=[{'text':i.text,'start_time':i.start_time,'end_time':i.end_time} for i in batch[0].items] if len(batch)==1 else []
            raw_file=raw/(name+'.json')
            freeze(raw_file,{'anchorId':anchor_id,'normalizedAnchorSha256':anchor_sha,
                'prefixFrames':prefix,'contextSha256':file_sha(context),'items':items})
            target=[{'text':i['text'],'start_time':number(i['start_time'])-prefix/16000,
                     'end_time':number(i['end_time'])-prefix/16000} for i in items[len(segments[0]):]]
            attempt: dict[str,object]={'anchorId':anchor_id,'rawSha256':file_sha(raw_file),
                'zeroUnitIndices':[i for i,t in enumerate(target) if t['start_time']==t['end_time']]}
            try:
                if len(items)!=sum(len(s) for s in segments):
                    raise ValueError('Context unit count mismatch')
                bounds=boundaries_v2(target,len(target_pcm)/32000)
                estimates.append(bounds)
                attempt.update(status='valid',speechStartMs=bounds[0],speechEndMs=bounds[1])
            except ValueError as error:
                estimates.append(None)
                attempt.update(status='invalid',reason=str(error))
            attempts.append(attempt)
        result={'id':recording.id,'lang':recording.lang,'attempts':attempts,
                **compare_contexts(estimates[0],estimates[1])}
        results.append(result)
        freeze(raw/(recording.id+'.comparison.json'),result)
        print(f"Stability {len(results)}/60 {recording.id}: {result['stable']}",flush=True)
    summary={'method':plan['method'],'commit':commit,'filesSha256':provenance,
        'rawDirectory':str(raw),'count':len(results),'stableCount':sum(r['stable'] is True for r in results),
        'allStable':len(results)==60 and all(r['stable'] is True for r in results),
        'timestampGridMs':80,'replacementPublished':False,'samples':results}
    freeze(raw/'summary.json',summary)
    freeze(REPO/f'reports/P4-01/reference-stability-{stamp}.json',summary)
    print(f"Completed stability QA: {summary['stableCount']}/60; no references replaced",flush=True)


if __name__=='__main__':
    main()
