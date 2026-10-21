"""Preregistered continuous recording import; candidates stay outside Git."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import importlib
import importlib.metadata
import json
import os
from pathlib import Path
import shutil
import subprocess
from typing import Protocol, cast
import wave
from align_user import Aligner, Factory, REVISION, Torch, verify_models
from reference_data import file_sha, freeze, inside, number, obj, objects, read_json, text

REPO = Path(__file__).resolve().parents[5]
DOC = REPO / 'reports/P4-01/CONTINUOUS-IMPORT-PREREGISTRATION.md'
PROMPTS = REPO / 'packages/voice/src/asr/reading-prompts.json'

class Tokenizer(Protocol):
    def encode_timestamp(self, text: str, language: str) -> tuple[list[str], str]: ...

class CoarseAligner(Aligner, Protocol):
    aligner_processor: Tokenizer

def sentence_lengths(units: list[list[str]], joint: list[str]) -> list[int]:
    # Context may change boundaries inside a sentence, never its characters.
    if ''.join(joint) != ''.join(u for sentence in units for u in sentence):
        raise ValueError('Group tokenization changed text')
    ends: dict[int, int] = {}
    characters = 0
    for index, token in enumerate(joint):
        characters += len(token)
        ends[characters] = index+1
    counts: list[int] = []
    target, previous = 0, 0
    for sentence in units:
        target += sum(len(token) for token in sentence)
        if target not in ends:
            raise ValueError('Joint unit crosses a sentence boundary')
        counts.append(ends[target]-previous)
        previous = ends[target]
    if any(count <= 0 for count in counts):
        raise ValueError('Empty sentence token group')
    return counts

def candidate_bounds(items: list[dict[str, object]], duration: float) -> tuple[float, float, list[int]]:
    # Only coarse audio preparation; these spans NEVER become timing references.
    if not items:
        raise ValueError('Empty candidate alignment')
    previous_start, previous_end = 0.0, 0.0
    zero_units: list[int] = []
    for index, item in enumerate(items):
        start, end = number(item['start_time']), number(item['end_time'])
        if start < previous_start or end < previous_end or end < start or start < 0 or end > duration:
            raise ValueError('Invalid candidate span; preserve failure')
        if end == start:
            zero_units.append(index)
        previous_start, previous_end = start, end
    start, end = number(items[0]['start_time']), number(items[-1]['end_time'])
    if end <= start:
        raise ValueError('Non-positive complete candidate interval')
    return start*1000, end*1000, zero_units

def check_method() -> str:
    files = [DOC, Path(__file__), PROMPTS]
    paths = [str(p.relative_to(REPO)) for p in files]
    subprocess.run(['git', 'ls-files', '--error-unmatch', *paths], cwd=REPO, capture_output=True, check=True)
    subprocess.run(['git', 'diff', '--exit-code', 'HEAD', '--', *paths], cwd=REPO, capture_output=True, check=True)
    return subprocess.check_output(['git', 'log', '-1', '--format=%H', '--', *paths], cwd=REPO, text=True).strip()

def extract(source: Path, target: Path, start: int, stop: int) -> None:
    with wave.open(str(source), 'rb') as audio:
        if audio.getframerate() != 16000 or audio.getnchannels() != 1 or audio.getsampwidth() != 2:
            raise ValueError('Canonical PCM format mismatch')
        audio.setpos(start)
        pcm = audio.readframes(stop-start)
    if len(pcm) != (stop-start)*2:
        raise ValueError('PCM slice incomplete')
    with target.open('xb') as output:
        with wave.open(output, 'wb') as stream:
            stream.setnchannels(1)
            stream.setsampwidth(2)
            stream.setframerate(16000)
            stream.writeframes(pcm)

def promote(root: Path, archive: Path) -> None:
    manifest = obj(read_json(archive / 'import-candidates.json'))
    if manifest['methodSha256'] != file_sha(DOC):
        raise ValueError('Import method changed')
    rows = objects(manifest['samples'])
    prompts = objects(read_json(PROMPTS))
    if len(rows) != 60 or {text(r['id']) for r in rows} != {text(r['id']) for r in prompts}:
        raise ValueError('Candidate coverage incomplete')
    indexed = {text(r['id']): r for r in rows}
    for prompt in prompts:
        row = indexed[text(prompt['id'])]
        source = inside(archive, text(row['candidatePath']))
        target = inside(root, text(prompt['path']))
        if target.exists() or file_sha(source) != row['sha256']:
            raise ValueError('Existing user WAV or changed candidate; no overwrite')
        if target.with_suffix('.txt').read_text(encoding='utf-8-sig').strip() != prompt['text']:
            raise ValueError('Human transcript differs; preserve and review before promotion')
    for prompt in prompts:
        row = indexed[text(prompt['id'])]
        target = inside(root, text(prompt['path']))
        with inside(archive, text(row['candidatePath'])).open('rb') as src, target.open('xb') as dst:
            shutil.copyfileobj(src, dst)
    print('Promoted 60 WAVs; run pnpm asr:user to register, then independent per-clip alignment')

def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--promote', action='store_true')
    args = parser.parse_args()
    if os.environ.get('CI'):
        raise ValueError('Local-only import')
    archive = cast(Path, args.archive).resolve()
    local = Path(os.environ['LOCALAPPDATA'])
    root = Path(os.environ.get('NEMU_ASR_DATA', str(local / 'Nemu/asr-evaluation'))).resolve()
    commit = check_method()
    if args.promote:
        promote(root, archive)
        return
    source = archive / 'original.m4a'
    canonical = archive / 'continuous-16k.wav'
    if file_sha(source) != '58a8fe9653a2cdac58475a53aeea3e2b132594e641b94efc3e31178e033d87f0':
        raise ValueError('Source differs from preregistration')
    if (archive / 'import-candidates.json').exists():
        raise ValueError('Candidates already frozen; no overwrite')
    if file_sha(canonical) != 'ae86600f0bf28cd2eabaeb9b7133d7001e0e4f29553c693c15ee8eb805fab2ed':
        raise ValueError('Canonical recording changed')
    with wave.open(str(canonical), 'rb') as audio:
        frames = audio.getnframes()
    cache = local / 'Nemu/models/evaluation/qwen3-forced-aligner' / REVISION
    verify_models(cache, False)
    os.environ.update(CUDA_VISIBLE_DEVICES='', HF_HUB_OFFLINE='1', TRANSFORMERS_OFFLINE='1')
    for package, expected in {'qwen-asr': '0.0.6', 'torch': '2.10.0+cpu', 'transformers': '4.57.6'}.items():
        if importlib.metadata.version(package) != expected:
            raise ValueError(f'Wrong package version: {package}')
    torch = cast(Torch, importlib.import_module('torch'))
    torch.set_num_threads(2)
    torch.set_num_interop_threads(1)
    torch.manual_seed(0)
    torch.use_deterministic_algorithms(True)
    factory = cast(Factory, getattr(importlib.import_module('qwen_asr'), 'Qwen3ForcedAligner'))
    aligner = cast(CoarseAligner, factory.from_pretrained(str(cache), dtype=torch.float32, device_map='cpu', attn_implementation='eager', local_files_only=True))
    attempt = archive / ('import-attempt-' + datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ'))
    attempt.mkdir()
    base_prompts = objects(read_json(PROMPTS))
    indexed_prompts = {text(r['id']): r for r in base_prompts}
    prompts = [indexed_prompts[f'user-{lang}-{number:03d}'] for number in range(1,21) for lang in ('zh','en','ja')]
    candidates: list[dict[str, object]] = []
    cursor = 0
    previous_end = 0.0
    for group_index in range(0, 60, 3):
        group = prompts[group_index:group_index+3]
        language = 'Japanese'
        units = [aligner.aligner_processor.encode_timestamp(text(p['text']), language)[0] for p in group]
        joined = '\n'.join(text(p['text']) for p in group)
        joint_units = aligner.aligner_processor.encode_timestamp(joined, language)[0]
        unit_counts = sentence_lengths(units, joint_units)
        stop = min(frames, cursor+45*16000)
        wav = attempt / f'window-{group_index//3+1:02d}.wav'
        extract(canonical, wav, cursor, stop)
        aligned = aligner.align(audio=str(wav), text=joined, language=language)
        if len(aligned) != 1:
            raise ValueError('Unexpected alignment batch size')
        items = [{'text': i.text, 'start_time': i.start_time, 'end_time': i.end_time} for i in aligned[0].items]
        (wav.with_suffix('.json')).write_text(json.dumps(items, ensure_ascii=False, indent=2), encoding='utf-8')
        if len(aligned) != 1 or len(items) != len(joint_units):
            raise ValueError('Unexpected alignment unit count')
        offset = 0
        for prompt, unit_count in zip(group, unit_counts, strict=True):
            start_ms, end_ms, zero_units = candidate_bounds(items[offset:offset+unit_count], (stop-cursor)/16000)
            offset += unit_count
            start, end = cursor/16000+start_ms/1000, cursor/16000+end_ms/1000
            if start < previous_end or end > stop/16000-1:
                raise ValueError(f'Overlapping/edge candidate: {prompt["id"]}')
            candidates.append({**prompt, 'speechStartSample': round(start*16000), 'speechEndSample': round(end*16000), 'coarseZeroDurationUnits': zero_units, 'notTimingReference': True})
            previous_end = end
            print(f'Located {prompt["id"]}: {start:.3f}..{end:.3f}', flush=True)
        cursor = round(previous_end*16000)
    for index, row in enumerate(candidates):
        speech_start, speech_end = int(cast(int, row['speechStartSample'])), int(cast(int, row['speechEndSample']))
        left = 0 if index == 0 else (int(cast(int,candidates[index-1]['speechEndSample']))+speech_start)//2
        right = frames if index == 59 else (speech_end+int(cast(int,candidates[index+1]['speechStartSample'])))//2
        start, stop = max(left,speech_start-16000), min(right,speech_end+16000)
        path = attempt / (text(row['id'])+'.wav')
        extract(canonical, path, start, stop)
        row.update(startSample=start, stopSample=stop, sha256=file_sha(path), candidatePath=path.relative_to(archive).as_posix())
    freeze(archive/'import-candidates.json', {'methodCommit':commit,'methodSha256':file_sha(DOC),'sourceSha256':file_sha(source),'canonicalSha256':file_sha(canonical),'purpose':'coarse-audio-candidates-not-valid-timing-references','samples':candidates})
    print('60 candidates frozen outside Git; awaiting user recording/order confirmation before promotion')

if __name__ == '__main__':
    main()
