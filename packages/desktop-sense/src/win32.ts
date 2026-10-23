import { createRequire } from 'node:module';
import type * as Koffi from 'koffi';

const require = createRequire(import.meta.url);
export function loadKoffi(): typeof Koffi {
  return require('koffi') as typeof Koffi;
}

let cached: ReturnType<typeof loadBindings> | undefined;

/** Loading DLLs does not enumerate windows or read user context. */
export function win32Bindings(): ReturnType<typeof loadBindings> {
  if (process.platform !== 'win32') throw new Error('Desktop sensing requires Windows');
  return (cached ??= loadBindings());
}

function loadBindings() {
  const koffi = loadKoffi();
  const user = koffi.load('user32.dll'),
    kernel = koffi.load('kernel32.dll');
  const shell = koffi.load('shell32.dll'),
    dwm = koffi.load('dwmapi.dll'),
    shcore = koffi.load('shcore.dll');
  const rect = koffi.struct('NemuSenseRect', {
    left: 'int32_t',
    top: 'int32_t',
    right: 'int32_t',
    bottom: 'int32_t',
  });
  koffi.struct('NemuSensePoint', { x: 'int32_t', y: 'int32_t' });
  const monitor = koffi.struct('NemuSenseMonitor', {
    cbSize: 'uint32_t',
    rcMonitor: rect,
    rcWork: rect,
    flags: 'uint32_t',
  });
  const lastInput = koffi.struct('NemuSenseLastInput', { cbSize: 'uint32_t', dwTime: 'uint32_t' });
  const gui = koffi.struct('NemuSenseGui', {
    cbSize: 'uint32_t',
    flags: 'uint32_t',
    hwndActive: 'uintptr_t',
    hwndFocus: 'uintptr_t',
    hwndCapture: 'uintptr_t',
    hwndMenuOwner: 'uintptr_t',
    hwndMoveSize: 'uintptr_t',
    hwndCaret: 'uintptr_t',
    rcCaret: rect,
  });
  const appbar = koffi.struct('NemuSenseAppBar', {
    cbSize: 'uint32_t',
    hwnd: 'uintptr_t',
    callback: 'uint32_t',
    edge: 'uint32_t',
    rc: rect,
    param: 'intptr_t',
  });
  koffi.proto('int __stdcall NemuSenseEnumWindow(uintptr_t hwnd, intptr_t param)');
  koffi.proto(
    'int __stdcall NemuSenseEnumMonitor(uintptr_t monitor, uintptr_t dc, void *rect, intptr_t param)',
  );
  const winEvent = koffi.proto(
    'void __stdcall NemuSenseWinEvent(uintptr_t hook, uint32_t event, uintptr_t hwnd, int32_t object, int32_t child, uint32_t thread, uint32_t time)',
  );
  return {
    sizes: {
      monitor: koffi.sizeof(monitor),
      lastInput: koffi.sizeof(lastInput),
      gui: koffi.sizeof(gui),
      appbar: koffi.sizeof(appbar),
    },
    winEvent,
    enumWindows: user.func(
      'int __stdcall EnumWindows(NemuSenseEnumWindow *callback, intptr_t param)',
    ),
    enumMonitors: user.func(
      'int __stdcall EnumDisplayMonitors(uintptr_t dc, void *clip, NemuSenseEnumMonitor *callback, intptr_t param)',
    ),
    visible: user.func('int __stdcall IsWindowVisible(uintptr_t hwnd)'),
    iconic: user.func('int __stdcall IsIconic(uintptr_t hwnd)'),
    frame: dwm.func(
      'int32_t __stdcall DwmGetWindowAttribute(uintptr_t hwnd, uint32_t attr, _Out_ NemuSenseRect *rect, uint32_t size)',
    ),
    cloaked: dwm.func(
      'int32_t __stdcall DwmGetWindowAttribute(uintptr_t hwnd, uint32_t attr, _Out_ uint32_t *value, uint32_t size)',
    ),
    threadProcess: user.func(
      'uint32_t __stdcall GetWindowThreadProcessId(uintptr_t hwnd, _Out_ uint32_t *pid)',
    ),
    openProcess: kernel.func(
      'uintptr_t __stdcall OpenProcess(uint32_t access, int inherit, uint32_t pid)',
    ),
    imageName: kernel.func(
      'int __stdcall QueryFullProcessImageNameW(uintptr_t process, uint32_t flags, _Out_ char16_t *buffer, _Inout_ uint32_t *size)',
    ),
    closeHandle: kernel.func('int __stdcall CloseHandle(uintptr_t handle)'),
    className: user.func(
      'int __stdcall GetClassNameW(uintptr_t hwnd, _Out_ char16_t *buffer, int count)',
    ),
    title: user.func(
      'int __stdcall GetWindowTextW(uintptr_t hwnd, _Out_ char16_t *buffer, int count)',
    ),
    foreground: user.func('uintptr_t __stdcall GetForegroundWindow()'),
    monitorInfo: user.func(
      'int __stdcall GetMonitorInfoW(uintptr_t monitor, _Inout_ NemuSenseMonitor *info)',
    ),
    dpi: shcore.func(
      'int32_t __stdcall GetDpiForMonitor(uintptr_t monitor, int type, _Out_ uint32_t *x, _Out_ uint32_t *y)',
    ),
    appbar: shell.func(
      'uintptr_t __stdcall SHAppBarMessage(uint32_t message, _Inout_ NemuSenseAppBar *data)',
    ),
    notification: shell.func(
      'int32_t __stdcall SHQueryUserNotificationState(_Out_ uint32_t *state)',
    ),
    lastInput: user.func('int __stdcall GetLastInputInfo(_Inout_ NemuSenseLastInput *info)'),
    tickCount: kernel.func('uint32_t __stdcall GetTickCount()'),
    gui: user.func('int __stdcall GetGUIThreadInfo(uint32_t thread, _Inout_ NemuSenseGui *info)'),
    clientToScreen: user.func(
      'int __stdcall ClientToScreen(uintptr_t hwnd, _Inout_ NemuSensePoint *point)',
    ),
    toPhysical: user.func(
      'int __stdcall LogicalToPhysicalPointForPerMonitorDPI(uintptr_t hwnd, _Inout_ NemuSensePoint *point)',
    ),
    windowDpiContext: user.func('intptr_t __stdcall GetWindowDpiAwarenessContext(uintptr_t hwnd)'),
    threadDpiContext: user.func(
      'intptr_t __stdcall SetThreadDpiAwarenessContext(intptr_t context)',
    ),
    hook: user.func(
      'uintptr_t __stdcall SetWinEventHook(uint32_t min, uint32_t max, uintptr_t module, NemuSenseWinEvent *callback, uint32_t process, uint32_t thread, uint32_t flags)',
    ),
    unhook: user.func('int __stdcall UnhookWinEvent(uintptr_t hook)'),
  };
}

