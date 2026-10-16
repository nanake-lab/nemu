import { describe, it, expect, vi } from 'vitest';
import type { PhysicalState } from '@nemu/contracts';
import { PhysicalStateController } from './state-controller.js';
import { isPerformableMotion, isRequestableMotion, MOTION_SLOTS } from './slots.js';

describe('PhysicalStateController & XState FSM', () => {
  const allStates: PhysicalState[] = [
    'idle',
    'listening',
    'speaking',
    'sitting',
    'walking',
    'dragged',
    'falling',
    'sleeping',
    'peeking',
    'hidden',
  ];

  it('verifies 30 motion slots metadata according to CONTRACTS.md 1.8', () => {
    const slotKeys = Object.keys(MOTION_SLOTS);
    expect(slotKeys.length).toBe(30);

    // 10 performable slots
    const performable = slotKeys.filter(isPerformableMotion);
    expect(performable.length).toBe(10);
    expect(performable).toContain('wave');
    expect(performable).toContain('nod');
    expect(performable).toContain('react_surprised');
    expect(performable).not.toContain('idle_stand');
    expect(performable).not.toContain('fidget_yawn');

    // 16 requestable slots
    const requestable = slotKeys.filter(isRequestableMotion);
    expect(requestable.length).toBe(16);
    expect(requestable).toContain('fidget_yawn');
    expect(requestable).toContain('wave');
    expect(requestable).toContain('react_happy_hop');
    expect(requestable).not.toContain('idle_stand');
    expect(requestable).not.toContain('dangle');
    expect(requestable).not.toContain('sleep_loop');
  });

  describe('Model-based state pair and resilience testing', () => {
    it('supports direct state forcing from any state to any other state', () => {
      const controller = new PhysicalStateController();

      for (const fromState of allStates) {
        for (const toState of allStates) {
          controller.forceState(fromState, 'setup');
          expect(controller.getCurrentState()).toBe(fromState);

          controller.forceState(toState, 'test_transition');
          expect(controller.getCurrentState()).toBe(toState);
        }
      }

      controller.dispose();
    });

    it('safely ignores illegal/unhandled events in each state without throwing', () => {
      const controller = new PhysicalStateController();

      // In sleeping state, arbitrary speech done or vad speech end should not throw and not change state
      controller.forceState('sleeping', 'test');
      expect(controller.getCurrentState()).toBe('sleeping');

      controller.send({ type: 'SPEECH_DONE' });
      expect(controller.getCurrentState()).toBe('sleeping');

      controller.send({ type: 'VAD_SPEECH_END' });
      expect(controller.getCurrentState()).toBe('sleeping');

      controller.send({ type: 'ACTION_PEEK', side: 'left' });
      expect(controller.getCurrentState()).toBe('sleeping');

      // In hidden state, drag move should be ignored
      controller.forceState('hidden', 'test');
      controller.send({ type: 'DRAG_MOVE', screenX: 100, screenY: 200 });
      expect(controller.getCurrentState()).toBe('hidden');

      controller.dispose();
    });
  });

  describe('Integrated scenario playback', () => {
    it('scenario 1: complete dialogue lifecycle (idle -> listening -> speaking -> idle)', () => {
      const transitionHistory: PhysicalState[] = [];
      const controller = new PhysicalStateController({
        onStateTransition: (_, to) => {
          transitionHistory.push(to);
        },
      });

      expect(controller.getCurrentState()).toBe('idle');

      // User starts speaking (voice.vad speechStart)
      controller.handleAppBusEvent({
        v: 2,
        id: '01HXYZ00000000000000000001',
        ts: Date.now(),
        source: 'voice',
        type: 'voice.vad',
        payload: { state: 'speechStart' },
      });
      expect(controller.getCurrentState()).toBe('listening');

      // User finishes speaking (voice.vad speechEnd) -> remains listening awaiting synthesis
      controller.handleAppBusEvent({
        v: 2,
        id: '01HXYZ00000000000000000002',
        ts: Date.now(),
        source: 'voice',
        type: 'voice.vad',
        payload: { state: 'speechEnd' },
      });
      expect(controller.getCurrentState()).toBe('listening');

      // Audio starts playing on stage
      controller.send({ type: 'SPEECH_STARTED', sentenceId: 'sent-1' });
      expect(controller.getCurrentState()).toBe('speaking');

      // Audio finishes playing on stage
      controller.send({ type: 'SPEECH_DONE', sentenceId: 'sent-1', interrupted: false });
      expect(controller.getCurrentState()).toBe('idle');

      expect(transitionHistory).toEqual(['listening', 'speaking', 'idle']);

      controller.dispose();
    });

    it('scenario 2: sleep and wake-up lifecycle (idle -> sleeping -> wake_up -> idle)', () => {
      const gestureFn = vi.fn();
      const transitionHistory: PhysicalState[] = [];

      const controller = new PhysicalStateController({
        playL1Gesture: gestureFn,
        onStateTransition: (_, to) => {
          transitionHistory.push(to);
        },
      });

      expect(controller.getCurrentState()).toBe('idle');

      // Core action: sleep
      controller.send({ type: 'ACTION_SLEEP' });
      expect(controller.getCurrentState()).toBe('sleeping');

      // Core action: wake
      controller.send({ type: 'ACTION_WAKE' });
      expect(controller.getCurrentState()).toBe('idle');
      expect(gestureFn).toHaveBeenCalledWith('wake_up', { durationSec: 1.2 });

      expect(transitionHistory).toEqual(['sleeping', 'idle']);

      controller.dispose();
    });

    it('scenario 3: drag, fall, and land lifecycle (idle -> dragged -> falling -> landed -> idle)', () => {
      const gestureFn = vi.fn();
      const controller = new PhysicalStateController({
        playL1Gesture: gestureFn,
      });

      // Start drag
      controller.handleDrag('start', { screenX: 200, screenY: 300 });
      expect(controller.getCurrentState()).toBe('dragged');

      // Move drag
      controller.handleDrag('move', { screenX: 250, screenY: 350 });
      expect(controller.getCurrentState()).toBe('dragged');

      // Release in air (landing on ground)
      controller.handleDrag('end', { landingTarget: 'ground' });
      expect(controller.getCurrentState()).toBe('falling');

      // Hit ground
      controller.send({ type: 'LANDED' });
      expect(controller.getCurrentState()).toBe('idle');
      expect(gestureFn).toHaveBeenCalledWith('land', { durationSec: 0.6 });

      controller.dispose();
    });

    it('scenario 4: drag and drop onto window edge (idle -> dragged -> sitting)', () => {
      const controller = new PhysicalStateController();

      controller.handleDrag('start');
      expect(controller.getCurrentState()).toBe('dragged');

      // Release onto windowEdge platform
      controller.handleDrag('end', { landingTarget: 'windowEdge' });
      expect(controller.getCurrentState()).toBe('sitting');

      // Wake/stand up from sitting
      controller.send({ type: 'ACTION_WAKE' });
      expect(controller.getCurrentState()).toBe('idle');

      controller.dispose();
    });

    it('scenario 5: desktop DND fullscreen/presentation hides stage, meeting/busy does not', () => {
      const controller = new PhysicalStateController();

      // Meeting mode does NOT hide
      controller.handleAppBusEvent({
        v: 2,
        id: '01HXYZ00000000000000000003',
        ts: Date.now(),
        source: 'main',
        type: 'desktop.dnd',
        payload: { state: 'meeting' },
      });
      expect(controller.getCurrentState()).toBe('idle');

      // Fullscreen mode DOES hide
      controller.handleAppBusEvent({
        v: 2,
        id: '01HXYZ00000000000000000004',
        ts: Date.now(),
        source: 'main',
        type: 'desktop.dnd',
        payload: { state: 'fullscreen' },
      });
      expect(controller.getCurrentState()).toBe('hidden');

      // Exiting fullscreen restores visibility
      controller.handleAppBusEvent({
        v: 2,
        id: '01HXYZ00000000000000000005',
        ts: Date.now(),
        source: 'main',
        type: 'desktop.dnd',
        payload: { state: 'none' },
      });
      expect(controller.getCurrentState()).toBe('idle');

      controller.dispose();
    });
  });
});
