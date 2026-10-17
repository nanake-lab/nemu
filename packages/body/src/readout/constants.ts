import type { BodyReadout } from '@nemu/contracts';

/**
 * Standard neutral BodyReadout.
 * Produced when core state is unmodulated, quarantined, disconnected,
 * or when m = 0 (CONTRACTS.md Section 7).
 */
export const NEUTRAL_BODY_READOUT: BodyReadout = Object.freeze({
  version: '1.0',
  expressionBaseline: Object.freeze({}),
  idleTempo: 1.0,
  earPose: 0.0,
  tailAmplitude: 0.5,
  gazeSociability: 0.5,
  posture: 0.0,
});

/**
 * Specification boundaries for BodyReadout parameters (CONTRACTS.md 7).
 */
export const READOUT_BOUNDS = Object.freeze({
  posture: { min: -1.0, max: 1.0, neutral: 0.0 },
  idleTempo: { min: 0.5, max: 1.5, neutral: 1.0 },
  earPose: { min: -1.0, max: 1.0, neutral: 0.0 },
  tailAmplitude: { min: 0.0, max: 1.0, neutral: 0.5 },
  gazeSociability: { min: 0.0, max: 1.0, neutral: 0.5 },
  expressionBaseline: { min: 0.0, max: 0.4, neutral: 0.0 },
});

export const DEFAULT_SMOOTHING_DURATION_SEC = 2.0;
