// Run with the local serve-review.mjs fixture on port 5186. Serial browser check.
import assert from 'node:assert/strict';
import { chromium } from '@playwright/test';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const output = resolve('reports/P2-05/idle-preview');
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true, args: ['--ignore-gpu-blocklist', '--use-gl=angle'] });
const errors = [];
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1050 } });
  page.on('pageerror', e => errors.push(e.message));
  await page.goto('http://127.0.0.1:5186/test/jitter/review.html');
  await page.waitForFunction(() => window.__P205_READY__ === true);
  assert.equal(await page.locator('#idle').isChecked(), false);
  await page.locator('#idle').check();
  await page.waitForFunction(() => window.__p205Idle.elapsed > .2);
  await page.locator('#preset').selectOption('calm');
  assert.equal(await page.locator('#idle').isChecked(), true);
  await page.waitForFunction(() => window.__p205Idle.elapsed > .2);
  await page.locator('#slow-turn').check();
  await page.locator('#stop').click();
  assert.equal(await page.locator('#idle').isChecked(), false);
  const paused = await page.evaluate(() => window.__p205Idle.elapsed);
  await page.waitForTimeout(200);
  assert.equal(await page.evaluate(() => window.__p205Idle.elapsed), paused);

  // Real loaded VRM, fixed dt, actual runtime layers and spring solver.
  const observations = [];
  for (const preset of ['light', 'standard', 'calm']) {
    await page.locator('#preset').selectOption(preset);
    const observation = await page.evaluate(() => {
      const idle = window.__p205Idle, vrm = window.__p205Review.currentVrm;
      idle.reset();
      const base = vrm.humanoid.getNormalizedBoneNode('hips').position.clone();
      let maxBlink = 0, maxSway = 0, maxChest = 0, maxTurn = 0;
      const tail = [...vrm.springBoneManager.joints].find(j => j.bone.name.startsWith('NemuTail'));
      let tailTravel = 0;
      const last = tail.bone.quaternion.clone();
      for (let frame = 0; frame < 900; frame++) {
        idle.gaze = { x: frame < 450 ? -.8 : .8, y: .25 };
        idle.step(1 / 60, true);
        const hips = vrm.humanoid.getNormalizedBoneNode('hips');
        maxSway = Math.max(maxSway, hips.position.distanceTo(base));
        maxChest = Math.max(maxChest, Math.abs(vrm.humanoid.getNormalizedBoneNode('chest').rotation.x));
        maxTurn = Math.max(maxTurn, Math.abs(hips.rotation.y));
        maxBlink = Math.max(maxBlink, vrm.expressionManager.getValue('blink'));
        tailTravel += tail.bone.quaternion.angleTo(last); last.copy(tail.bone.quaternion);
        for (const joint of vrm.springBoneManager.joints) {
          if (!joint.bone.quaternion.toArray().every(Number.isFinite)) throw new Error(`Nonfinite ${joint.bone.name}`);
        }
      }
      window.__p205Review.render();
      return { maxBlink, maxSway, maxChest, maxTurn, tailTravel,
        gaze: idle.body.getGazeAngles(), tailStiffness: tail.settings.stiffness };
    });
    assert.ok(observation.maxBlink > .5, 'runtime blink must close eyes');
    assert.ok(observation.maxSway > .001 && observation.maxSway < .02, 'sway must move without accumulating');
    assert.ok(observation.maxChest > .005 && observation.maxChest < .1, 'breathing must move without accumulating');
    assert.ok(observation.maxTurn > .3 && observation.maxTurn < .4);
    assert.ok(observation.gaze.headYaw > 0 && observation.gaze.eyeYaw > observation.gaze.headYaw);
    assert.ok(observation.tailTravel > .01);
    observations.push({ preset, ...observation });
  }
  for (const background of ['white', 'black']) {
    if (background === 'black') await page.locator('#background').click();
    await page.screenshot({ path: resolve(output, `preset-calm_bg-${background}_idle-15s_turn-on.png`) });
  }
  await page.locator('#idle').check();
  const bounds = await page.locator('#review-canvas').boundingBox();
  await page.mouse.move(bounds.x + bounds.width * .9, bounds.y + bounds.height * .4);
  await page.waitForFunction(() => window.__p205Idle.gaze.x > .7);
  await page.locator('#play').click();
  assert.equal(await page.locator('#idle').isChecked(), false);
  await page.locator('#stop').click();
  assert.deepEqual(errors, []);
  await writeFile(resolve(output, 'checks.json'), JSON.stringify({
    passed: true, checks: ['paused default', 'idle toggle', 'preset resumes idle', 'stop cancels loop',
      '900 fixed steps per preset', 'bounded breathing and sway', 'blink', 'gaze', 'slow turn',
      'finite spring orientations', 'pointer input', 'diagnostic excludes idle'], observations,
    limitation: 'Fixture integration check only; not P2-05 jitter acceptance or aesthetic approval.',
  }, null, 2) + '\n');
  console.log(JSON.stringify({ passed: true, observations }, null, 2));
} finally { await browser.close(); }
