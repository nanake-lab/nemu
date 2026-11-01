// P6-17: browser-safe boundary validation must agree with the wire codecs.
import { readFileSync } from 'node:fs';
import { wireInputSchema } from '../../packages/contracts/src/generated/echo/browser.js';

type Vector = { request: { name: string; op: string; value?: Record<string, unknown> } };
const schema = wireInputSchema('TouchObservationV2');
let count = 0;
for (const [file, accepted] of [
  ['legal', true],
  ['illegal', false],
] as const) {
  const vectors = JSON.parse(
    readFileSync(`core/contracts/golden/${file}.json`, 'utf8'),
  ) as Vector[];
  for (const { request: q } of vectors) {
    if (q.name !== 'TouchObservationV2' || q.op !== 'encode' || !q.value) continue;
    if (schema.safeParse(q.value).success !== accepted)
      throw new Error(`Browser touch schema disagrees: ${file} ${JSON.stringify(q.value)}`);
    count += 1;
  }
}
// JSON cannot represent nonfinite values, so exercise them at the native boundary.
for (const elapsed_ms of [NaN, Infinity, -Infinity]) {
  if (
    schema.safeParse({
      schema_version: 1,
      region: 'head',
      kind: 8,
      contact_id: 'contact-A',
      elapsed_ms,
    }).success
  )
    throw new Error('Browser touch schema accepted nonfinite elapsed_ms');
  count += 1;
}
if (count === 3) throw new Error('No frozen touch browser checks ran');
console.log(`P6-17 browser-safe schema: ${count} checks PASS`);
