import type { BodyReadout, Emotion } from '@nemu/contracts';
import type {
  CoreConnectionStatus,
  ReadoutApplicatorOptions,
  ExpressionBaselineMap,
} from './types.js';
import {
  NEUTRAL_BODY_READOUT,
  READOUT_BOUNDS,
  DEFAULT_SMOOTHING_DURATION_SEC,
} from './constants.js';

function clamp(val: number, min: number, max: number): number {
  if (Number.isNaN(val)) return min;
  return Math.max(min, Math.min(max, val));
}

/**
 * Sanitizes and strictly bounds a candidate BodyReadout per CONTRACTS.md 7.
 */
export function sanitizeBodyReadout(input?: Partial<BodyReadout> | null): BodyReadout {
  if (!input) {
    return { ...NEUTRAL_BODY_READOUT };
  }

  const posture = clamp(
    input.posture ?? READOUT_BOUNDS.posture.neutral,
    READOUT_BOUNDS.posture.min,
    READOUT_BOUNDS.posture.max,
  );

  const idleTempo = clamp(
    input.idleTempo ?? READOUT_BOUNDS.idleTempo.neutral,
    READOUT_BOUNDS.idleTempo.min,
    READOUT_BOUNDS.idleTempo.max,
  );

  const earPose = clamp(
    input.earPose ?? READOUT_BOUNDS.earPose.neutral,
    READOUT_BOUNDS.earPose.min,
    READOUT_BOUNDS.earPose.max,
  );

  const tailAmplitude = clamp(
    input.tailAmplitude ?? READOUT_BOUNDS.tailAmplitude.neutral,
    READOUT_BOUNDS.tailAmplitude.min,
    READOUT_BOUNDS.tailAmplitude.max,
  );

  const gazeSociability = clamp(
    input.gazeSociability ?? READOUT_BOUNDS.gazeSociability.neutral,
    READOUT_BOUNDS.gazeSociability.min,
    READOUT_BOUNDS.gazeSociability.max,
  );

  const expressionBaseline: ExpressionBaselineMap = {};
  if (input.expressionBaseline && typeof input.expressionBaseline === 'object') {
    for (const [key, rawVal] of Object.entries(input.expressionBaseline)) {
      if (typeof rawVal === 'number' && !Number.isNaN(rawVal)) {
        const clampedVal = clamp(
          rawVal,
          READOUT_BOUNDS.expressionBaseline.min,
          READOUT_BOUNDS.expressionBaseline.max,
        );
        if (clampedVal > 0.001) {
          expressionBaseline[key as Emotion] = clampedVal;
        }
      }
    }
  }

  return {
    version: input.version ?? NEUTRAL_BODY_READOUT.version,
    posture,
    idleTempo,
    earPose,
    tailAmplitude,
    gazeSociability,
    expressionBaseline,
  };
}

/**
 * Checks whether two BodyReadouts are structurally and numerically equal.
 */
export function areReadoutsEqual(a: BodyReadout, b: BodyReadout): boolean {
  if (
    Math.abs(a.posture - b.posture) > 1e-5 ||
    Math.abs(a.idleTempo - b.idleTempo) > 1e-5 ||
    Math.abs(a.earPose - b.earPose) > 1e-5 ||
    Math.abs(a.tailAmplitude - b.tailAmplitude) > 1e-5 ||
    Math.abs(a.gazeSociability - b.gazeSociability) > 1e-5 ||
    a.version !== b.version
  ) {
    return false;
  }

  const keysA = Object.keys(a.expressionBaseline) as Emotion[];
  const keysB = Object.keys(b.expressionBaseline) as Emotion[];
  const allKeys = new Set([...keysA, ...keysB]);

  for (const k of allKeys) {
    const valA = a.expressionBaseline[k] ?? 0;
    const valB = b.expressionBaseline[k] ?? 0;
    if (Math.abs(valA - valB) > 1e-5) {
      return false;
    }
  }

  return true;
}

/**
 * ReadoutApplicator
 *
 * Implements P3-03 Readout Application logic:
 * - Maps BodyReadout to L2, L3, L4 and idle scheduler parameters.
 * - Idempotent: applying the same readout twice produces identical results without accumulation.
 * - Fixed animation smoothing (~2.0 s) between transitions (animation interpolation, not persistent state).
 * - Fallbacks to neutral values when unmodulated, quarantined, or disconnected.
 */
export class ReadoutApplicator {
  private readonly smoothingDurationSec: number;
  private readonly onReadoutUpdated?: ((readout: BodyReadout) => void) | undefined;
  private readonly onIdleTempoUpdated?: ((idleTempo: number) => void) | undefined;

  private currentReadout: BodyReadout;
  private startReadout: BodyReadout;
  private targetReadout: BodyReadout;

  private transitionElapsedSec = 0;
  private isTransitioning = false;

