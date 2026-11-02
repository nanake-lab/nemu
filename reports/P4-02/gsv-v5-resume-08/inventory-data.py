import pathlib, hashlib,json,importlib.metadata as m
site=pathlib.Path(r'D:\Dev\cache\p4-02\gsv-v5-venv\Lib\site-packages')
rows=[]
for name in ['g2p-en','fast-langdetect','wordsegment','jieba','jieba-fast','pypinyin','cn2an','opencc','resampy']:
 dist=m.distribution(name)
 for f in dist.files or []:
  p=dist.locate_file(f)
  if p.is_file() and not str(f).endswith(('.py','.pyc','.pyd','.dll')) and '.dist-info/' not in str(f):
   rows.append({'package':name,'version':dist.version,'file':str(f),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
pathlib.Path(__file__).with_name('bundled-data-hashes.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
print('files',len(rows))
