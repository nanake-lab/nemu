import { chromium } from '@playwright/test';
import { createServer } from 'vite';
import { resolve } from 'node:path';
import { writeFile } from 'node:fs/promises';
import assert from 'node:assert/strict';

const server = await createServer({ root: resolve('apps/stage'), configFile: false,
  server: { host: '127.0.0.1', port: 0, strictPort: true, open: false } });
let browser;
try {
  await server.listen();
  const port = server.httpServer.address().port;
  browser = await chromium.launch({ headless: true,
    args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${port}/?deterministic=1&layers=L0,L2,L3,L4`);
  await page.waitForFunction(() => window.__STAGE_READY__ === true);
  const readout = { version: 'wire-test-v2', facing: 0.2, tension: 0.8, energy: 0.9 };
  const post = async message => {
    await page.evaluate(message => window.postMessage(message, '*'), message);
    await page.waitForTimeout(50);
  };
  const command = { v: 2, id: 'wire', type: 'stage.readout.v2', payload: { readout } };
  await post(command);
  const sample = () => page.evaluate(() => {
    const app = window.__stageApp;
    for (let i = 0; i < 130; i++) app.scheduler.step(16.67);
    return { target: app.readoutApplicator.getTargetReadout(),
      current: app.readoutApplicator.getCurrentReadout(),
      earPose: app.compositor.l4.getEffectiveEarPose(),
      tailAmplitude: app.compositor.l4.getEffectiveTailAmplitude(),
      hasBaselineInput: 'setExpressionBaseline' in app.compositor.l3 };
  });
  const v2 = await sample(); assert.deepEqual(v2.current, readout);
  assert.equal(v2.earPose, -0.4); assert.equal(v2.tailAmplitude, 0.3); assert.equal(v2.hasBaselineInput, false);
  await post({ ...command, payload: { readout: { ...readout, energy: 2 } } });
  assert.deepEqual((await sample()).current, readout);
  await post({ v: 2, id: 'old', type: 'stage.readout', payload: { readout: {
    version: 'legacy', expressionBaseline: { happy: 0.4 }, posture: 1, idleTempo: 1.5,
    earPose: 1, tailAmplitude: 1, gazeSociability: 1,
  } } });
  const legacy = await sample();
  assert.deepEqual([legacy.current.facing, legacy.current.tension, legacy.current.energy], [0.5, 0, 0.5]);
  await post(command);
  await post({ v: 2, id: 'status', type: 'stage.event', payload: { event: {
    v: 2, id: '01ARZ3NDEKTSV4RRFFQ69G5FAV', ts: 0, source: 'main', type: 'core.runState',
    payload: { states: ['CORE_UNMODULATED'], speechMode: 'A_LOCAL_STRICT', strict: true,
      lineage: 'REHEARSAL', autonomy: 'paused', reasons: [] },
  } } });
  const unmodulated = await sample();
  assert.deepEqual([unmodulated.current.facing, unmodulated.current.tension, unmodulated.current.energy], [0.5, 0, 0.5]);
  await post(command); assert.deepEqual((await sample()).current, unmodulated.current);
  await writeFile('reports/P3-06/stage-wire.json', JSON.stringify({ passed: true, v2, legacy, unmodulated }, null, 2));
  console.log('Stage window-message successor, legacy neutral fallback, invalid rejection, and m=0 gate passed.');
} finally { await browser?.close(); await server.close(); }
