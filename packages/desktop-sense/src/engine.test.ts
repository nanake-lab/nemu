import { expect, test, vi } from 'vitest';
import { SenseEngine, startSenseLoop, type SenseSink } from './engine.js';
import { fixtureDesktop, fixtureWindow } from './fixtures.js';
import type { DesktopSource, DesktopSample, DesktopObservation } from './types.js';
import { appBusPayloads } from '@nemu/contracts';

function setup() {
  let sample = fixtureDesktop();
  const observations: DesktopObservation[] = [],
    body = vi.fn(),
    bus = vi.fn();
  const sink: SenseSink = {
    body,
    bus: (type, payload) => {
      bus(type, payload);
      if (type === 'desktop.observation')
        observations.push(appBusPayloads['desktop.observation'].parse(payload));
    },
  };
  const source: DesktopSource = {
    sample: vi.fn(() => sample),
    readTitle: vi.fn(() => 'SYNTHETIC SECRET'),
    subscribe: vi.fn(() => vi.fn()),
    dispose: vi.fn(),
  };
  return {
    source,
    sink,
    observations,
    body,
    bus,
    set: (value: DesktopSample) => {
      sample = value;
    },
  };
}
test('title read is denied by default; revocation is checked both before reading and before emission', () => {
  const t = setup(),
    granted = vi.fn(() => false);
  const engine = new SenseEngine(
    t.source,
    { idleThresholdMs: 1000, titleAuthorized: granted },
    t.sink,
  );
  engine.tick();
  expect(t.source.readTitle).not.toHaveBeenCalled();
  expect(JSON.stringify(t.observations)).not.toContain('SECRET');
  granted.mockReturnValue(true);
  t.set(fixtureDesktop({ foreground: '101', windows: [fixtureWindow({ hwnd: '101' })] }));
  engine.tick();
  expect(t.observations.at(-1)).toMatchObject({
    kind: 'foreground_changed',
    title: 'SYNTHETIC SECRET',
  });
  t.source.readTitle = vi.fn(() => {
    granted.mockReturnValue(false);
    return 'REVOKED SECRET';
  });
  t.set(fixtureDesktop({ foreground: '102', windows: [fixtureWindow({ hwnd: '102' })] }));
  engine.tick();
  expect(JSON.stringify(t.observations)).not.toContain('REVOKED');
});
test('core-bound observations use an allowlist; geometry, handles and paths never go on the bus', () => {
  const t = setup();
  t.set(fixtureDesktop({ caret: { x: 6789, y: 3210, w: 2, h: 20 } }));
  const engine = new SenseEngine(t.source, { idleThresholdMs: 1000 }, t.sink);
  engine.tick();
  expect(t.body).toHaveBeenCalledWith(
    expect.objectContaining({ caret: expect.any(Object), platforms: expect.any(Array) }),
  );
  for (const payload of t.observations)
    expect(
      Object.keys(payload).every((k) =>
        ['kind', 'processName', 'title', 'dnd', 'idleMs'].includes(k),
      ),
    ).toBe(true);
  expect(JSON.stringify(t.bus.mock.calls)).not.toMatch(
    /platforms|caret|bounds|hwnd|windowClass|dpi|6789/,
  );
  // Wider untrusted snapshots are rejected rather than spread into any event.
  t.source.sample = () => ({ ...fixtureDesktop(), surprise: 'content' }) as DesktopSample;
  expect(() => engine.tick()).toThrow();
});
test('low-frequency transitions emit once; geometry motion does not create semantic history', () => {
  const t = setup(),
    engine = new SenseEngine(t.source, { idleThresholdMs: 1000 }, t.sink);
  engine.tick();
  const initial = t.observations.length;
  for (let i = 0; i < 100; i++) {
    t.set(
      fixtureDesktop({ windows: [fixtureWindow({ bounds: { x: i, y: 100, w: 600, h: 400 } })] }),
    );
    engine.tick();
  }
  expect(t.observations).toHaveLength(initial);
  t.set(fixtureDesktop({ idleMs: 1000 }));
  engine.tick();
  engine.tick();
  t.set(fixtureDesktop({ idleMs: 0 }));
  engine.tick();
  expect(t.observations.filter((o) => o.kind.startsWith('idle_')).map((o) => o.kind)).toEqual([
    'idle_started',
    'idle_ended',
  ]);
  t.set(
    fixtureDesktop({
      windows: [fixtureWindow({ processName: 'zoom.exe', windowClass: 'ZPContentViewWndClass' })],
    }),
  );
  engine.tick();
  engine.tick();
  t.set(fixtureDesktop());
  engine.tick();
  expect(t.observations.filter((o) => o.kind.startsWith('meeting_')).map((o) => o.kind)).toEqual([
    'meeting_started',
    'meeting_ended',
  ]);
  engine.setLocked(true);
  engine.setLocked(true);
  const reads = vi.mocked(t.source.sample).mock.calls.length;
  engine.tick();
  expect(vi.mocked(t.source.sample)).toHaveBeenCalledTimes(reads);
  expect(t.body.mock.calls.at(-1)![0]).toEqual({ platforms: [], dnd: 'busy' });
  engine.setLocked(false);
  engine.tick();
  expect(t.observations.filter((o) => o.kind.startsWith('session_')).map((o) => o.kind)).toEqual([
    'session_locked',
    'session_unlocked',
  ]);
  expect(t.observations.every((o) => !/absen|expect|miss/.test(o.kind))).toBe(true);
});
test('unknown idle is not a fabricated return; fault clears geometry and gates busy', () => {
  const t = setup(),
    engine = new SenseEngine(t.source, { idleThresholdMs: 1000 }, t.sink);
  t.set(fixtureDesktop({ idleMs: 2000 }));
  engine.tick();
  t.set(fixtureDesktop({ idleMs: null }));
  engine.tick();
  expect(t.observations.filter((o) => o.kind === 'idle_ended')).toHaveLength(0);
  engine.failClosed();
  expect(t.body.mock.calls.at(-1)![0]).toEqual({ platforms: [], dnd: 'busy' });
});
test('title authorization revocation is not evidence that a meeting ended', () => {
  const t = setup();
  let authorized = true;
  t.source.readTitle = () => 'Synthetic meeting';
  t.set(fixtureDesktop({ windows: [fixtureWindow({ processName: 'ms-teams.exe' })] }));
  const engine = new SenseEngine(
    t.source,
    { idleThresholdMs: 1000, titleAuthorized: () => authorized },
    t.sink,
  );
  engine.tick();
  authorized = false;
  engine.tick();
  expect(t.observations.filter((o) => o.kind === 'meeting_ended')).toHaveLength(0);
  expect(t.body.mock.calls.at(-1)![0].dnd).toBe('meeting');
});
test('event storms coalesce; shutdown releases timers, hook and source exactly once', () => {
  vi.useFakeTimers();
  try {
    const t = setup();
    let invalidate = () => {};
    const unhook = vi.fn();
    t.source.subscribe = (fn) => {
      invalidate = fn;
      return unhook;
    };
    const engine = new SenseEngine(t.source, { idleThresholdMs: 1000 }, t.sink);
    const stop = startSenseLoop(t.source, engine, vi.fn());
    for (let i = 0; i < 10000; i++) invalidate();
    expect(t.source.sample).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(100);
    expect(t.source.sample).toHaveBeenCalledTimes(2);
    invalidate();
    stop();
    stop();
    vi.advanceTimersByTime(5000);
    expect(t.source.sample).toHaveBeenCalledTimes(2);
    expect(unhook).toHaveBeenCalledTimes(1);
    expect(t.source.dispose).toHaveBeenCalledTimes(1);
    expect(vi.getTimerCount()).toBe(0);
  } finally {
    vi.useRealTimers();
  }
});
