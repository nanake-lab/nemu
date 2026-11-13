import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { render } from './study.mjs';

// Mechanical check on synthetic text ONLY. Uses already installed Playwright, no test runner.
const [dependencyRoot, outputDir, browserPath, mode = 'smoke'] = process.argv.slice(2);
if (!dependencyRoot || !outputDir || !browserPath) throw new Error('Usage: browser-check.mjs DEPENDENCY_ROOT NEW_OUTPUT_DIR BROWSER_EXECUTABLE [timer]');
const require = createRequire(join(resolve(dependencyRoot), 'package.json'));
const playwright = require(require.resolve('playwright', { paths: [require.resolve('@playwright/test')] }));
mkdirSync(outputDir);
const pairs = Array.from({ length: 12 }, (_, i) => ({
  position: i + 1, screened: i === 1, context: '这里是合成占位场景，只检查页面，不是正式盲测材料。',
  left: '这段左侧占位文字仅用于检查排版和保存。', right: '这段右侧占位文字仅用于检查排版和保存。',
}));
const html = render(readFileSync(new URL('page.html', import.meta.url), 'utf8'), readFileSync(new URL('app.js', import.meta.url), 'utf8'), pairs);
const fixture = join(outputDir, 'synthetic.html');
writeFileSync(fixture, html);
const browser = await playwright.chromium.launch({ executablePath: browserPath, headless: true, args: ['--disable-gpu', '--disable-software-rasterizer'] });
const context = await browser.newContext({ acceptDownloads: true, viewport: { width: 1280, height: 960 } });
const page = await context.newPage();
const errors = [], remote = [];
page.on('pageerror', (error) => errors.push(String(error)));
page.on('request', (request) => { if (/^https?:/.test(request.url())) remote.push(request.url()); });
await page.goto(pathToFileURL(resolve(fixture)).href);
assert.equal(await page.locator('#start').isDisabled(), true);
await page.screenshot({ path: join(outputDir, 'intro.png'), fullPage: true });
await page.locator('#approved').check();
await page.locator('#start').click();
const startTime = performance.now();
await page.waitForTimeout(1100);
assert.match(await page.locator('#clock').textContent(), /剩余 13:5[89]/);
await page.screenshot({ path: join(outputDir, 'comparison.png'), fullPage: true });
if (mode === 'timer') {
  process.stdout.write('Synthetic 14-minute real-time timer started; no actual study opened.\n');
  await page.locator('#done').waitFor({ state: 'visible', timeout: 850000 });
} else {
  await page.locator('[data-quality="left"]').click();
  assert.match(await page.locator('#progress').textContent(), /第 3 /);
  await page.locator('#recognized').check();
  await page.locator('[data-quality="ok"]').click();
  await page.locator('[data-choice="tie"]').click();
  for (let i = 3; i < 12; i++) {
    await page.locator('[data-quality="ok"]').click();
    await page.locator('[data-choice="left"]').click();
  }
}
await page.locator('#done').waitFor({ state: 'visible' });
const wallMs = performance.now() - startTime;
const downloadEvent = page.waitForEvent('download');
await page.locator('#download').click();
const download = await downloadEvent;
const resultPath = join(outputDir, 'synthetic-result.json');
await download.saveAs(resultPath);
const result = JSON.parse(readFileSync(resultPath, 'utf8'));
assert.equal(result.answers.length, 12);
assert.equal(result.status, mode === 'timer' ? 'timeout' : 'complete');
if (mode === 'timer') {
  assert(result.elapsedMs >= 840000 && result.elapsedMs < 845000);
  assert(wallMs >= 839000);
} else {
  assert.equal(result.answers[0].choice, null);
  assert.equal(result.answers[0].quality, 'left');
  assert.equal(result.answers[1].quality, 'screened');
  assert.equal(result.answers[2].choice, 'tie');
  assert.equal(result.answers[2].recognized, true);
}
assert.deepEqual(Object.keys(result), ['version', 'materialHash', 'status', 'elapsedMs', 'answers']);
assert.equal(remote.length, 0); assert.deepEqual(errors, []);
await page.locator('#saved').check();
await page.screenshot({ path: join(outputDir, 'saved.png'), fullPage: true });
await page.setViewportSize({ width: 600, height: 900 });
await page.screenshot({ path: join(outputDir, 'narrow-saved.png'), fullPage: true });
const summary = { mode, fixtureOnly: true, browserVersion: browser.version(), wallMs, savedBytes: readFileSync(resultPath).length, remoteRequests: remote.length, pageErrors: errors, status: result.status };
writeFileSync(join(outputDir, 'summary.json'), JSON.stringify(summary, null, 2) + '\n');
process.stdout.write(JSON.stringify(summary) + '\n');
await browser.close();
