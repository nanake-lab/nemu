"""CPU tokenizer preflight only; native GGUF token boundaries remain runtime gates."""
from __future__ import annotations

import argparse
import hashlib
import importlib
import json
from pathlib import Path
from typing import Protocol, cast

from c1_fixtures import make_jobs


class Encoded(Protocol):
    @property
    def ids(self) -> list[int]: ...


class TokenizerHandle(Protocol):
    def encode(self, text: str, *, add_special_tokens: bool) -> Encoded: ...


class TokenizerFactory(Protocol):
    def from_file(self, path: str) -> TokenizerHandle: ...


def sha(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream,'sha256').hexdigest()


def check(output: Path) -> None:
    root = Path(__file__).resolve().parents[2]
    inventory = json.loads((root/'reports/P0-12/preparation/model-inventory.json').read_text(encoding='utf-8-sig'))
    entry = next(item for item in inventory['files'] if item['path']=='tokenizer.json')
    path = Path(inventory['target'])/'tokenizer.json'
    if sha(path) != entry['sha256']:
        raise ValueError('Changed official P0-12 tokenizer')
    # Existing tokenizers lacks py.typed; isolate its three used members explicitly.
    factory = cast(TokenizerFactory,importlib.import_module('tokenizers').Tokenizer)
    tokenizer = factory.from_file(str(path))
    records = []
    for job in make_jobs():
        prefix = tokenizer.encode(job['prompt'],add_special_tokens=False).ids
        complete = tokenizer.encode(job['prompt']+job['continuation'],add_special_tokens=False).ids
        if complete[:len(prefix)] != prefix or not prefix or len(prefix)>448 or len(complete)>460:
            raise ValueError('Unstable boundary or context overflow: '+job['id'])
        size = len(complete)-len(prefix)
        if job['kind']!='free' and not 0<size<=112:
            raise ValueError('Scored content outside C1 budget')
        records.append({'id':job['id'],'prompt_tokens':len(prefix),'continuation_tokens':size})
    with output.open('x',encoding='utf-8',newline='\n') as stream:
        json.dump({'method':'HF tokenizers CPU; GGUF runtime must independently agree',
                   'tokenizer':str(path),'sha256':sha(path),'count':len(records),'records':records},stream,indent=2)
        stream.write('\n')


if __name__ == '__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--output',required=True,type=Path)
    check(parser.parse_args().output)
