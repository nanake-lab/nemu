import { expect, test, vi } from 'vitest';
import { EventEmitter } from 'node:events';
import { startDesktopSense } from './index.js';
import { AppBus } from '../bus/bus.js';
import { fixtureDesktop } from '../../../../packages/desktop-sense/src/fixtures.js';

test('desktop wiring routes geometry only to stage and removes all session listeners on close', () => {
  vi.useFakeTimers();
  try {
    const source = {
      sample: vi.fn(fixtureDesktop),
      readTitle: vi.fn(),
      subscribe: vi.fn(() => vi.fn()),
      dispose: vi.fn(),
    };
    const bus = new AppBus({ enableValidation: true });
    const publish = vi.spyOn(bus, 'publish'),
      stage = vi.fn(),
      session = new EventEmitter();
    const stop = startDesktopSense({
      source,
      settings: { idleThresholdMs: 1000 },
      bus,
      sendStage: stage,
      session,
      onFault: vi.fn(),
    });
    expect(stage).toHaveBeenCalledWith(
      'stage.desktop.update',
      expect.objectContaining({ platforms: expect.any(Array) }),
    );
    expect(JSON.stringify(publish.mock.calls)).not.toMatch(/platforms|caret|hwnd|bounds/);
    const sent = stage.mock.calls.length;
    bus.publish(
      bus.createEvent('stage', 'stage.ready', {
        avatarName: 'synthetic',
        boundsPx: { x: 0, y: 0, w: 10, h: 10 },
      }),
    );
    expect(stage).toHaveBeenCalledTimes(sent + 1);
    session.emit('lock-screen');
    session.emit('unlock-screen');
    stop();
    stop();
    expect(session.eventNames()).toEqual([]);
    expect(vi.getTimerCount()).toBe(0);
    expect(source.dispose).toHaveBeenCalledTimes(1);
  } finally {
    vi.useRealTimers();
  }
});

test('startup hook failure reaches the shell catch after disposing the source without timers', () => {
  vi.useFakeTimers();
  try {
    const source = {
      sample: vi.fn(fixtureDesktop),
      readTitle: vi.fn(),
      subscribe: vi.fn(() => {
        throw new Error('synthetic hook failure');
      }),
      dispose: vi.fn(),
    };
    const session = new EventEmitter();
    expect(() =>
      startDesktopSense({
        source,
        settings: { idleThresholdMs: 1000 },
        bus: new AppBus(),
        sendStage: vi.fn(),
        session,
        onFault: vi.fn(),
      }),
    ).toThrow('synthetic hook failure');
    expect(source.dispose).toHaveBeenCalledTimes(1);
    expect(session.eventNames()).toEqual([]);
    expect(vi.getTimerCount()).toBe(0);
  } finally {
    vi.useRealTimers();
  }
});
