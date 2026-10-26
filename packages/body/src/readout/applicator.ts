import { bodyReadoutV2Schema, type BodyReadoutV2 } from '@nemu/contracts';
import type { CoreConnectionStatus, ReadoutApplicatorOptions } from './types.js';
import { NEUTRAL_BODY_READOUT, DEFAULT_SMOOTHING_DURATION_SEC } from './constants.js';

/** Reject invalid/v1 shapes as a whole; never project old fields into v2. */
export function sanitizeBodyReadout(input?: unknown): BodyReadoutV2 {
  const parsed = bodyReadoutV2Schema.safeParse(input);
  return parsed.success ? parsed.data : { ...NEUTRAL_BODY_READOUT };
}

export function areReadoutsEqual(a: BodyReadoutV2, b: BodyReadoutV2): boolean {
  return (
    a.version === b.version &&
    a.facing === b.facing &&
    a.tension === b.tension &&
    a.energy === b.energy
  );
}

/** Only frame-local animation interpolation; no persistence or accumulated drive. */
export class ReadoutApplicator {
  private readonly smoothingDurationSec: number;
  private readonly onReadoutUpdated: ReadoutApplicatorOptions['onReadoutUpdated'];
  private currentReadout: BodyReadoutV2 = { ...NEUTRAL_BODY_READOUT };
  private startReadout: BodyReadoutV2 = { ...NEUTRAL_BODY_READOUT };
  private targetReadout: BodyReadoutV2 = { ...NEUTRAL_BODY_READOUT };
  private transitionElapsedSec = 0;
  private isTransitioning = false;

  constructor(options?: ReadoutApplicatorOptions) {
    this.smoothingDurationSec = options?.smoothingDurationSec ?? DEFAULT_SMOOTHING_DURATION_SEC;
    if (!Number.isFinite(this.smoothingDurationSec) || this.smoothingDurationSec < 0) {
      throw new RangeError('Invalid readout animation duration');
    }
    this.onReadoutUpdated = options?.onReadoutUpdated;
  }

  applyReadout(
    readout?: BodyReadoutV2 | null,
    status: CoreConnectionStatus = 'CONNECTED',
    instant = false,
  ): boolean {
    const next = sanitizeBodyReadout(status === 'CONNECTED' ? readout : null);
    if (areReadoutsEqual(next, this.targetReadout) && !(instant && this.isTransitioning)) {
      return false;
    }
    this.startReadout = { ...this.currentReadout };
    this.targetReadout = next;
    this.transitionElapsedSec = 0;
    this.isTransitioning = !instant && this.smoothingDurationSec > 0;
    if (!this.isTransitioning) {
      this.currentReadout = { ...next };
      this.publish();
    }
    return true;
  }

  update(deltaSec: number): BodyReadoutV2 {
    if (this.isTransitioning && Number.isFinite(deltaSec) && deltaSec > 0) {
      this.transitionElapsedSec += deltaSec;
      const t = Math.min(1, this.transitionElapsedSec / this.smoothingDurationSec);
      const s = t * t * (3 - 2 * t);
      const lerp = (a: number, b: number) => a + s * (b - a);
      this.currentReadout =
        t === 1
          ? { ...this.targetReadout }
          : {
              version: this.targetReadout.version,
              facing: lerp(this.startReadout.facing, this.targetReadout.facing),
              tension: lerp(this.startReadout.tension, this.targetReadout.tension),
              energy: lerp(this.startReadout.energy, this.targetReadout.energy),
            };
      this.isTransitioning = t < 1;
      this.publish();
    }
    return this.getCurrentReadout();
  }

  private publish(): void {
    this.onReadoutUpdated?.({ ...this.currentReadout });
  }
  getCurrentReadout(): BodyReadoutV2 {
    return { ...this.currentReadout };
  }
  getTargetReadout(): BodyReadoutV2 {
    return { ...this.targetReadout };
  }
  isInterpolating(): boolean {
    return this.isTransitioning;
  }
  resetToNeutral(instant = false): void {
    this.applyReadout(null, 'DISCONNECTED', instant);
  }
}
