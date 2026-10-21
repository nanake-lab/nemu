// P0-07 hardware-render load using the unchanged stage placeholder.
// Usage: node render_load.mjs <new-ready-json> <stop-file>
import { spawn, execFileSync } from 'node:child_process';
import { existsSync, writeFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';
import { chromium } from '@playwright/test';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const [readyArg, stopArg] = process.argv.slice(2);
if (!readyArg || !stopArg) throw new Error('ready JSON and stop-file paths required');
const ready = resolve(readyArg);
const stop = resolve(stopArg);
if (existsSync(ready) || existsSync(stop)) throw new Error('use new output paths');
if (process.platform !== 'win32') throw new Error('measurement requires the Windows GPU host');
for (const name of ['Runner.Listener.exe', 'Runner.Worker.exe']) {
  const rows = execFileSync('tasklist', ['/FI', `IMAGENAME eq ${name}`, '/FO', 'CSV', '/NH'], {
    encoding: 'utf8',
    windowsHide: true,
  });
  if (rows.toLowerCase().includes(`"${name.toLowerCase()}"`)) {
    throw new Error('runner must be offline before local rendering');
  }
}
const server = spawn(
  process.execPath,
  [
    resolve(root, 'node_modules/vite/bin/vite.js'),
    '--host',
    '127.0.0.1',
    '--port',
    '5174',
    '--strictPort',
  ],
  {
    cwd: resolve(root, 'apps/stage'),
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
  },
);
server.stdout.on('data', (data) => process.stdout.write(data));
server.stderr.on('data', (data) => process.stderr.write(data));
let browser;
let stopping = false;
process.on('SIGINT', () => {
  stopping = true;
});
process.on('SIGTERM', () => {
  stopping = true;
});
try {
  const deadline = Date.now() + 30_000;
  while (true) {
    if (server.exitCode !== null) throw new Error('stage server exited');
    try {
      const result = await fetch('http://127.0.0.1:5174');
      if (result.ok) break;
    } catch {
      /* server has not bound its port yet */
    }
    if (Date.now() > deadline) throw new Error('stage server readiness timed out');
    await delay(250);
  }
  browser = await chromium.launch({ headless: true, args: ['--use-angle=d3d11'] });
  const page = await browser.newPage({
    viewport: { width: 2560, height: 1440 },
    deviceScaleFactor: 1,
  });
  await page.goto('http://127.0.0.1:5174/?bg=white&framing=fullBody');
  await page.waitForFunction(() => window.__STAGE_READY__ === true, null, { timeout: 30_000 });
  const state = await page.evaluate(() => {
    const canvas = document.querySelector('canvas');
    const gl = canvas?.getContext('webgl2');
    if (!gl) throw new Error('WebGL2 required');
    const ext = gl.getExtension('WEBGL_debug_renderer_info');
    if (!ext) throw new Error('renderer identity unavailable');
    return {
      renderer: gl.getParameter(ext.UNMASKED_RENDERER_WEBGL),
      vendor: gl.getParameter(ext.UNMASKED_VENDOR_WEBGL),
      canvasWidth: canvas.width,
      canvasHeight: canvas.height,
      dpr: devicePixelRatio,
    };
  });
  if (!/NVIDIA.*3080/i.test(state.renderer) || /swiftshader|software/i.test(state.renderer)) {
    throw new Error(`unexpected renderer: ${state.renderer}`);
  }
  const cdp = await browser.newBrowserCDPSession();
  const info = await cdp.send('SystemInfo.getInfo');
  const frameStart = await page.evaluate(
    () => window.__stageApp.renderer.renderer.info.render.frame,
  );
  await delay(1000);
  const frameEnd = await page.evaluate(() => window.__stageApp.renderer.renderer.info.render.frame);
  if (!(frameEnd > frameStart)) throw new Error('stage is not continuously rendering');
  writeFileSync(
    ready,
    JSON.stringify(
      {
        startedAt: new Date().toISOString(),
        browser: browser.version(),
        frameStart,
        frameEnd,
        state,
        gpu: info.gpu,
        stage: 'sample.vrm',
        mode: 'continuous-hardware-rendering-not-visual-regression',
      },
      null,
      2,
    ),
    { flag: 'wx' },
  );
  while (!stopping && !existsSync(stop)) {
    if (server.exitCode !== null || !browser.isConnected()) throw new Error('render load exited');
    await delay(500);
  }
} finally {
  await browser?.close();
  server.kill();
}
