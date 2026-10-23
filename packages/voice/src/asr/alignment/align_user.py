"""Local-only, CPU-only reference generation. Must precede DUT measurements."""
from __future__ import annotations
import argparse
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
from typing import Protocol, cast
import wave
from reference_data import (ANNOTATION_V2 as ANNOTATION, METHOD_V2 as METHOD, boundaries_v2 as boundaries, canonicalize_v2 as canonicalize, file_sha, freeze,
                            inside, obj, objects, read_json, recordings, text)

REPO = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
PLAN = REPO / 'reports/P4-01/alignment-preregistration-v2.json'
METHOD_DOC = REPO / 'reports/P4-01/ALIGNMENT-PREREGISTRATION-V2.md'
REVISION = 'c7cbfc2048c462b0d63a45797104fc9db3ad62b7'
LANGUAGES = {'zh': 'Chinese', 'en': 'English', 'ja': 'Japanese'}


class Span(Protocol):
    text: str
    start_time: float
    end_time: float


class Aligned(Protocol):
    items: list[Span]


class Aligner(Protocol):
    def align(self, *, audio: str, text: str, language: str) -> list[Aligned]: ...


class Factory(Protocol):
    def from_pretrained(self, path: str, **options: object) -> Aligner: ...


class Torch(Protocol):
    float32: object
    def set_num_threads(self, value: int) -> None: ...
    def set_num_interop_threads(self, value: int) -> None: ...
    def manual_seed(self, value: int) -> object: ...
    def use_deterministic_algorithms(self, value: bool) -> None: ...


def committed_method() -> dict[str, str]:
    files = [PLAN, METHOD_DOC, HERE / 'manifest.json', HERE / 'pyproject.toml',
             HERE / 'uv.lock', HERE.parent / 'cache_paths.py', Path(__file__), HERE / 'reference_data.py',
             HERE / 'import_continuous.py', REPO / 'reports/P4-01/CONTINUOUS-IMPORT-PREREGISTRATION.md']
    paths = [str(p.relative_to(REPO)) for p in files]
    subprocess.run(['git', 'ls-files', '--error-unmatch', *paths], cwd=REPO,
                   check=True, capture_output=True)
    subprocess.run(['git', 'diff', '--exit-code', 'HEAD', '--', *paths], cwd=REPO,
                   check=True, capture_output=True)
    commit = subprocess.check_output(['git', 'log', '-1', '--format=%H', '--', *paths],
                                     cwd=REPO, text=True).strip()
    return {'commit': commit, 'planSha256': file_sha(PLAN), 'methodSha256': file_sha(METHOD_DOC),
            'manifestSha256': file_sha(HERE / 'manifest.json'), 'uvLockSha256': file_sha(HERE / 'uv.lock')}


def verify_models(cache: Path, download: bool) -> None:
    manifest = obj(read_json(HERE / 'manifest.json'))
    if manifest['revision'] != REVISION:
        raise ValueError('Model revision differs from preregistration')
    cache.mkdir(parents=True, exist_ok=True)
    for entry in objects(manifest['files']):
        target = inside(cache, text(entry['path']))
        expected = entry['sha256']
        if target.exists() and file_sha(target) == expected:
            continue
        if not download:
            raise ValueError(f'Model missing/mismatched: {entry["path"]}; run pnpm asr:align:models')
        partial = target.with_suffix(target.suffix + '.partial')
        subprocess.run(['curl.exe' if os.name == 'nt' else 'curl', '--fail', '--location',
                        '--retry', '3', '--silent', '--show-error', '--output', str(partial),
                        text(entry['url'])], check=True)
        if file_sha(partial) != expected:
            raise ValueError(f'Downloaded model hash mismatch: {entry["path"]}')
        partial.replace(target)
    config = obj(read_json(cache / 'config.json'))
    if config.get('timestamp_segment_time') != 80:
        raise ValueError('Unexpected timestamp grid')


