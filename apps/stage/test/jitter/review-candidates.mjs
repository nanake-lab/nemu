// P2-05 review-only fixture: use the existing review bench, never change visual goldens.
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { chromium } from '@playwright/test';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const root = process.cwd();
const products = resolve(root, 'tools/asset-pipeline/.cache/candidates-v4');
const override = process.env.P205_TRIAL ? JSON.parse(await readFile(process.env.P205_TRIAL, 'utf8')) : null;
const output = resolve(root, 'reports/P2-05', override?.label ?? 'visual-v4');
const variants = override?.variants ?? ['short-thick-standard', 'standard-standard', 'long-slender-standard', 'standard-light', 'standard-calm'];
await mkdir(output, { recursive: true });
const server = await createServer({
  root: resolve(root, 'apps/stage'), configFile: resolve(root, 'apps/stage/vite.config.ts'),
  server: { host: '127.0.0.1', port: 5185, strictPort: true, fs: { allow: [root] } },
});
await server.listen();
const browser = await chromium.launch({ headless: true, args: ['--ignore-gpu-blocklist', '--use-gl=angle'] });
const observations = [];
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1050 }, deviceScaleFactor: 1 });
  page.on('pageerror', (error) => { throw error; });
  await page.goto('http://127.0.0.1:5185/review.html?deterministic=1');
  await page.waitForFunction(() => window.__REVIEW_READY__ === true);
  for (const variant of variants) {
    const bytes = await readFile(resolve(products, variant, 'nemu.vrm'));
    const json = JSON.parse(bytes.subarray(20, 20 + bytes.readUInt32LE(12)).toString('utf8'));
    const modelBase = `/@fs/${products.replaceAll('\\', '/')}/${variant}`;
    await page.evaluate(async ({ modelBase, variant, json, override }) => {
      const app = window.__reviewApp;
      app.stop();
      await app.loadModel(`${modelBase}/nemu.vrm`, `${modelBase}/nemu.profile.json`);
      app.stop();
      const vrm = app.currentVrm;
      if (vrm.meta.version !== `P2-05-candidate-${variant}`) throw new Error(`Wrong model: ${vrm.meta.version}`);
      const expected = new Map(json.extensions.VRMC_springBone.springs.flatMap((s) =>
        s.joints.map((j) => [json.nodes[j.node].name, j])));
      for (const joint of vrm.springBoneManager.joints) {
        const data = expected.get(joint.bone.name);
        if (!data || Math.abs(joint.settings.stiffness - (data.stiffness ?? 1)) > 1e-6 ||
            Math.abs(joint.settings.dragForce - (data.dragForce ?? .5)) > 1e-6) throw new Error(`Settings mismatch: ${joint.bone.name}`);
      }
      if (override?.headCollider) {
        const headName = vrm.humanoid.getRawBoneNode('head').name;
        const headColliders = vrm.springBoneManager.colliders.filter((c) => c.parent.name === headName && c.shape.type === 'sphere');
        if (headColliders.length !== 1) throw new Error('Expected one head sphere');
        headColliders[0].shape.offset.fromArray(override.headCollider.offset);
        headColliders[0].shape.radius = override.headCollider.radius;
      }
      vrm.humanoid.resetNormalizedPose();
      vrm.humanoid.getNormalizedBoneNode('leftUpperArm').rotation.z = -1.05;
      vrm.humanoid.getNormalizedBoneNode('rightUpperArm').rotation.z = 1.05;
      vrm.humanoid.update();
      vrm.scene.updateMatrixWorld(true);
      vrm.springBoneManager.reset();
      app.turntable.target.set(0, .80, 0);
      app.turntable.setDistance(5.8);
      app.setLightingPreset('threePoint');
      document.querySelector('.floating-toolbar').style.display = 'none';
      document.querySelector('.tab-btn[data-tab="tab-springs"]').click();
      document.getElementById('model-badge').textContent = `CANDIDATE / ${variant} / rest pose / not approved`;
    }, { modelBase, variant, json, override });
    if (!override && variant.endsWith('-standard')) {
      for (const bg of ['white', 'black']) for (const angle of ['front', 'threeQuarter', 'back']) {
        await page.evaluate(({ bg, angle }) => { const app = window.__reviewApp; app.setBackgroundPreset(bg); app.scene.background = null; app.renderer.renderer.setClearColor(bg === 'white' ? 0xffffff : 0x0a0b0e, 1); app.turntable.setPreset(angle, false); app.render(); }, { bg, angle });
        await page.screenshot({ path: resolve(output, `deterministic-1_model-${variant}_bg-${bg}_framing-fullBody_angle-${angle}_expr-neutral_t-0.png`) });
      }
    }
    const result = await page.evaluate(({ variant }) => {
      const app = window.__reviewApp;
      const vrm = app.currentVrm;
      const manager = vrm.springBoneManager;
      const joints = [...manager.joints];
      const names = joints.map((j) => j.bone.name);
      const previous = joints.map((j) => j.bone.quaternion.clone());
      const samples = [];
      let maxSpeed = 0;
      const dt = 1 / 120;
      // Hips translation inside authored Root centers models pointer displacement; head/spine add grab/turn.
      // Authored spring centers stay unchanged. Record this exact stimulus rather than implying OS mouse coverage.
      const head = vrm.humanoid.getNormalizedBoneNode('head');
      const spine = vrm.humanoid.getNormalizedBoneNode('spine');
      const hips = vrm.humanoid.getNormalizedBoneNode('hips');
      const hipOrigin = hips.position.clone();
      for (let frame = 0; frame < 35 * 120; frame++) {
        const t = frame * dt;
        const driven = t < 30;
        hips.position.copy(hipOrigin);
        hips.position.x += driven ? .24 * Math.sin(2 * Math.PI * 1.3 * t) : 0;
        hips.position.y += driven ? .12 * Math.sin(2 * Math.PI * 2.1 * t) : 0;
        head.rotation.z = driven ? .18 * Math.sin(2 * Math.PI * 1.7 * t) : 0;
        spine.rotation.z = driven ? .10 * Math.sin(2 * Math.PI * 1.3 * t) : 0;
        vrm.humanoid.update();
        vrm.scene.updateMatrixWorld(true);
        manager.update(dt);
        const speeds = joints.map((joint, index) => {
          const q = joint.bone.quaternion;
          if (![q.x, q.y, q.z, q.w].every(Number.isFinite)) throw new Error(`Nonfinite joint ${joint.bone.name} frame ${frame}`);
          const speed = previous[index].angleTo(q) / dt;
          previous[index].copy(q);
          maxSpeed = Math.max(maxSpeed, speed);
          return speed;
        });
        samples.push(speeds);
      }
      hips.position.copy(hipOrigin);
      const windows = Array.from({ length: 35 }, (_, second) => {
        const values = samples.slice(second * 120, (second + 1) * 120).flat();
        return { second, rmsAngularSpeed: Math.sqrt(values.reduce((s, v) => s + v * v, 0) / values.length), maxAngularSpeed: Math.max(...values) };
      });
      const gl = app.renderer.renderer.getContext();
      const info = gl.getExtension('WEBGL_debug_renderer_info');
      return { variant, hz: 120, drivenSeconds: 30, settlingSeconds: 5, maxAngularSpeed: maxSpeed,
        renderer: info ? gl.getParameter(info.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER),
        finite: true, jointNames: names, windows, angularSpeedSamples: samples,
        settings: joints.map((j) => ({ bone: j.bone.name, stiffness: j.settings.stiffness, dragForce: j.settings.dragForce,
          gravityPower: j.settings.gravityPower, gravityDir: j.settings.gravityDir.toArray() })) };
    }, { variant });
    assert.equal(result.finite, true);
    assert.ok(result.jointNames.some((name) => name.startsWith('NemuTail')));
    await writeFile(resolve(output, `jitter-${variant}.json`), `${JSON.stringify(result)}\n`);
    observations.push({ ...result, override, angularSpeedSamples: undefined, settings: undefined });
    console.log(`${variant}: ${result.jointNames.length} joints, finite, max ${result.maxAngularSpeed.toFixed(4)} rad/s`);
  }
  await writeFile(resolve(output, 'observations.json'), `${JSON.stringify(observations, null, 2)}\n`);
} finally {
  await browser.close();
  await server.close();
}
