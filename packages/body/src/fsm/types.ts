import type { PhysicalState, Anchor, GazeTarget } from '@nemu/contracts';

export type { PhysicalState };

export interface PhysicalMachineContext {
  previousState?: PhysicalState | undefined;
  lastLandingTarget?: 'ground' | 'windowEdge' | 'platform' | undefined;
  peekSide?: 'left' | 'right' | undefined;
  targetAnchor?: Anchor | undefined;
}

export type PhysicalMachineEvent =
  | { type: 'VAD_SPEECH_START' }
  | { type: 'VAD_SPEECH_END' }
  | { type: 'SPEECH_STARTED'; sentenceId?: string | undefined }
  | { type: 'SPEECH_DONE'; sentenceId?: string | undefined; interrupted?: boolean | undefined }
  | { type: 'BARGE_IN'; at?: number | undefined }
  | { type: 'DRAG_START'; screenX?: number | undefined; screenY?: number | undefined }
  | { type: 'DRAG_MOVE'; screenX?: number | undefined; screenY?: number | undefined; vx?: number | undefined; vy?: number | undefined }
  | { type: 'DRAG_END'; landingTarget?: 'ground' | 'windowEdge' | 'platform' | undefined; vx?: number | undefined; vy?: number | undefined }
  | { type: 'LANDED' }
  | { type: 'WAKE_ANIM_DONE' }
  | { type: 'ACTION_SLEEP' }
  | { type: 'ACTION_WAKE' }
  | { type: 'ACTION_PEEK'; side?: 'left' | 'right' | undefined }
  | { type: 'ACTION_GOTO'; anchor: Anchor }
  | { type: 'ACTION_SIT'; platformId?: string | undefined }
  | { type: 'ACTION_DONE' }
  | { type: 'DND_CHANGE'; mode: 'none' | 'fullscreen' | 'presentation' | 'meeting' | 'busy' }
  | { type: 'FORCE_STATE'; state: PhysicalState; reason: string };

export interface PhysicalStateActions {
  playL0Motion?: ((slotId: string, options?: { loop?: boolean | undefined; fadeSec?: number | undefined } | undefined) => void) | undefined;
  playL1Gesture?: ((slotId: string, options?: { durationSec?: number | undefined } | undefined) => void) | undefined;
  setGaze?: ((target: GazeTarget, holdMs?: number | undefined) => void) | undefined;
  setFpsTier?: ((tier: 'active' | 'idle' | 'sleep' | 'hidden') => void) | undefined;
  onStateTransition?: ((from: PhysicalState, to: PhysicalState, reason?: string | undefined) => void) | undefined;
}
