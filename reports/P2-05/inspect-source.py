import bpy,json
from pathlib import Path
bpy.data.batch_remove(list(bpy.data.objects))
bpy.ops.import_scene.vrm(filepath='D:/Dev/nemu/content/nemu/source/nemu-vroid-export.vrm',use_addon_preferences=False,extract_textures_into_folder=False,enable_mtoon_outline_preview=False)
a=next(o for o in bpy.data.objects if o.type=='ARMATURE')
result={'armature':a.name,'bones':{},'weightedBounds':{}}
for name in ['J_Bip_C_Hips','J_Bip_C_Head','J_Sec_Hair1_23','J_Sec_Hair2_23','J_Sec_Hair1_24','J_Sec_Hair2_24']:
 b=a.data.bones[name]; result['bones'][name]={'head':list(b.head_local),'tail':list(b.tail_local)}
for suffix in ['23','24']:
 points=[]
 for o in bpy.data.objects:
  if o.type!='MESH':continue
  ids={g.index for g in o.vertex_groups if g.name.endswith('_'+suffix)}
  for v in o.data.vertices:
   if any(g.group in ids and g.weight>0.05 for g in v.groups):points.append(o.matrix_world@v.co)
 result['weightedBounds'][suffix]={'vertices':len(points),'min':[min(p[i] for p in points) for i in range(3)],'max':[max(p[i] for p in points) for i in range(3)]}
Path('reports/P2-05/import-inspection.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
bpy.context.preferences.filepaths.save_version=0
bpy.ops.wm.save_as_mainfile(filepath=str(Path('tools/asset-pipeline/.cache/source-inspection.blend').resolve()))
print(json.dumps(result))
