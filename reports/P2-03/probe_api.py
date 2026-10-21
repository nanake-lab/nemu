import bpy, json, struct
from pathlib import Path
p=Path('apps/stage/public/sample.vrm')
b=p.read_bytes(); n=struct.unpack_from('<I',b,12)[0]; j=json.loads(b[20:20+n]); print('META',json.dumps(j['extensions']['VRMC_vrm']['meta']));print('MATERIALS',[m['name'] for m in j['materials']]);print('PRESETS',list(j['extensions']['VRMC_vrm']['expressions']['preset']))
bpy.data.batch_remove(list(bpy.data.objects))
bpy.ops.import_scene.vrm(filepath=str(p.resolve()),use_addon_preferences=False,enable_mtoon_outline_preview=False)
a=next(o for o in bpy.data.objects if o.type=='ARMATURE')
def show(label,x):
 print(label, x.bl_rna.identifier, [(p.identifier,p.type) for p in x.bl_rna.properties])
show('EXT',a.data.vrm_addon_extension); v=a.data.vrm_addon_extension.vrm1
show('META_PROP',v.meta); show('EXPRS',v.expressions); show('EXPR',v.expressions.preset.happy)
x=v.expressions.custom.add(); show('CUSTOM',x); show('MORPH',x.morph_target_binds.add()); show('COLOR',x.material_color_binds.add());show('TRANSFORM',x.texture_transform_binds.add())
s=a.data.vrm_addon_extension.spring_bone1; show('SPRING_EXT',s);c=s.colliders.add();show('COLLIDER',c);show('SPHERE',c.shape.sphere);show('CAPSULE',c.shape.capsule);g=s.collider_groups.add();show('GROUP',g);show('GROUP_COLLIDER',g.colliders.add());sp=s.springs.add();show('SPRING',sp);show('JOINT',sp.joints.add());show('REF',sp.collider_groups.add())
m=next(m for m in bpy.data.materials if m.vrm_addon_extension.mtoon1.enabled);show('MAT',m.vrm_addon_extension.mtoon1);show('MTOON',m.vrm_addon_extension.mtoon1.extensions.vrmc_materials_mtoon)
print('BONES', [b.name for b in a.data.bones]); print('HAIR_MAT',m.name)
bpy.ops.wm.save_as_mainfile(filepath=str(Path('.cache/P2-03-probe.blend').resolve()))



