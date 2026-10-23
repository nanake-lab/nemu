import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { resolve, relative, isAbsolute } from 'node:path';
import { fileURLToPath } from 'node:url';
import { z } from 'zod';
import plan from '../../../../reports/P4-01/alignment-preregistration-v4.json' with { type: 'json' };
import registration from '../../../../reports/P4-01/hybrid-reference-registration-v2.json' with { type: 'json' };
import { timingReferenceSchema } from './timing-reference.js';

const digest = z.string().regex(/^[a-f0-9]{64}$/);
const registry = z
  .strictObject({
    method: z.literal('P4-01-hybrid-audible-reference-v2'),
    count: z.literal(60),
    referencesPath: z.literal('timing-references-hybrid-v2.json'),
    referencesSha256: digest,
    annotationsPath: z.string(),
    annotationsSha256: digest,
    inputsPath: z.literal('alignment-inputs-v3.json'),
    inputsSha256: digest,
    manualIds: z.array(z.string()).length(22),
    automaticCount: z.literal(38),
    manualCount: z.literal(22),
    originalAuditPassed: z.literal(false),
    auditResolution: z.string().min(1),
    sourceFiles: z.array(z.strictObject({ path: z.string(), sha256: digest })).length(5),
    referenceUncertainty: z.string().min(1),
  })
  .parse(registration);
const repo = fileURLToPath(new URL('../../../../', import.meta.url));
const sha = (bytes: Buffer | string) => createHash('sha256').update(bytes).digest('hex');
export function withinReferenceRoot(root: string, path: string): string {
  const target = resolve(root, path),
    rel = relative(resolve(root), target);
  if (isAbsolute(rel) || rel === '..' || rel.startsWith('../') || rel.startsWith('..\\'))
    throw new Error('Reference path escapes evaluation root');
  return target;
}
const schema = z
  .strictObject({
    ...timingReferenceSchema.shape,
    annotation: z.enum(['qwen3-stable-primary-v1', 'human-audible-v1']),
  })
  .refine((r) => r.speechEndMs > r.speechStartMs, 'Non-positive speech interval');
export type HybridReference = z.infer<typeof schema>;
export function validateHybridReferences(raw: unknown): HybridReference[] {
  const rows = z.array(schema).length(60).parse(raw);
  const manual = new Set(registry.manualIds);
  if (manual.size !== 22 || new Set(rows.map((r) => r.id)).size !== 60)
    throw new Error('Duplicate reference IDs or invalid source allocation');
  for (const [i, row] of rows.entries()) {
    const expected = plan.samples[i];
    if (
      !expected ||
      row.id !== expected.id ||
      row.lang !== expected.lang ||
      row.sourcePath !== expected.path ||
      row.path !== `alignment-input-v3/${row.id}.wav`
    )
      throw new Error('Hybrid sample/order/path differs from frozen inputs');
    if (row.annotation !== (manual.has(row.id) ? 'human-audible-v1' : 'qwen3-stable-primary-v1'))
      throw new Error('Hybrid reference source allocation changed');
  }
  if (rows.filter((r) => r.annotation === 'human-audible-v1').length !== 22)
    throw new Error('Expected 22 human and 38 automatic references');
  return rows;
}
export async function loadHybridReferences(root: string) {
  const bytes = await readFile(withinReferenceRoot(root, registry.referencesPath));
  if (sha(bytes) !== registry.referencesSha256)
    throw new Error('Hybrid reference SHA-256 mismatch');
  for (const [path, hash] of [
    [registry.annotationsPath, registry.annotationsSha256],
    [registry.inputsPath, registry.inputsSha256],
  ] as const)
    if (sha(await readFile(withinReferenceRoot(root, path))) !== hash)
      throw new Error('Hybrid annotation/input provenance changed');
  for (const source of registry.sourceFiles)
    if (sha(await readFile(withinReferenceRoot(repo, source.path))) !== source.sha256)
      throw new Error('Hybrid preregistration/evidence changed');
  const references = validateHybridReferences(JSON.parse(bytes.toString('utf8')));
  for (const ref of references) {
    if (
      sha(await readFile(withinReferenceRoot(root, ref.path))) !== ref.sha256 ||
      sha(await readFile(withinReferenceRoot(root, ref.sourcePath))) !== ref.sourceSha256
    )
      throw new Error(`Hybrid audio changed: ${ref.id}`);
    const human = (
      await readFile(withinReferenceRoot(root, ref.sourcePath.replace(/\.wav$/, '.txt')), 'utf8')
    )
      .replace(/^\uFEFF/u, '')
      .trim();
    if (sha(human) !== ref.transcriptSha256)
      throw new Error(`Hybrid registered text changed: ${ref.id}`);
  }
  return {
    references,
    method: registry.method,
    referenceSha256: registry.referencesSha256,
    referenceUncertainty: registry.referenceUncertainty,
    automaticCount: 38,
    humanCount: 22,
  };
}
