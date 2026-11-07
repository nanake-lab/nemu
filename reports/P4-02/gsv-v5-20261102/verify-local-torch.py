import base64, csv, hashlib, json
from pathlib import Path
root=Path(r'D:\Dev\cache\uv')
results=[]
for name,archive,expected in [('torch','zzl3a3CpbUHvsyFq','fbde8f6a9ec8c76979a0d14df21c10b9e5cab6f0d106a73ca73e2179bc597cae'),('torchaudio','2ZkY3wtoi3xNd49H','9a890dfbc7a3301f5ce7930f3ce452841f0c34e51686609628e99ed52c5cc775')]:
    folder=root/'archive-v0'/archive
    info=folder/f'{name}-2.10.0+cu128.dist-info'
    checked=0; total=0; failures=[]
    for relative,digest,size in csv.reader((info/'RECORD').open(newline='',encoding='utf-8')):
        if not digest: continue
        file=folder/relative
        if not file.is_file(): failures.append({'file':relative,'error':'missing'}); continue
        algorithm,encoded=digest.split('=',1)
        with file.open('rb') as stream: actual=hashlib.file_digest(stream,algorithm).digest()
        if base64.urlsafe_b64encode(actual).rstrip(b'=').decode()!=encoded or file.stat().st_size!=int(size): failures.append({'file':relative,'error':'digest_or_size'})
        checked+=1; total+=file.stat().st_size
    cache=root/'wheels-v6'/'index'/'d2bd0b84f216183d'/name/f'2.10.0+cu128-cp312-cp312-win_amd64.http'
    raw=cache.read_bytes()
    results.append({'name':name,'version':'2.10.0+cu128','archive':str(folder),'files_checked':checked,'bytes_checked':total,'failures':failures,'record_sha256':hashlib.sha256((info/'RECORD').read_bytes()).hexdigest(),'official_wheel_sha256':expected,'cached_http_contains_official_hash':expected.encode() in raw or bytes.fromhex(expected) in raw,'cache_http_sha256':hashlib.sha256(raw).hexdigest(),'wheel_bytes_present':False})
out=Path(r'D:\Dev\cache\p4-02\gsv-v5-preparation\local-torch-verification.json')
out.write_text(json.dumps(results,indent=2),encoding='utf-8')
print(json.dumps(results,indent=2))
raise SystemExit(bool(any(r['failures'] for r in results)))
