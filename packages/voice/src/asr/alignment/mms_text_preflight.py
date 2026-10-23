"""Vocabulary preflight only: no acoustic model download, audio or DUT results."""
from __future__ import annotations
import hashlib
import importlib.metadata
import json
from pathlib import Path
import sys
from typing import Protocol, cast
import importlib
sys.path.insert(0,str(Path(__file__).resolve().parent.parent))
from cache_paths import configure_cache_environment, data_directory
from reference_data import objects, read_json, text

class Romanizer(Protocol):
    def romanize_string(self, value: str, *, lcode: str) -> str: ...


def main() -> None:
    configure_cache_environment()
    if importlib.metadata.version('uroman')!='1.3.1.1':
        raise ValueError('Expected uroman 1.3.1.1')
    factory=getattr(importlib.import_module('uroman'),'Uroman')
    u=cast(Romanizer,factory())
    allowed=set("abcdefghijklmnopqrstuvwxyz'")
    report=[]
    for row in objects(read_json(data_directory()/'user.json')):
        language=text(row['lang'])
        approved=text(row['text'])
        roman=u.romanize_string(approved,lcode={'zh':'zho','en':'eng','ja':'jpn'}[language]).lower()
        unknown=sorted({c for c in roman if (c.isalnum() or c=="'") and c not in allowed})
        report.append({'id':row['id'],'lang':language,'unknownCharacters':unknown,
            'transcriptSha256':hashlib.sha256(approved.encode()).hexdigest(),
            'romanizationSha256':hashlib.sha256(roman.encode()).hexdigest()})
    output={'tool':'uroman','version':'1.3.1.1','acousticModelDownloaded':False,
        'count':len(report),'coveredCount':sum(not r['unknownCharacters'] for r in report),
        'samples':report}
    path=Path(__file__).resolve().parents[5]/'reports/P4-01/mms-text-preflight.json'
    path.write_text(json.dumps(output,indent=2)+'\n',encoding='utf-8')
    print(f"Coverage {output['coveredCount']}/{len(report)}; no characters dropped or model downloaded")


if __name__=='__main__':
    main()
