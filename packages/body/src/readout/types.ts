import type { BodyReadout, Emotion } from '@nemu/contracts';

export type CoreConnectionStatus =
  | 'CONNECTED'
  | 'DISCONNECTED'
  | 'CORE_UNMODULATED'
  | 'CORE_QUARANTINED';

export interface ReadoutApplicatorOptions {
  /**
   * Fixed smoothing transition duration in seconds (default: 2.0s per CONTRACTS.md 7).
   */
  smoothingDurationSec?: number | undefined;

  /**
   * Callback fired whenever smoothed readout values update.
   */
  onReadoutUpdated?: ((readout: BodyReadout) => void) | undefined;

  /**
   * Callback fired when smoothed idleTempo updates.
   */
  onIdleTempoUpdated?: ((idleTempo: number) => void) | undefined;
}

export type ExpressionBaselineMap = Partial<Record<Emotion, number>>;