  constructor(options?: ReadoutApplicatorOptions) {
    this.smoothingDurationSec = options?.smoothingDurationSec ?? DEFAULT_SMOOTHING_DURATION_SEC;
    this.onReadoutUpdated = options?.onReadoutUpdated;
    this.onIdleTempoUpdated = options?.onIdleTempoUpdated;

    this.currentReadout = { ...NEUTRAL_BODY_READOUT };
    this.startReadout = { ...NEUTRAL_BODY_READOUT };
    this.targetReadout = { ...NEUTRAL_BODY_READOUT };
  }

  /**
   * Applies incoming BodyReadout or resets to neutral if core is disconnected or unmodulated.
   * Returns true if target changed, false if idempotent no-op.
   */
  applyReadout(
    readout?: BodyReadout | null,
    status: CoreConnectionStatus = 'CONNECTED',
    instant = false,
  ): boolean {
    const effectiveInput =
      status === 'CONNECTED' && readout != null ? readout : NEUTRAL_BODY_READOUT;

    const sanitized = sanitizeBodyReadout(effectiveInput);

    // Idempotency check: if identical to current target, do not reset interpolation
    if (areReadoutsEqual(sanitized, this.targetReadout)) {
      return false;
    }

    if (instant || this.smoothingDurationSec <= 0) {
      this.currentReadout = sanitized;
      this.startReadout = sanitized;
      this.targetReadout = sanitized;
      this.isTransitioning = false;
      this.transitionElapsedSec = this.smoothingDurationSec;

      this.onReadoutUpdated?.(this.currentReadout);
      this.onIdleTempoUpdated?.(this.currentReadout.idleTempo);
      return true;
    }

    // Start 2s smooth transition from current interpolated values to new target
    this.startReadout = {
      ...this.currentReadout,
      expressionBaseline: { ...this.currentReadout.expressionBaseline },
    };
    this.targetReadout = sanitized;
    this.transitionElapsedSec = 0;
    this.isTransitioning = true;

    return true;
  }

  /**
   * Updates smoothing interpolation per animation frame.
   */
  update(deltaSec: number): BodyReadout {
    if (!this.isTransitioning) {
      return this.currentReadout;
    }

    this.transitionElapsedSec += Math.max(0, deltaSec);
    const progress = Math.min(1.0, this.transitionElapsedSec / this.smoothingDurationSec);

    // Smooth Hermite interpolation (smoothstep) S-curve
    const s = progress * progress * (3.0 - 2.0 * progress);

    const lerp = (start: number, end: number) => start + s * (end - start);

    // Interpolate scalar parameters
    const posture = lerp(this.startReadout.posture, this.targetReadout.posture);
    const idleTempo = lerp(this.startReadout.idleTempo, this.targetReadout.idleTempo);
    const earPose = lerp(this.startReadout.earPose, this.targetReadout.earPose);
    const tailAmplitude = lerp(this.startReadout.tailAmplitude, this.targetReadout.tailAmplitude);
    const gazeSociability = lerp(
      this.startReadout.gazeSociability,
      this.targetReadout.gazeSociability,
    );

    // Interpolate expression baselines
    const expressionBaseline: ExpressionBaselineMap = {};
    const emotions = new Set<Emotion>([
      ...(Object.keys(this.startReadout.expressionBaseline) as Emotion[]),
      ...(Object.keys(this.targetReadout.expressionBaseline) as Emotion[]),
    ]);

    for (const emo of emotions) {
      const startVal = this.startReadout.expressionBaseline[emo] ?? 0;
      const targetVal = this.targetReadout.expressionBaseline[emo] ?? 0;
      const currentVal = lerp(startVal, targetVal);
      if (currentVal > 0.001) {
        expressionBaseline[emo] = currentVal;
      }
    }

    this.currentReadout = {
      version: this.targetReadout.version,
      posture,
      idleTempo,
      earPose,
      tailAmplitude,
      gazeSociability,
      expressionBaseline,
    };

    if (progress >= 1.0) {
      this.isTransitioning = false;
      this.currentReadout = {
        ...this.targetReadout,
        expressionBaseline: { ...this.targetReadout.expressionBaseline },
      };
    }

    this.onReadoutUpdated?.(this.currentReadout);
    this.onIdleTempoUpdated?.(this.currentReadout.idleTempo);

    return this.currentReadout;
  }

  getCurrentReadout(): BodyReadout {
    return {
      ...this.currentReadout,
      expressionBaseline: { ...this.currentReadout.expressionBaseline },
    };
  }

  getTargetReadout(): BodyReadout {
    return {
      ...this.targetReadout,
      expressionBaseline: { ...this.targetReadout.expressionBaseline },
    };
  }

  isInterpolating(): boolean {
    return this.isTransitioning;
  }

  resetToNeutral(instant = false): void {
    this.applyReadout(NEUTRAL_BODY_READOUT, 'CONNECTED', instant);
  }
}
