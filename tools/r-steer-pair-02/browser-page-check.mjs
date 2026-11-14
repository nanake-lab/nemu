import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { render } from './build-page.mjs';

const out = 'D:/Dev/cache/R-STEER-PAIR-02/page-check-02';
mkdirSync(out);
const require = createRequire('D:/Dev/nemu/package.json');
const playwright = require(require.resolve('playwright', { paths: [require.resolve('@playwright/test')] }));
const fixture = join(out, 'synthetic.html');
const pairs = Array.from({ length: 22 }, (_, i) => ({ position: i + 1, screened: false,
  context: '这是合成检查材料，不是正式题目。', left: '这段左侧占位文字仅检查排版和保存。', right: '这段右侧占位文字仅检查排版和保存。' }));
writeFileSync(fixture, render({ version: 'synthetic-only', question: '合成检查问题', pairs }));
const browser = await playwright.chromium.launch({ executablePath: 'D:/Dev/cache/playwright/chromium-1248/chrome-win64/chrome.exe',
  headless: true, args: ['--disable-gpu', '--disable-software-rasterizer'] });
const context = await browser.newContext({ acceptDownloads: true, viewport: { width: 1280, height: 960 } });
const page = await context.newPage(), errors = [], remote = [];
page.on('pageerror', (e) => errors.push(String(e)));
page.on('request', (r) => { if (/^https?:/.test(r.url())) remote.push(r.url()); });
async function download(name) {
  const event = page.waitForEvent('download');
  await page.locator('#download').click();
  const artifact = await event, file = join(out, name);
  await artifact.saveAs(file);
  return JSON.parse(readFileSync(file, 'utf8'));
}
try {
  await page.goto(pathToFileURL(fixture).href);
  assert(await page.locator('#start').isDisabled());
  assert.match(await page.locator('#intro').textContent(), /22 对.*没有作答时限/);
  await page.screenshot({ path: join(out, 'synthetic-intro.png'), fullPage: true });
  await page.locator('#approved').check(); await page.locator('#start').click();
  await page.locator('#familiar').check();
  await page.locator('[data-quality="left"]').click();
  assert.match(await page.locator('#progress').textContent(), /第 2 /);
  assert(!(await page.locator('#familiar').isChecked()));
  await page.locator('#conditionKnown').check();
  await page.locator('[data-quality="ok"]').click();
  await page.locator('[data-choice="tie"]').click();
  await page.screenshot({ path: join(out, 'synthetic-comparison.png'), fullPage: true });
  await page.setViewportSize({ width: 600, height: 900 });
  await page.screenshot({ path: join(out, 'synthetic-narrow.png'), fullPage: true });
  for (let i = 2; i < 22; i++) {
    await page.locator('[data-quality="ok"]').click();
    await page.locator('[data-choice="left"]').click();
  }
  const smoke = await download('synthetic-complete.json');
  assert.equal(smoke.status, 'complete'); assert.equal(smoke.answers.length, 22);
  assert.equal(smoke.answers[0].choice, null); assert.equal(smoke.answers[0].quality, 'left');
  assert.equal(smoke.answers[0].familiar, true); assert.equal(smoke.answers[0].conditionKnown, false);
  assert.equal(smoke.answers[1].choice, 'tie'); assert.equal(smoke.answers[1].conditionKnown, true);
  await page.locator('#saved').check();
  await page.screenshot({ path: join(out, 'synthetic-saved.png'), fullPage: true });
  writeFileSync(join(out, 'smoke-summary.json'), JSON.stringify({ passed: true, fixtureOnly: true, answers: 22, remoteRequests: remote.length, errors }, null, 2));
  console.log('Synthetic 22-pair flow, quality rejection, tie, distinct recognition fields and JSON download passed.');

  // Inspect the real landing page only. Never press its start button or view its pairs.
  await page.goto(pathToFileURL('D:/Dev/cache/R-STEER-PAIR-02/delivery-v2-unlimited/comparison.html').href);
  assert(await page.locator('#start').isDisabled()); assert(await page.locator('#trial').isHidden());
  assert.equal(await page.locator('#left').textContent(), '');
  assert.equal(await page.locator('#right').textContent(), '');
  await page.screenshot({ path: join(out, 'actual-landing-only.png'), fullPage: true });
  assert.equal(remote.length, 0); assert.deepEqual(errors, []);
  assert(Number.isFinite(smoke.elapsedMs) && smoke.elapsedMs > 0);
  assert.equal(await page.locator('#clock').count(), 0);
  const summary = { passed: true, responseDeadline: null, resultElapsedMs: smoke.elapsedMs,
    browser: browser.version(), remoteRequests: remote.length, pageErrors: errors, syntheticOnlyAnswers: true,
    actualPageInspection: 'landing_only', actualTrialsStarted: 0 };
  writeFileSync(join(out, 'summary.json'), JSON.stringify(summary, null, 2) + '\n');
  console.log(JSON.stringify(summary));
} finally { await browser.close(); }
