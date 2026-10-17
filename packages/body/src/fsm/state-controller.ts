import { createActor, type ActorRefFrom } from 'xstate';
import type { AppBusEvent, ForwardedAppBusEvent, PhysicalState } from '@nemu/contracts';
import { createPhysicalStateMachine } from './machine.js';
import type {
  PhysicalMachineEvent,
  PhysicalMachineContext,
  PhysicalStateActions,
} from './types.js';

export class PhysicalStateController {
  private readonly machine: ReturnType<typeof createPhysicalStateMachine>;
  private readonly actor: ActorRefFrom<ReturnType<typeof createPhysicalStateMachine>>;

  private currentState: PhysicalState = 'idle';
  private listeners: Array<(state: PhysicalState, prev: PhysicalState) => void> = [];

  constructor(actions?: PhysicalStateActions | undefined) {
    this.machine = createPhysicalStateMachine(actions);
    this.actor = createActor(this.machine);

    this.actor.subscribe((snapshot) => {
      const nextState = snapshot.value as PhysicalState;
      if (nextState && nextState !== this.currentState && nextState !== ('handleForce' as PhysicalState)) {
        const prev = this.currentState;
        this.currentState = nextState;
        for (const listener of this.listeners) {
          listener(nextState, prev);
        }
      }
    });

    this.actor.start();
  }

  getCurrentState(): PhysicalState {
    return this.currentState;
  }

  getContext(): PhysicalMachineContext {
    return this.actor.getSnapshot().context;
  }

  send(event: PhysicalMachineEvent): void {
    try {
      this.actor.send(event);
    } catch (err) {
      console.warn('PhysicalStateController ignored unhandled/illegal event:', event, err);
    }
  }

  forceState(state: PhysicalState, reason = 'manual'): void {
    this.send({ type: 'FORCE_STATE', state, reason });
  }

  onTransition(callback: (state: PhysicalState, prev: PhysicalState) => void): () => void {
    this.listeners.push(callback);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== callback);
    };
  }

  handleAppBusEvent(event: ForwardedAppBusEvent | AppBusEvent): void {
    switch (event.type) {
      case 'voice.vad': {
        const payload = event.payload as { state?: 'speechStart' | 'speechEnd' } | undefined;
        if (payload?.state === 'speechStart') {
          this.send({ type: 'VAD_SPEECH_START' });
        } else if (payload?.state === 'speechEnd') {
          this.send({ type: 'VAD_SPEECH_END' });
        }
        break;
      }
      case 'voice.bargeIn': {
        this.send({ type: 'BARGE_IN', at: event.ts });
        break;
      }
      case 'desktop.dnd': {
        const payload = event.payload as { state?: 'none' | 'fullscreen' | 'presentation' | 'meeting' | 'busy' } | undefined;
        if (payload?.state) {
          this.send({ type: 'DND_CHANGE', mode: payload.state });
        }
        break;
      }
    }
  }

  handleDrag(phase: 'start' | 'move' | 'end', options?: {
    screenX?: number | undefined;
    screenY?: number | undefined;
    vx?: number | undefined;
    vy?: number | undefined;
    landingTarget?: 'ground' | 'windowEdge' | 'platform' | undefined;
  } | undefined): void {
    if (phase === 'start') {
      this.send({ type: 'DRAG_START', screenX: options?.screenX, screenY: options?.screenY });
    } else if (phase === 'move') {
      this.send({
        type: 'DRAG_MOVE',
        screenX: options?.screenX,
        screenY: options?.screenY,
        vx: options?.vx,
        vy: options?.vy,
      });
    } else if (phase === 'end') {
      this.send({
        type: 'DRAG_END',
        landingTarget: options?.landingTarget,
        vx: options?.vx,
        vy: options?.vy,
      });
    }
  }

  dispose(): void {
    this.actor.stop();
    this.listeners = [];
  }
}
