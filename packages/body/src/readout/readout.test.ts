import { describe, it, expect, vi } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { resolve, join } from 'node:path';
import type { BodyReadout } from '@nemu/contracts';
import {
  ReadoutApplicator,
  sanitizeBodyReadout,
  areReadoutsEqual,
} from './applicator.js';
import {
  NEUTRAL_BODY_READOUT,
  READOUT_BOUNDS,
  DEFAULT_SMOOTHING_DURATION_SEC,
} from './constants.js';

describe('P3-03 Readout Applicator', () => {
  describe('Boundary Clamping (sanitizeBodyReadout)', () => {
    it('clamps values that exceed maximum bounds', () => {
      const overBounds: Partial<BodyReadout> = {
        posture: 3.5,
        idleTempo: 5.0,
        earPose: 2.0,
        tailAmplitude: 1.8,
        gazeSociability: 2.5,
        expressionBaseline: {
          happy: 0.9,
          relaxed: 1.2,
        },
      };

      const sanitized = sanitizeBodyReadout(overBounds);

      expect(sanitized.posture).toBe(READOUT_BOUNDS.posture.max); // 1.0
      expect(sanitized.idleTempo).toBe(READOUT_BOUNDS.idleTempo.max); // 1.5
      expect(sanitized.earPose).toBe(READOUT_BOUNDS.earPose.max); // 1.0
      expect(sanitized.tailAmplitude).toBe(READOUT_BOUNDS.tailAmplitude.max); // 1.0
      expect(sanitized.gazeSociability).toBe(READOUT_BOUNDS.gazeSociability.max); // 1.0
      expect(sanitized.expressionBaseline.happy).toBe(READOUT_BOUNDS.expressionBaseline.max); // 0.4
      expect(sanitized.expressionBaseline.relaxed).toBe(READOUT_BOUNDS.expressionBaseline.max); // 0.4
    });

    it('clamps values that fall below minimum bounds', () => {
      const underBounds: Partial<BodyReadout> = {
        posture: -4.0,
        idleTempo: 0.1,
        earPose: -3.0,
        tailAmplitude: -0.5,
        gazeSociability: -1.0,
        expressionBaseline: {
          sad: -0.5,
        },
      };

      const sanitized = sanitizeBodyReadout(underBounds);

      expect(sanitized.posture).toBe(READOUT_BOUNDS.posture.min); // -1.0
      expect(sanitized.idleTempo).toBe(READOUT_BOUNDS.idleTempo.min); // 0.5
      expect(sanitized.earPose).toBe(READOUT_BOUNDS.earPose.min); // -1.0
      expect(sanitized.tailAmplitude).toBe(READOUT_BOUNDS.tailAmplitude.min); // 0.0
      expect(sanitized.gazeSociability).toBe(READOUT_BOUNDS.gazeSociability.min); // 0.0
      expect(sanitized.expressionBaseline.sad).toBeUndefined(); // <= 0.001 is omitted
    });

    it('handles NaN and undefined inputs safely by substituting neutral values', () => {
      const nanInput: Partial<BodyReadout> = {
        posture: Number.NaN,
        idleTempo: Number.NaN,
        earPose: Number.NaN,
        tailAmplitude: Number.NaN,
        gazeSociability: Number.NaN,
      };

      const sanitized = sanitizeBodyReadout(nanInput);

      expect(sanitized.posture).toBe(READOUT_BOUNDS.posture.min);
      expect(sanitized.idleTempo).toBe(READOUT_BOUNDS.idleTempo.min);
      expect(sanitized.earPose).toBe(READOUT_BOUNDS.earPose.min);
      expect(sanitized.tailAmplitude).toBe(READOUT_BOUNDS.tailAmplitude.min);
      expect(sanitized.gazeSociability).toBe(READOUT_BOUNDS.gazeSociability.min);
    });
  });

  describe('Neutral Readout & Fallback', () => {
    it('produces neutral posture and neutral parameters for null / undefined input', () => {
      const sanitized = sanitizeBodyReadout(null);

      expect(sanitized.posture).toBe(0.0);
      expect(sanitized.idleTempo).toBe(1.0);
      expect(sanitized.earPose).toBe(0.0);
      expect(sanitized.tailAmplitude).toBe(0.5);
      expect(sanitized.gazeSociability).toBe(0.5);
      expect(Object.keys(sanitized.expressionBaseline)).toHaveLength(0);
      expect(sanitized).toEqual(NEUTRAL_BODY_READOUT);
    });

    it('resets to neutral readout when core is disconnected or quarantined', () => {
      const applicator = new ReadoutApplicator({ smoothingDurationSec: 0 });

      // Apply modulated readout
      applicator.applyReadout({
        version: '1.0',
        posture: 0.8,
        idleTempo: 1.4,
        earPose: -0.6,
        tailAmplitude: 0.9,
        gazeSociability: 0.8,
        expressionBaseline: { happy: 0.35 },
      });

      expect(applicator.getCurrentReadout().posture).toBe(0.8);

      // Now core status becomes CORE_QUARANTINED
      applicator.applyReadout(null, 'CORE_QUARANTINED', true);
      const readoutQuarantined = applicator.getCurrentReadout();
      expect(readoutQuarantined.posture).toBe(0.0);
      expect(readoutQuarantined.idleTempo).toBe(1.0);
      expect(readoutQuarantined.earPose).toBe(0.0);
      expect(readoutQuarantined.tailAmplitude).toBe(0.5);
      expect(readoutQuarantined.gazeSociability).toBe(0.5);

      // Core status becomes CORE_UNMODULATED
      applicator.applyReadout(null, 'CORE_UNMODULATED', true);
      expect(applicator.getCurrentReadout().posture).toBe(0.0);

      // Core status becomes DISCONNECTED
      applicator.applyReadout(null, 'DISCONNECTED', true);
      expect(applicator.getCurrentReadout().posture).toBe(0.0);
    });
  });

  describe('Idempotence (Rule 4 & Spec Requirement)', () => {
    it('applying the same readout twice produces identical results without accumulation or reset', () => {
      const applicator = new ReadoutApplicator({ smoothingDurationSec: 2.0 });

      const testReadout: BodyReadout = {
        version: '1.0',
        posture: 0.6,
        idleTempo: 1.2,
        earPose: 0.5,
        tailAmplitude: 0.7,
        gazeSociability: 0.8,
        expressionBaseline: { relaxed: 0.25 },
      };

      // First application: starts transition
      const changed1 = applicator.applyReadout(testReadout);
      expect(changed1).toBe(true);
      expect(applicator.isInterpolating()).toBe(true);

      // Advance by 1 second (halfway through 2s transition)
      applicator.update(1.0);
      const midReadout = applicator.getCurrentReadout();
      expect(midReadout.posture).toBeGreaterThan(0.0);
      expect(midReadout.posture).toBeLessThan(0.6);

      // Second application with identical readout: must be idempotent no-op
      const changed2 = applicator.applyReadout(testReadout);
      expect(changed2).toBe(false);

      // Smoothing continues seamlessly towards original target without resetting to start
      applicator.update(1.0);
      expect(applicator.isInterpolating()).toBe(false);
      expect(areReadoutsEqual(applicator.getCurrentReadout(), testReadout)).toBe(true);
    });
  });

  describe('Smoothing Transition (2.0s Animation Curve)', () => {
    it('smooths transitions over default 2 seconds', () => {
      const onReadoutUpdated = vi.fn();
      const onIdleTempoUpdated = vi.fn();

      const applicator = new ReadoutApplicator({
        smoothingDurationSec: DEFAULT_SMOOTHING_DURATION_SEC,
        onReadoutUpdated,
        onIdleTempoUpdated,
      });

      const target: BodyReadout = {
        version: '1.0',
        posture: 1.0,
        idleTempo: 1.5,
        earPose: 1.0,
        tailAmplitude: 1.0,
        gazeSociability: 1.0,
        expressionBaseline: { happy: 0.4 },
      };

      applicator.applyReadout(target);

      // t = 0
      expect(applicator.getCurrentReadout().posture).toBe(0.0);

      // t = 1.0s (50% elapsed, smoothstep(0.5) = 0.5)
      applicator.update(1.0);
      const half = applicator.getCurrentReadout();
      expect(half.posture).toBeCloseTo(0.5, 2);
      expect(half.idleTempo).toBeCloseTo(1.25, 2); // 1.0 + 0.5 * 0.5
      expect(half.expressionBaseline.happy).toBeCloseTo(0.2, 2);

      // t = 2.0s (100% elapsed)
      applicator.update(1.0);
      const final = applicator.getCurrentReadout();
      expect(final.posture).toBeCloseTo(1.0, 4);
      expect(final.idleTempo).toBeCloseTo(1.5, 4);
      expect(final.expressionBaseline.happy).toBeCloseTo(0.4, 4);
      expect(applicator.isInterpolating()).toBe(false);

      expect(onReadoutUpdated).toHaveBeenCalled();
      expect(onIdleTempoUpdated).toHaveBeenCalled();
    });
  });

  describe('Structural Check (No persistent mood/favorability/intimacy in client code)', () => {
    it('verifies that terminal, body, and stage packages do not store hidden mood/favorability variables', () => {
      // Golden Rule 4 & Invariant I18: No hidden persistent state in terminal / client code.
      const searchDirs = [
        resolve(__dirname, '../../src'),
        resolve(__dirname, '../../../avatar-runtime/src'),
        resolve(__dirname, '../../../../apps/stage/src'),
      ];

      const forbiddenPersistentKeywords = [
        'localStorage.setItem("favorability"',
        'localStorage.setItem("affection"',
        'localStorage.setItem("intimacy"',
        'localStorage.setItem("mood"',
        'sessionStorage.setItem("favorability"',
        'sessionStorage.setItem("mood"',
      ];

      function scanDir(dir: string): void {
        const entries = readdirSync(dir);
        for (const entry of entries) {
          const fullPath = join(dir, entry);
          const stat = statSync(fullPath);
          if (stat.isDirectory()) {
            scanDir(fullPath);
          } else if (entry.endsWith('.ts') && !entry.endsWith('.test.ts')) {
            const content = readFileSync(fullPath, 'utf-8');
            for (const kw of forbiddenPersistentKeywords) {
              expect(content.includes(kw)).toBe(false);
            }
          }
        }
      }

      for (const dir of searchDirs) {
        scanDir(dir);
      }
    });
  });
});
