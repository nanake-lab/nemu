import { describe, it, expect } from 'vitest';
import { computeLatencyStats } from './LatencyPanel.js';
import type { LatencyRecord } from './types.js';

describe('computeLatencyStats', () => {
  it('handles empty records cleanly', () => {
    const stats = computeLatencyStats([]);
    expect(stats.count).toBe(0);
    expect(stats.p50).toBe(0);
    expect(stats.p90).toBe(0);
  });

  it('computes accurate p50 and p90 for 10 records', () => {
    const records: LatencyRecord[] = [
      100, 200, 300, 400, 500, 600, 700, 800, 900, 1000,
    ].map((d, i) => ({
      utteranceKey: `utt-${i}`,
      ts: Date.now() + i * 1000,
      marks: { vadEnd: 0, audioStart: d },
      totalDurationMs: d,
    }));

    const stats = computeLatencyStats(records);
    expect(stats.count).toBe(10);
    expect(stats.min).toBe(100);
    expect(stats.max).toBe(1000);
    // p50 is index Math.floor(10 * 0.5) = 5 -> 600
    expect(stats.p50).toBe(600);
    // p90 is index Math.floor(10 * 0.9) = 9 -> 1000
    expect(stats.p90).toBe(1000);
    expect(stats.avg).toBe(550);
  });

  it('computes percentiles for exactly 50 records', () => {
    const records: LatencyRecord[] = [];
    for (let i = 1; i <= 50; i++) {
      records.push({
        utteranceKey: `utt-${i}`,
        ts: Date.now() + i,
        marks: { vadEnd: 0, firstChunk: i * 20 },
        totalDurationMs: i * 20,
      });
    }

    const stats = computeLatencyStats(records);
    expect(stats.count).toBe(50);
    expect(stats.min).toBe(20);
    expect(stats.max).toBe(1000);
    // 50 * 0.5 = 25 -> 26th element (index 25) = 26 * 20 = 520
    expect(stats.p50).toBe(520);
    // 50 * 0.9 = 45 -> 46th element (index 45) = 46 * 20 = 920
    expect(stats.p90).toBe(920);
  });
});
