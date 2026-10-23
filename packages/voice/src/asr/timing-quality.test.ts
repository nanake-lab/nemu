import { describe, expect, it } from 'vitest';
import { assertReferenceQuality } from './timing-reference.js';
describe('known failed independent references', () => {
  it('blocks the frozen reference whose full QA failed', () => {
    expect(() =>
      assertReferenceQuality('d6889a9d9f2fd7d102e517d914da83f85e96648196579913f5d530a5dae03409'),
    ).toThrow('Timing reference QA failed');
  });
  it('does not confuse an unrelated digest with the rejected artifact', () => {
    expect(() => assertReferenceQuality('a'.repeat(64))).not.toThrow();
  });
});
