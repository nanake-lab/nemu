import { writeFile, mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { serverAndBrowser, assertQuiet } from '../materials/audit-capture.mjs';
const dir = resolve('reports/P2-06/revision2/visual');
await mkdir(dir, { recursive: true });
const jobs = [];
if (process.env.P206_DIAGNOSTIC === '3') {
  jobs.push({ variant:'B', diagnostic:'lip-smoothed', bg:'white', framing:'face' });
} else if (process.env.P206_DIAGNOSTIC === '2') {
  jobs.push({ revision:'r1', variant:'B', diagnostic:'no-normal', bg:'white', framing:'face' });
} else if (process.env.P206_DIAGNOSTIC === '1') {
  for (const diagnostic of ['control', 'mouth-outline', 'skin-outline', 'all-outline', 'hide-mouth'])
    jobs.push({ revision: 'r1', variant: 'B', diagnostic, bg: 'white', framing: 'face' });
} else {
  for (const expr of ['nemuSmug', 'nemuCatMouth', 'nemuSparkle', 'nemuBlush', 'nemuTeary', 'nemuSleepy', 'nemuPout', 'nemuAngryMark'])
    for (const variant of ['A', 'B', 'C']) jobs.push({ expr, variant, weight: '1', bg: 'white', framing: 'face' });
  for (const emo of ['neutral', 'happy', 'relaxed', 'angry', 'worried', 'excited', 'sleepy', 'surprised', 'sad', 'shy', 'smug', 'pout'])
    for (const variant of ['A', 'B']) jobs.push({ emo, variant, bg: 'white', framing: 'face' });
  for (const expr of ['nemuTeary', 'nemuAngryMark', 'nemuPout']) jobs.push({ expr, variant: 'C', weight: '1', bg: 'black', framing: 'face' });
  for (const bg of ['white','black','gradient']) jobs.push({ variant:'B', bg, framing:'upperBody' });
}
const { server, browser } = await serverAndBrowser();
const results = [], errors = [];
try {
  const page = await browser.newPage({ viewport: { width: 440, height: 500 }, deviceScaleFactor: 1 });
  page.on('pageerror', e => { errors.push(String(e)); console.error('PAGE ERROR', String(e)); });
  page.on('requestfailed', request => console.error('REQUEST FAILED', request.url(), request.failure()));
  for (const job of jobs) {
    assertQuiet();
    const params = new URLSearchParams({ deterministic: '1', ...job });
    const url = 'http://127.0.0.1:5188/tools/asset-pipeline/expressions/revision2.html?' + params;
    await page.goto(url, { waitUntil:'commit' });
    await page.waitForFunction(() => window.__P206_READY__, null, { timeout: 60000 });
    const first = await page.screenshot();
    await page.evaluate(() => window.__P206_RENDER__());
    const second = await page.screenshot();
    if (!first.equals(second)) throw Error('Non-deterministic ' + params);
    const file = [...params].map(([k, v]) => `${k}-${v}`).join('_') + '.png';
    await writeFile(resolve(dir, file), first);
    const runtime = await page.evaluate(() => window.__P206_REPORT__);
    if (!runtime.triangles) throw Error('Empty render');
    results.push({ ...job, file, url, deterministic: true, runtime });
    console.log(file);
  }
  assertQuiet();
  if (errors.length) throw Error(errors.join('\n'));
  await writeFile(resolve(dir, process.env.P206_DIAGNOSTIC ? `diagnostic-${process.env.P206_DIAGNOSTIC}.json` : 'screenshots.json'), JSON.stringify({ results, errors }, null, 2));
} finally { await browser.close(); await server.close(); }
