"""Independent fixed-context alignment. No DUT outputs or latency-dependent choices."""
from __future__ import annotations
from datetime import datetime, timezone
import importlib
import importlib.metadata
import json
import os
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from cache_paths import data_directory, evaluation_directory, configure_cache_environment
import subprocess
from typing import cast
import wave
from align_user import Factory, LANGUAGES, REVISION, Torch, verify_models
from import_continuous import CoarseAligner
from segmented_tokens import FixedSegments
from reference_data import (boundaries_v2, canonicalize_v3, contextual_pcm, file_sha,
                            freeze, inside, number, obj, objects, read_json, recordings, text)

REPO = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
PLAN = REPO/'reports/P4-01/alignment-preregistration-v4.json'
DOC = REPO/'reports/P4-01/ALIGNMENT-PREREGISTRATION-V4.md'
METHOD = 'P4-01-segmented-context-alignment-v4'
ANNOTATION = 'qwen3-segmented-context-v4'

def pcm(path: Path) -> bytes:
    with wave.open(str(path),'rb') as w:
        if w.getframerate()!=16000 or w.getnchannels()!=1 or w.getsampwidth()!=2:
            raise ValueError('Expected mono 16 kHz PCM16')
        return w.readframes(w.getnframes())

def write_pcm(path: Path, value: bytes) -> None:
    with path.open('xb') as output:
        with wave.open(output,'wb') as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(16000)
            w.writeframes(value)

def method() -> dict[str,str]:
    files=[PLAN,DOC,Path(__file__),HERE/'reference_data.py',HERE/'import_continuous.py',
           HERE/'align_user.py',HERE.parent/'cache_paths.py',HERE/'segmented_tokens.py',HERE/'manifest.json',HERE/'uv.lock',HERE/'pyproject.toml',
           REPO/'packages/voice/src/asr/public-data.lock.json']
    paths=[str(p.relative_to(REPO)) for p in files]
    subprocess.run(['git','ls-files','--error-unmatch',*paths],cwd=REPO,capture_output=True,check=True)
    subprocess.run(['git','diff','--exit-code','HEAD','--',*paths],cwd=REPO,capture_output=True,check=True)
    commit=subprocess.check_output(['git','log','-1','--format=%H','--',*paths],cwd=REPO,text=True).strip()
    return {'commit':commit,'planSha256':file_sha(PLAN),'methodSha256':file_sha(DOC),
            'manifestSha256':file_sha(HERE/'manifest.json'),'uvLockSha256':file_sha(HERE/'uv.lock')}

