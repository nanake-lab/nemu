import { chromium } from '@playwright/test';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import { resolve } from 'node:path';

const probe = process.argv.includes('--probe');
const times = probe ? [0.1, 1, 3] : [0.1, 90, 179];
const root = resolve(probe ? 'reports/P3-06/video-probe' : 'reports/P3-06/videos-final');
const manifest = JSON.parse(await readFile(resolve(root, 'manifest.json'), 'utf8'));
const browser = await chromium.launch({ headless: true });
const output = [];
await mkdir(resolve(root, 'frames'), { recursive: true });
try {
  for (const item of manifest.results) {
    const bytes = await readFile(resolve(root, item.file));
    assert.equal(createHash('sha256').update(bytes).digest('hex'), item.sha256);
    const page = await browser.newPage();
    const result = await page.evaluate(async ({ base64, times }) => {
      const bytes = Uint8Array.from(atob(base64), char => char.charCodeAt(0));
      const url = URL.createObjectURL(new Blob([bytes], { type: 'video/webm' }));
      const video = document.createElement('video'); document.body.append(video);
      const wait = (target, event) => Promise.race([
        new Promise(resolveEvent => target.addEventListener(event, resolveEvent, { once: true })),
        new Promise((_, reject) => setTimeout(() => reject(new Error(`Timed out: ${event}`)), 20000)),
      ]);
      const loaded = wait(video, 'loadeddata'); video.src = url; await loaded;
      const seek = async time => {
        const done = wait(video, 'seeked'); video.currentTime = time; await done;
      };
      // MediaRecorder WebM omits duration metadata; seek to EOF to measure decoded extent.
      if (!Number.isFinite(video.duration)) await seek(1e9);
      const duration = video.duration;
      if (!Number.isFinite(duration)) throw new Error('Cannot determine decoded video duration');
      const canvas = document.createElement('canvas');
      canvas.width = video.videoWidth; canvas.height = video.videoHeight;
      const context = canvas.getContext('2d');
      const frames = [];
      for (const time of times) {
        await seek(time); context.drawImage(video, 0, 0);
        frames.push({ time, png: canvas.toDataURL('image/png').split(',')[1] });
      }
      URL.revokeObjectURL(url);
      return { duration, width: canvas.width, height: canvas.height, frames };
    }, { base64: bytes.toString('base64'), times });
    assert(result.duration >= (probe ? 3 : 180), 'Decoded recording is shorter than required');
    const hashes = [];
    for (const frame of result.frames) {
      const png = Buffer.from(frame.png, 'base64');
      await writeFile(resolve(root, 'frames', `${item.group}-t${frame.time}.png`), png);
      hashes.push(createHash('sha256').update(png).digest('hex'));
    }
    assert(new Set(hashes).size > 1, 'Video must contain changing rendered frames');
    output.push({ file: item.file, duration: result.duration, width: result.width,
      height: result.height, frames: times, frameHashes: hashes });
    await page.close();
  }
  await writeFile(resolve(root, 'verification.json'), JSON.stringify(output, null, 2));
  console.log(JSON.stringify(output, null, 2));
} finally { await browser.close(); }
