import base64,csv,hashlib,json,shutil
from pathlib import Path
source=Path(r'D:\Dev\cache\uv\archive-v0')
target=Path(r'D:\Dev\cache\p4-02\gsv-v5-venv\Lib\site-packages')
results=[]
for name,key in [('torch','zzl3a3CpbUHvsyFq'),('torchaudio','2ZkY3wtoi3xNd49H')]:
    folder=source/key
    record=folder/f'{name}-2.10.0+cu128.dist-info/RECORD'
    rows=list(csv.reader(record.open(encoding='utf-8',newline='')))
    for relative,digest,size in rows:
        file=folder/relative
        if digest:
            with file.open('rb') as stream: value=hashlib.file_digest(stream,'sha256').digest()
            assert base64.urlsafe_b64encode(value).rstrip(b'=').decode()==digest.split('=',1)[1],relative
            assert file.stat().st_size==int(size),relative
        assert not (target/relative).exists(),relative
    for relative,digest,size in rows:
        dst=target/relative
        dst.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(folder/relative,dst)
        if digest:
            with dst.open('rb') as stream: value=hashlib.file_digest(stream,'sha256').digest()
            assert base64.urlsafe_b64encode(value).rstrip(b'=').decode()==digest.split('=',1)[1],relative
    results.append({'package':name,'version':'2.10.0+cu128','source':str(folder),'target':str(target),'record_sha256':hashlib.sha256(record.read_bytes()).hexdigest(),'copied_and_verified_files':len(rows),'scope':'Python library payload; console entry point wrappers not created'})
Path(r'D:\Dev\cache\p4-02\gsv-v5-resume-02\torch-reuse.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
print(json.dumps(results,indent=2))
