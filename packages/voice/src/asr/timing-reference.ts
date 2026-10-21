import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { resolve, relative, isAbsolute } from 'node:path';
import { z } from 'zod';
import plan from '../../../../reports/P4-01/alignment-preregistration.json' with { type: 'json' };
const sha = z.string().regex(/^[a-f0-9]{64}$/);
export const timingReferenceSchema = z
  .strictObject({
    id: z.string(),
    lang: z.enum(['zh', 'en', 'ja']),
    sourcePath: z.string(),
    sourceSha256: sha,
    transcriptSha256: sha,
    path: z.string(),
    sha256: sha,
    speechStartMs: z.number().nonnegative(),
    speechEndMs: z.number().positive(),
    annotation: z.literal('qwen3-forced-alignment-v1'),
  })
  .refine((row) => row.speechEndMs > row.speechStartMs, 'Non-positive speech interval');
export type TimingReference = z.infer<typeof timingReferenceSchema>;
export function validateTimingReferences(raw: unknown): TimingReference[] {
  const refs = z.array(timingReferenceSchema).length(60).parse(raw);
  if (new Set(refs.map((r) => r.id)).size !== 60) throw new Error('Duplicate timing IDs');
  for (const [i, ref] of refs.entries()) {
    const expected = plan.samples[i];
    if (
      !expected ||
      ref.id !== expected.id ||
      ref.lang !== expected.lang ||
      ref.sourcePath !== expected.path ||
      ref.path !== `alignment-input/${ref.id}.wav`
    )
      throw new Error('Timing samples/order differ from preregistration');
  }
  return refs;
}
const hash = (bytes: Buffer | string) => createHash('sha256').update(bytes).digest('hex');
function within(root: string, path: string): string {
  const target = resolve(root, path),
    rel = relative(resolve(root), target);
  if (isAbsolute(rel) || rel.startsWith('..'))
    throw new Error('Timing input escapes evaluation directory');
  return target;
}
export async function loadTimingReferences(root: string): Promise<TimingReference[]> {
  const bytes = await readFile(within(root, 'timing-references.json'));
  const refs = validateTimingReferences(JSON.parse(bytes.toString('utf8')));
  const meta = z
    .object({
      method: z.literal('P4-01-forced-alignment-v1'),
      tool: z.literal('qwen-asr'),
      toolVersion: z.literal('0.0.6'),
      modelRevision: z.literal('c7cbfc2048c462b0d63a45797104fc9db3ad62b7'),
      referenceResolutionMs: z.literal(80),
      count: z.literal(60),
      commit: z.string().regex(/^[a-f0-9]{40}$/),
      planSha256: sha,
      methodSha256: sha,
      manifestSha256: sha,
      uvLockSha256: sha,
      referencesSha256: sha,
      inputsSha256: sha,
    })
    .parse(JSON.parse(await readFile(within(root, 'timing-references.meta.json'), 'utf8')));
  const checks: Array<[string, Buffer]> = [
    [meta.referencesSha256, bytes],
    [meta.inputsSha256, await readFile(within(root, 'alignment-inputs.json'))],
    [
      meta.planSha256,
      await readFile(
        new URL('../../../../reports/P4-01/alignment-preregistration.json', import.meta.url),
      ),
    ],
    [
      meta.methodSha256,
      await readFile(
        new URL('../../../../reports/P4-01/ALIGNMENT-PREREGISTRATION.md', import.meta.url),
      ),
    ],
    [meta.manifestSha256, await readFile(new URL('./alignment/manifest.json', import.meta.url))],
    [meta.uvLockSha256, await readFile(new URL('./alignment/uv.lock', import.meta.url))],
  ];
  for (const [expected, content] of checks)
    if (hash(content) !== expected)
      throw new Error('Timing provenance hash mismatch; do not measure changed inputs/method');
  for (const ref of refs) {
    if (
      hash(await readFile(within(root, ref.path))) !== ref.sha256 ||
      hash(await readFile(within(root, ref.sourcePath))) !== ref.sourceSha256
    )
      throw new Error(`Timing audio changed: ${ref.id}`);
    const human = (await readFile(within(root, ref.sourcePath.replace(/\.wav$/, '.txt')), 'utf8'))
      .replace(/^\uFEFF/u, '')
      .trim();
    if (hash(human) !== ref.transcriptSha256)
      throw new Error(`Human transcript changed: ${ref.id}`);
  }
  return refs;
}
