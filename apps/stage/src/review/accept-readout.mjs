// P3-06: headless-only rendering/recording of the actual review bench.
import { chromium } from '@playwright/test';
import { createServer } from 'vite';
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';

const probeMode = process.argv.includes('--probe-video');
const videoMode = process.argv.includes('--videos') || probeMode;
const root = resolve('reports/P3-06');
const run = probeMode ? 'video-probe' : videoMode ? 'videos-final' : 'preview-final';
const directory = resolve(root, run);
await mkdir(directory, { recursive: true });
const vrmPath = process.env.NEMU_P306_VRM;
const profilePath = process.env.NEMU_P306_PROFILE;
assert(vrmPath && profilePath, 'Set NEMU_P306_VRM and NEMU_P306_PROFILE to existing read-only assets');
const vrm = await readFile(vrmPath);
const profileBytes = await readFile(profilePath);
const profile = JSON.parse(profileBytes.toString('utf8'));
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const assetHashes = { vrm: sha256(vrm), profile: sha256(profileBytes) };
const groups = [
  { id: 'A', facing: 0.85, tension: -0.65, energy: 0.25 },
  { id: 'B', facing: 0.25, tension: 0.7, energy: 0.85 },
];
const server = await createServer({
  root: resolve('apps/stage'), configFile: false,
  server: { host: '127.0.0.1', port: 5186, strictPort: true, open: false },
  plugins: [{ name: 'p306-read-only-assets', configureServer(instance) {
    instance.middlewares.use((req, res, next) => {
      if (req.url === '/p306-avatar.vrm') { res.setHeader('Content-Type', 'application/octet-stream'); res.end(vrm); }
      else if (req.url === '/p306-profile.json') { res.setHeader('Content-Type', 'application/json'); res.end(profileBytes); }
      else next();
    });
  } }],
});
let browser;
const evidence = { mode: run, startedAt: new Date().toISOString(), assetHashes, profileIdleSet: profile.idleSet,
  groups, results: [], browserErrors: [],
  basePose: 'capture-only synthetic neutral-arm fixture; source profile idleSet is unchanged; not motion-library acceptance',
  recordingWallSeconds: probeMode ? 4 : videoMode ? 181 : 0,
  recordingBackground: 'opaque #f4f3f5 canvas composite' };
