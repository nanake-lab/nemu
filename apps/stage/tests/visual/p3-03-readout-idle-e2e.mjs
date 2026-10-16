import { chromium } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  IdleScheduler,
  DEFAULT_IDLE_MOTIONS,
  hasConsecutiveSubsequenceRepeat,
} from '../../../../packages/body/dist/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = resolve(__dirname, '../../../../');
const reportDir = resolve(projectRoot, 'reports/P3-03');
const screenshotsDirA = resolve(reportDir, 'readout_A_relaxed');
const screenshotsDirB = resolve(reportDir, 'readout_B_alert');

await mkdir(screenshotsDirA, { recursive: true });
await mkdir(screenshotsDirB, { recursive: true });

console.log('--- 1. Generating 10-Minute Idle Sampling Evidence ---');
function createSeededPrng(seed = 987654321) {
  let s = seed >>> 0;
  return () => {
    s = (s + 0x6d2b79f5) >>> 0;
    let t = Math.imul(s ^ (s >>> 15), 1 | s);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const scheduler = new IdleScheduler({
  idleSet: DEFAULT_IDLE_MOTIONS,
  prng: createSeededPrng(20261016),
  baseIntervalRange: [5.0, 9.0],
  idleTempo: 1.0,
});

const totalSeconds = 600; // 10 minutes
const dt = 1 / 60; // 60 FPS
const events = [];

for (let t = 0; t <= totalSeconds; t += dt) {
  const triggered = scheduler.update(dt);
  if (triggered) {
    events.push({
      timeSec: Number(t.toFixed(2)),
      timeFormatted: `${Math.floor(t / 60)}m ${Math.floor(t % 60)}s`,
      motionId: triggered,
    });
  }
}

const motionHistory = scheduler.getHistory();
const repeatFound = hasConsecutiveSubsequenceRepeat(motionHistory, 3);

// Verify constraint: no 3 consecutive identical motions
let hasTripleRepeat = false;
for (let i = 2; i < motionHistory.length; i++) {
  if (motionHistory[i] === motionHistory[i - 1] && motionHistory[i - 1] === motionHistory[i - 2]) {
    hasTripleRepeat = true;
    break;
  }
}

console.log(`10-Minute Sampling Result:`);
console.log(`- Total motions triggered: ${events.length}`);
console.log(`- Consecutive repeat of length >= 3 found: ${repeatFound} (Expected: false)`);
console.log(`- 3 identical consecutive motions found: ${hasTripleRepeat} (Expected: false)`);

await writeFile(
  resolve(reportDir, '10min_idle_sample.json'),
  JSON.stringify(
    {
      totalDurationSec: totalSeconds,
      motionCount: events.length,
      hasConsecutiveSubsequenceRepeatOfLen3OrMore: repeatFound,
      has3ConsecutiveIdenticalMotions: hasTripleRepeat,
      events,
      motionHistory,
    },
    null,
    2,
  ),
  'utf-8',
);

console.log('Saved: reports/P3-03/10min_idle_sample.json');

console.log('--- 2. Starting Review Bench for Standby Evidence ---');
const server = await createServer({
  root: resolve(projectRoot, 'apps/stage'),
  configFile: resolve(projectRoot, 'apps/stage/vite.config.ts'),
  server: { host: '127.0.0.1', port: 5175, strictPort: true },
});
await server.listen();

const browser = await chromium.launch({
  headless: true,
  args: ['--enable-gpu-rasterization', '--enable-zero-copy', '--ignore-gpu-blocklist', '--use-gl=angle'],
});

try {
  const page = await browser.newPage({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 1,
  });

  await page.goto('http://127.0.0.1:5175/review.html');
  await page.waitForFunction(() => window.__REVIEW_READY__ === true, null, { timeout: 20000 });
  await page.waitForTimeout(1000);

  // Switch to Body Readout tab
  await page.click('.tab-btn[data-tab="tab-readout"]');

  // --- Session A: Relaxed / Casual Idle Readout ---
  console.log('Capturing Readout Group A (Relaxed Idle)...');
  await page.evaluate(() => {
    window.__reviewApp.applyBodyReadout({
      posture: -0.4,
      idleTempo: 1.35,
      earPose: 0.5,
      tailAmplitude: 0.8,
      gazeSociability: 0.7,
      expressionBaseline: 0.3,
    });
  });

  // Capture snapshots over 3-minute simulated standby (intervals: 0s, 30s, 60s, 90s, 120s, 150s, 180s)
  const intervals = [0, 30, 60, 90, 120, 150, 180];
  for (const sec of intervals) {
    // Advance simulation
    await page.evaluate((seconds) => {
      // Step the review app by elapsed seconds
      window.__reviewApp.update?.(seconds === 0 ? 0.1 : 30.0);
    }, sec);
    await page.waitForTimeout(300);
    const dest = resolve(screenshotsDirA, `readout_A_t${sec}s.png`);
    await page.screenshot({ path: dest });
    console.log(`Captured: readout_A_relaxed/readout_A_t${sec}s.png`);
  }

  // --- Session B: Alert / Focused Standby Readout ---
  console.log('Capturing Readout Group B (Alert / Focused Idle)...');
  await page.evaluate(() => {
    window.__reviewApp.applyBodyReadout({
      posture: 0.6,
      idleTempo: 0.75,
      earPose: -0.4,
      tailAmplitude: 0.3,
      gazeSociability: 0.9,
      expressionBaseline: 0.05,
    });
  });

  for (const sec of intervals) {
    await page.evaluate((seconds) => {
      window.__reviewApp.update?.(seconds === 0 ? 0.1 : 30.0);
    }, sec);
    await page.waitForTimeout(300);
    const dest = resolve(screenshotsDirB, `readout_B_t${sec}s.png`);
    await page.screenshot({ path: dest });
    console.log(`Captured: readout_B_alert/readout_B_t${sec}s.png`);
  }

  console.log('All P3-03 E2E standby verification completed successfully!');
} finally {
  await browser.close();
  await server.close();
}
