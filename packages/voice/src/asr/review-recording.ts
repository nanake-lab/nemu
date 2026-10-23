/** Local review draft only: never an evaluation transcript or timing reference. */
import { spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { assertQuiet } from './quiet.js';
import { cacheDirectory, verifyModels } from './models.js';
import { SenseVoiceProvider } from './sensevoice.js';
if (process.env.CI) throw new Error('Private recording review is local-only');
const archive = resolve(process.argv[2] ?? '');
const source = join(archive, 'original.m4a');
const sourceSha256 = createHash('sha256')
  .update(await readFile(source))
  .digest('hex');
if (sourceSha256 !== '58a8fe9653a2cdac58475a53aeea3e2b132594e641b94efc3e31178e033d87f0')
  throw new Error('Unexpected source recording');
assertQuiet();
await verifyModels();
const decoded = spawnSync(
  'ffmpeg',
  ['-v', 'error', '-i', source, '-ar', '16000', '-ac', '1', '-f', 'f32le', '-'],
  { maxBuffer: 32 * 1024 * 1024 },
);
if (decoded.status !== 0) throw new Error('Review audio decoding failed');
const pcm = new Float32Array(
  decoded.stdout.buffer.slice(
    decoded.stdout.byteOffset,
    decoded.stdout.byteOffset + decoded.stdout.byteLength,
  ),
);
const provider = new SenseVoiceProvider(cacheDirectory());
const rows: Array<{
  windowStartSeconds: number;
  windowEndSeconds: number;
  language: string;
  text: string;
}> = [];
for (let start = 0; start < pcm.length; start += 22 * 16000) {
  assertQuiet();
  const end = Math.min(start + 24 * 16000, pcm.length);
  const result = await provider.recognize(pcm.subarray(start, end));
  rows.push({
    windowStartSeconds: start / 16000,
    windowEndSeconds: end / 16000,
    language: result.lang,
    text: result.text,
  });
  console.info(`Review draft window ${rows.length} complete`);
}
const stamp = new Date().toISOString().replace(/[:.]/g, '-');
await writeFile(
  join(archive, `review-draft-${stamp}.json`),
  JSON.stringify(
    {
      kind: 'unverified-asr-draft-not-ground-truth',
      sourceSha256,
      tool: 'sherpa-onnx-node 1.13.8 / SenseVoice int8',
      warning:
        'Fixed file windows only; never use this text or these window offsets as evaluation references. Overlapping windows duplicate boundary content.',
      rows,
    },
    null,
    2,
  ) + '\n',
  { flag: 'wx' },
);
await writeFile(
  join(archive, `review-draft-${stamp}.md`),
  '# 用户核对用机器转写草稿\n\n未经人工核对；不作为错误率参考稿或延迟时标。24 秒固定窗口、相邻重叠 2 秒；窗口交界重复不能当成实际重读。\n\n' +
    rows
      .map(
        (r) =>
          `## 文件窗口 ${r.windowStartSeconds.toFixed(1)}–${r.windowEndSeconds.toFixed(1)} 秒 (${r.language})\n\n${r.text}\n`,
      )
      .join('\n'),
  { flag: 'wx' },
);
console.info('Private review draft saved outside Git; no reference or user registration changed');