export function withPhysicalPixels<T>(api: ReturnType<typeof win32Bindings>, work: () => T): T {
  // PMv2 affects this thread only and is restored synchronously; no OS/user settings.
  const old: number | bigint = api.threadDpiContext(-4);
  if (!old) throw new Error('PMv2 context unavailable');
  let result: T;
  try {
    result = work();
  } catch (error) {
    api.threadDpiContext(old);
    throw error;
  }
  if (!api.threadDpiContext(old)) throw new Error('DPI context restore failed');
  return result;
}

import type { Rect } from '@nemu/contracts';
import {
  desktopSampleSchema,
  type DesktopSample,
  type DesktopSource,
  type WindowSample,
} from './types.js';

type Handle = number | bigint;
const emptyRect = () => ({ left: 0, top: 0, right: 0, bottom: 0 });
const toRect = (r: ReturnType<typeof emptyRect>): Rect => ({
  x: r.left,
  y: r.top,
  w: r.right - r.left,
  h: r.bottom - r.top,
});
const text = (buffer: Buffer, length: number) =>
  buffer.toString('utf16le', 0, Math.max(0, length) * 2);

export function idleElapsed(now: number, last: number): number {
  return (now - last) >>> 0; // DWORD clock wraps after 49.7 days.
}

/** Production source. Tests never call sample() against the user's desktop. */
export function createWin32Source(
  options: { titleAuthorized?: () => boolean; excludeProcessId?: number } = {},
): DesktopSource {
  return createSourceWithBindings(win32Bindings(), options);
}

