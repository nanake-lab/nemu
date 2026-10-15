export interface FrameStats {
  fps: number;
  p95FrameMs: number;
  renderedFrames: number;
}

export type StatsCallback = (stats: FrameStats) => void;
export type RenderCallback = (deltaTimeSec: number) => void;

export class FrameScheduler {
  private targetFps: number;
  private isPaused = false;
  private isDeterministic = false;

  private rafId: number | null = null;
  private lastTimestamp = 0;
  private accumulator = 0;

  // Frame statistics tracking
  private recentFrameDurations: number[] = [];
  private readonly maxSamples = 120;
  private frameCountInWindow = 0;
  private lastStatsReportTime = 0;
  private totalRenderedFrames = 0;

  private readonly onRender: RenderCallback;
  private readonly onStats?: StatsCallback | undefined;
  private readonly statsIntervalMs = 5000;

  constructor(options: {
    targetFps?: number;
    deterministic?: boolean;
    onRender: RenderCallback;
    onStats?: StatsCallback;
  }) {
    this.targetFps = options.targetFps ?? 60;
    this.isDeterministic = options.deterministic ?? false;
    this.onRender = options.onRender;
    this.onStats = options.onStats;
  }

  setTargetFps(fps: number): void {
    if (fps <= 0) {
      this.isPaused = true;
    } else {
      this.isPaused = false;
      this.targetFps = fps;
    }
  }

  setPaused(paused: boolean): void {
    this.isPaused = paused;
  }

  getTargetFps(): number {
    return this.targetFps;
  }

  start(): void {
    if (this.isDeterministic) {
      // In deterministic mode, frames are driven manually via step()
      return;
    }

    if (this.rafId !== null) return;
    this.lastTimestamp = performance.now();
    this.lastStatsReportTime = this.lastTimestamp;

    const loop = (timestamp: number) => {
      this.rafId = requestAnimationFrame(loop);

      if (this.isPaused) {
        this.lastTimestamp = timestamp;
        return;
      }

      const elapsed = timestamp - this.lastTimestamp;
      this.lastTimestamp = timestamp;

      // Prevent spiral of death on long freezes/tab-switches
      const clampedElapsed = Math.min(elapsed, 250);
      this.accumulator += clampedElapsed;

      const targetInterval = 1000 / this.targetFps;

      if (this.accumulator >= targetInterval) {
        // Record render time
        const startRender = performance.now();

        // Perform frame update
        const deltaSec = targetInterval / 1000;
        this.onRender(deltaSec);

        const frameDuration = performance.now() - startRender;
        this.recordFrameDuration(frameDuration);

        this.accumulator -= targetInterval;
        // If still backlog exists after 1 frame, drain excess to maintain stability
        if (this.accumulator > targetInterval) {
          this.accumulator = this.accumulator % targetInterval;
        }

        this.totalRenderedFrames++;
        this.frameCountInWindow++;

        // Periodic 5s stats report
        if (timestamp - this.lastStatsReportTime >= this.statsIntervalMs) {
          this.reportStats(timestamp);
        }
      }
    };

    this.rafId = requestAnimationFrame(loop);
  }

  stop(): void {
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
  }

  /**
   * Deterministically advances the frame by deltaMs.
   */
  step(deltaMs: number): void {
    const deltaSec = deltaMs / 1000;
    const startRender = performance.now();
    this.onRender(deltaSec);
    const frameDuration = performance.now() - startRender;
    this.recordFrameDuration(frameDuration);
    this.totalRenderedFrames++;
  }

  private recordFrameDuration(durationMs: number): void {
    this.recentFrameDurations.push(durationMs);
    if (this.recentFrameDurations.length > this.maxSamples) {
      this.recentFrameDurations.shift();
    }
  }

  private reportStats(currentTimestamp: number): void {
    const timeDelta = (currentTimestamp - this.lastStatsReportTime) / 1000;
    const actualFps = timeDelta > 0 ? this.frameCountInWindow / timeDelta : 0;
    const p95 = this.calculateP95FrameTime();

    if (this.onStats) {
      this.onStats({
        fps: Math.round(actualFps * 10) / 10,
        p95FrameMs: Math.round(p95 * 100) / 100,
        renderedFrames: this.totalRenderedFrames,
      });
    }

    this.frameCountInWindow = 0;
    this.lastStatsReportTime = currentTimestamp;
  }

  calculateP95FrameTime(): number {
    if (this.recentFrameDurations.length === 0) return 0;
    const sorted = [...this.recentFrameDurations].sort((a, b) => a - b);
    const p95Index = Math.min(sorted.length - 1, Math.floor(sorted.length * 0.95));
    return sorted[p95Index] ?? 0;
  }

  getRecentStats(): FrameStats {
    return {
      fps: this.targetFps,
      p95FrameMs: this.calculateP95FrameTime(),
      renderedFrames: this.totalRenderedFrames,
    };
  }
}
