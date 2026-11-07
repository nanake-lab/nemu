import base64,csv,hashlib,json,shutil
from pathlib import Path
root=Path(r'D:\Dev\cache\uv\archive-v0'); target=Path(r'D:\Dev\cache\p4-02\gsv-v5-venv\Lib\site-packages')
keys=['b-UwTluh8WeNNhWR','bNPKZTq0krYXg0zO','buoPi3n84VQH1oLc','CanYmGqnDSwuf8JX','clx8hf-_qsZYuVrw','FWdXoiMGs3Yud6wq','halNDmtV1wpkkpST','mkzo6RUs3QUhpS9C','nHwIi85VXjfR4-uw']
result=[]
for key in keys:
    folder=root/key; record=next(folder.glob('*.dist-info/RECORD')); rows=list(csv.reader(record.open(encoding='utf-8',newline='')))
    for relative,digest,size in rows:
        assert not relative.startswith('../'),relative
        src=folder/relative
        if digest:
            with src.open('rb') as f: h=hashlib.file_digest(f,'sha256').digest()
            assert base64.urlsafe_b64encode(h).rstrip(b'=').decode()==digest.split('=',1)[1],relative
            assert src.stat().st_size==int(size),relative
        assert not (target/relative).exists(),relative
    for relative,digest,size in rows:
        dst=target/relative; dst.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(folder/relative,dst)
        if digest:
            with dst.open('rb') as f: h=hashlib.file_digest(f,'sha256').digest()
            assert base64.urlsafe_b64encode(h).rstrip(b'=').decode()==digest.split('=',1)[1],relative
    result.append({'source':str(folder),'distribution':record.parent.name,'files':len(rows),'record_sha256':hashlib.sha256(record.read_bytes()).hexdigest()})
Path(r'D:\Dev\cache\p4-02\gsv-v5-resume-02\torch-dependencies-reuse.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
print('All cached dependency files verified and copied.')