def main() -> None:
    configure_cache_environment()
    parser = argparse.ArgumentParser()
    parser.add_argument('--download', action='store_true')
    parser.add_argument('--preflight', action='store_true')
    args = parser.parse_args()
    if os.environ.get('CI'):
        raise ValueError('Alignment and downloads are local-only; disabled in CI')
    root = data_directory().resolve()
    cache = evaluation_directory() / 'qwen3-forced-aligner' / REVISION
    if args.download:
        verify_models(cache, True)
        print('Independent evaluation model verified; no alignment or measurement was run')
        return
    plan = obj(read_json(PLAN))
    if plan['method'] != METHOD:
        raise ValueError('Unexpected preregistration method')
    selected = recordings(root, plan)
    if args.preflight:
        print(f'Validated {len(selected)}/60 preregistered recordings and human transcripts')
        return
    method = committed_method()
    if (root / 'timing-references.json').exists():
        raise ValueError('References already exist; preserve previous attempt instead of overwriting')
    verify_models(cache, False)
    freeze(root / 'alignment-source-inputs.json', [{'id': r.id, 'lang': r.lang, 'sourcePath': r.source_path, 'sourceSha256': r.source_sha256, 'transcriptSha256': r.transcript_sha256} for r in selected])
    canonical = [(recording, canonicalize(recording, root)) for recording in selected]
    frozen = [{'id': r.id, 'lang': r.lang, 'sourcePath': r.source_path,
               'sourceSha256': r.source_sha256, 'transcriptSha256': r.transcript_sha256,
               'path': p.relative_to(root).as_posix(), 'sha256': file_sha(p)} for r, p in canonical]
    freeze(root / 'alignment-inputs-v2.json', frozen)
    os.environ['CUDA_VISIBLE_DEVICES'] = ''
    os.environ['HF_HUB_OFFLINE'] = '1'
    os.environ['TRANSFORMERS_OFFLINE'] = '1'
    for package, expected in {'qwen-asr': '0.0.6', 'torch': '2.10.0+cpu', 'transformers': '4.57.6'}.items():
        if importlib.metadata.version(package) != expected:
            raise ValueError(f'Wrong preregistered package version: {package}')
    torch = cast(Torch, importlib.import_module('torch'))
    torch.set_num_threads(2)
    torch.set_num_interop_threads(1)
    torch.manual_seed(0)
    torch.use_deterministic_algorithms(True)
    factory = cast(Factory, getattr(importlib.import_module('qwen_asr'), 'Qwen3ForcedAligner'))
    aligner = factory.from_pretrained(str(cache), dtype=torch.float32, device_map='cpu',
                                      attn_implementation='eager', local_files_only=True)
    output: list[dict[str, object]] = []
    diagnostics: list[dict[str, object]] = []
    raw_root = root / ('alignment-raw-' + datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ'))
    raw_root.mkdir()
    for (recording, wav), frozen_row in zip(canonical, frozen, strict=True):
        aligned = aligner.align(audio=str(wav), text=recording.transcript,
                                language=LANGUAGES[recording.lang])
        if len(aligned) != 1:
            raise ValueError(f'Unexpected alignment batch size: {recording.id}')
        items = [{'text': i.text, 'start_time': i.start_time, 'end_time': i.end_time}
                 for i in aligned[0].items]
        (raw_root / f'{recording.id}.json').write_text(json.dumps(items, ensure_ascii=False, indent=2), encoding='utf-8')
        with wave.open(str(wav), 'rb') as stream:
            duration = stream.getnframes() / stream.getframerate()
        zeros = [i for i, item in enumerate(items) if item['end_time'] == item['start_time']]
        try:
            start, end = boundaries(items, duration)
        except ValueError as error:
            diagnostics.append({'id': recording.id, 'lang': recording.lang, 'status': 'invalid',
                                'error': str(error), 'zeroUnitIndices': zeros,
                                'unitCount': len(items), 'sourceSha256': recording.source_sha256,
                                'rawSha256': file_sha(raw_root / f'{recording.id}.json')})
            print(f'Strict alignment invalid: {recording.id}; raw retained', flush=True)
            continue
        diagnostics.append({'id': recording.id, 'lang': recording.lang, 'status': 'valid',
                            'zeroUnitIndices': zeros, 'unitCount': len(items),
                            'sourceSha256': recording.source_sha256,
                            'rawSha256': file_sha(raw_root / f'{recording.id}.json')})
        output.append({**frozen_row, 'speechStartMs': start, 'speechEndMs': end,
                       'annotation': ANNOTATION})
        print(f'Aligned {recording.id}; raw spans remain outside Git')
    freeze(raw_root / 'diagnostics.json', {'method': method, 'count': len(diagnostics),
        'validCount': len(output), 'invalidCount': len(selected)-len(output),
        'validReferencesCreated': False, 'samples': diagnostics})
    if len(output) != len(selected):
        raise ValueError(f'Strict alignment invalid for {len(selected)-len(output)}/60 samples; all raw diagnostics preserved')
    metadata = {**method, 'method': METHOD, 'tool': 'qwen-asr', 'toolVersion': '0.0.6',
                'modelRevision': REVISION, 'referenceResolutionMs': 80, 'count': len(output),
                'referenceKind': 'independent-model-estimate-not-manual-ground-truth',
                'canonicalTailPaddingMs': 1000, 'quantizationWarnings': [d for d in diagnostics if d['zeroUnitIndices']],
                'createdAt': datetime.now(timezone.utc).isoformat(), 'pythonVersion': sys.version,
                'rawDirectory': raw_root.name,
                'environment': {d.metadata['Name']: d.version for d in importlib.metadata.distributions()}}
    # The final references appear only if every one of the 60 alignments succeeds.
    freeze(root / 'timing-references.json', output)
    metadata['referencesSha256'] = file_sha(root / 'timing-references.json')
    metadata['inputsSha256'] = file_sha(root / 'alignment-inputs-v2.json')
    metadata['ffmpegVersion'] = subprocess.check_output(['ffmpeg', '-version'], text=True).splitlines()[0]
    (root / 'timing-references.meta.json').write_text(json.dumps(metadata, indent=2) + '\n', encoding='utf-8')
    print(f'Created 60 independent references under {root}; aligner exits before DUT measurement')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f'Alignment preparation blocked: {error}', file=sys.stderr)
        raise SystemExit(1) from error
