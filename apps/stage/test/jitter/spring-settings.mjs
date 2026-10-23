export function springCategory(name) {
  if (name.startsWith('NemuTail')) return 'tail';
  if (name.startsWith('J_Sec_Hair') && (name.endsWith('_23') || name.endsWith('_24'))) return 'ear';
  if (name.includes('Hair')) return 'hair';
  if (name.includes('Skirt')) return 'skirt';
  return null;
}

export function applySpringSettings(vrm, parameters) {
  const counts = { hair: 0, ear: 0, tail: 0, skirt: 0 };
  for (const joint of vrm.springBoneManager.joints) {
    const kind = springCategory(joint.bone.name);
    if (!kind) continue;
    const p = parameters[kind];
    joint.settings.stiffness = p.stiffness;
    joint.settings.dragForce = p.dragForce;
    joint.settings.gravityPower = p.gravityPower;
    // Candidate configs use Blender axes; glTF runtime uses Y up.
    joint.settings.gravityDir.set(p.gravityDir[0], p.gravityDir[2], -p.gravityDir[1]);
    counts[kind]++;
  }
  for (const [kind, count] of Object.entries(counts)) {
    if (!count) throw new Error(`No ${kind} joints in candidate model`);
  }
  return counts;
}
