import importlib.metadata as m, json, pathlib
out=[]
for d in sorted(m.distributions(), key=lambda x:x.metadata['Name'].lower()):
 p=d.metadata
 licenses=p.get_all('License-Expression') or [c for c in p.get_all('Classifier',[]) if c.startswith('License ::')]
 out.append({'name':p['Name'],'version':d.version,'license': licenses or [(p.get('License') or 'UNSPECIFIED').splitlines()[0]], 'license_files':p.get_all('License-File',[])})
pathlib.Path(r'D:\Dev\cache\p4-02\gsv-v5-resume-06\installed-dependencies.json').write_text(json.dumps(out,indent=2,ensure_ascii=False),encoding='utf-8')
