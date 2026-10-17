import type { GazeTarget, Emotion } from '@nemu/contracts';

export interface ActionResultPayload {
  actionId: string;
  status: 'succeeded' | 'failed' | 'cancelled';
  detail?: string | undefined;
}

export interface ActionExecutorOptions {
  onActionResult?: ((result: ActionResultPayload) => void) | undefined;
  onPlayMotion?: ((motionId: string, durationSec?: number | undefined) => void) | undefined;
  onGaze?: ((target: GazeTarget, holdMs?: number | undefined) => void) | undefined;
}

export interface CueSchedulerOptions {
  onPerformAct?: ((motionId: string) => void) | undefined;
  onPerformLook?: ((target: GazeTarget) => void) | undefined;
  onPerformEmo?: ((emotion: Emotion, weight?: number | undefined) => void) | undefined;
}
