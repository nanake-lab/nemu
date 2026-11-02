import ast,hashlib,json,subprocess,sys
from pathlib import Path
from typing import Any
root=Path.cwd(); evidence=root/'reports/P0-12/typecheck-03'
before=json.loads((evidence/'before.json').read_text())
class StripTypes(ast.NodeTransformer):
 def visit_ImportFrom(self,node):
  if node.module=='typing':return None
  return self.generic_visit(node)
 def visit_Import(self,node):
  if all(n.name=='csv' for n in node.names):return None
  return node
 def visit_AnnAssign(self,node):
  if node.value is None:return None
  return ast.Assign(targets=[self.visit(node.target)],value=self.visit(node.value))
 def visit_Call(self,node):
  node=self.generic_visit(node)
  if isinstance(node.func,ast.Name) and node.func.id=='cast':return node.args[1]
  return node
checks=[]
for name in before['files']:
 old=subprocess.check_output(['git','show',before['head']+':'+name]).decode('utf-8')
 new=Path(name).read_text(encoding='utf-8')
 a=ast.dump(StripTypes().visit(ast.parse(old)),include_attributes=False)
 b=ast.dump(StripTypes().visit(ast.parse(new)),include_attributes=False)
 checks.append(dict(path=name,runtimeAstEqualExcludingTypingAndUnusedCsv=a==b,beforeSha256=hashlib.sha256(old.encode()).hexdigest(),afterSha256=hashlib.sha256(Path(name).read_bytes()).hexdigest()))
(evidence/'ast-comparison.json').write_text(json.dumps(checks,indent=2),encoding='utf-8',newline='\n')
if not all(r['runtimeAstEqualExcludingTypingAndUnusedCsv'] for r in checks):raise SystemExit('Runtime AST changed')
sys.path.insert(0,str(root/'spikes/carrier-engine'))
import summarize_visual_injection as injection
injection.OUT=evidence/'replay-injection'
injection.main()
import summarize_visual_image_timing as images
replay_image=evidence/'replay-image';replay_image.mkdir()
original_write=images.write_json
def redirect(path: Path,value: object)->None:
 original_write(replay_image/path.name,value)
images.write_json=redirect
images.main()
pairs=[('injection','reports/P0-12/results/injection-summary.json',injection.OUT/'injection-summary.json'),('image timing','reports/P0-12/continuation-02/image-timing-boundaries-v2.json',replay_image/'image-timing-boundaries-v2.json')]
results=[]
def numeric(value: Any,path: str='$')->dict[str,Any]:
 if isinstance(value,dict):return {k:v for name,item in value.items() for k,v in numeric(item,path+'/'+name).items()}
 if isinstance(value,list):return {k:v for i,item in enumerate(value) for k,v in numeric(item,path+'/'+str(i)).items()}
 if isinstance(value,(int,float)) and not isinstance(value,bool):return {path:value}
 return {}
for label,oldname,newpath in pairs:
 oldpath=root/oldname;a=json.loads(oldpath.read_text(encoding='utf-8'));b=json.loads(newpath.read_text(encoding='utf-8'))
 na,nb=numeric(a),numeric(b); differences=[dict(path=k,before=na.get(k),after=nb.get(k)) for k in sorted(na.keys()|nb.keys()) if na.get(k)!=nb.get(k)]
 results.append(dict(label=label,original=oldname,replay=str(newpath.relative_to(root)),jsonEqual=a==b,byteEqual=oldpath.read_bytes()==newpath.read_bytes(),numericValues=len(na),differences=differences))
unchanged=[]
for entry in before['bindings']:
 if entry['path'].startswith('reports/'):
  unchanged.append(dict(path=entry['path'],unchanged=hashlib.sha256((root/entry['path']).read_bytes()).hexdigest()==entry['sha256']))
record=dict(runtimeAstChecks=checks,replays=results,reportAndPriorSummaryHashes=unchanged,allNumbersUnchanged=all(r['jsonEqual'] and not r['differences'] for r in results) and all(r['unchanged'] for r in unchanged),gpuUsed=False,method='Import unchanged calculation functions; redirect only output paths. No GPU/model entrypoints called.')
(evidence/'numeric-comparison.json').write_text(json.dumps(record,ensure_ascii=False,indent=2),encoding='utf-8',newline='\n')
print(json.dumps(dict(replays=results,allNumbersUnchanged=record['allNumbersUnchanged']),ensure_ascii=False))
if not record['allNumbersUnchanged']:raise SystemExit(1)
