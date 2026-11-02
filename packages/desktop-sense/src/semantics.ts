import type { DndState } from '@nemu/contracts';
import type { DesktopSample, WindowSample } from './types.js';

export function isMeeting(window: WindowSample, title?: string): boolean {
  const process = window.processName?.toLowerCase();
  if (
    process === 'zoom.exe' &&
    /^(ZPContentViewWndClass|ZPPTMainFrmWndClassEx)$/.test(window.windowClass)
  )
    return true;
  if (!title) return false;
  if (process === 'ms-teams.exe' || process === 'teams.exe') {
    return /(?:meeting|会议|会議|call|通话|通話)/i.test(title);
  }
  if (process === 'zoom.exe') return /(?:zoom meeting|zoom 会议|zoom ミーティング)/i.test(title);
  return (
    /^(chrome|msedge|firefox)\.exe$/.test(process ?? '') &&
    /(?:^|[—–| -])Meet\s*[-–—]\s*[a-z]{3}-[a-z]{4}-[a-z]{3}\b/i.test(title)
  );
}

export function detectDnd(
  sample: DesktopSample,
  meeting: boolean,
  quietHours: boolean,
  locked: boolean,
): DndState {
  if (quietHours) return 'quietHours';
  if (locked) return 'busy';
  if (meeting) return 'meeting';
  const front = sample.windows.find(
    (w) => w.hwnd === sample.foreground && w.visible && !w.cloaked && !w.minimized,
  );
  if (
    sample.notificationState === 4 ||
    (front?.processName?.toLowerCase() === 'powerpnt.exe' && front.windowClass === 'screenClass')
  )
    return 'presentation';
  if (
    front &&
    sample.monitors.some(
      (m) =>
        Math.abs(front.bounds.x - m.bounds.x) <= 2 &&
        Math.abs(front.bounds.y - m.bounds.y) <= 2 &&
        Math.abs(front.bounds.w - m.bounds.w) <= 2 &&
        Math.abs(front.bounds.h - m.bounds.h) <= 2,
    )
  )
    return 'fullscreen';
  if (sample.notificationState === 3) return 'fullscreen';
  // Query failure and Windows' blocked notification states are conservative gates.
  if (sample.notificationState === null || [1, 2, 6].includes(sample.notificationState))
    return 'busy';
  return 'none';
}
