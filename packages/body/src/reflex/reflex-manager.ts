import type { GazeTarget, PhysicalState } from '@nemu/contracts';
import type { DragEventPayload, HitEventPayload, ReflexOptions } from './types.js';

interface PendingTimer {
  remainingMs: number;
  action: () => void;
}

export class ReflexManager {
  private readonly speechEndResponseMs: number;
  private readonly flingVelocityThreshold: number;
  private readonly clock: () => number;

  private readonly onGaze?: ((target: GazeTarget, holdMs?: number | undefined) => void) | undefined;
  private readonly onGesture?: ((gestureSlot: string, durationSec?: number | undefined) => void) | undefined;
  private readonly onEarFlick?: (() => void) | undefined;
  private readonly onLatencySample?: ((metric: string, elapsedMs: number) => void) | undefined;

  private pendingTimers: PendingTimer[] = [];
  private lastSpeechEndTime: number | null = null;
  private currentState: PhysicalState = 'idle';

  constructor(options?: ReflexOptions | undefined) {
    this.speechEndResponseMs = Math.min(300, Math.max(50, options?.speechEndResponseMs ?? 200));
    this.flingVelocityThreshold = options?.flingVelocityThresholdPx ?? 1200;
    this.clock = options?.clock ?? (() => (typeof performance !== 'undefined' ? performance.now() : Date.now()));

    this.onGaze = options?.onGaze;
    this.onGesture = options?.onGesture;
    this.onEarFlick = options?.onEarFlick;
    this.onLatencySample = options?.onLatencySample;
  }

  /**
   * Reflex 1: Non-verbal response to voice.vad speechEnd.
   * Within <= 300 ms, look at user and gently nod.
   */
  handleSpeechEnd(timestamp?: number | undefined): void {
    const triggerTime = timestamp ?? this.clock();
    this.lastSpeechEndTime = triggerTime;

    // Schedule gaze and nod within speechEndResponseMs
    this.scheduleTimer(this.speechEndResponseMs, () => {
      const now = this.clock();
      const elapsedMs = now - triggerTime;
      this.onLatencySample?.('speechEnd_to_head_turn', elapsedMs);

      // Look at user and nod
      this.onGaze?.('user', 2500);
      this.onGesture?.('nod', 0.8);
    });
  }

  /**
   * Reflex 2: Pat on head/ears causes an ear flick.
   */
  handleHit(payload: HitEventPayload): void {
    if (payload.kind === 'pat' || payload.region.toLowerCase().includes('ear')) {
      this.onEarFlick?.();
      this.onGesture?.('fidget_ear_scratch', 0.8);
    }
  }

  /**
   * Reflex 3: Fast drag fling triggers react_surprised.
   */
  handleDrag(payload: DragEventPayload): void {
    if (payload.phase === 'end') {
      const vx = payload.vx ?? 0;
      const vy = payload.vy ?? 0;
      const speed = Math.sqrt(vx * vx + vy * vy);

      if (speed >= this.flingVelocityThreshold) {
        this.onGesture?.('react_surprised', 1.0);
      }
    }
  }

  /**
   * Reflex 4: Natural gaze behavior based on state.
   */
  handleStateChange(state: PhysicalState): void {
    this.currentState = state;
    if (state === 'speaking') {
      this.onGaze?.('user');
    } else if (state === 'idle') {
      this.onGaze?.('cursor');
    }
  }

  getCurrentState(): PhysicalState {
    return this.currentState;
  }

  getLastSpeechEndTime(): number | null {
    return this.lastSpeechEndTime;
  }

  /**
   * Advances simulation timers by deltaSec seconds.
   */
  update(deltaSec: number): void {
    if (deltaSec <= 0 || this.pendingTimers.length === 0) return;

    const deltaMs = deltaSec * 1000;
    const remaining: PendingTimer[] = [];

    for (const timer of this.pendingTimers) {
      timer.remainingMs -= deltaMs;
      if (timer.remainingMs <= 0) {
        timer.action();
      } else {
        remaining.push(timer);
      }
    }

    this.pendingTimers = remaining;
  }

  private scheduleTimer(delayMs: number, action: () => void): void {
    this.pendingTimers.push({
      remainingMs: Math.max(0, delayMs),
      action,
    });
  }

  reset(): void {
    this.pendingTimers = [];
    this.lastSpeechEndTime = null;
  }
}
