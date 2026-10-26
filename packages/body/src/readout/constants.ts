import type { BodyReadoutV2 } from '@nemu/contracts';

/** CONTRACTS §7: missing input or m = 0 uses the neutral triple. */
export const NEUTRAL_BODY_READOUT: Readonly<BodyReadoutV2> = Object.freeze({
  version: 'body-neutral-v2',
  facing: 0.5,
  tension: 0,
  energy: 0.5,
});

export const DEFAULT_SMOOTHING_DURATION_SEC = 2;
