import { setup, assign } from 'xstate';
import type {
  PhysicalMachineContext,
  PhysicalMachineEvent,
  PhysicalState,
  PhysicalStateActions,
} from './types.js';

export function createPhysicalStateMachine(actions?: PhysicalStateActions | undefined) {
  const notifyTransition = (from: PhysicalState | undefined, to: PhysicalState) => {
    if (from && from !== to) {
      actions?.onStateTransition?.(from, to);
    }
  };

  return setup({
    types: {
      context: {} as PhysicalMachineContext,
      events: {} as PhysicalMachineEvent,
    },
    actions: {
      onEnterIdle: ({ context }) => {
        actions?.playL0Motion?.('idle_stand', { loop: true });
        actions?.setFpsTier?.('idle');
        notifyTransition(context.previousState, 'idle');
      },
      onEnterListening: ({ context }) => {
        actions?.playL0Motion?.('idle_stand', { loop: true });
        actions?.setFpsTier?.('active');
        notifyTransition(context.previousState, 'listening');
      },
      onEnterSpeaking: ({ context }) => {
        actions?.playL0Motion?.('idle_stand', { loop: true });
        actions?.setFpsTier?.('active');
        notifyTransition(context.previousState, 'speaking');
      },
      onEnterSitting: ({ context }) => {
        actions?.playL0Motion?.('sit_edge_loop', { loop: true });
        actions?.setFpsTier?.('idle');
        notifyTransition(context.previousState, 'sitting');
      },
      onEnterWalking: ({ context }) => {
        actions?.playL0Motion?.('walk', { loop: true });
        actions?.setFpsTier?.('active');
        notifyTransition(context.previousState, 'walking');
      },
      onEnterDragged: ({ context }) => {
        actions?.playL0Motion?.('dangle', { loop: true });
        actions?.setFpsTier?.('active');
        notifyTransition(context.previousState, 'dragged');
      },
      onEnterFalling: ({ context }) => {
        actions?.playL0Motion?.('fall_loop', { loop: true });
        actions?.setFpsTier?.('active');
        notifyTransition(context.previousState, 'falling');
      },
      onLanded: () => {
        actions?.playL1Gesture?.('land', { durationSec: 0.6 });
      },
      onEnterSleeping: ({ context }) => {
        actions?.playL0Motion?.('sleep_loop', { loop: true });
        actions?.setFpsTier?.('sleep');
        notifyTransition(context.previousState, 'sleeping');
      },
      onWakeUp: () => {
        actions?.playL1Gesture?.('wake_up', { durationSec: 1.2 });
      },
      onEnterPeeking: ({ context }) => {
        actions?.playL0Motion?.('peek_loop', { loop: true });
        actions?.setFpsTier?.('active');
        notifyTransition(context.previousState, 'peeking');
      },
      onEnterHidden: ({ context }) => {
        actions?.setFpsTier?.('hidden');
        notifyTransition(context.previousState, 'hidden');
      },
    },
    guards: {
      isDndHidden: (_, params: { mode: string }) =>
        params.mode === 'fullscreen' || params.mode === 'presentation',
      isDndVisible: (_, params: { mode: string }) =>
        params.mode === 'none' || params.mode === 'meeting' || params.mode === 'busy',
      isEdgeLanding: (_, params: { target?: string | undefined }) =>
        params.target === 'windowEdge' || params.target === 'platform',
      isTargetState: ({ event }, params: { expected: PhysicalState }) => {
        return (event as { state?: PhysicalState }).state === params.expected;
      },
    },
  }).createMachine({
    id: 'nemuPhysicalState',
    initial: 'idle',
    context: {
      previousState: undefined,
      lastLandingTarget: undefined,
      peekSide: undefined,
      targetAnchor: undefined,
    },
    on: {
      FORCE_STATE: [
        {
          guard: { type: 'isTargetState', params: { expected: 'idle' } },
          target: '.idle',
          actions: assign(({ event }) => ({ previousState: (event as { state: PhysicalState }).state })),
        },
        {
          guard: { type: 'isTargetState', params: { expected: 'listening' } },
          target: '.listening',
          actions: assign(({ event }) => ({ previousState: (event as { state: PhysicalState }).state })),
        },
        {
          guard: { type: 'isTargetState', params: { expected: 'speaking' } },
          target: '.speaking',
          actions: assign(({ event }) => ({ previousState: (event as { state: PhysicalState }).state })),
        },
        {
          guard: { type: 'isTargetState', params: { expected: 'sitting' } },
          target: '.sitting',
          actions: assign(({ event }) => ({ previousState: (event as { state: PhysicalState }).state })),
        },
        {
          guard: { type: 'isTargetState', params: { expected: 'walking' } },
          target: '.walking',
          actions: assign(({ event }) => ({ previousState: (event as { state: PhysicalState }).state })),
        },
        {
          guard: { type: 'isTargetState', params: { expected: 'dragged' } },
          target: '.dragged',
          actions: assign(({ event }) => ({ previousState: (event as { state: PhysicalState }).state })),
        },
        {
          guard: { type: 'isTargetState', params: { expected: 'falling' } },
          target: '.falling',
          actions: assign(({ event }) => ({ previousState: (event as { state: PhysicalState }).state })),
        },
        {
          guard: { type: 'isTargetState', params: { expected: 'sleeping' } },
          target: '.sleeping',
          actions: assign(({ event }) => ({ previousState: (event as { state: PhysicalState }).state })),
        },
        {
          guard: { type: 'isTargetState', params: { expected: 'peeking' } },
          target: '.peeking',
          actions: assign(({ event }) => ({ previousState: (event as { state: PhysicalState }).state })),
        },
        {
          guard: { type: 'isTargetState', params: { expected: 'hidden' } },
          target: '.hidden',
          actions: assign(({ event }) => ({ previousState: (event as { state: PhysicalState }).state })),
        },
      ],
    },
    states: {
      idle: {
        entry: 'onEnterIdle',
        on: {
          VAD_SPEECH_START: {
            target: 'listening',
            actions: assign({ previousState: 'idle' }),
          },
          SPEECH_STARTED: {
            target: 'speaking',
            actions: assign({ previousState: 'idle' }),
          },
          DRAG_START: {
            target: 'dragged',
            actions: assign({ previousState: 'idle' }),
          },
          ACTION_SLEEP: {
            target: 'sleeping',
            actions: assign({ previousState: 'idle' }),
          },
          ACTION_PEEK: {
            target: 'peeking',
            actions: assign(({ event }) => ({
              previousState: 'idle' as PhysicalState,
              peekSide: event.side ?? 'left',
            })),
          },
          ACTION_GOTO: {
            target: 'walking',
            actions: assign(({ event }) => ({
              previousState: 'idle' as PhysicalState,
              targetAnchor: event.anchor,
            })),
          },
          ACTION_SIT: {
            target: 'sitting',
            actions: assign({ previousState: 'idle' }),
          },
          DND_CHANGE: {
            guard: { type: 'isDndHidden', params: ({ event }) => ({ mode: event.mode }) },
            target: 'hidden',
            actions: assign({ previousState: 'idle' }),
          },
        },
      },

      listening: {
        entry: 'onEnterListening',
        on: {
          SPEECH_STARTED: {
            target: 'speaking',
            actions: assign({ previousState: 'listening' }),
          },
          VAD_SPEECH_END: {
            // Remains listening awaiting core synthesis or fallback to idle on action done
          },
          ACTION_DONE: {
            target: 'idle',
            actions: assign({ previousState: 'listening' }),
          },
          BARGE_IN: {
            target: 'listening',
          },
          DRAG_START: {
            target: 'dragged',
            actions: assign({ previousState: 'listening' }),
          },
          ACTION_SLEEP: {
            target: 'sleeping',
            actions: assign({ previousState: 'listening' }),
          },
          DND_CHANGE: {
            guard: { type: 'isDndHidden', params: ({ event }) => ({ mode: event.mode }) },
            target: 'hidden',
            actions: assign({ previousState: 'listening' }),
          },
        },
      },

      speaking: {
        entry: 'onEnterSpeaking',
        on: {
          SPEECH_DONE: [
            {
              guard: ({ context }) => context.previousState === 'sitting',
              target: 'sitting',
              actions: assign({ previousState: 'speaking' }),
            },
            {
              target: 'idle',
              actions: assign({ previousState: 'speaking' }),
            },
          ],
          BARGE_IN: {
            target: 'listening',
            actions: assign({ previousState: 'speaking' }),
          },
          VAD_SPEECH_START: {
            target: 'listening',
            actions: assign({ previousState: 'speaking' }),
          },
          DRAG_START: {
            target: 'dragged',
            actions: assign({ previousState: 'speaking' }),
          },
          DND_CHANGE: {
            guard: { type: 'isDndHidden', params: ({ event }) => ({ mode: event.mode }) },
            target: 'hidden',
            actions: assign({ previousState: 'speaking' }),
          },
        },
      },

      sitting: {
        entry: 'onEnterSitting',
        on: {
          VAD_SPEECH_START: {
            target: 'listening',
            actions: assign({ previousState: 'sitting' }),
          },
          SPEECH_STARTED: {
            target: 'speaking',
            actions: assign({ previousState: 'sitting' }),
          },
          DRAG_START: {
            target: 'dragged',
            actions: assign({ previousState: 'sitting' }),
          },
          ACTION_WAKE: {
            target: 'idle',
            actions: assign({ previousState: 'sitting' }),
          },
          ACTION_GOTO: {
            target: 'walking',
            actions: assign(({ event }) => ({
              previousState: 'sitting' as PhysicalState,
              targetAnchor: event.anchor,
            })),
          },
          ACTION_PEEK: {
            target: 'peeking',
            actions: assign(({ event }) => ({
              previousState: 'sitting' as PhysicalState,
              peekSide: event.side ?? 'left',
            })),
          },
          ACTION_SIT: {
            target: 'sitting',
            actions: assign({ previousState: 'sitting' }),
          },
          ACTION_SLEEP: {
            target: 'sleeping',
            actions: assign({ previousState: 'sitting' }),
          },
          DND_CHANGE: {
            guard: { type: 'isDndHidden', params: ({ event }) => ({ mode: event.mode }) },
            target: 'hidden',
            actions: assign({ previousState: 'sitting' }),
          },
        },
      },

      walking: {
        entry: 'onEnterWalking',
        on: {
          ACTION_DONE: {
            target: 'idle',
            actions: assign({ previousState: 'walking' }),
          },
          ACTION_SIT: {
            target: 'sitting',
            actions: assign({ previousState: 'walking' }),
          },
          ACTION_SLEEP: {
            target: 'sleeping',
            actions: assign({ previousState: 'walking' }),
          },
          ACTION_PEEK: {
            target: 'peeking',
            actions: assign(({ event }) => ({
              previousState: 'walking' as PhysicalState,
              peekSide: event.side ?? 'left',
            })),
          },
          VAD_SPEECH_START: {
            target: 'listening',
            actions: assign({ previousState: 'walking' }),
          },
          SPEECH_STARTED: {
            target: 'speaking',
            actions: assign({ previousState: 'walking' }),
          },
          DRAG_START: {
            target: 'dragged',
            actions: assign({ previousState: 'walking' }),
          },
          DND_CHANGE: {
            guard: { type: 'isDndHidden', params: ({ event }) => ({ mode: event.mode }) },
            target: 'hidden',
            actions: assign({ previousState: 'walking' }),
          },
        },
      },

      dragged: {
        entry: 'onEnterDragged',
        on: {
          DRAG_MOVE: {
            // Remains dragged
          },
          DRAG_END: [
            {
              guard: { type: 'isEdgeLanding', params: ({ event }) => ({ target: event.landingTarget }) },
              target: 'sitting',
              actions: assign(({ event }) => ({
                previousState: 'dragged' as PhysicalState,
                lastLandingTarget: event.landingTarget,
              })),
            },
            {
              target: 'falling',
              actions: assign(({ event }) => ({
                previousState: 'dragged' as PhysicalState,
                lastLandingTarget: event.landingTarget ?? 'ground',
              })),
            },
          ],
          DND_CHANGE: {
            guard: { type: 'isDndHidden', params: ({ event }) => ({ mode: event.mode }) },
            target: 'hidden',
            actions: assign({ previousState: 'dragged' }),
          },
        },
      },

      falling: {
        entry: 'onEnterFalling',
        on: {
          LANDED: {
            target: 'idle',
            actions: ['onLanded', assign({ previousState: 'falling' })],
          },
          DRAG_START: {
            target: 'dragged',
            actions: assign({ previousState: 'falling' }),
          },
          DND_CHANGE: {
            guard: { type: 'isDndHidden', params: ({ event }) => ({ mode: event.mode }) },
            target: 'hidden',
            actions: assign({ previousState: 'falling' }),
          },
        },
      },

      sleeping: {
        entry: 'onEnterSleeping',
        on: {
          ACTION_WAKE: {
            target: 'idle',
            actions: ['onWakeUp', assign({ previousState: 'sleeping' })],
          },
          DRAG_START: {
            // Physical drag wakes her up
            target: 'dragged',
            actions: ['onWakeUp', assign({ previousState: 'sleeping' })],
          },
          DND_CHANGE: {
            guard: { type: 'isDndHidden', params: ({ event }) => ({ mode: event.mode }) },
            target: 'hidden',
            actions: assign({ previousState: 'sleeping' }),
          },
        },
      },

      peeking: {
        entry: 'onEnterPeeking',
        on: {
          ACTION_DONE: {
            target: 'idle',
            actions: assign({ previousState: 'peeking' }),
          },
          ACTION_SIT: {
            target: 'sitting',
            actions: assign({ previousState: 'peeking' }),
          },
          ACTION_GOTO: {
            target: 'walking',
            actions: assign(({ event }) => ({
              previousState: 'peeking' as PhysicalState,
              targetAnchor: event.anchor,
            })),
          },
          VAD_SPEECH_START: {
            target: 'listening',
            actions: assign({ previousState: 'peeking' }),
          },
          DRAG_START: {
            target: 'dragged',
            actions: assign({ previousState: 'peeking' }),
          },
          ACTION_SLEEP: {
            target: 'sleeping',
            actions: assign({ previousState: 'peeking' }),
          },
          DND_CHANGE: {
            guard: { type: 'isDndHidden', params: ({ event }) => ({ mode: event.mode }) },
            target: 'hidden',
            actions: assign({ previousState: 'peeking' }),
          },
        },
      },

      hidden: {
        entry: 'onEnterHidden',
        on: {
          DND_CHANGE: {
            guard: { type: 'isDndVisible', params: ({ event }) => ({ mode: event.mode }) },
            target: 'idle',
            actions: assign({ previousState: 'hidden' }),
          },
        },
      },
    },
  });
}