function assertRunnerIdle() {
  const result = execFileSync('powershell.exe', ['-NoProfile', '-Command',
    "@(Get-Process Runner.Worker -ErrorAction SilentlyContinue).Count"], { encoding: 'utf8', windowsHide: true });
  assert.equal(Number(result.trim()), 0, 'Runner.Worker is active; no local rendering window');
}
try {
  assertRunnerIdle();
  await server.listen();
  browser = await chromium.launch({ headless: true,
    args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
  for (const group of groups) {
    assertRunnerIdle();
    const page = await browser.newPage({ viewport: { width: 1280, height: 960 }, deviceScaleFactor: 1 });
    await page.addInitScript(() => {
      let state = 306;
      Math.random = () => { state = (Math.imul(1664525, state) + 1013904223) >>> 0; return state / 4294967296; };
    });
    page.on('pageerror', error => evidence.browserErrors.push(String(error)));
    await page.goto('http://127.0.0.1:5186/review.html?model=/p306-avatar.vrm&profile=/p306-profile.json');
    await page.waitForFunction(() => window.__REVIEW_READY__ === true, null, { timeout: 30000 });
    await page.addStyleTag({ content: '.floating-toolbar { display: none !important; }' });
    await page.evaluate(async () => {
      const { installReadoutIdleFixture } = await import('/src/review/readout-idle-fixture.ts');
      installReadoutIdleFixture(window.__reviewApp.currentVrm, window.__reviewApp.compositor);
    });
    await page.evaluate(async group => {
      const { Vector3 } = await import('/node_modules/.vite/deps/three.js');
      const app = window.__reviewApp;
      if (!app.currentVrm) throw new Error('Review model did not load');
      app.stop();
      app.turntable.setPreset('threeQuarter', false);
      app.turntable.setDistance(app.currentProfile.heightM * 3.6);
      app.compositor.setGaze('away');
      app.applyBodyReadout({ facing: group.facing, tension: group.tension, energy: group.energy });
      app.readoutApplicator.update(2);
      for (let i = 0; i < 120; i++) app.update(1 / 60);
      app.currentVrm.scene.updateMatrixWorld(true);
      for (const side of ['left', 'right']) {
        const hand = app.currentVrm.humanoid.getNormalizedBoneNode(`${side}Hand`);
        const arm = app.currentVrm.humanoid.getNormalizedBoneNode(`${side}UpperArm`);
        if (hand.getWorldPosition(new Vector3()).y >= arm.getWorldPosition(new Vector3()).y) {
          throw new Error('Idle observation fixture must lower both hands');
        }
      }
      app.render();
      document.querySelector('[data-tab="tab-readout"]').click();
      for (const key of ['facing', 'tension', 'energy']) {
        const input = document.getElementById(`slider-${key}`);
        input.value = String(group[key]); input.dispatchEvent(new Event('input'));
      }
    }, group);
    const actual = await page.evaluate(() => ({
      readout: window.__reviewApp.readoutApplicator.getTargetReadout(),
      mapping: window.__reviewApp.compositor.getReadoutMapping(),
      rangeCount: document.querySelectorAll('#tab-readout input[type="range"]').length,
    }));
    assert.equal(actual.rangeCount, 3);
    for (const key of ['facing', 'tension', 'energy']) assert.equal(actual.readout[key], group[key]);
    if (!videoMode) {
      for (const bg of ['white', 'black']) {
        await page.evaluate(bg => { const app = window.__reviewApp; app.setBackgroundPreset(bg); app.render(); }, bg);
        const name = `bg-${bg}_framing-fullBody_view-threeQuarter_facing-${group.facing}_tension-${group.tension}_energy-${group.energy}.png`;
        const first = await page.screenshot();
        assert.deepEqual(await page.screenshot(), first, 'Frozen preview must be repeatable');
        await writeFile(resolve(directory, name), first);
      }
      // Both preset UI and neutral reset feed the same applicator.
      await page.click('#btn-reset-readout');
      const reset = await page.evaluate(() => { window.__reviewApp.readoutApplicator.update(2);
        return window.__reviewApp.readoutApplicator.getCurrentReadout(); });
      assert.equal(reset.facing, 0.5); assert.equal(reset.tension, 0); assert.equal(reset.energy, 0.5);
    } else {
      await page.evaluate(recordingMs => {
        const app = window.__reviewApp; app.setBackgroundPreset('gradient'); app.start();
        const canvas = document.getElementById('review-canvas');
        // Encode an opaque 2D composite; direct transparent WebGL capture can corrupt VP8 edges.
        const capture = document.createElement('canvas');
        capture.width = canvas.width; capture.height = canvas.height;
        const context = capture.getContext('2d', { alpha: false });
        let drawFrame;
        const draw = () => {
          context.fillStyle = '#f4f3f5'; context.fillRect(0, 0, capture.width, capture.height);
          context.drawImage(canvas, 0, 0); drawFrame = requestAnimationFrame(draw);
        };
        draw();
        const stream = capture.captureStream(15);
        const recorder = new MediaRecorder(stream, { mimeType: 'video/webm;codecs=vp8', videoBitsPerSecond: 1500000 });
        const chunks = [];
        window.__p306Recording = new Promise((resolveVideo, reject) => {
          recorder.ondataavailable = event => { if (event.data.size) chunks.push(event.data); };
          recorder.onerror = event => reject(new Error(String(event)));
          recorder.onstop = () => {
            const blob = new Blob(chunks, { type: 'video/webm' });
            const reader = new FileReader();
            reader.onload = () => resolveVideo(String(reader.result).split(',')[1]);
            reader.readAsDataURL(blob); stream.getTracks().forEach(track => track.stop());
            cancelAnimationFrame(drawFrame);
          };
        });
        // Keep a one-second capture buffer: the first encoded frame can start after recorder.start.
        recorder.start(1000); setTimeout(() => recorder.stop(), recordingMs);
      }, probeMode ? 4000 : 181000);
      if (probeMode) await new Promise(resolveTick => setTimeout(resolveTick, 4000));
      for (let second = 30; !probeMode && second <= 180; second += 30) {
        await new Promise(resolveTick => setTimeout(resolveTick, 30000));
        assertRunnerIdle(); console.log(`Group ${group.id}: ${second}/180 wall-clock seconds`);
      }
      const bytes = Buffer.from(await page.evaluate(() => window.__p306Recording), 'base64');
      const name = `readout-${group.id}_facing-${group.facing}_tension-${group.tension}_energy-${group.energy}_180s.webm`;
      await writeFile(resolve(directory, name), bytes);
      evidence.results.push({ group: group.id, file: name, bytes: bytes.length, sha256: sha256(bytes), ...actual });
    }
    await page.close();
  }
  assert.deepEqual(evidence.browserErrors, []);
  assert.equal(sha256(await readFile(vrmPath)), assetHashes.vrm);
  assert.equal(sha256(await readFile(profilePath)), assetHashes.profile);
  evidence.passed = true;
} catch (error) {
  evidence.passed = false; evidence.error = String(error); throw error;
} finally {
  evidence.finishedAt = new Date().toISOString();
  await writeFile(resolve(directory, 'manifest.json'), JSON.stringify(evidence, null, 2));
  await browser?.close(); await server.close();
}
