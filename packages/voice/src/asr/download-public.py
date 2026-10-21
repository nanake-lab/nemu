# /// script
# requires-python = ">=3.12"
# dependencies = ["pyarrow==23.0.1"]
# ///
"""Local-only FLEURS first 30 test rows per language; never selects on ASR quality."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import subprocess
import pyarrow.parquet as pq  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[4]
CACHE = Path(os.environ.get('NEMU_ASR_DATA', str(Path(os.environ.get('LOCALAPPDATA', str(Path.home()))) / 'Nemu' / 'asr-evaluation')))
REV = '168de341b3db6859a9bac1c50a2ef5e3b47647e0'
SHARDS = [('zh', 'cmn_hans_cn', '87c0aebbe183f3a36ac87b5c3421b6ab57036824744ff695029a3f858e7622fd'), ('en', 'en_us', '6428a4d04d3aac29e16b45e039bb1470a8bd7aa334cf92f7984c9c520d1f234d'), ('ja', 'ja_jp', 'e954b67e934b9a31d7a74a070a75225379660d50c8d1aacdb755852c57f23e6b')]

def sha(path: Path) -> str:
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main() -> None:
    if os.environ.get('CI'):
        raise RuntimeError('Public audio download is local-only, disabled in CI')
    CACHE.mkdir(parents=True, exist_ok=True)
    rows: list[dict[str, object]] = []
    for lang, config, expected in SHARDS:
        target = CACHE / f'{config}.parquet'
        url = f'https://huggingface.co/datasets/google/fleurs/resolve/{REV}/{config}/test/0000.parquet'
        if not target.exists() or sha(target) != expected:
            print(f'Downloading pinned FLEURS {config}', flush=True)
            partial = target.with_suffix('.partial')
            subprocess.run(['curl.exe' if os.name == 'nt' else 'curl', '--fail', '--location', '--retry', '3', '--silent', '--show-error', '--output', str(partial), url], check=True)
            if sha(partial) != expected:
                raise RuntimeError(f'Shard hash mismatch {config}')
            partial.replace(target)
        batch = next(pq.ParquetFile(target).iter_batches(batch_size=30)).to_pylist()
        for i, row in enumerate(batch):
            audio = row['audio']['bytes']
            name = f'public/{lang}/{i+1:03}.wav'
            path = CACHE / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(audio)
            rows.append(dict(id=f'public-{lang}-{i+1:03}', lang=lang, path=name, sha256=sha(path), text=row['transcription'], source='google/fleurs', license='CC-BY-4.0', revision=REV, shard_sha256=expected, row_index=i, dataset_id=row['id'], source_url=url))
    lock = ROOT / 'packages/voice/src/asr/public-data.lock.json'
    if lock.exists():
        original = json.loads(lock.read_text(encoding='utf-8'))
        if rows != original:
            raise RuntimeError('Frozen public sample metadata/hash mismatch; not replacing lock')
    else:
        lock.write_text(json.dumps(rows, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    (CACHE / 'public.json').write_text(json.dumps(rows, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    print(f'90 verified public samples: {CACHE}', flush=True)

if __name__ == '__main__':
    main()
