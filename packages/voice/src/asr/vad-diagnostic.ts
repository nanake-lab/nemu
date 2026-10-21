import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import sherpa from 'sherpa-onnx-node';
import { asrDataDirectory } from './cache.js';
import { cacheDirectory, verifyModels } from './models.js';
import { loadTimingReferences } from './timing-reference.js';
import { assertQuiet } from './quiet.js';
const configs = [
  { id: 'baseline', threshold: 0.5, speech: 0.2, silence: 0.2 },
  { id: 'window-accounting', threshold: 0.5, speech: 0.168, silence: 0.168 },
  { id: 'posterior-035', threshold: 0.35, speech: 0.168, silence: 0.168 },
  { id: 'posterior-020', threshold: 0.2, speech: 0.168, silence: 0.168 },
  { id: 'shorter-endpoint', threshold: 0.35, speech: 0.168, silence: 0.128 },
];
assertQuiet();
await verifyModels();
const root = asrDataDirectory(),
  refs = await loadTimingReferences(root);
const audio = new Map<string, Float32Array>();
for (const ref of refs) {
  const decoded = spawnSync('ffmpeg', [
    '-v',
    'error',
    '-i',
    join(root, ref.path),
    '-ar',
    '16000',
    '-ac',
    '1',
    '-f',
    'f32le',
    '-',
  ]);
  if (decoded.status !== 0) throw new Error(`Decode failed: ${ref.id}`);
  const bytes = decoded.stdout;
  audio.set(
    ref.id,
    new Float32Array(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)),
  );
}
const output = [];
for (const config of configs) {
  const rows = [];
  for (const ref of refs) {
    assertQuiet();
    const make = (barge: boolean) =>
      new sherpa.Vad(
        {
          sileroVad: {
            model: join(cacheDirectory(), 'silero_vad.onnx'),
            threshold: barge ? config.threshold : 0.5,
            minSilenceDuration: barge ? 0.032 : config.silence,
            minSpeechDuration: barge ? config.speech : 0.032,
            windowSize: 512,
            maxSpeechDuration: 20,
          },
          sampleRate: 16000,
          numThreads: 1,
          provider: 'cpu',
          debug: 0,
        },
        30,
      );
    const vad = make(false),
      barge = make(true),
      pcm = audio.get(ref.id);
    if (!pcm) throw new Error('Missing PCM');
    let detected = false,
      end: number | null = null,
      first: number | null = null;
    for (let offset = 0; offset < pcm.length + 16000; offset += 320) {
      const block = new Float32Array(320);
      block.set(pcm.subarray(offset, offset + 320));
      vad.acceptWaveform(block);
      barge.acceptWaveform(block);
      const now = (offset + 320) / 16,
        next = vad.isDetected();
      if (detected && !next) end = now;
      if (first === null && barge.isDetected()) first = now;
      detected = next;
      while (!vad.isEmpty()) vad.pop();
      while (!barge.isEmpty()) barge.pop();
    }
    rows.push({
      id: ref.id,
      lang: ref.lang,
      vadEndMs: end === null ? null : end - ref.speechEndMs,
      bargeInMs: first === null ? null : first - ref.speechStartMs,
    });
  }
  const ends = rows.flatMap((r) => (r.vadEndMs === null ? [] : [r.vadEndMs])).sort((a, b) => a - b);
  output.push({
    config,
    count: rows.length,
    medianVadEndMs: ((ends[29] ?? 0) + (ends[30] ?? 0)) / 2,
    bargeOver250: rows.filter((r) => r.bargeInMs !== null && r.bargeInMs > 250).length,
    prematureBarge: rows.filter((r) => r.bargeInMs !== null && r.bargeInMs < 0).length,
    prematureEnd: rows.filter((r) => r.vadEndMs !== null && r.vadEndMs < 0).length,
    maximumBargeMs: Math.max(...rows.flatMap((r) => (r.bargeInMs === null ? [] : [r.bargeInMs]))),
    rows,
  });
  console.log(`Diagnostic ${config.id} complete: 60/60`);
}
await writeFile(
  'reports/P4-01/vad-diagnostic-grid.json',
  JSON.stringify(
    {
      status: 'DIAGNOSTIC_ONLY_NOT_ACCEPTANCE',
      method:
        'Fixed global grid; sample clock without realtime delivery, ASR load or IPC; no tuning of references or sample selection',
      referenceSha256: JSON.parse(await readFile(join(root, 'timing-references.meta.json'), 'utf8'))
        .referencesSha256,
      configs: output,
    },
    null,
    2,
  ),
);
