"""Build a self-contained, blind offline annotation page on D; no model inference."""
from __future__ import annotations
import base64
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import wave
import zlib
import numpy as np
from numpy.typing import NDArray
sys.path.insert(0,str(Path(__file__).resolve().parent.parent))
from cache_paths import configure_cache_environment, data_directory
from manual_reference import PLAN_PATH, REPO, selection
from reference_data import file_sha, inside, obj, objects, read_json, recordings, text


def chunk(kind: bytes, data: bytes) -> bytes:
    return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))


def spectrum_png(samples: NDArray[np.int16]) -> bytes:
    """Fixed-scale 32 ms Hann STFT / 5 ms hop; visual aid, never boundary detection."""
    values = samples.astype(np.float64)/32768
    padded = np.pad(values,(256,256))
    frames = np.lib.stride_tricks.sliding_window_view(padded,512)[::80]
    power = np.abs(np.fft.rfft(frames*np.hanning(512),axis=1))/256
    db = 20*np.log10(np.maximum(power,1e-9))
    level = np.clip((db+90)/75,0,1).T[::-1]
    # Fixed dark-blue -> amber scale, no per-recording gain normalization.
    rgb = np.stack((15+230*level,22+165*level,35+45*level),axis=2).astype(np.uint8)
    height,width = rgb.shape[:2]
    scanlines = b''.join(b'\0'+row.tobytes() for row in rgb)
    return (b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,2,0,0,0))
            +chunk(b'IDAT',zlib.compress(scanlines))+chunk(b'IEND',b''))


def main() -> None:
    configure_cache_environment()
    plan, _ = selection()
    relative_files = ['reports/P4-01/manual-reference-plan.json',
                      'reports/P4-01/MANUAL-REFERENCE-PREREGISTRATION.md']
    subprocess.run(['git','ls-files','--error-unmatch',*relative_files],cwd=REPO,check=True,capture_output=True)
    subprocess.run(['git','diff','--exit-code','HEAD','--',*relative_files],cwd=REPO,check=True,capture_output=True)
    root = data_directory().resolve()
    original_plan = obj(read_json(REPO/'reports/P4-01/alignment-preregistration-v4.json'))
    approved = {r.id:r for r in recordings(root,original_plan)}
    inputs_path = root/'alignment-inputs-v3.json'
    if file_sha(inputs_path) != plan['canonicalInputsSha256']:
        raise ValueError('Canonical input manifest changed')
    inputs = {text(r['id']):r for r in objects(read_json(inputs_path))}
    output = inside(root,'manual-labels')
    output.mkdir(exist_ok=True)
    tasks: list[dict[str,object]] = []
    identity: list[dict[str,object]] = []
    for value in plan['presentationOrder'] if isinstance(plan['presentationOrder'],list) else []:
        identifier = text(value)
        row = inputs[identifier]
        recording = approved[identifier]
        if row['sourceSha256'] != recording.source_sha256 or row['transcriptSha256'] != recording.transcript_sha256:
            raise ValueError('User-approved recording changed')
        wav = inside(root,text(row['path']))
        if file_sha(wav) != row['sha256']:
            raise ValueError('Canonical audio hash changed')
        with wave.open(str(wav),'rb') as stream:
            if (stream.getframerate(),stream.getnchannels(),stream.getsampwidth()) != (16000,1,2):
                raise ValueError('Expected canonical mono PCM16 at 16 kHz')
            count = stream.getnframes()
            pcm = stream.readframes(count)
        task = {'id':identifier,'lang':recording.lang,'sha256':row['sha256'],
                'frameCount':count,'sampleRate':16000,'transcript':recording.transcript,
                'wav':base64.b64encode(wav.read_bytes()).decode('ascii'),
                'spectrum':base64.b64encode(spectrum_png(np.frombuffer(pcm,dtype='<i2'))).decode('ascii')}
        tasks.append(task)
        identity.append({k:task[k] for k in ('id','lang','sha256','frameCount','sampleRate')})
    if len(tasks) != 26:
        raise ValueError('Expected 26 blind tasks')
    payload = {'method':plan['method'],'planSha256':file_sha(PLAN_PATH),'sampleRate':16000,
               'tasks':tasks}
    template = (REPO/'packages/voice/web/annotation/index.html').read_text(encoding='utf-8')
    serialized = json.dumps(payload,ensure_ascii=True,separators=(',',':')).replace('<','\\u003c')
    html = template.replace('__ANNOTATION_PAYLOAD__',serialized)
    destination = output/'annotation.html'
    # Page rebuilds never change or remove any annotations in this directory.
    destination.write_text(html,encoding='utf-8')
    (output/'input-identities.json').write_text(json.dumps(identity,indent=2)+'\n',encoding='utf-8')
    manifest = {'method':plan['method'],'planSha256':file_sha(PLAN_PATH),
                'pagePath':str(destination),'pageSha256':file_sha(destination),'bytes':destination.stat().st_size,
                'taskCount':26,'networkPolicy':'CSP connect-src none; embedded media; no server',
                'automaticPointsEmbedded':False,'browserValidation':'file protocol blocked by automation policy',
                'inputIdentitiesSha256':hashlib.sha256((output/'input-identities.json').read_bytes()).hexdigest(),
                'sourceSha256':{'generator':file_sha(Path(__file__)),
                               'template':file_sha(REPO/'packages/voice/web/annotation/index.html'),
                               'referenceHelpers':file_sha(Path(__file__).with_name('manual_reference.py'))},
                'methodDocSha256':file_sha(REPO/'reports/P4-01/MANUAL-REFERENCE-PREREGISTRATION.md')}
    (REPO/'reports/P4-01/manual-page-build.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print(f'Offline blind annotation page ready: {destination}; 26 tasks; no automatic markers')


if __name__=='__main__':
    main()