/** Dependency seam for restricted synthetic-window acceptance; never exported by package entry. */
export function createSourceWithBindings(
  api: ReturnType<typeof win32Bindings>,
  options: { titleAuthorized?: () => boolean; excludeProcessId?: number } = {},
): DesktopSource {
  const koffi = loadKoffi();
  let disposed = false;
  const subscriptions = new Set<() => void>();
  function processName(hwnd: Handle): string | undefined {
    const pid = [0];
    if (!api.threadProcess(hwnd, pid) || pid[0] === (options.excludeProcessId ?? process.pid))
      return undefined;
    const handle: Handle = api.openProcess(0x1000, 0, pid[0]);
    if (!handle) return undefined;
    try {
      const buffer = Buffer.alloc(65536),
        size = [32768];
      if (!api.imageName(handle, 0, buffer, size)) return undefined;
      return text(buffer, size[0]!).split(/[\\/]/).at(-1);
    } finally {
      api.closeHandle(handle);
    }
  }
  function caret(): Rect | undefined {
    const info = {
      cbSize: api.sizes.gui,
      flags: 0,
      hwndActive: 0,
      hwndFocus: 0,
      hwndCapture: 0,
      hwndMenuOwner: 0,
      hwndMoveSize: 0,
      hwndCaret: 0,
      rcCaret: emptyRect(),
    };
    if (!api.gui(0, info) || !info.hwndCaret || !(info.flags & 1)) return undefined;
    // rcCaret is in the target window's logical coordinates, not the caller's DPI.
    const old: Handle = api.threadDpiContext(api.windowDpiContext(info.hwndCaret));
    if (!old) return undefined;
    try {
      const a = { x: info.rcCaret.left, y: info.rcCaret.top },
        b = { x: info.rcCaret.right, y: info.rcCaret.bottom };
      if (
        !api.clientToScreen(info.hwndCaret, a) ||
        !api.clientToScreen(info.hwndCaret, b) ||
        !api.toPhysical(info.hwndCaret, a) ||
        !api.toPhysical(info.hwndCaret, b)
      )
        return undefined;
      if (b.x < a.x || b.y < a.y) return undefined;
      return { x: a.x, y: a.y, w: b.x - a.x, h: b.y - a.y };
    } finally {
      api.threadDpiContext(old);
    }
  }
  return {
    sample() {
      if (disposed) throw new Error('Sense source disposed');
      return withPhysicalPixels(api, () => {
        const windows: WindowSample[] = [],
          monitors: DesktopSample['monitors'] = [],
          taskbars: DesktopSample['taskbars'] = [];
        let z = 0;
        if (
          !api.enumWindows((hwnd: Handle) => {
            const order = z++;
            if (!api.visible(hwnd)) return 1;
            const pid = [0];
            if (
              !api.threadProcess(hwnd, pid) ||
              pid[0] === (options.excludeProcessId ?? process.pid)
            )
              return 1;
            const cloak = [0],
              bounds = emptyRect();
            if (
              api.cloaked(hwnd, 14, cloak, 4) !== 0 ||
              cloak[0] !== 0 ||
              api.frame(hwnd, 9, bounds, 16) !== 0
            )
              return 1;
            const r = toRect(bounds);
            if (r.w <= 0 || r.h <= 0) return 1;
            const buffer = Buffer.alloc(512),
              count: number = api.className(hwnd, buffer, 256);
            const windowClass = text(buffer, count),
              id = String(hwnd);
            if (windowClass === 'Shell_TrayWnd' || windowClass === 'Shell_SecondaryTrayWnd') {
              taskbars.push({ hwnd: id, bounds: r });
              return 1;
            }
            if (['Progman', 'WorkerW'].includes(windowClass)) return 1;
            const name = processName(hwnd);
            windows.push({
              hwnd: id,
              windowClass,
              bounds: r,
              visible: true,
              minimized: Boolean(api.iconic(hwnd)),
              cloaked: false,
              z: order,
              ...(name ? { processName: name } : {}),
            });
            return 1;
          }, 0)
        )
          throw new Error('Window enumeration failed');
        if (
          !api.enumMonitors(
            0,
            null,
            (handle: Handle) => {
              const info = {
                cbSize: api.sizes.monitor,
                rcMonitor: emptyRect(),
                rcWork: emptyRect(),
                flags: 0,
              };
              if (!api.monitorInfo(handle, info)) return 1;
              const x = [96],
                y = [96];
              if (api.dpi(handle, 0, x, y) !== 0) {
                x[0] = 96;
                y[0] = 96;
              }
              monitors.push({
                id: String(handle),
                bounds: toRect(info.rcMonitor),
                workArea: toRect(info.rcWork),
                dpiX: x[0]!,
                dpiY: y[0]!,
              });
              return 1;
            },
            0,
          ) ||
          monitors.length === 0
        )
          throw new Error('Monitor enumeration failed');
        if (taskbars.length === 0) {
          const bar = {
            cbSize: api.sizes.appbar,
            hwnd: 0,
            callback: 0,
            edge: 0,
            rc: emptyRect(),
            param: 0,
          };
          if (api.appbar(5, bar) && toRect(bar.rc).w > 0 && toRect(bar.rc).h > 0)
            taskbars.push({ hwnd: String(bar.hwnd), bounds: toRect(bar.rc) });
        }
        const notification = [0],
          notificationState = api.notification(notification) === 0 ? notification[0]! : null;
        const input = { cbSize: api.sizes.lastInput, dwTime: 0 };
        const idleMs = api.lastInput(input)
          ? idleElapsed(api.tickCount() as number, input.dwTime)
          : null;
        const foreground: Handle = api.foreground(),
          caretRect = caret();
        return desktopSampleSchema.parse({
          windows,
          monitors,
          taskbars,
          foreground: foreground ? String(foreground) : null,
          ...(caretRect ? { caret: caretRect } : {}),
          idleMs,
          notificationState,
        });
      });
    },
    readTitle(hwnd) {
      if (disposed || options.titleAuthorized?.() !== true || !/^\d+$/.test(hwnd)) return undefined;
      const buffer = Buffer.alloc(8192),
        count: number = api.title(BigInt(hwnd), buffer, 4096);
      if (options.titleAuthorized?.() !== true) return undefined;
      return text(buffer, count) || undefined;
    },
    subscribe(invalidate) {
      if (disposed) throw new Error('Sense source disposed');
      const callback = koffi.register(
        (_hook: Handle, event: number, _hwnd: Handle, object: number) => {
          if (event === 3 || object === 0 || object === -8) invalidate();
        },
        koffi.pointer(api.winEvent),
      );
      const hooks: Handle[] = [];
      const stop = () => {
        for (const hook of hooks.splice(0)) api.unhook(hook);
        koffi.unregister(callback);
        subscriptions.delete(stop);
      };
      for (const [min, max] of [
        [3, 3],
        [0x8000, 0x800b],
        [0x8017, 0x8018],
      ]) {
        const hook: Handle = api.hook(min, max, 0, callback, 0, 0, 2); // OUTOFCONTEXT | SKIPOWNPROCESS
        if (!hook) {
          stop();
          throw new Error('WinEvent hook unavailable');
        }
        hooks.push(hook);
      }
      subscriptions.add(stop);
      return () => {
        if (subscriptions.has(stop)) stop();
      };
    },
    dispose() {
      if (disposed) return;
      for (const stop of subscriptions) stop();
      disposed = true;
    },
  };
}

