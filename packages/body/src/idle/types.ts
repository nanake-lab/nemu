export interface IdleSetItem {
  motionId: string;
  weight: number;
  cooldownS: number;
}

export interface IdleSchedulerOptions {
  /**
   * Available idle motions and their parameters from NemuProfile.idleSet.
   */
  idleSet?: IdleSetItem[] | readonly IdleSetItem[] | undefined;

  /**
   * Base interval range between idle motion triggers in seconds [min, max].
   * Defaults to [6.0, 10.0].
   */
  baseIntervalRange?: [number, number] | undefined;

  /**
   * Initial idle tempo multiplier (0.5..1.5, default: 1.0).
   */
  idleTempo?: number | undefined;

  /**
   * Injectable pseudorandom number generator returning [0, 1).
   * In deterministic mode, provide a seeded generator.
   */
  prng?: (() => number) | undefined;

  /**
   * Callback fired when an idle motion is triggered.
   */
  onTriggerMotion?: ((motionId: string) => void) | undefined;
}
