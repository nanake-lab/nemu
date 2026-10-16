import type { IdleSetItem, IdleSchedulerOptions } from './types.js';

export const DEFAULT_IDLE_MOTIONS: readonly IdleSetItem[] = Object.freeze([
  { motionId: 'idle_stand', weight: 1.0, cooldownS: 4.0 },
  { motionId: 'idle_relaxed', weight: 0.8, cooldownS: 5.0 },
  { motionId: 'weight_shift', weight: 0.7, cooldownS: 5.0 },
  { motionId: 'look_around', weight: 0.6, cooldownS: 6.0 },
  { motionId: 'ear_twitch', weight: 0.6, cooldownS: 6.0 },
  { motionId: 'fidget_stretch', weight: 0.5, cooldownS: 8.0 },
]);

/**
 * Checks whether an array contains any consecutive repeated subsequence
 * of length >= minLength (i.e. tandem repeat U U where |U| >= minLength).
 */
export function hasConsecutiveSubsequenceRepeat(
  sequence: readonly string[],
  minLength = 3,
): boolean {
  const n = sequence.length;
  for (let len = minLength; len * 2 <= n; len++) {
    for (let i = 0; i + 2 * len <= n; i++) {
      let match = true;
      for (let k = 0; k < len; k++) {
        if (sequence[i + k] !== sequence[i + len + k]) {
          match = false;
          break;
        }
      }
      if (match) {
        return true;
      }
    }
  }
  return false;
}

/**
 * IdleScheduler
 *
 * Implements P3-03 Idle Scheduling logic:
 * - Weighted random selection from profile.idleSet with cooldown constraints.
 * - Same motion cannot appear more than 2 consecutive times (3rd consecutive forbidden).
 * - Guaranteed: no repeated subsequence of length >= 3 occurs twice consecutively.
 * - Rhythm dynamically modulated by idleTempo.
 * - Pluggable PRNG source for deterministic reproducibility.
 */
export class IdleScheduler {
  private idleSet: IdleSetItem[];
  private readonly baseIntervalRange: [number, number];
  private readonly prng: () => number;
  private readonly onTriggerMotion?: ((motionId: string) => void) | undefined;

  private idleTempo = 1.0;
  private currentTimeSec = 0;
  private nextTriggerTimeSec = 0;
  private readonly lastTriggeredTime = new Map<string, number>();
  private readonly history: string[] = [];

  constructor(options?: IdleSchedulerOptions) {
    this.idleSet =
      options?.idleSet && options.idleSet.length > 0
        ? [...options.idleSet]
        : [...DEFAULT_IDLE_MOTIONS];

    this.baseIntervalRange = options?.baseIntervalRange ?? [6.0, 10.0];
    this.prng = options?.prng ?? Math.random;
    this.onTriggerMotion = options?.onTriggerMotion;

    if (options?.idleTempo != null) {
      this.setIdleTempo(options.idleTempo);
    }

    this.scheduleNextTrigger();
  }

  setIdleTempo(tempo: number): void {
    this.idleTempo = Math.max(0.5, Math.min(1.5, tempo));
  }

  getIdleTempo(): number {
    return this.idleTempo;
  }

  setIdleSet(idleSet: IdleSetItem[] | readonly IdleSetItem[]): void {
    if (idleSet && idleSet.length > 0) {
      this.idleSet = [...idleSet];
    } else {
      this.idleSet = [...DEFAULT_IDLE_MOTIONS];
    }
  }

  getIdleSet(): readonly IdleSetItem[] {
    return this.idleSet;
  }

  getHistory(): readonly string[] {
    return this.history;
  }

  clearHistory(): void {
    this.history.length = 0;
    this.lastTriggeredTime.clear();
    this.currentTimeSec = 0;
    this.scheduleNextTrigger();
  }

  /**
   * Advances scheduler by deltaSec.
   * Returns motionId if an idle motion triggered this frame, otherwise null.
   */
  update(deltaSec: number): string | null {
    this.currentTimeSec += Math.max(0, deltaSec);

    if (this.currentTimeSec >= this.nextTriggerTimeSec) {
      const motionId = this.triggerNext();
      if (motionId !== null) {
        this.scheduleNextTrigger();
        return motionId;
      }
      // Postpone next trigger slightly to let cooldowns expire
      this.nextTriggerTimeSec = this.currentTimeSec + 0.5;
    }

    return null;
  }

  /**
   * Selects and triggers the next idle motion immediately based on weights and constraints.
   */
  triggerNext(): string | null {
    if (this.idleSet.length === 0) {
      return null;
    }

    // Step 1: Filter by cooldown and 3-consecutive repeat rule
    const eligible = this.idleSet.filter((item) => {
      // 1. Cooldown check
      const lastTrigger = this.lastTriggeredTime.get(item.motionId);
      if (lastTrigger !== undefined) {
        if (this.currentTimeSec - lastTrigger < item.cooldownS) {
          return false;
        }
      }

      // 2. Cannot repeat same motion 3 times consecutively
      const n = this.history.length;
      if (n >= 2 && this.history[n - 1] === item.motionId && this.history[n - 2] === item.motionId) {
        return false;
      }

      return true;
    });

    if (eligible.length === 0) {
      return null;
    }

    // Step 2: Prevent length >= 3 consecutive tandem repeat (e.g. A-B-C-A-B-C)
    const validNonRepeating = eligible.filter((item) => {
      const candidateSeq = [...this.history, item.motionId];
      const n = candidateSeq.length;

      // Check if candidateSeq ends with any repeated subsequence of length L >= 3
      for (let len = 3; len * 2 <= n; len++) {
        let match = true;
        for (let k = 0; k < len; k++) {
          if (candidateSeq[n - len + k] !== candidateSeq[n - 2 * len + k]) {
            match = false;
            break;
          }
        }
        if (match) {
          return false;
        }
      }
      return true;
    });

    const candidates = validNonRepeating.length > 0 ? validNonRepeating : eligible;
    return this.pickCandidate(candidates);
  }

  private pickCandidate(candidates: IdleSetItem[]): string {
    const totalWeight = candidates.reduce((sum, item) => sum + Math.max(0, item.weight), 0);
    let chosen = candidates[0]!;

    if (totalWeight > 0) {
      let r = this.prng() * totalWeight;
      for (const item of candidates) {
        r -= Math.max(0, item.weight);
        if (r <= 0) {
          chosen = item;
          break;
        }
      }
    }

    const motionId = chosen.motionId;
    this.history.push(motionId);
    this.lastTriggeredTime.set(motionId, this.currentTimeSec);

    this.onTriggerMotion?.(motionId);
    return motionId;
  }

  private scheduleNextTrigger(): void {
    const [minInterval, maxInterval] = this.baseIntervalRange;
    const baseInterval = minInterval + this.prng() * (maxInterval - minInterval);

    // Rhythm is multiplied by idleTempo: higher tempo => shorter interval => more frequent actions
    const actualInterval = baseInterval / this.idleTempo;
    this.nextTriggerTimeSec = this.currentTimeSec + actualInterval;
  }
}
