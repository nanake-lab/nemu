/** Reassess frozen evidence under ADR 0017 without loading models or replaying audio. */
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { z } from 'zod';
import { assessLatency } from './latency-acceptance.js';

const source = 'reports/P4-01/latency-attempt-2026-10-22T09-39-20-062Z.json';
const bytes = await readFile(source);
const report = z
  .object({
    status: z.literal('FAIL'),
    referenceSha256: z.literal('f039ae6a3faa10c25f4cac6740b79e5931f83364cf1dfe03575b2c0178108f8e'),
    results: z
      .array(
        z.object({
          id: z.string().min(1),
          vadEndMs: z.number().finite().nullable(),
          bargeInMs: z.number().finite().nullable(),
        }),
      )
      .length(60),
  })
  .parse(JSON.parse(bytes.toString('utf8')) as unknown);
const assessment = assessLatency(report.results);
const output =
  JSON.stringify(
    {
      reassessedOn: '2026-10-23',
      kind: 'acceptance reassessment only; no new measurement or inference',
      source,
      sourceSha256: createHash('sha256').update(bytes).digest('hex'),
      sourceStatusPreserved: report.status,
      referenceSha256: report.referenceSha256,
      ...assessment,
      status: assessment.passed ? 'PASS' : 'FAIL',
    },
    null,
    2,
  ) + '\n';
const destination = 'reports/P4-01/latency-acceptance-adr0017.json';
let existing: string | undefined;
try {
  existing = await readFile(destination, 'utf8');
} catch (error) {
  if (!(error instanceof Error && 'code' in error && error.code === 'ENOENT')) throw error;
}
if (existing !== undefined && existing !== output) throw new Error('Frozen reassessment differs');
if (existing === undefined) await writeFile(destination, output, { flag: 'wx' });
console.info(output);
if (!assessment.passed) process.exitCode = 1;