def main() -> None:
    configure_cache_environment()
    if os.environ.get('CI'):
        raise ValueError('Independent alignment is local-only')
    root = data_directory().resolve()
    plan=obj(read_json(PLAN))
    if plan['method']!=METHOD:
        raise ValueError('Unexpected fixed-context plan')
    provenance=method()
    selected=recordings(root,plan)
    if (root/'timing-references.json').exists():
        raise ValueError('Preserve existing references before starting a new method version')
    public=objects(read_json(REPO/'packages/voice/src/asr/public-data.lock.json'))
    anchors=[r for r in public if r['id']=='public-zh-001']
    if len(anchors)!=1:
        raise ValueError('Missing unique preregistered anchor')
    anchor=anchors[0]
    anchor_path=inside(root,text(anchor['path']))
    if file_sha(anchor_path)!=anchor['sha256'] or anchor['sha256']!=obj(plan['context'])['anchorSha256']:
        raise ValueError('Public anchor SHA-256 mismatch')
    normalized_anchor=root/'alignment-context-anchor/public-zh-001-pcm16.wav'
    normalized_anchor.parent.mkdir(parents=True,exist_ok=True)
    if not normalized_anchor.exists():
        subprocess.run(['ffmpeg','-v','error','-i',str(anchor_path),'-ar','16000',
                        '-ac','1','-c:a','pcm_s16le',str(normalized_anchor)],check=True,capture_output=True)
    anchor_pcm=pcm(normalized_anchor)
    freeze(root/'alignment-context-anchor/snapshot.json',{
        'sourceSha256':file_sha(anchor_path),'normalizedSha256':file_sha(normalized_anchor),
        'frameCount':len(anchor_pcm)//2,'format':'mono 16 kHz PCM16; no trimming or gain'})
    canonical=[(r,canonicalize_v3(r,root)) for r in selected]
    frozen=[{'id':r.id,'lang':r.lang,'sourcePath':r.source_path,'sourceSha256':r.source_sha256,
             'transcriptSha256':r.transcript_sha256,'path':p.relative_to(root).as_posix(),
             'sha256':file_sha(p)} for r,p in canonical]
    freeze(root/'alignment-inputs-v3.json',frozen)
    cache = evaluation_directory() / 'qwen3-forced-aligner' / REVISION
    verify_models(cache,False)
    os.environ.update(CUDA_VISIBLE_DEVICES='',HF_HUB_OFFLINE='1',TRANSFORMERS_OFFLINE='1')
    for package,version in {'qwen-asr':'0.0.6','torch':'2.10.0+cpu','transformers':'4.57.6'}.items():
        if importlib.metadata.version(package)!=version:
            raise ValueError(f'Wrong package version: {package}')
    torch=cast(Torch,importlib.import_module('torch'))
    torch.set_num_threads(2)
    torch.set_num_interop_threads(1)
    torch.manual_seed(0)
    torch.use_deterministic_algorithms(True)
    factory=cast(Factory,getattr(importlib.import_module('qwen_asr'),'Qwen3ForcedAligner'))
    aligner=cast(CoarseAligner,factory.from_pretrained(str(cache),dtype=torch.float32,
        device_map='cpu',attn_implementation='eager',local_files_only=True))
    raw=root/('alignment-context-raw-'+datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ'))
    raw.mkdir()
    output: list[dict[str,object]]=[]
    diagnostics: list[dict[str,object]]=[]
    for (recording,wav),row in zip(canonical,frozen,strict=True):
        target_pcm=pcm(wav)
        context_pcm,prefix_frames=contextual_pcm(anchor_pcm,target_pcm)
        context_wav=raw/(recording.id+'.context.wav')
        write_pcm(context_wav,context_pcm)
        full_text=text(anchor['text'])+'。\n'+recording.transcript
        original_encoder=aligner.aligner_processor
        per=[original_encoder.encode_timestamp(text(anchor['text']),'Chinese')[0],
             original_encoder.encode_timestamp(recording.transcript,LANGUAGES[recording.lang])[0]]
        counts=[len(segment) for segment in per]
        fixed=FixedSegments(original_encoder,full_text,per)
        aligner.aligner_processor=fixed
        try:
            batch=aligner.align(audio=str(context_wav),text=full_text,language='Japanese')
        finally:
            aligner.aligner_processor=original_encoder
        if len(batch)!=1 or len(batch[0].items)!=sum(counts):
            raise ValueError('Context unit count mismatch')
        items=[{'text':i.text,'start_time':i.start_time,'end_time':i.end_time} for i in batch[0].items]
        raw_file=raw/(recording.id+'.json')
        freeze(raw_file,{'prefixFrames':prefix_frames,'contextSha256':file_sha(context_wav),'items':items})
        offset=prefix_frames/16000
        target=[{'text':item['text'],'start_time':number(item['start_time'])-offset,
                 'end_time':number(item['end_time'])-offset} for item in items[counts[0]:]]
        zeros=[i for i,item in enumerate(target) if item['start_time']==item['end_time']]
        try:
            start,end=boundaries_v2(target,len(target_pcm)/32000)
        except ValueError as error:
            diagnostics.append({'id':recording.id,'status':'invalid','reason':str(error),
                                'zeroUnitIndices':zeros,'rawSha256':file_sha(raw_file)})
            print(f'Context reference invalid: {recording.id}; raw retained',flush=True)
            continue
        output.append({**row,'speechStartMs':start,'speechEndMs':end,'annotation':ANNOTATION})
        diagnostics.append({'id':recording.id,'status':'valid','zeroUnitIndices':zeros,
                            'rawSha256':file_sha(raw_file)})
        print(f'Context aligned {recording.id}',flush=True)
    freeze(raw/'diagnostics.json',{'method':provenance,'count':len(diagnostics),
        'validCount':len(output),'invalidCount':60-len(output),'samples':diagnostics})
    if len(output)!=60:
        raise ValueError(f'Context references invalid for {60-len(output)}/60; no references published')
    freeze(root/'timing-references.json',output)
    meta={**provenance,'method':METHOD,'tool':'qwen-asr','toolVersion':'0.0.6',
          'modelRevision':REVISION,'referenceResolutionMs':80,'count':60,
          'createdAt':datetime.now(timezone.utc).isoformat(),'contextAnchorId':'public-zh-001',
          'contextAnchorSha256':anchor['sha256'],'normalizedAnchorSha256':file_sha(normalized_anchor),'contextGapSamples':12800,
          'contextPrefixSamples':len(anchor_pcm)//2+12800,'canonicalLeadingPaddingMs':1000,
          'canonicalTailPaddingMs':1000,'rawDirectory':raw.name,
          'referenceKind':'independent-model-estimate-not-manual-ground-truth',
          'quantizationWarnings':[d for d in diagnostics if d['zeroUnitIndices']],
          'referencesSha256':file_sha(root/'timing-references.json'),
          'inputsSha256':file_sha(root/'alignment-inputs-v3.json'),
          'environment':{d.metadata['Name']:d.version for d in importlib.metadata.distributions()}}
    freeze(root/'timing-references.meta.json',meta)
    print('Published 60 fixed-context references; target input remains byte-identical for DUT',flush=True)

if __name__=='__main__':
    try:
        main()
    except (ValueError,OSError,subprocess.CalledProcessError) as error:
        print(f'Fixed-context reference generation failed: {error}',file=sys.stderr)
        raise SystemExit(1) from error
