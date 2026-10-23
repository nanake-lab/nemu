import { describe, it, expect } from 'vitest';
import { PcmFramer, workletSource } from '../../web/mic-worklet.js';
import { MicController } from '../../web/mic.js';
import { editDistance, units } from './scoring.js';
import { normalizeLanguage, CloudAsrProvider } from './provider.js';
describe('P4-01 capture and scoring', () => {
  it.each([16000, 44100, 48000])(
    'maintains 20 ms framing across worklet quanta at %i Hz',
    (rate) => {
      const framer = new PcmFramer(rate);
      const frames: Int16Array[] = [];
      for (let offset = 0; offset < rate; offset += 128)
        frames.push(...framer.push(new Float32Array(Math.min(128, rate - offset)).fill(0.5)));
      expect(frames).toHaveLength(50);
      expect(
        frames.every((f) => f.length === 320 && [...f].every((v) => Math.abs(v - 16384) <= 1)),
      ).toBe(true);
    },
  );
  it('clamps clipping with signed full scale', () => {
    const frames = new PcmFramer(16000).push(
      Float32Array.from({ length: 640 }, (_, i) => (i < 320 ? -2 : 2)),
    );
    expect(frames[0]?.[0]).toBe(-32768);
    expect(frames[1]?.[0]).toBe(32767);
  });
  it('emits valid executable worklet JS', () => {
    expect(workletSource()).toContain("registerProcessor('nemu-mic'");
    expect(() => new Function(workletSource())).not.toThrow();
  });
  it('stops late permission resolution after disable', async () => {
    let complete: (() => void) | undefined;
    let stopped = 0;
    let started: (() => void) | undefined;
    const didStart = new Promise<void>((r) => {
      started = r;
    });
    const mic = new MicController({ postMessage: () => {} }, async () => {
      await new Promise<void>((r) => {
        complete = r;
        started?.();
      });
      return {
        stop: async () => {
          stopped++;
        },
      };
    });
    const opening = mic.set(true);
    await didStart;
    const closing = mic.set(false);
    complete?.();
    await opening;
    await closing;
    expect(stopped).toBe(1);
  });
  it('counts substitutions deletions insertions and case/punctuation normalization', () => {
    expect(editDistance(['a', 'b', 'c'], ['a', 'x', 'c', 'd'])).toBe(2);
    expect(editDistance([], ['x'])).toBe(1);
    expect(units('Hello, WORLD!', 'en')).toEqual(['hello', 'world']);
    expect(units('今日は、晴れ。', 'ja')).toEqual([...'今日は晴れ']);
    expect(editDistance(units('今天晴天', 'zh'), units('今天雨天', 'zh'))).toBe(1);
  });
  it('marks unsupported language mappings and never silently uploads audio', () => {
    expect(normalizeLanguage('yue', '你好')).toEqual({ lang: 'zh', mapped: true });
    expect(normalizeLanguage('ko', '한국어')).toEqual({ lang: 'en', mapped: true });
    expect(normalizeLanguage('ja', '日本語')).toEqual({ lang: 'ja', mapped: false });
    expect(() => new CloudAsrProvider().start()).toThrow('not configured');
  });
});
