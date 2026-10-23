/** Private comparison draft; never a scoring reference or latency annotation. */
import { readFile, writeFile } from 'node:fs/promises';
import { resolve, join, relative, isAbsolute } from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { z } from 'zod';
import { SenseVoiceProvider } from './sensevoice.js';
import { cacheDirectory, verifyModels } from './models.js';
import { assertQuiet } from './quiet.js';
import { units } from './scoring.js';
if (process.env.CI) throw new Error('Private candidate review is local-only');
const root = resolve(process.argv[2] ?? '');
const sha = z.string().regex(/^[a-f0-9]{64}$/);
const manifest = z
  .object({
    sourceSha256: sha,
    samples: z
      .array(
        z.object({
          id: z.string(),
          lang: z.enum(['zh', 'en', 'ja']),
          text: z.string(),
          candidatePath: z.string(),
          sha256: sha,
          notTimingReference: z.literal(true),
          coarseZeroDurationUnits: z.array(z.number().int()),
        }),
      )
      .length(60),
  })
  .parse(JSON.parse(await readFile(join(root, 'import-candidates.json'), 'utf8')));
if (new Set(manifest.samples.map((r) => r.id)).size !== 60)
  throw new Error('Duplicate candidate IDs');
assertQuiet();
await verifyModels();
const provider = new SenseVoiceProvider(cacheDirectory());
const rows = [];
for (const row of manifest.samples) {
  assertQuiet();
  const wav = resolve(root, row.candidatePath),
    rel = relative(root, wav);
  if (isAbsolute(rel) || rel.startsWith('..')) throw new Error('Candidate path escape');
  if (
    createHash('sha256')
      .update(await readFile(wav))
      .digest('hex') !== row.sha256
  )
    throw new Error('Candidate SHA-256 mismatch');
  const decoded = spawnSync(
    'ffmpeg',
    ['-v', 'error', '-i', wav, '-ar', '16000', '-ac', '1', '-f', 'f32le', '-'],
    { maxBuffer: 16 * 1024 * 1024 },
  );
  if (decoded.status !== 0) throw new Error('Candidate decode failed');
  const pcm = new Float32Array(
    decoded.stdout.buffer.slice(
      decoded.stdout.byteOffset,
      decoded.stdout.byteOffset + decoded.stdout.byteLength,
    ),
  );
  const result = await provider.recognize(pcm);
  rows.push({
    id: row.id,
    lang: row.lang,
    path: wav.replaceAll('\\', '/'),
    sha256: row.sha256,
    expectedPrompt: row.text,
    machineDraft: result.text,
    predictedLang: result.lang,
    promptMatchesDraft:
      JSON.stringify(units(row.text, row.lang)) === JSON.stringify(units(result.text, row.lang)),
    coarseZeroDurationUnits: row.coarseZeroDurationUnits,
    referenceEligible: false,
  });
  console.info(`Private candidate review ${rows.length}/60 complete`);
}
const stamp = new Date().toISOString().replace(/[:.]/g, '-');
const value = {
  kind: 'unverified-machine-draft-not-ground-truth',
  sourceSha256: manifest.sourceSha256,
  rows,
};
await writeFile(
  join(root, `candidate-review-${stamp}.json`),
  JSON.stringify(value, null, 2) + '\n',
  { flag: 'wx' },
);
const escape = (s: string) => s.replaceAll('|', '\\|').replaceAll('\n', ' ');
const heading =
  '# 逐句录音核对稿\n\n仅机器草稿：不能据此自动修改人工参考稿，不能作 CER/WER 或延迟参考。机器稿与提示不同可能是识别错误、重读或实际读法变化，需要结合原音频判断。粗定位零时长警告仍保留。\n\n';
await writeFile(
  join(root, `candidate-review-${stamp}.md`),
  heading +
    '| ID | 朗读提示 | 机器转写（未核实） | WAV |\n|---|---|---|---|\n' +
    rows
      .map(
        (r) =>
          `| ${r.id} | ${escape(r.expectedPrompt)} | ${escape(r.machineDraft)} | [试听](${r.path}) |`,
      )
      .join('\n') +
    '\n',
  { flag: 'wx' },
);
console.info(
  `Saved 60 private review rows; exact normalized prompt matches ${rows.filter((r) => r.promptMatchesDraft).length}/60 (not an acceptance result)`,
);
