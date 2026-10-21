import { describe, it, expect } from 'vitest';
import { replayPcm } from './replay.js';
import { validateTimingReferences } from './timing-reference.js';
import plan from '../../../../reports/P4-01/alignment-preregistration-v3.json' with { type: 'json' };
describe('Independent alignment measurement validity', () => {
  it('sends complete 20 ms packets at their deadlines, including tail silence', async () => {
    let clock = 0;
    const times: number[] = [];
    const result = await replayPcm(
      new Int16Array(320),
      (block) => {
        expect(block).toHaveLength(320);
        times.push(clock);
      },
      async () => {},
      () => clock,
      async (ms) => {
        clock += ms;
      },
    );
    expect(times).toHaveLength(51);
    expect(times[0]).toBe(20);
    expect(times.at(-1)).toBe(1020);
    expect(result.maximumLatenessMs).toBe(0);
  });
  it('rejects lagging realtime input rather than catching up in a burst', async () => {
    let clock = 0,
      frames = 0;
    await expect(
      replayPcm(
        new Int16Array(320),
        () => {
          frames++;
        },
        async () => {},
        () => clock,
        async (ms) => {
          clock += ms + 25;
        },
      ),
    ).rejects.toThrow('late');
    expect(frames).toBe(0);
  });
  it('blocks missing coverage, duplicate IDs and legacy self-asserted annotations', () => {
    const refs = plan.samples.map((r) => ({
      id: r.id,
      lang: r.lang,
      sourcePath: r.path,
      sourceSha256: 'a'.repeat(64),
      transcriptSha256: 'b'.repeat(64),
      path: `alignment-input-v3/${r.id}.wav`,
      sha256: 'c'.repeat(64),
      speechStartMs: 800,
      speechEndMs: 2000,
      annotation: 'qwen3-context-alignment-v3',
    }));
    expect(validateTimingReferences(refs)).toHaveLength(60);
    expect(() =>
      validateTimingReferences(
        refs.map((r) => ({ ...r, annotation: 'qwen3-forced-alignment-v1' })),
      ),
    ).toThrow();
    expect(() => validateTimingReferences(refs.slice(1))).toThrow();
    expect(() =>
      validateTimingReferences(refs.map((r, i) => (i === 1 ? { ...r, id: refs[0]?.id } : r))),
    ).toThrow('Duplicate');
    expect(() =>
      validateTimingReferences(
        refs.map((r) => ({ ...r, annotation: 'verified-acoustic-boundary' })),
      ),
    ).toThrow();
  });
});
