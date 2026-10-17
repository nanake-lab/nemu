import type { GazeTarget } from '@nemu/contracts';

export interface ReflexOptions {
  speechEndResponseMs?: number | undefined; // Target latency for gaze/nod after speechEnd (default 200ms, max 300ms)
  flingVelocityThresholdPx?: number | undefined; // Speed threshold to trigger react_surprised (default 1200 px/s)
  onGaze?: ((target: GazeTarget, holdMs?: number | undefined) => void) | undefined;
  onGesture?: ((gestureSlot: string, durationSec?: number | undefined) => void) | undefined;
  onEarFlick?: (() => void) | undefined;
  onLatencySample?: ((metric: string, elapsedMs: number) => void) | undefined;
  clock?: (() => number) | undefined;
}

export interface HitEventPayload {
  region: string;
  kind: 'click' | 'doubleClick' | 'hoverStart' | 'hoverEnd' | 'pat';
}

export interface DragEventPayload {
  phase: 'start' | 'move' | 'end';
  screenX?: number | undefined;
  screenY?: number | undefined;
  vx?: number | undefined;
  vy?: number | undefined;
}
