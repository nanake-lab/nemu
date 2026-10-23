import { test } from 'node:test';
import assert from 'node:assert/strict';
import { motionRange, rms, slope, spectrum } from './settling-metrics.mjs';
test('constant coordinates have zero residual spectrum without numerical DC leakage', () => {
  assert.deepEqual(spectrum(Array.from({ length: 600 }, () => [1.234, -.456, .03]), 120).dominantHz, null);
});
test('a known 18 Hz oscillation is retained above the reporting band', () => {
  const s = spectrum(Array.from({ length: 600 }, (_, i) => [.002 * Math.sin(2 * Math.PI * 18 * i / 120), 0, 0]), 120);
  assert.equal(s.dominantHz, 18); assert.ok(s.above15HzPowerFraction > .99);
  assert.ok(Math.abs(s.above15HzWindowedRms - .002 / Math.sqrt(2)) < 1e-9);
});
test('low-frequency oscillation, RMS and signed trend are measured', () => {
  const s = spectrum(Array.from({ length: 600 }, (_, i) => [.003 * Math.sin(2 * Math.PI * 2 * i / 120), 0, 0]), 120);
  assert.equal(s.dominantHz, 2); assert.ok(s.above15HzPowerFraction < 1e-6);
  assert.equal(slope([3, 2, 1]), -1); assert.equal(rms([3, -3]), 3);
});
test('geometric range preserves tiny angle and displacement instead of rounding them away', () => {
  const a = 1e-9;
  const range = motionRange([[0, 0, 0], [1e-12, 0, 0]], [[1, 0, 0], [Math.cos(a), Math.sin(a), 0]]);
  assert.equal(range.endpointPeakToPeakM, 1e-12); assert.ok(Math.abs(range.directionPeakToPeakDeg - a * 180 / Math.PI) < 1e-20);
});
