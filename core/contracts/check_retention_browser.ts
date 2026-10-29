// Execute the browser-safe Zod schema in Node; no browser or application is opened.
import { readFileSync } from 'node:fs';
import { wireInputSchema } from '../../packages/contracts/src/generated/echo/browser.js';
import { fromJson } from '../../packages/contracts/src/generated/echo/bridge.js';
import type { MessageMap } from '../../packages/contracts/src/generated/echo/schema.js';

type Vector = { request: { name: string; op: string; value?: Record<string, unknown> } };
const names = new Set(['EvidenceAllocationV3', 'TendencyV3', 'EvidenceGroupV3', 'RSnapshotV3']);
let count = 0;
for (const [file, accepted] of [['legal', true], ['illegal', false]] as const) {
  const vectors = JSON.parse(readFileSync(`core/contracts/golden/${file}.json`, 'utf8')) as Vector[];
  for (const { request: q } of vectors) {
    if (!names.has(q.name) || q.op !== 'encode' || !q.value) continue;
    const input = fromJson(q.name, q.value);
    const schema = wireInputSchema(q.name as keyof MessageMap);
    if (schema.safeParse(input).success !== accepted)
      throw new Error(`Browser schema disagrees: ${file} ${q.name}`);
    count += 1;
    if (accepted && q.name === 'EvidenceAllocationV3') {
      for (const actual_retention of [0, -0, -1, 1.0000000000000002, NaN, Infinity, -Infinity]) {
        if (schema.safeParse({ ...input, actual_retention }).success)
          throw new Error('Browser schema accepted invalid q');
        count += 1;
      }
    }
    if (accepted && q.name === 'TendencyV3') {
      for (const evidence_absolute of [-1, NaN, Infinity, -Infinity]) {
        if (schema.safeParse({ ...input, evidence_absolute }).success)
          throw new Error('Browser schema accepted invalid V');
        count += 1;
      }
    }
  }
}
if (count === 0) throw new Error('No V3 browser checks ran');
console.log(`P5-34 browser-safe schema: ${count} checks PASS`);
