import hashlib,json
from collections import defaultdict
from datetime import datetime,timezone
from pathlib import Path
root=Path(r"D:\Dev\cache\P0-10\benchmark-run03-audit-01")
src=root/"audit.json"
raw=src.read_bytes()
data=json.loads(raw.decode("utf-8"))
def sha(b): return hashlib.sha256(b).hexdigest()
t0=datetime.now(timezone.utc).isoformat().replace("+00:00","Z")
out={"method":"Formal-only projection of already independently revalidated rows in audit.json; excludes prepared warmup-* IDs and does not alter source records.",
 "createdUtc":t0,"sourcePath":str(src),"sourceSha256":sha(raw),"quantizations":{}}
for q,report in data["quantizations"].items():
 rows=[x for x in report["irlRevalidation"]["allCandidateResults"] if not x["id"].startswith("warmup-")]
 groups=defaultdict(lambda:defaultdict(lambda:{
  "rows":0,"independentSchemaValidTrue":0,"independentSchemaValidFalse":0,
  "sampleSchemaValidTrue":0,"sampleSchemaValidFalse":0,
  "independentSourceBindingValidTrue":0,"independentSourceBindingValidFalse":0,
  "sampleSourceBindingValidTrue":0,"sampleSourceBindingValidFalse":0,
  "booleanComparisonMismatches":0,"wireSchemaValidTrue":0,"wireSchemaValidFalse":0,
  "compactSchemaValidTrue":0,"compactSchemaValidFalse":0}))
 mismatches=[]; invalid=[]
 for r in rows:
  g=groups[r["arm"]][r["lang"]]; g["rows"]+=1
  g["independentSchemaValidTrue" if r["independentFullSchemaValid"] else "independentSchemaValidFalse"]+=1
  if r["sampleRowSchemaValid"] is True:g["sampleSchemaValidTrue"]+=1
  elif r["sampleRowSchemaValid"] is False:g["sampleSchemaValidFalse"]+=1
  g["independentSourceBindingValidTrue" if r["independentSourceBindingValid"] else "independentSourceBindingValidFalse"]+=1
  if r["sampleRowSourceBindingValid"] is True:g["sampleSourceBindingValidTrue"]+=1
  elif r["sampleRowSourceBindingValid"] is False:g["sampleSourceBindingValidFalse"]+=1
  if not r["bothBooleansMatchSample"]:
   g["booleanComparisonMismatches"]+=1;mismatches.append(r)
  w=r["independentWireSchemaValid"]
  if w is not None:g["wireSchemaValidTrue" if w else "wireSchemaValidFalse"]+=1
  c=r["independentCompactSchemaValid"]
  if c is not None:g["compactSchemaValidTrue" if c else "compactSchemaValidFalse"]+=1
  if not r["independentFullSchemaValid"] or not r["independentSourceBindingValid"]:invalid.append(r)
 out["quantizations"][q]={"formalRows":len(rows),"byArmAndLanguage":{
  a:{l:v for l,v in sorted(ls.items())} for a,ls in sorted(groups.items())},
  "booleanMismatchCount":len(mismatches),"booleanMismatches":mismatches,
  "invalidCandidateCount":len(invalid),"invalidCandidates":invalid,
  "allFormalRows":rows}
out["endedUtc"]=datetime.now(timezone.utc).isoformat().replace("+00:00","Z")
out["methodScript"]={"path":str(root/"filter_formal_irl.py"),
 "bytes":(root/"filter_formal_irl.py").stat().st_size,
 "sha256":sha((root/"filter_formal_irl.py").read_bytes())}
dest=root/"formal-irl-revalidation.json"
with dest.open("x",encoding="utf-8",newline="\n") as f:
 json.dump(out,f,ensure_ascii=False,indent=2,allow_nan=False);f.write("\n")
print(json.dumps({q:{"formalRows":v["formalRows"],"booleanMismatchCount":v["booleanMismatchCount"],
 "invalidCandidateCount":v["invalidCandidateCount"],"byArmAndLanguage":v["byArmAndLanguage"]}
 for q,v in out["quantizations"].items()},ensure_ascii=False))
