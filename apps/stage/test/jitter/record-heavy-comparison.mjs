// Deterministic fixture capture. No production code, scene edits, or retiming.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
const { chromium } = createRequire(import.meta.resolve('@playwright/test'))('playwright');
const output = resolve('reports/P2-05/heavy-comparison');
await mkdir(output, { recursive: true });
process.env.TEMP = 'D:/Dev/cache/tmp'; process.env.TMP = process.env.TEMP;
await mkdir(process.env.TEMP, { recursive: true });
const browser = await chromium.launch({ headless: true, args: ['--ignore-gpu-blocklist', '--use-gl=angle'] });
const errors = [], samples = [];
let encoder;
try {
  const page = await browser.newPage({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
  page.on('pageerror', e => errors.push(e.message));
  await page.goto('http://127.0.0.1:5186/test/jitter/heavy-comparison.html');
  await page.waitForFunction(() => window.__HEAVY_READY__ === true, { timeout: 60000 });
  await page.evaluate(() => document.body.classList.add('capture'));
  const meta = await page.evaluate(() => {
    const c = window.__heavyCompare;
    return { fps: c.fps, duration: c.duration, viewport: [innerWidth, innerHeight],
      renderers: c.rigs.map(({ app, candidate, counts }) => {
        const gl = app.renderer.renderer.getContext(), ext = gl.getExtension('WEBGL_debug_renderer_info');
        return { id: candidate.id, counts, renderer: ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER) };
      }),
    };
  });
  if (process.argv.includes('--probe')) {
    for (const [frame, label] of [[0, 'start'], [300, 'turn'], [420, 'drag'], [465, 'settle'], [600, 'rest']]) {
      await page.evaluate(f => window.__heavyCompare.stepToFrame(f), frame);
      await page.screenshot({ path: resolve(output, `probe-${label}.png`) });
    }
    assert.deepEqual(errors, []);
    console.log(JSON.stringify(meta, null, 2));
  } else {
    const destination = resolve(output, 'A-B-C_same-motion_24s_30fps.mp4');
    encoder = spawn('D:/Tools/ffmpeg/bin/ffmpeg.exe', ['-y', '-hide_banner', '-loglevel', 'warning',
      '-f', 'image2pipe', '-framerate', String(meta.fps), '-vcodec', 'png', '-i', 'pipe:0',
      '-an', '-c:v', 'libx264', '-preset', 'fast', '-crf', '18', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', destination],
    { windowsHide: true, stdio: ['pipe', 'ignore', 'pipe'] });
    let encoderLog = ''; encoder.stderr.on('data', chunk => { encoderLog += chunk; });
    const completion = once(encoder, 'close');
    for (let frame = 0; frame < meta.duration * meta.fps; frame++) {
      const sampled = await page.evaluate(f => {
        const c = window.__heavyCompare; c.stepToFrame(f);
        return c.sample().map(({ id, parts }) => {
          for (const part of Object.values(parts)) {
            if (!part.tip.every(Number.isFinite) || !part.rotations.flat().every(Number.isFinite)) throw new Error('Nonfinite spring state');
          }
          return { id, tips: Object.fromEntries(Object.entries(parts).map(([name, p]) => [name, p.tip])) };
        });
      }, frame);
      samples.push({ frame, time: frame / meta.fps, variants: sampled });
      const png = await page.screenshot({ type: 'png' });
      if (!encoder.stdin.write(png)) await once(encoder.stdin, 'drain');
      if ([0, 300, 420, 465, 600, 719].includes(frame)) await writeFile(resolve(output, `bg-white_frame-${frame}.png`), png);
      if (frame % 90 === 0) console.log(`Captured ${frame}/${meta.duration * meta.fps} frames`);
    }
    encoder.stdin.end();
    const [code] = await completion;
    await writeFile(resolve(output, 'ffmpeg.log'), encoderLog);
    assert.equal(code, 0, encoderLog);
    assert.deepEqual(errors, []);
    await page.evaluate(() => window.__heavyCompare.setBackground(true));
    await page.screenshot({ path: resolve(output, 'bg-black_frame-719.png') });
    const bytes = await readFile(destination);
    await writeFile(resolve(output, 'capture.json'), JSON.stringify({ ...meta, frames: samples.length,
      warmupSeconds: 4, physicsHz: 120, phases: { idle: [0, 4], turn: [4, 12], drag: [12, 15], settle: [15, 24] },
      sha256: createHash('sha256').update(bytes).digest('hex'), errors,
      limitation: 'Candidate comparison only; no explicit mass/delay parameter, no jitter acceptance or aesthetic approval.',
    }, null, 2) + '\n');
    await writeFile(resolve(output, 'tip-samples.json'), JSON.stringify(samples) + '\n');
    console.log(`Video complete: ${destination}`);
  }
} finally {
  encoder?.stdin.destroy();
  if (encoder && encoder.exitCode === null) encoder.kill();
  await browser.close();
}
