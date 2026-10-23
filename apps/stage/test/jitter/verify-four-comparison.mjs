import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { gunzipSync } from 'node:zlib';
import { resolve } from 'node:path';
const { chromium } = createRequire(import.meta.resolve('@playwright/test'))('playwright');
const root = 'reports/P2-05/four-review';
const comparison = JSON.parse(await readFile(`${root}/comparison.json`, 'utf8'));
for (const group of comparison.groups) {
  const compressed = await readFile(group.compressedRawPath);
  assert.equal(createHash('sha256').update(compressed).digest('hex'), group.compressedSha256);
  assert.equal(createHash('sha256').update(gunzipSync(compressed)).digest('hex'), group.rawSha256);
}
const capture = JSON.parse(await readFile(`${root}/video/capture.json`, 'utf8'));
const bytes = await readFile(`${root}/video/calm-B-C-D_same-motion_24s_30fps.mp4`);
assert.equal(createHash('sha256').update(bytes).digest('hex'), capture.sha256);
const probe = JSON.parse(execFileSync('D:/Tools/ffmpeg/bin/ffprobe.exe', ['-v', 'error', '-count_frames',
  '-show_entries', 'stream=codec_name,width,height,avg_frame_rate,nb_read_frames:format=duration',
  '-of', 'json', resolve(`${root}/video/calm-B-C-D_same-motion_24s_30fps.mp4`)], { windowsHide: true, encoding: 'utf8' }));
assert.deepEqual(probe.streams, [{ codec_name: 'h264', width: 2560, height: 1080, avg_frame_rate: '30/1', nb_read_frames: '720' }]);
assert.equal(Number(probe.format.duration), 24);
assert.deepEqual(capture.errors, []);
process.env.TEMP = 'D:/Dev/cache/tmp'; process.env.TMP = process.env.TEMP;
const browser = await chromium.launch({ headless: true, args: ['--ignore-gpu-blocklist', '--use-gl=angle'] });
try {
  const page = await browser.newPage({ viewport: { width: 2560, height: 1080 } });
  const errors = []; page.on('pageerror', e => errors.push(e.message));
  await page.goto('http://127.0.0.1:5186/test/jitter/heavy-comparison.html?four=1');
  await page.waitForFunction(() => window.__HEAVY_READY__ === true);
  const rigs = await page.evaluate(() => window.__heavyCompare.rigs.map(({ app, candidate }) => ({
    id: candidate.id, displayedPeak: candidate.maximumHairFrameDegrees,
    joints: [...app.currentVrm.springBoneManager.joints].map(j => ({ name: j.bone.name,
      stiffness: j.settings.stiffness, dragForce: j.settings.dragForce, gravityPower: j.settings.gravityPower })),
  })));
  assert.deepEqual(rigs.map(r => r.id), comparison.groups.map(g => g.id));
  for (const [i, rig] of rigs.entries()) {
    assert.equal(rig.displayedPeak, comparison.groups[i].maximumHairFrameDegrees);
    for (const joint of rig.joints) {
      const part = joint.name.startsWith('NemuTail') ? 'tail' : /^J_Sec_Hair.*_(23|24)$/.test(joint.name) ? 'ear' : joint.name.includes('Hair') ? 'hair' : joint.name.includes('Skirt') ? 'skirt' : null;
      if (!part) continue;
      for (const key of ['stiffness', 'dragForce', 'gravityPower']) assert.equal(joint[key], comparison.groups[i].parameters[part][key]);
    }
  }
  await page.goto('http://127.0.0.1:5186/test/jitter/review.html');
  await page.waitForFunction(() => window.__P205_READY__ === true);
  assert.equal(await page.locator('#preset').inputValue(), 'calm');
  const response = await page.request.get('http://127.0.0.1:5186/p205/four-comparison.mp4');
  assert.equal(response.status(), 200); assert.equal(response.headers()['content-type'], 'video/mp4');
  await page.evaluate(() => {
    const video = document.createElement('video'); video.muted = true; video.src = '/p205/four-comparison.mp4'; document.body.replaceChildren(video);
  });
  await page.waitForFunction(() => document.querySelector('video').readyState >= 2);
  const playback = await page.evaluate(async () => { const v = document.querySelector('video'); await v.play(); return { duration: v.duration, width: v.videoWidth, height: v.videoHeight }; });
  await page.waitForFunction(() => document.querySelector('video').currentTime > .15);
  await page.evaluate(() => document.querySelector('video').pause());
  assert.deepEqual(playback, { duration: 24, width: 2560, height: 1080 });
  assert.deepEqual(errors, []);
  await writeFile(`${root}/verification.json`, JSON.stringify({ passed: true, probe, playback,
    checks: ['four compressed and raw hashes', 'all 720 encoded video frames decoded', 'four groups all live K/D/G verified',
      'diagnostic peaks displayed', 'B not preselected as final', 'served MP4 MIME and browser playback'], errors,
  }, null, 2) + '\n');
  console.log('Four-group evidence, live settings and video verified. No drag diagnostic rerun.');
} finally { await browser.close(); }
