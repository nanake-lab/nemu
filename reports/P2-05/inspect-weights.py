import bpy,json
from pathlib import Path
out=[]
for suffix in range(1,25):
 ending=f'_{suffix:02d}'; points=[]
 for o in bpy.data.objects:
  if o.type!='MESH':continue
  ids={g.index for g in o.vertex_groups if g.name.startswith('J_Sec_Hair') and g.name.endswith(ending)}
  for v in o.data.vertices:
   if any(g.group in ids and g.weight>0.1 for g in v.groups):points.append(o.matrix_world@v.co)
 out.append({'chain':suffix,'n':len(points),'min':[round(min(p[i] for p in points),4) for i in range(3)],'max':[round(max(p[i] for p in points),4) for i in range(3)]})
Path('reports/P2-05/hair-weighted-bounds.json').write_text(json.dumps(out,indent=2),encoding='utf-8')
print(json.dumps(out))
