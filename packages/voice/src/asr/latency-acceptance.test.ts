import { expect, it } from 'vitest';
import { assessLatency } from './latency-acceptance.js';

const rows = () =>
  Array.from({ length: 60 }, (_, index) => ({
    id: `sample-${index}`,
    vadEndMs: 350,
    bargeInMs: 300,
  }));

it('accepts inclusive ADR medians, reports early outliers and retains late tails', () => {
  const data = rows();
  data[0]!.bargeInMs = -5074;
  data[1]!.bargeInMs = 1300;
  expect(assessLatency(data)).toMatchObject({
    passed: true,
    medianVadEndMs: 350,
    medianBargeInMs: 300,
    earlyBargeInCount: 1,
    earlyBargeInRatio: 1 / 60,
  });
  expect(data[0]!.bargeInMs).toBe(-5074);
});

it('rejects either median above its target', () => {
  expect(assessLatency(rows().map((row) => ({ ...row, vadEndMs: 350.001 }))).passed).toBe(false);
  expect(assessLatency(rows().map((row) => ({ ...row, bargeInMs: 300.001 }))).passed).toBe(false);
});

it('does not pass missing, duplicate, invalid or premature end measurements', () => {
  expect(assessLatency(rows().slice(1)).passed).toBe(false);
  expect(assessLatency(rows().map((row) => ({ ...row, id: 'duplicate' }))).passed).toBe(false);
  expect(assessLatency(rows().map((row) => ({ ...row, bargeInMs: null }))).passed).toBe(false);
  const data = rows();
  data[0]!.vadEndMs = -1;
  expect(assessLatency(data).passed).toBe(false);
  data[0]!.vadEndMs = Infinity;
  expect(() => assessLatency(data)).toThrow('Nonfinite');
});
