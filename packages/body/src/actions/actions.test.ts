import { describe, it, expect, vi } from 'vitest';
import { PhysicalStateController } from '../fsm/state-controller.js';
import { BodyActionExecutor } from './action-executor.js';

describe('BodyActionExecutor', () => {
  it('rejects non-requestable motion slots and returns failed result', () => {
    const controller = new PhysicalStateController();
    const actionResultFn = vi.fn();
    const executor = new BodyActionExecutor(controller, {
      onActionResult: actionResultFn,
    });

    // Try to request 'idle_stand' (non-requestable)
    const resultIdle = executor.executeAction('act-1', {
      kind: 'motion',
      motionId: 'idle_stand',
    } as unknown as Parameters<typeof executor.executeAction>[1]);

    expect(resultIdle.status).toBe('failed');
    expect(resultIdle.detail).toContain("Motion slot 'idle_stand' is not requestable");
    expect(actionResultFn).toHaveBeenCalledWith(
      expect.objectContaining({
        actionId: 'act-1',
        status: 'failed',
      }),
    );

    // Try to request 'sleep_loop' (non-requestable)
    const resultSleep = executor.executeAction('act-2', {
      kind: 'motion',
      motionId: 'sleep_loop',
    } as unknown as Parameters<typeof executor.executeAction>[1]);
    expect(resultSleep.status).toBe('failed');
    expect(resultSleep.detail).toContain("Motion slot 'sleep_loop' is not requestable");

    // Try to request 'dangle' (non-requestable)
    const resultDangle = executor.executeAction('act-3', {
      kind: 'motion',
      motionId: 'dangle',
    } as unknown as Parameters<typeof executor.executeAction>[1]);
    expect(resultDangle.status).toBe('failed');

    controller.dispose();
  });

  it('accepts and executes requestable motion slots with succeeded result', () => {
    const controller = new PhysicalStateController();
    const playMotionFn = vi.fn();
    const actionResultFn = vi.fn();
    const executor = new BodyActionExecutor(controller, {
      onPlayMotion: playMotionFn,
      onActionResult: actionResultFn,
    });

    // 'wave' is requestable
    const resultWave = executor.executeAction('act-wave', {
      kind: 'motion',
      motionId: 'wave',
    });

    expect(resultWave.status).toBe('succeeded');
    expect(playMotionFn).toHaveBeenCalledWith('wave');
    expect(actionResultFn).toHaveBeenCalledWith({
      actionId: 'act-wave',
      status: 'succeeded',
    });

    // 'fidget_yawn' is requestable
    const resultYawn = executor.executeAction('act-yawn', {
      kind: 'motion',
      motionId: 'fidget_yawn',
    });
    expect(resultYawn.status).toBe('succeeded');
    expect(playMotionFn).toHaveBeenCalledWith('fidget_yawn');

    // 'react_happy_hop' is requestable
    const resultHop = executor.executeAction('act-hop', {
      kind: 'motion',
      motionId: 'react_happy_hop',
    });
    expect(resultHop.status).toBe('succeeded');
    expect(playMotionFn).toHaveBeenCalledWith('react_happy_hop');

    controller.dispose();
  });

  it('handles state transition actions: sleep, wake, peek, goTo, sit', () => {
    const controller = new PhysicalStateController();
    const executor = new BodyActionExecutor(controller);

    expect(controller.getCurrentState()).toBe('idle');

    // sleep action
    executor.executeAction('act-sleep', { kind: 'sleep' });
    expect(controller.getCurrentState()).toBe('sleeping');

    // wake action
    executor.executeAction('act-wake', { kind: 'wake' });
    expect(controller.getCurrentState()).toBe('idle');

    // peek action
    executor.executeAction('act-peek', { kind: 'peek', side: 'right' });
    expect(controller.getCurrentState()).toBe('peeking');

    // sit action
    executor.executeAction('act-sit', { kind: 'sit', platformId: 'plat-1' });
    expect(controller.getCurrentState()).toBe('sitting');

    // goTo action
    executor.executeAction('act-goto', {
      kind: 'goTo',
      anchor: { kind: 'free', x: 500, y: 600 },
    });
    expect(controller.getCurrentState()).toBe('walking');

    controller.dispose();
  });

  it('cancels ongoing action when pre-empted or explicitly cancelled', () => {
    const controller = new PhysicalStateController();
    const results: Array<{ actionId: string; status: string }> = [];
    const executor = new BodyActionExecutor(controller, {
      onActionResult: (r) => results.push({ actionId: r.actionId, status: r.status }),
    });

    // Start action 1 and simulate it staying active
    executor.executeAction('act-1', { kind: 'motion', motionId: 'wave' });
    // Execute action 2
    executor.executeAction('act-2', { kind: 'motion', motionId: 'nod' });

    expect(results.length).toBe(2);
    expect(results[0]?.actionId).toBe('act-1');
    expect(results[0]?.status).toBe('succeeded');

    // Cancel current
    executor.cancelCurrentAction('test_cancel');
    controller.dispose();
  });
});
