import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { join, resolve, relative, isAbsolute } from 'node:path';
import { spawnSync } from 'node:child_process';
import { setTimeout as delay } from 'node:timers/promises';
import { z } from 'zod';
import { assertQuiet } from './quiet.js';
import { cacheDirectory, verifyModels } from './models.js';
import { SenseVoiceProvider } from './sensevoice.js';
const root =
  process.env.NEMU_ASR_DATA ?? join(process.env.LOCALAPPDATA ?? '', 'Nemu/asr-evaluation');
const referenceSchema = z.array(
  z.strictObject({
    id: z.string(),
    path: z.string(),
    sha256: z.string().regex(/^[a-f0-9]{64}$/),
    speechStartMs: z.number().nonnegative(),
    speechEndMs: z.number().positive(),
    annotation: z.literal('verified-acoustic-boundary'),
  }),
);
let references: z.infer<typeof referenceSchema>;
try {
  references = referenceSchema.parse(
    JSON.parse(await readFile(join(root, 'timing-references.json'), 'utf8')),
  );
} catch (e) {
  if ((e as NodeJS.ErrnoException).code !== 'ENOENT') throw e;
  await writeFile(
    'reports/P4-01/latency.json',
    JSON.stringify(
      {
        status: 'BLOCKED',
        reason:
          'No independently verified syllable onset/end annotations. Energy thresholds and the tested VAD itself are not ground truth.',
        localReference: 'timing-references.json',
        results: [],
      },
      null,
      2,
    ),
  );
  throw new Error('Timing reference annotations absent; not claiming latency acceptance', { cause: e });
}
assertQuiet();
await verifyModels();
const results = [];
const provider = new SenseVoiceProvider(cacheDirectory(), {
  onBargeIn: () => {
    if (bargeAt === undefined) bargeAt = performance.now();
  },
});
let bargeAt: number | undefined;
for (const ref of references) {
  assertQuiet();
  if (ref.speechEndMs <= ref.speechStartMs) throw new Error('Invalid acoustic interval');
  const path = resolve(root, ref.path);
  const withinRoot = relative(resolve(root), path);
  if (isAbsolute(withinRoot) || withinRoot.startsWith('..'))
    throw new Error('Timing audio path escapes local evaluation directory');
  if (
    createHash('sha256')
      .update(await readFile(path))
      .digest('hex') !== ref.sha256
  )
    throw new Error(`Audio hash mismatch ${ref.id}`);
  const audio = spawnSync(
    'ffmpeg',
    ['-v', 'error', '-i', path, '-ar', '16000', '-ac', '1', '-f', 's16le', '-'],
    { maxBuffer: 32 * 1024 * 1024 },
  );
  if (audio.status !== 0) throw new Error(`Decode failed ${ref.id}`);
  const pcm = new Int16Array(
    audio.stdout.buffer.slice(
      audio.stdout.byteOffset,
      audio.stdout.byteOffset + audio.stdout.byteLength,
    ),
  );
  const ends: number[] = [];
  bargeAt = undefined;
  provider.setPlayback(ref.id, true);
  const session = provider.start({ lang: 'auto', mode: 'continuous' });
  const start = performance.now();
  session.onVad((state) => {
    if (state === 'speechEnd') ends.push(performance.now());
  });
  for (let offset = 0; offset < pcm.length + 16000; offset += 320) {
    if (offset % 16000 === 0) assertQuiet();
    await delay(Math.max(0, start + (offset + 320) / 16 - performance.now()));
    const block = new Int16Array(320);
    block.set(pcm.subarray(offset, offset + 320));
    session.feed(block);
  }
  const endAt = ends.find((at) => at >= start + ref.speechEndMs);
  results.push({
    id: ref.id,
    sha256: ref.sha256,
    bargeInMs: bargeAt === undefined ? null : bargeAt - start - ref.speechStartMs,
    vadEndMs: endAt === undefined ? null : endAt - start - ref.speechEndMs,
  });
  session.close();
  provider.setPlayback(ref.id, false);
}
const vad = results.flatMap((r) => (r.vadEndMs === null ? [] : [r.vadEndMs])).sort((a, b) => a - b);
const median = vad.length
  ? ((vad[Math.floor((vad.length - 1) / 2)] ?? 0) + (vad[Math.floor(vad.length / 2)] ?? 0)) / 2
  : null;
const passed =
  results.length > 0 &&
  vad.length === results.length &&
  median !== null &&
  median <= 300 &&
  results.every((r) => r.bargeInMs !== null && r.bargeInMs >= 200 && r.bargeInMs <= 250);
await writeFile(
  'reports/P4-01/latency.json',
  JSON.stringify(
    { measuredAt: new Date().toISOString(), medianVadEndMs: median, passed, results },
    null,
    2,
  ),
);
if (!passed) process.exitCode = 1;
