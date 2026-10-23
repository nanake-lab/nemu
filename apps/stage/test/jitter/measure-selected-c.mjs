// One CLI-only 30 s drive + 5 s settling run on the exported selected C model.
import assert from 'node:assert/strict';
import { readFile, writeFile, access } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';
import { Vector3 } from 'three';
import { loadPhysicsVrm } from './load-physics-vrm.mjs';
import { springCategory } from './spring-settings.mjs';
const output = 'reports/P2-05/selected-C-final';
const rawPath = `${output}/jitter-raw.json.gz`;
let exists = false; try { await access(rawPath); exists = true; } catch { /* first run */ }
assert.equal(exists, false, 'Do not overwrite or repeat the selected C measurement');
const model = 'D:/Dev/cache/P2-05/selected-C-final/export/nemu.vrm';
const { vrm, bytes } = await loadPhysicsVrm(model);
assert.equal(vrm.meta.version, 'P2-05-C-calmHeavy1');
const parameters = JSON.parse(await readFile('reports/P2-05/preview-data.json', 'utf8')).presets.calmHeavy1;
const manager = vrm.springBoneManager, joints = [...manager.joints];
const settings = joints.map(j => ({ bone: j.bone.name, part: springCategory(j.bone.name) ?? 'other',
  stiffness: j.settings.stiffness, dragForce: j.settings.dragForce,
  gravityPower: j.settings.gravityPower, gravityDir: j.settings.gravityDir.toArray(),
  lengthM: j.initialLocalChildPosition.length(),
}));
for (const s of settings) {
  if (!parameters[s.part]) continue;
  for (const key of ['stiffness', 'dragForce', 'gravityPower']) assert.ok(Math.abs(s[key] - parameters[s.part][key]) < 1e-6, `${s.bone}.${key}`);
  assert.ok(s.gravityDir.every((v, i) => Math.abs(v - [0, -1, 0][i]) < 1e-12), `${s.bone}.gravityDir`);
}
vrm.humanoid.resetNormalizedPose();
vrm.humanoid.getNormalizedBoneNode('leftUpperArm').rotation.z = -1.05;
vrm.humanoid.getNormalizedBoneNode('rightUpperArm').rotation.z = 1.05;
vrm.humanoid.update(); vrm.scene.updateMatrixWorld(true); manager.reset();
const head = vrm.humanoid.getNormalizedBoneNode('head');
const spine = vrm.humanoid.getNormalizedBoneNode('spine');
const hips = vrm.humanoid.getNormalizedBoneNode('hips'), origin = hips.position.clone();
function geometry(joint) {
  const tail = joint._currentTail.clone();
  if (joint.center) tail.applyMatrix4(joint.center.matrixWorld);
  const bone = joint.bone.getWorldPosition(new Vector3());
  return { tail, direction: tail.clone().sub(bone).normalize() };
}
const previous = joints.map(j => ({ q: j.bone.quaternion.clone(), ...geometry(j) }));
const angularSpeed = [], endpointSpeed = [], directionSpeed = [];
const settleTailPositions = [], settleDirections = [], settleAngularVelocities = [];
const peaks = settings.map(() => ({ angleRad: 0, frame: 0 }));
const hz = 120, dt = 1 / hz;
for (let frame = 0; frame < 35 * hz; frame++) {
  const t = frame * dt, driven = t < 30;
  hips.position.copy(origin);
  hips.position.x += driven ? .24 * Math.sin(2 * Math.PI * 1.3 * t) : 0;
  hips.position.y += driven ? .12 * Math.sin(2 * Math.PI * 2.1 * t) : 0;
  head.rotation.z = driven ? .18 * Math.sin(2 * Math.PI * 1.7 * t) : 0;
  spine.rotation.z = driven ? .10 * Math.sin(2 * Math.PI * 1.3 * t) : 0;
  vrm.humanoid.update(); vrm.scene.updateMatrixWorld(true); manager.update(dt);
  const speed = [], pointSpeed = [], segmentSpeed = [], positions = [], directions = [], velocities = [];
  for (const [i, joint] of joints.entries()) {
    const q = joint.bone.quaternion;
    assert.ok(q.toArray().every(Number.isFinite), `Nonfinite quaternion ${joint.bone.name} frame ${frame}`);
    const delta = previous[i].q.clone().invert().multiply(q).normalize();
    if (delta.w < 0) delta.set(-delta.x, -delta.y, -delta.z, -delta.w);
    const sine = Math.hypot(delta.x, delta.y, delta.z), angle = 2 * Math.atan2(sine, delta.w);
    const factor = sine > 1e-12 ? angle / sine / dt : 0;
    const g = geometry(joint);
    assert.ok([...g.tail.toArray(), ...g.direction.toArray()].every(Number.isFinite), `Nonfinite geometry ${joint.bone.name}`);
    speed.push(angle / dt);
    pointSpeed.push(g.tail.distanceTo(previous[i].tail) / dt);
    segmentSpeed.push(Math.atan2(g.direction.clone().cross(previous[i].direction).length(), g.direction.dot(previous[i].direction)) / dt);
    if (angle > peaks[i].angleRad) peaks[i] = { angleRad: angle, frame };
    if (!driven) {
      positions.push(g.tail.toArray()); directions.push(g.direction.toArray());
      velocities.push([delta.x * factor, delta.y * factor, delta.z * factor]);
    }
    previous[i] = { q: q.clone(), ...g };
  }
  angularSpeed.push(speed); endpointSpeed.push(pointSpeed); directionSpeed.push(segmentSpeed);
  if (!driven) { settleTailPositions.push(positions); settleDirections.push(directions); settleAngularVelocities.push(velocities); }
}
const raw = Buffer.from(JSON.stringify({
  hz, drivenSeconds: 30, settlingSeconds: 5, model,
  modelSha256: createHash('sha256').update(bytes).digest('hex'), settings,
  angularSpeed, endpointSpeed, directionSpeed, settleTailPositions, settleDirections, settleAngularVelocities,
  localRotationPeaks: peaks.map((p, i) => ({ bone: settings[i].bone, part: settings[i].part,
    maximumFrameDegrees: p.angleRad * 180 / Math.PI, frame: p.frame, timeSeconds: p.frame / hz })),
  environment: { browserLaunched: false, rendererCreated: false, node: process.version, three: '0.174.0', threeVrm: '3.5.5',
    textures: 'Undecoded placeholder textures only; GLB nodes/skins/transforms and spring settings/colliders unchanged.' },
  finite: true, runs: 1,
}) + '\n');
const compressed = gzipSync(raw, { mtime: 0 });
await writeFile(rawPath, compressed, { flag: 'wx' });
await writeFile(`${output}/measurement.json`, JSON.stringify({
  model, modelSha256: createHash('sha256').update(bytes).digest('hex'), hz, frames: 4200, jointCount: joints.length,
  parametersVerified: true, noParameterOverrides: true, runs: 1, finite: true,
  rawPath, rawSha256: createHash('sha256').update(raw).digest('hex'), compressedBytes: compressed.length,
  compressedSha256: createHash('sha256').update(compressed).digest('hex'),
  localRotationPeaks: JSON.parse(raw).localRotationPeaks,
}, null, 2) + '\n');
console.log(`Selected C: one run, ${joints.length} joints, 4200 finite frames, ${compressed.length} raw gzip bytes.`);
