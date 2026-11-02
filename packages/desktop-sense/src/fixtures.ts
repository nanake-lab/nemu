import type { DesktopSample, WindowSample } from './types.js';

/** Synthetic data only. This file performs no OS calls. */
export function fixtureWindow(overrides: Partial<WindowSample> = {}): WindowSample {
  return {
    hwnd: '100',
    processName: 'synthetic.exe',
    windowClass: 'SyntheticWindow',
    bounds: { x: 100, y: 100, w: 600, h: 400 },
    visible: true,
    minimized: false,
    cloaked: false,
    z: 0,
    ...overrides,
  };
}
export function fixtureDesktop(overrides: Partial<DesktopSample> = {}): DesktopSample {
  return {
    windows: [fixtureWindow()],
    monitors: [
      {
        id: 'synthetic-primary',
        bounds: { x: 0, y: 0, w: 2560, h: 1440 },
        workArea: { x: 0, y: 0, w: 2560, h: 1400 },
        dpiX: 96,
        dpiY: 96,
      },
    ],
    taskbars: [{ hwnd: '200', bounds: { x: 0, y: 1400, w: 2560, h: 40 } }],
    foreground: '100',
    idleMs: 0,
    notificationState: 5,
    ...overrides,
  };
}
