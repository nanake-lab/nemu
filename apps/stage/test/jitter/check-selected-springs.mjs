// Selected B, actual rebuilt model, original 30 s drag + 5 s settle stimulus.
// Retain signed rotation velocities; do not turn finite-only checks into acceptance.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { gzipSync } from 'node:zlib';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
const { chromium } = createRequire(import.meta.resolve('@playwright/test'))('playwright');
const sourceControl = process.argv.includes('--source');
const sourceBParameters = process.argv.includes('--b-parameters');
const noColliders = process.argv.includes('--no-colliders');
const presetArgument = process.argv.find(a => a.startsWith('--preset='));
const comparisonPreset = presetArgument?.slice('--preset='.length);
const product = resolve('D:/Dev/cache/P2-05/selected-B-build');
const variant = sourceBParameters ? 'diagnosis-source-B-params' : noColliders ? 'diagnosis-B-no-colliders' : sourceControl ? 'source-control' : 'selected-B';
const output = comparisonPreset ? resolve('reports/P2-05/four-review', comparisonPreset) : resolve('reports/P2-05', variant);
await mkdir(output, { recursive: true });
const input = sourceControl ? 'D:/Dev/nemu/content/nemu/source/nemu-vroid-export.vrm' : resolve(product, 'nemu.vrm');
const bytes = await readFile(input);
const inputJson = JSON.parse(bytes.subarray(20, 20 + bytes.readUInt32LE(12)).toString('utf8'));
const profile = await readFile(resolve(product, 'nemu.profile.json'));
const data = JSON.parse(await readFile('reports/P2-05/preview-data.json', 'utf8'));
if (comparisonPreset && !data.fourComparison.some(c => c.id === comparisonPreset)) throw new Error('Unknown comparison preset');
const parameters = data.presets[comparisonPreset ?? 'heavyBalanced'];
process.env.TEMP = 'D:/Dev/cache/tmp'; process.env.TMP = process.env.TEMP;
const browser = await chromium.launch({ headless: true, args: ['--ignore-gpu-blocklist', '--use-gl=angle'] });
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1050 } });
  const errors = []; page.on('pageerror', e => errors.push(e.message));
  await page.route('**/selected-B.vrm', route => route.fulfill({ contentType: 'application/octet-stream', body: bytes }));
  await page.route('**/selected-B.profile.json', route => route.fulfill({ contentType: 'application/json', body: profile }));
  await page.goto('http://127.0.0.1:5186/test/jitter/review.html');
  await page.waitForFunction(() => window.__P205_READY__ === true);
  const result = await page.evaluate(async ({ parameters, sourceControl, sourceBParameters, noColliders, comparisonPreset, expectedVersion }) => {
    const app = window.__p205Review;
    await app.loadModel('/selected-B.vrm', '/selected-B.profile.json'); app.stop();
    const vrm = app.currentVrm, manager = vrm.springBoneManager;
    if (vrm.meta.version !== expectedVersion) throw new Error('Wrong comparison model loaded');
    const joints = [...manager.joints];
    function category(name) {
      if (name.startsWith('NemuTail')) return 'tail';
      if (name.startsWith('J_Sec_Hair') && /_(23|24)$/.test(name)) return 'ear';
      if (name.includes('Hair')) return 'hair';
      if (name.includes('Skirt')) return 'skirt';
      return 'other';
    }
    if (sourceBParameters || comparisonPreset) for (const joint of joints) {
      const p = parameters[category(joint.bone.name)]; if (!p) continue;
      for (const key of ['stiffness', 'dragForce', 'gravityPower']) joint.settings[key] = p[key];
      joint.settings.gravityDir.set(p.gravityDir[0], p.gravityDir[2], -p.gravityDir[1]);
    }
    if (noColliders) for (const joint of joints) joint.colliderGroups = [];
    for (const joint of sourceControl && !sourceBParameters ? [] : joints) {
      const p = parameters[category(joint.bone.name)];
      if (!p) continue;
      for (const key of ['stiffness', 'dragForce', 'gravityPower']) {
        if (Math.abs(joint.settings[key] - p[key]) > 1e-6) throw new Error(`Rebuilt setting mismatch ${joint.bone.name}.${key}`);
      }
    }
    vrm.humanoid.resetNormalizedPose();
    vrm.humanoid.getNormalizedBoneNode('leftUpperArm').rotation.z = -1.05;
    vrm.humanoid.getNormalizedBoneNode('rightUpperArm').rotation.z = 1.05;
    vrm.humanoid.update(); vrm.scene.updateMatrixWorld(true); manager.reset();
    const geometry = joints.filter(j => category(j.bone.name) === 'hair').map(j => {
      const local = j.initialLocalChildPosition.clone();
      const direction = j.bone.localToWorld(local.clone()).sub(j.bone.getWorldPosition(local.clone())).normalize();
      return { bone: j.bone.name, child: j.child?.name ?? null, localLength: local.length(),
        axisGravityDot: direction.dot(j.settings.gravityDir), hitRadius: j.settings.hitRadius,
        colliderCount: j.colliderGroups.reduce((sum, group) => sum + group.colliders.length, 0) };
    });
    const head = vrm.humanoid.getNormalizedBoneNode('head');
    const spine = vrm.humanoid.getNormalizedBoneNode('spine');
    const hips = vrm.humanoid.getNormalizedBoneNode('hips'), origin = hips.position.clone();
    const previous = joints.map(j => j.bone.quaternion.clone());
    const samples = [], vectors = [], hz = 120, dt = 1 / hz;
    let hairPeakDebug = null, hairPeakSpeed = 0;
    for (let frame = 0; frame < 35 * hz; frame++) {
      const t = frame * dt, driven = t < 30;
      hips.position.copy(origin);
      hips.position.x += driven ? .24 * Math.sin(2 * Math.PI * 1.3 * t) : 0;
      hips.position.y += driven ? .12 * Math.sin(2 * Math.PI * 2.1 * t) : 0;
      head.rotation.z = driven ? .18 * Math.sin(2 * Math.PI * 1.7 * t) : 0;
      spine.rotation.z = driven ? .10 * Math.sin(2 * Math.PI * 1.3 * t) : 0;
      vrm.humanoid.update(); vrm.scene.updateMatrixWorld(true); manager.update(dt);
      const velocity = [], speed = [];
      for (let i = 0; i < joints.length; i++) {
        const q = joints[i].bone.quaternion;
        if (!q.toArray().every(Number.isFinite)) throw new Error(`Nonfinite ${joints[i].bone.name} frame ${frame}`);
        const delta = previous[i].clone().invert().multiply(q).normalize();
        if (delta.w < 0) delta.set(-delta.x, -delta.y, -delta.z, -delta.w);
        const sine = Math.hypot(delta.x, delta.y, delta.z);
        const angle = 2 * Math.atan2(sine, delta.w);
        const factor = sine > 1e-12 ? angle / sine / dt : 0;
        velocity.push([delta.x * factor, delta.y * factor, delta.z * factor]); speed.push(angle / dt);
        if (category(joints[i].bone.name) === 'hair' && angle / dt > hairPeakSpeed) {
          const joint = joints[i]; hairPeakSpeed = angle / dt;
          const point = joint._currentTail.clone(); if (joint.center) point.applyMatrix4(joint.center.matrixWorld);
          const origin = joint.bone.getWorldPosition(point.clone());
          const restAxis = joint._boneAxis.clone().transformDirection(joint._initialLocalMatrix).transformDirection(joint._parentMatrixWorld);
          const contacts = joint.colliderGroups.flatMap(g => g.colliders).map(collider => ({
            bone: collider.parent?.name, shape: collider.shape.type, radius: collider.shape.radius,
            offset: collider.shape.offset.toArray(), tail: collider.shape.tail?.toArray(),
            distance: collider.shape.calculateCollision(collider.colliderMatrix, point, joint.settings.hitRadius, point.clone()),
          }));
          hairPeakDebug = { frame, bone: joint.bone.name, speed: hairPeakSpeed,
            boneLength: joint._worldSpaceBoneLength, localChild: joint.initialLocalChildPosition.toArray(),
            restAxisGravityDot: restAxis.dot(joint.settings.gravityDir),
            restAxisSolvedDirectionDot: restAxis.dot(point.clone().sub(origin).normalize()),
            contacts };
        }
        previous[i].copy(q);
      }
      samples.push(speed); vectors.push(velocity);
    }
    const regions = {};
    for (const part of ['hair', 'ear', 'tail', 'skirt', 'other']) {
      const indexes = joints.map((j, i) => category(j.bone.name) === part ? i : -1).filter(i => i >= 0);
      if (!indexes.length) continue;
      let peak = { speed: 0, frame: 0, bone: '' };
      for (let frame = 0; frame < samples.length; frame++) for (const index of indexes) {
        if (samples[frame][index] > peak.speed) peak = { speed: samples[frame][index], frame, bone: joints[index].bone.name };
      }
      const rmsBySecond = Array.from({ length: 35 }, (_, second) => {
        let sum = 0; for (let f = second * hz; f < (second + 1) * hz; f++) for (const i of indexes) sum += samples[f][i] ** 2;
        return Math.sqrt(sum / (hz * indexes.length));
      });
      regions[part] = { joints: indexes.length, peak, rmsBySecond };
    }
    app.turntable.target.set(0, .8, 0); app.turntable.setDistance(5.8); app.turntable.setPreset('threeQuarter', false); app.render();
    return { hz, drivenSeconds: 30, settlingSeconds: 5, jointNames: joints.map(j => j.bone.name), regions,
      geometry, hairPeakDebug,
      settings: joints.map(j => ({ bone: j.bone.name, stiffness: j.settings.stiffness, dragForce: j.settings.dragForce,
        gravityPower: j.settings.gravityPower, gravityDir: j.settings.gravityDir.toArray() })),
      angularSpeedSamples: samples, angularVelocityVectors: vectors,
      acceptance: 'Pending high-frequency and divergence assessment; finite values alone do not pass.', finite: true };
  }, { parameters, sourceControl, sourceBParameters, noColliders, comparisonPreset, expectedVersion: inputJson.extensions.VRMC_vrm.meta.version });
  assert.deepEqual(errors, []); assert.equal(result.finite, true);
  const raw = Buffer.from(JSON.stringify(result) + '\n');
  await writeFile(resolve(output, comparisonPreset ? 'jitter.json.gz' : sourceControl ? 'jitter-source.json.gz' : 'jitter-B.json.gz'), gzipSync(raw, { mtime: 0 }), { flag: 'wx' });
  const summary = { ...result, angularSpeedSamples: undefined, angularVelocityVectors: undefined,
    modelSha256: createHash('sha256').update(bytes).digest('hex'), rawSha256: createHash('sha256').update(raw).digest('hex'), errors,
    sourceControl, sourceBParameters, noColliders, variant, input, comparisonPreset,
    parameterApplication: comparisonPreset ? 'Review-only K/D/G and gravityDir override on the identical rebuilt B model; topology, colliders and radii unchanged.' : null,
    maximumHairFrameDegrees: result.regions.hair.peak.speed / result.hz * 180 / Math.PI,
    profileUse: sourceBParameters ? 'Rebuilt profile only frames the camera; K/D/G overridden with B, native collider/radius/topology preserved.' : sourceControl ? 'Rebuilt profile only frames the camera; animation compositor remains stopped; authored source springs/settings/colliders unchanged.' : 'Rebuilt profile',
    hasGeneratedTail: result.jointNames.some(name => name.startsWith('NemuTail')) };
  if (sourceControl) assert.equal(summary.hasGeneratedTail, false);
  await writeFile(resolve(output, 'jitter-summary.json'), JSON.stringify(summary, null, 2) + '\n');
  await page.screenshot({ path: resolve(output, 'bg-white_after-35s.png') });
  console.log(JSON.stringify(summary, null, 2));
} finally { await browser.close(); }
