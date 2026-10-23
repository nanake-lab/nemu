import { describe, expect, it } from 'vitest';
import { tmpdir } from 'node:os';
import plan from '../../../../reports/P4-01/alignment-preregistration-v4.json' with { type: 'json' };
import registration from '../../../../reports/P4-01/hybrid-reference-registration-v2.json' with { type: 'json' };
import { validateHybridReferences, withinReferenceRoot } from './hybrid-reference.js';

const fixture = () =>
  plan.samples.map((sample) => ({
    id: sample.id,
    lang: sample.lang,
    sourcePath: sample.path,
    sourceSha256: 'a'.repeat(64),
    transcriptSha256: 'b'.repeat(64),
    path: `alignment-input-v3/${sample.id}.wav`,
    sha256: 'c'.repeat(64),
    speechStartMs: 100,
    speechEndMs: 200,
    annotation: registration.manualIds.includes(sample.id)
      ? 'human-audible-v1'
      : 'qwen3-stable-primary-v1',
  }));
describe('frozen hybrid measurement references', () => {
  it('requires all sixty with the exact 38/22 sources including the confirmed correction', () => {
    const rows = validateHybridReferences(fixture());
    expect(rows.filter((r) => r.annotation === 'human-audible-v1')).toHaveLength(22);
    expect(rows.find((r) => r.id === 'user-ja-013')?.annotation).toBe('human-audible-v1');
  });
  it('rejects silent source changes and legacy annotations', () => {
    for (const annotation of ['qwen3-stable-primary-v1', 'qwen3-segmented-context-v4']) {
      const rows = fixture();
      rows.find((r) => r.id === 'user-ja-013')!.annotation = annotation;
      expect(() => validateHybridReferences(rows)).toThrow();
    }
  });
  it('rejects missing, reordered, duplicated or escaped samples', () => {
    const missing = fixture();
    missing.pop();
    expect(() => validateHybridReferences(missing)).toThrow();
    const swapped = fixture();
    [swapped[0], swapped[1]] = [swapped[1]!, swapped[0]!];
    expect(() => validateHybridReferences(swapped)).toThrow();
    const duplicate = fixture();
    duplicate[1] = duplicate[0]!;
    expect(() => validateHybridReferences(duplicate)).toThrow();
    const escaped = fixture();
    escaped[0]!.path = '../outside.wav';
    expect(() => validateHybridReferences(escaped)).toThrow();
  });
  it('rejects nonfinite or reversed boundaries without clamping', () => {
    for (const [start, end] of [
      [NaN, 200],
      [0, Infinity],
      [300, 200],
      [-1, 200],
    ]) {
      const rows = fixture();
      rows[0]!.speechStartMs = start!;
      rows[0]!.speechEndMs = end!;
      expect(() => validateHybridReferences(rows)).toThrow();
    }
  });
  it('guards paths at the filesystem boundary', () => {
    expect(() => withinReferenceRoot(tmpdir(), '../escaped')).toThrow();
    expect(withinReferenceRoot(tmpdir(), 'synthetic.wav')).toContain('synthetic.wav');
  });
});
