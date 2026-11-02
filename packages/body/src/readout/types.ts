import type { BodyReadoutV2 } from '@nemu/contracts';

export type CoreConnectionStatus =
  'CONNECTED' | 'DISCONNECTED' | 'CORE_UNMODULATED' | 'MODULATION_NOT_ENABLED';

export interface ReadoutApplicatorOptions {
  /** Fixed animation transition, not core state or learned parameters. */
  smoothingDurationSec?: number | undefined;
  onReadoutUpdated?: ((readout: BodyReadoutV2) => void) | undefined;
}
