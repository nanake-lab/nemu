import { expect, test, vi } from 'vitest';
import { readFocusProbe } from './focus-probe.js';
const ok = (value: number, size = 4, status = 0) => ({ status, size, value, changeStamp: 7 });
test('only recognized Windows 10 builds call WNF; invalid formats and read failures stay unknown', () => {
  for (const value of [0, 1, 2])
    expect(
      readFocusProbe('10.0.19045', { windows10: () => ok(value), windows11: vi.fn() }).state,
    ).toBe(value ? 'active' : 'inactive');
  for (const raw of [ok(3), ok(0, 0), ok(0, 8), ok(0, 4, -1)])
    expect(readFocusProbe('10.0.19045', { windows10: () => raw, windows11: vi.fn() }).state).toBe(
      'unknown',
    );
  expect(
    readFocusProbe('10.0.19045', {
      windows10: () => {
        throw new Error('missing DLL');
      },
      windows11: vi.fn(),
    }).state,
  ).toBe('unknown');
  for (const version of ['10.0.19044', '10.0.19046', '11.0.1', 'invalid']) {
    const windows10 = vi.fn(),
      windows11 = vi.fn();
    expect(readFocusProbe(version, { windows10, windows11 }).state).toBe('unknown');
    expect(windows10).not.toHaveBeenCalled();
    expect(windows11).not.toHaveBeenCalled();
  }
});
test('Windows 11 only uses public FocusSessionManager; no WNF fallback even on failure', () => {
  const windows10 = vi.fn();
  expect(
    readFocusProbe('10.0.26100', {
      windows10,
      windows11: () => ({ state: 'active', source: 'focus-session-manager', reason: 'ok' }),
    }).state,
  ).toBe('active');
  expect(
    readFocusProbe('10.0.26100', {
      windows10,
      windows11: () => {
        throw new Error('unavailable');
      },
    }).state,
  ).toBe('unknown');
  expect(readFocusProbe('10.0.22621', { windows10, windows11: vi.fn() }).state).toBe('unknown');
  expect(windows10).not.toHaveBeenCalled();
});
