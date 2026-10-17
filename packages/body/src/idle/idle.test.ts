import { describe, it, expect } from 'vitest';
import {
  IdleScheduler,
  DEFAULT_IDLE_MOTIONS,
  hasConsecutiveSubsequenceRepeat,
} from './scheduler.js';
import type { IdleSetItem } from './types.js';

function createSeededPrng(seed = 123456789): () => number {
  let s = seed >>> 0;
  return () => {
    s = (s + 0x6d2b79f5) >>> 0;
    let t = Math.imul(s ^ (s >>> 15), 1 | s);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

describe('P3-03 Idle Scheduler', () => {
  describe('Subsequence Repeat Detection (hasConsecutiveSubsequenceRepeat)', () => {
    it('detects consecutive repeated subsequences of length 3', () => {
      const tandem3 = ['idle_stand', 'wave', 'nod', 'idle_stand', 'wave', 'nod'];
      expect(hasConsecutiveSubsequenceRepeat(tandem3, 3)).toBe(true);
    });

    it('detects consecutive repeated subsequences of length 4', () => {
      const tandem4 = ['a', 'b', 'c', 'd', 'a', 'b', 'c', 'd'];
      expect(hasConsecutiveSubsequenceRepeat(tandem4, 3)).toBe(true);
      expect(hasConsecutiveSubsequenceRepeat(tandem4, 4)).toBe(true);
    });

    it('returns false when no consecutive repeated subsequence exists', () => {
      const cleanSeq = ['a', 'b', 'c', 'd', 'a', 'b', 'e', 'c', 'd'];
      expect(hasConsecutiveSubsequenceRepeat(cleanSeq, 3)).toBe(false);
    });

    it('does not flag repeated elements of length < minLength', () => {
      const singleRepeat = ['a', 'a', 'b', 'b'];
      expect(hasConsecutiveSubsequenceRepeat(singleRepeat, 3)).toBe(false);
    });
  });

  describe('Cooldown and Consecutive Constraints', () => {
    it('strictly forbids triggering the same motion 3 times in a row', () => {
      const prng = createSeededPrng(42);
      const scheduler = new IdleScheduler({
        idleSet: DEFAULT_IDLE_MOTIONS,
        prng,
        baseIntervalRange: [0.1, 0.2],
      });

      for (let i = 0; i < 200; i++) {
        scheduler.update(0.3); // enough to pass cooldown and interval
      }

      const seq = scheduler.getHistory();
      for (let i = 2; i < seq.length; i++) {
        const consecutive3 = seq[i] === seq[i - 1] && seq[i - 1] === seq[i - 2];
        expect(consecutive3).toBe(false);
      }
    });

    it('respects motion cooldown intervals', () => {
      const customIdleSet: IdleSetItem[] = [
        { motionId: 'm1', weight: 1.0, cooldownS: 5.0 },
        { motionId: 'm2', weight: 1.0, cooldownS: 5.0 },
        { motionId: 'm3', weight: 1.0, cooldownS: 5.0 },
      ];

      const prng = createSeededPrng(99);
      const scheduler = new IdleScheduler({
        idleSet: customIdleSet,
        prng,
        baseIntervalRange: [1.0, 2.0],
      });

      const lastSeen = new Map<string, number>();
      let currentTime = 0;

      for (let step = 0; step < 500; step++) {
        const dt = 0.5;
        currentTime += dt;
        const triggered = scheduler.update(dt);
        if (triggered) {
          const prev = lastSeen.get(triggered);
          if (prev !== undefined) {
            const item = customIdleSet.find((it) => it.motionId === triggered)!;
            expect(currentTime - prev).toBeGreaterThanOrEqual(item.cooldownS - 1e-4);
          }
          lastSeen.set(triggered, currentTime);
        }
      }
    });

    it('modulates trigger frequency based on idleTempo', () => {
      // High tempo (1.5) triggers more actions than low tempo (0.5) over same duration
      const prngFast = createSeededPrng(777);
      const schedulerFast = new IdleScheduler({
        prng: prngFast,
        idleTempo: 1.5,
        baseIntervalRange: [4.0, 6.0],
      });

      const prngSlow = createSeededPrng(777);
      const schedulerSlow = new IdleScheduler({
        prng: prngSlow,
        idleTempo: 0.5,
        baseIntervalRange: [4.0, 6.0],
      });

      const durationSec = 120;
      const dt = 0.1;
      let countFast = 0;
      let countSlow = 0;

      for (let t = 0; t < durationSec; t += dt) {
        if (schedulerFast.update(dt)) countFast++;
        if (schedulerSlow.update(dt)) countSlow++;
      }

      expect(countFast).toBeGreaterThan(countSlow);
      // Fast tempo interval is ~3x smaller than slow tempo interval, so countFast should be ~3x countSlow
      expect(countFast / countSlow).toBeGreaterThan(2.0);
    });

    it('is completely deterministic when seeded with fixed PRNG', () => {
      const s1 = new IdleScheduler({ prng: createSeededPrng(12345) });
      const s2 = new IdleScheduler({ prng: createSeededPrng(12345) });

      for (let i = 0; i < 600; i++) {
        s1.update(0.1);
        s2.update(0.1);
      }

      expect(s1.getHistory()).toEqual(s2.getHistory());
    });
  });

  describe('10-Minute Idle Simulation (M6 / P3-03 Acceptance)', () => {
    it('simulates 600 seconds without any consecutive repeated subsequence of length >= 3', () => {
      const prng = createSeededPrng(20261016);
      const scheduler = new IdleScheduler({
        idleSet: DEFAULT_IDLE_MOTIONS,
        prng,
        baseIntervalRange: [5.0, 9.0],
        idleTempo: 1.0,
      });

      const totalDurationSec = 600; // 10 minutes
      const deltaSec = 1 / 60; // 60 FPS
      const totalFrames = Math.round(totalDurationSec / deltaSec);

      let triggeredCount = 0;
      for (let f = 0; f < totalFrames; f++) {
        const motion = scheduler.update(deltaSec);
        if (motion) {
          triggeredCount++;
        }
      }

      const history = scheduler.getHistory();
      expect(history.length).toBe(triggeredCount);
      expect(history.length).toBeGreaterThan(40); // Over 10 mins with 7s avg interval: ~60-80 actions

      // Acceptance criterion 1: No consecutive repeated subsequence of length >= 3
      const hasRepeat = hasConsecutiveSubsequenceRepeat(history, 3);
      expect(hasRepeat).toBe(false);

      // Acceptance criterion 2: No motion appears 3 times consecutively
      for (let i = 2; i < history.length; i++) {
        const triple = history[i] === history[i - 1] && history[i - 1] === history[i - 2];
        expect(triple).toBe(false);
      }
    });
  });
});