/** Evidence-only fixture, intentionally absent from the public package exports.
 * Every content-bearing native query is restricted to windows owned by this fixture.
 */
export function createSyntheticWin32Fixture(count: number = 32): {
  source: DesktopSource;
  frames: { hwnd: string; expected: Rect; scale: number }[];
  dispose(): void;
} {
  if (!Number.isInteger(count) || count < 1 || count > 32) throw new Error('Invalid fixture size');
  const api = win32Bindings();
  const user = loadKoffi().load('user32.dll');
  const create = user.func(
    'uintptr_t __stdcall CreateWindowExW(uint32_t exStyle, const char16_t *className, const char16_t *title, uint32_t style, int x, int y, int w, int h, uintptr_t parent, uintptr_t menu, uintptr_t instance, void *param)',
  );
  const destroy = user.func('int __stdcall DestroyWindow(uintptr_t hwnd)');
  const owned = new Set<string>();
  const frames: { hwnd: string; expected: Rect; scale: number }[] = [];
  const cleanup = () => {
    for (const hwnd of owned) {
      if (!destroy(BigInt(hwnd))) throw new Error('Synthetic window cleanup failed');
      owned.delete(hwnd);
    }
  };
  try {
    for (let i = 0; i < count; i++) {
      const scale = [1, 1.25, 1.5, 2][i % 4]!;
      const expected = {
        x: (i % 8 < 4 ? -30000 : -14000) + (i % 4) * 1600,
        y: -10000 + Math.floor(i / 8) * 1800,
        w: 600 * scale,
        h: 400 * scale,
      };
      const hwnd: Handle = withPhysicalPixels(api, () =>
        create(
          0x08000080,
          'STATIC',
          'Nemu synthetic window',
          0x90000000,
          expected.x,
          expected.y,
          expected.w,
          expected.h,
          0,
          0,
          0,
          null,
        ),
      );
      if (!hwnd) throw new Error('Synthetic window creation failed');
      owned.add(String(hwnd));
      frames.push({ hwnd: String(hwnd), expected, scale });
    }
  } catch (error) {
    cleanup();
    throw error;
  }
  const restricted = {
    ...api,
    // Native enumeration itself is measured, but no other window is inspected.
    enumWindows: ((callback: (hwnd: Handle) => number) =>
      api.enumWindows(
        (hwnd: Handle) => (owned.has(String(hwnd)) ? callback(hwnd) : 1),
        0,
      )) as typeof api.enumWindows,
    enumMonitors: ((...args: unknown[]) => {
      (args[2] as (h: number) => void)(1);
      (args[2] as (h: number) => void)(2);
      return 1;
    }) as typeof api.enumMonitors,
    monitorInfo: ((handle: number, out: Record<string, unknown>) => {
      const r = {
        left: handle === 1 ? -32000 : -16000,
        top: -14000,
        right: handle === 1 ? -16000 : 0,
        bottom: 6000,
      };
      Object.assign(out, { rcMonitor: r, rcWork: r });
      return 1;
    }) as typeof api.monitorInfo,
    dpi: ((_h: unknown, _type: unknown, x: number[], y: number[]) => {
      x[0] = 96;
      y[0] = 96;
      return 0;
    }) as typeof api.dpi,
    appbar: (() => 0) as typeof api.appbar,
    notification: ((out: number[]) => {
      out[0] = 5;
      return 0;
    }) as typeof api.notification,
    lastInput: (() => 0) as typeof api.lastInput,
    foreground: (() => BigInt(frames[0]!.hwnd)) as typeof api.foreground,
    gui: (() => 0) as typeof api.gui,
  };
  const source = createSourceWithBindings(restricted, { excludeProcessId: -1 });
  return {
    source,
    frames,
    dispose() {
      try {
        source.dispose();
      } finally {
        cleanup();
      }
    },
  };
}

/** Loads the binary and makes one query without reading any desktop context. */
export function probeNativeRuntime(): {
  versions: NodeJS.ProcessVersions;
  binaries: string[];
  tick: number;
} {
  const api = win32Bindings();
  return {
    versions: process.versions,
    binaries: Object.keys(require.cache).filter(
      (path) => path.endsWith('.node') && path.includes('koffi'),
    ),
    tick: api.tickCount() as number,
  };
}
