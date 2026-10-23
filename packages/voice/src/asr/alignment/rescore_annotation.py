"""Version a literal human text correction and rescore existing inference, without models."""
from __future__ import annotations
from pathlib import Path
import sys
import unicodedata
sys.path.insert(0,str(Path(__file__).resolve().parent.parent))
from cache_paths import configure_cache_environment, data_directory
from manual_reference import REPO
from reference_data import file_sha, freeze, inside, number, obj, objects, read_json, text, text_sha


def ja_units(value: str) -> list[str]:
    return [c for c in unicodedata.normalize('NFKC',value).lower()
            if not c.isspace() and unicodedata.category(c)[0] not in ('P','S')]


def correction(old: str, note: str, old_edits: int, old_units: int) -> tuple[str,int,int]:
    if '\u6ca1\u6709\u598a' not in note or old.count('\u598a')!=1:
        raise ValueError('Expected one explicit human correction of the extra character')
    corrected=old.replace('\u598a','')
    if ja_units(corrected)!=ja_units(note.split('\uff0c')[0]):
        raise ValueError('Human note differs beyond the one literal correction')
    if old_edits!=0 or len(ja_units(old))!=old_units or len(ja_units(corrected))!=old_units-1:
        raise ValueError('Cannot recover the normalized old hypothesis from a nonzero/mismatched old score')
    # Levenshtein(old_ref, hyp)==0 proves equality of normalized strings.
    # Removing exactly one scored codepoint from ref makes the new distance exactly one.
    return corrected,1,old_units-1


def integer(value: object) -> int:
    n=number(value)
    if n<0 or n!=int(n):
        raise ValueError('Expected nonnegative integer score')
    return int(n)


def main() -> None:
    configure_cache_environment()
    root=data_directory().resolve()
    registration=obj(read_json(REPO/'reports/P4-01/hybrid-reference-registration-v2.json'))
    annotations=inside(root,text(registration['annotationsPath']))
    if file_sha(annotations)!=registration['annotationsSha256']:
        raise ValueError('Human annotation export changed')
    base_path=REPO/'reports/P4-01/evaluation.json'
    base=obj(read_json(base_path))
    results=objects(base['results'])
    users_path=root/'user.json'
    users=objects(read_json(users_path))
    public=objects(read_json(REPO/'packages/voice/src/asr/public-data.lock.json'))
    manifest={text(r['id']):r for r in public+users}
    if len(results)!=150 or len(manifest)!=150 or {r['id'] for r in results}!=set(manifest):
        raise ValueError('Expected the original complete 150-sample inference')
    for result in results:
        source=manifest[text(result['id'])]
        if result['sha256']!=source['sha256'] or file_sha(inside(root,text(source['path'])))!=source['sha256']:
            raise ValueError('Audio differs from the old inference')
    original=next(r for r in users if r['id']=='user-ja-013')
    old_score=next(r for r in results if r['id']=='user-ja-013')
    marking=next(r for r in objects(obj(read_json(annotations))['samples']) if r['id']=='user-ja-013')
    old=text(original['text'])
    frozen=next(r for r in objects(read_json(root/'alignment-inputs-v3.json')) if r['id']=='user-ja-013')
    if text_sha(old)!=frozen['transcriptSha256']:
        raise ValueError('Old reference differs from frozen registered text')
    corrected,edits,total=correction(old,text(marking['note']),integer(old_score['edits']),integer(old_score['referenceUnits']))
    old_text_sha=text_sha(old)
    original['text']=corrected
    old_score.update(edits=edits,referenceUnits=total)
    version=root/'user-reference-human-v2.json'
    freeze(version,users)
    languages=[]
    for lang in ('zh','en','ja'):
        selected=[r for r in results if r['lang']==lang]
        errors=sum(integer(r['edits']) for r in selected)
        units=sum(integer(r['referenceUnits']) for r in selected)
        public_count=sum(r['group']=='public' for r in selected)
        if len(selected)!=50 or public_count!=30:
            raise ValueError('Language sample allocation changed')
        languages.append({'lang':lang,'count':50,'publicCount':30,'userCount':20,
            'metric':'WER' if lang=='en' else 'CER','edits':errors,'referenceUnits':units,
            'errorRate':errors/units,'languageAccuracy':sum(r['lang']==r['predictedLang'] for r in selected)/50})
    accuracy=sum(r['lang']==r['predictedLang'] for r in results)/150
    passed=all(number(r['errorRate'])<=.1 for r in languages) and accuracy>=.98
    proof={'kind':'posthoc human reference correction; no new inference',
        'id':'user-ja-013','baseEvaluationSha256':file_sha(base_path),
        'baseUserManifestSha256':file_sha(users_path),'annotationsSha256':file_sha(annotations),
        'oldTextSha256':old_text_sha,'newTextSha256':text_sha(corrected),
        'oldEdits':0,'oldReferenceUnits':14,'newEdits':edits,'newReferenceUnits':total,
        'proof':'Old normalized edit distance 0 proves hypothesis equals old reference; remove one scored character => distance exactly 1',
        'newUserManifestPath':str(version),'newUserManifestSha256':file_sha(version),
        'oldRegistrationAndTimingSourcesUnchanged':True}
    freeze(version.with_suffix('.meta.json'),proof)
    output={**base,'status':'PASS' if passed else 'FAIL','languages':languages,
            'languageAccuracy':accuracy,'passed':passed,'results':results,'referenceCorrection':proof}
    freeze(REPO/'reports/P4-01/evaluation-human-reference-v2.json',output)
    print('Rescored 150 old inference outputs; no model loaded. Original results and registered text unchanged.')
    for row in languages:
        print(row['lang'],row['edits'],row['referenceUnits'],row['errorRate'])


if __name__=='__main__':
    main()
