import { assertQuiet } from './quiet.js';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { join, resolve, relative, isAbsolute } from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { z } from 'zod';
import { type Lang, langSchema } from '@nemu/contracts';
import { cacheDirectory, verifyModels } from './models.js';
import { SenseVoiceProvider } from './sensevoice.js';
import { editDistance, units } from './scoring.js';
const rowSchema = z.object({
  id: z.string(),
  path: z.string(),
  lang: langSchema,
  text: z.string(),
  sha256: z.string().regex(/^[a-f0-9]{64}$/),
});
const dataRoot =
  process.env.NEMU_ASR_DATA ?? join(process.env.LOCALAPPDATA ?? '', 'Nemu', 'asr-evaluation');
const report = resolve('reports/P4-01');
await mkdir(report, { recursive: true });
assertQuiet();
await verifyModels();
const publicRows = z
  .array(rowSchema)
  .parse(JSON.parse(await readFile(new URL('./public-data.lock.json', import.meta.url), 'utf8')));
let userRows: z.infer<typeof rowSchema>[] = [];
try {
  userRows = z
    .array(rowSchema)
    .parse(JSON.parse(await readFile(join(dataRoot, 'user.json'), 'utf8')));
} catch (e) {
  if ((e as NodeJS.ErrnoException).code !== 'ENOENT') throw e;
}
const ids = [...publicRows, ...userRows].map((row) => row.id);
if (new Set(ids).size !== ids.length) throw new Error('Duplicate evaluation IDs');
const provider = new SenseVoiceProvider(cacheDirectory());
const results: Array<{
  id: string;
  group: string;
  lang: Lang;
  sha256: string;
  predictedLang: Lang;
  edits: number;
  referenceUnits: number;
  hypothesis?: string;
}> = [];
for (const row of [...publicRows, ...userRows]) {
  const file = resolve(dataRoot, row.path);
  const rel = relative(resolve(dataRoot), file);
  if (isAbsolute(rel) || rel.startsWith('..'))
    throw new Error('Audio path must remain in local evaluation directory');
  const sha256 = createHash('sha256')
    .update(await readFile(file))
    .digest('hex');
  if (sha256 !== row.sha256) throw new Error(`Audio hash mismatch: ${row.id}`);
  const decoded = spawnSync(
    'ffmpeg',
    ['-v', 'error', '-i', file, '-ar', '16000', '-ac', '1', '-f', 'f32le', '-'],
    { maxBuffer: 32 * 1024 * 1024 },
  );
  if (decoded.status !== 0) throw new Error(`Audio decoding failed: ${row.id}`);
  const samples = new Float32Array(
    decoded.stdout.buffer.slice(
      decoded.stdout.byteOffset,
      decoded.stdout.byteOffset + decoded.stdout.byteLength,
    ),
  );
  const result = await provider.recognize(samples);
  const ref = units(row.text, row.lang),
    hyp = units(result.text, row.lang);
  const edits = editDistance(ref, hyp);
  results.push({
    id: row.id,
    group: row.id.startsWith('public-') ? 'public' : 'user',
    lang: row.lang,
    sha256,
    predictedLang: result.lang,
    edits,
    referenceUnits: ref.length,
    ...(row.id.startsWith('public-') ? { hypothesis: result.text } : {}),
  });
  console.info(`${row.id}: ${edits}/${ref.length}, language=${result.lang}`);
}
const languages = (['zh', 'en', 'ja'] as const).map((lang) => {
  const selected = results.filter((r) => r.lang === lang);
  const edits = selected.reduce((n, r) => n + r.edits, 0),
    total = selected.reduce((n, r) => n + r.referenceUnits, 0);
  return {
    lang,
    count: selected.length,
    publicCount: selected.filter((r) => r.group === 'public').length,
    userCount: selected.filter((r) => r.group === 'user').length,
    metric: lang === 'en' ? 'WER' : 'CER',
    edits,
    referenceUnits: total,
    errorRate: total ? edits / total : null,
    languageAccuracy: selected.filter((r) => r.lang === r.predictedLang).length / selected.length,
  };
});
const languageAccuracy = results.filter((r) => r.lang === r.predictedLang).length / results.length;
const complete = languages.every((r) => r.publicCount === 30 && r.userCount === 20);
const passed =
  complete &&
  languages.every((r) => r.errorRate !== null && r.errorRate <= 0.1) &&
  languageAccuracy >= 0.98;
await writeFile(
  join(report, 'evaluation.json'),
  JSON.stringify(
    {
      evaluatedAt: new Date().toISOString(),
      status: complete ? (passed ? 'PASS' : 'FAIL') : '待用户录音',
      normalization:
        'NFKC, lowercase, remove Unicode punctuation/symbols; zh/ja non-whitespace codepoints; en whitespace words; micro-average edits/reference units; no sample exclusion',
      languages,
      languageAccuracy,
      passed,
      results,
    },
    null,
    2,
  ) + '\n',
);
console.info(JSON.stringify({ languages, languageAccuracy, passed, complete }, null, 2));
if (complete && !passed) process.exitCode = 1;
