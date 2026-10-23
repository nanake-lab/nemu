import { expect, it } from 'vitest';
import { sampleStats } from './measurement-stats.js';
it('retains early values and interpolates without changing the input', () => {
  const values = [100, -10, 0, 10];
  expect(sampleStats(values)).toMatchObject({ count: 4, min: -10, median: 5, max: 100 });
  expect(sampleStats(values).p95).toBeCloseTo(86.5, 10);
  expect(values).toEqual([100, -10, 0, 10]);
});
it('does not turn missing or nonfinite measurements into zero', () => {
  expect(sampleStats([])).toMatchObject({ count: 0, median: null, p95: null });
  expect(() => sampleStats([NaN])).toThrow();
  expect(() => sampleStats([Infinity])).toThrow();
});
