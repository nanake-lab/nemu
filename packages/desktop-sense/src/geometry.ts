import type { Platform, Rect } from '@nemu/contracts';
import type { DesktopSample } from './types.js';

function intersect(a: Rect, b: Rect): Rect | undefined {
  const x = Math.max(a.x, b.x),
    y = Math.max(a.y, b.y);
  const w = Math.min(a.x + a.w, b.x + b.w) - x;
  const h = Math.min(a.y + a.h, b.y + b.h) - y;
  return w > 0 && h > 0 ? { x, y, w, h } : undefined;
}

/** All inputs already are virtual-desktop physical pixels; never multiply by DPI. */
export function buildPlatforms(sample: DesktopSample): Platform[] {
  const result: Platform[] = [];
  const blockers: Rect[] = [];
  for (const window of sample.windows.toSorted((a, b) => a.z - b.z)) {
    if (!window.visible || window.minimized || window.cloaked) continue;
    const r = window.bounds;
    for (const monitor of sample.monitors) {
      const clip = intersect(r, monitor.bounds);
      if (!clip || clip.y !== r.y) continue;
      let spans: [number, number][] = [[clip.x, clip.x + clip.w]];
      for (const front of blockers) {
        if (r.y < front.y || r.y >= front.y + front.h) continue;
        spans = spans.flatMap(([a, b]) => {
          if (front.x >= b || front.x + front.w <= a) return [[a, b]];
          const pieces: [number, number][] = [];
          if (a < front.x) pieces.push([a, front.x]);
          if (front.x + front.w < b) pieces.push([front.x + front.w, b]);
          return pieces;
        });
      }
      for (const [span, [x1, x2]] of spans.entries())
        result.push({
          id: `window:${window.hwnd}:${monitor.id}:${span}`,
          hwnd: window.hwnd,
          x1,
          x2,
          y: r.y,
          z: window.z,
        });
    }
    blockers.push(r);
  }
  for (const bar of sample.taskbars) {
    // Vertical taskbars offer their horizontal top edge as well.
    result.push({
      id: `taskbar:${bar.hwnd}`,
      hwnd: bar.hwnd,
      x1: bar.bounds.x,
      x2: bar.bounds.x + bar.bounds.w,
      y: bar.bounds.y,
      z: 0,
    });
  }
  for (const monitor of sample.monitors)
    result.push({
      id: `screen:${monitor.id}`,
      hwnd: '',
      x1: monitor.bounds.x,
      x2: monitor.bounds.x + monitor.bounds.w,
      y: monitor.bounds.y + monitor.bounds.h,
      z: Math.max(-1, ...sample.windows.map((w) => w.z)) + 1,
    });
  return result;
}
