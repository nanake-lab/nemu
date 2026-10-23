import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const { chromium } = createRequire(import.meta.resolve('@playwright/test'))('playwright');
const output = resolve('reports/P2-05/heavy-comparison');
const video = resolve(output, 'A-B-C_same-motion_24s_30fps.mp4');
const probe = JSON.parse(execFileSync('D:/Tools/ffmpeg/bin/ffprobe.exe', ['-v', 'error', '-count_frames',
  '-show_entries', 'stream=codec_type,codec_name,width,height,avg_frame_rate,nb_read_frames:format=duration',
  '-of', 'json', video], { windowsHide: true, encoding: 'utf8' }));
assert.equal(probe.streams.length, 1);
assert.deepEqual(probe.streams[0], { codec_name: 'h264', codec_type: 'video', width: 1920, height: 1080, avg_frame_rate: '30/1', nb_read_frames: '720' });
assert.equal(Number(probe.format.duration), 24);
const capture = JSON.parse(await readFile(resolve(output, 'capture.json'), 'utf8'));
assert.equal(capture.frames, 720); assert.deepEqual(capture.errors, []);
process.env.TEMP = 'D:/Dev/cache/tmp'; process.env.TMP = process.env.TEMP;
const browser = await chromium.launch({ headless: true, args: ['--ignore-gpu-blocklist', '--use-gl=angle'] });
try {
  const page = await browser.newPage({ viewport: { width: 1920, height: 1080 } });
  const errors = []; page.on('pageerror', e => errors.push(e.message));
  await page.goto('http://127.0.0.1:5186/test/jitter/heavy-comparison.html');
  await page.waitForFunction(() => window.__HEAVY_READY__ === true);
  const settings = await page.evaluate(() => window.__heavyCompare.rigs.map(({ app, candidate }) => {
    const joint = [...app.currentVrm.springBoneManager.joints].find(j => j.bone.name.startsWith('NemuTail'));
    return { id: candidate.id, stiffness: joint.settings.stiffness, drag: joint.settings.dragForce, gravity: joint.settings.gravityPower };
  }));
  assert.deepEqual(settings, [
    { id: 'calm', stiffness: 1.755, drag: .75, gravity: .035 },
    { id: 'heavyBalanced', stiffness: 1.2, drag: .83, gravity: .09 },
    { id: 'heavyDamped', stiffness: .95, drag: .88, gravity: .14 },
  ]);
  const tailTips = await page.evaluate(() => { const c = window.__heavyCompare; c.stepToFrame(420); return c.sample().map(s => s.parts.tail.tip); });
  assert.notDeepEqual(tailTips[0], tailTips[1]); assert.notDeepEqual(tailTips[1], tailTips[2]);
  await page.locator('#reset').click();
  await page.locator('#play').click();
  await page.waitForFunction(() => window.__heavyCompare.rigs[0].idle.elapsed > .1);
  await page.locator('#pause').click();
  const paused = await page.evaluate(() => window.__heavyCompare.rigs[0].idle.elapsed);
  await page.waitForTimeout(150);
  assert.equal(await page.evaluate(() => window.__heavyCompare.rigs[0].idle.elapsed), paused);
  await page.goto('http://127.0.0.1:5186/test/jitter/review.html');
  await page.waitForFunction(() => window.__P205_READY__ === true);
  assert.equal(await page.locator('#preset').inputValue(), 'calm');
  for (const [id, stiffness] of [['heavyBalanced', 1.2], ['heavyDamped', .95]]) {
    await page.locator('#preset').selectOption(id);
    assert.equal(await page.evaluate(() => [...window.__p205Review.currentVrm.springBoneManager.joints].find(j => j.bone.name.startsWith('NemuTail')).settings.stiffness), stiffness);
  }
  const response = await page.request.get('http://127.0.0.1:5186/p205/heavy-comparison.mp4');
  assert.equal(response.status(), 200); assert.equal(response.headers()['content-type'], 'video/mp4');
  await page.evaluate(() => {
    const video = document.createElement('video'); video.id = 'decode-check'; video.muted = true; video.src = '/p205/heavy-comparison.mp4'; document.body.replaceChildren(video);
  });
  await page.waitForFunction(() => document.querySelector('video').readyState >= 2);
  const playback = await page.evaluate(async () => { const v = document.querySelector('video'); await v.play(); return { duration: v.duration, width: v.videoWidth, height: v.videoHeight }; });
  await page.waitForFunction(() => document.querySelector('video').currentTime > .15);
  await page.evaluate(() => document.querySelector('video').pause());
  assert.deepEqual(playback, { duration: 24, width: 1920, height: 1080 });
  assert.deepEqual(errors, []);
  await writeFile(resolve(output, 'verification.json'), JSON.stringify({ passed: true, probe, settings, tailTips, playback,
    checks: ['all 720 encoded frames decoded by ffprobe', 'three actual tail settings', 'different tail responses under identical motion',
      'live comparison play/pause/reset', 'single preview default calm and both new presets', 'served MP4 MIME', 'browser H264 playback'], errors,
  }, null, 2) + '\n');
  console.log('Heavy comparison and browser video playback passed.');
} finally { await browser.close(); }
