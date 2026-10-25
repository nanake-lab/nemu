import {
  appBusPayloads,
  controlCommandPayloads,
  type AppBusPayloads,
  type DndState,
} from '@nemu/contracts';
import { buildPlatforms } from './geometry.js';
import { detectDnd, isMeeting } from './semantics.js';
import {
  desktopSampleSchema,
  type DesktopGeometry,
  type DesktopObservation,
  type DesktopSource,
  type SenseSettings,
} from './types.js';

export interface SenseSink {
  body(payload: DesktopGeometry): void;
  bus<K extends 'desktop.dnd' | 'desktop.idle' | 'desktop.observation'>(
    type: K,
    payload: AppBusPayloads[K],
  ): void;
}

/** Only transient edge detection state. No persistence, inference or authority store. */
export class SenseEngine {
  private foreground: string | null = null;
  private idle: boolean | undefined;
  private meeting = false;
  private meetingWindows = new Set<string>();
  private dnd: DndState | undefined;
  private locked = false;
  private geometry: string | undefined;

  constructor(
    private source: DesktopSource,
    private settings: SenseSettings,
    private sink: SenseSink,
  ) {
    if (!Number.isFinite(settings.idleThresholdMs) || settings.idleThresholdMs <= 0)
      throw new Error('Invalid idle threshold');
  }

  private observe(payload: DesktopObservation): void {
    // Rebuild from an allowlist even if called with a structurally wider object.
    const safe: DesktopObservation = { kind: payload.kind };
    if (payload.processName !== undefined) safe.processName = payload.processName;
    if (payload.title !== undefined && this.settings.titleAuthorized?.() === true)
      safe.title = payload.title;
    if (payload.dnd !== undefined) safe.dnd = payload.dnd;
    if (payload.idleMs !== undefined) safe.idleMs = payload.idleMs;
    this.sink.bus('desktop.observation', appBusPayloads['desktop.observation'].parse(safe));
  }

  setLocked(locked: boolean): void {
    if (locked === this.locked) return;
    this.locked = locked;
    this.observe({ kind: locked ? 'session_locked' : 'session_unlocked' });
    if (locked) this.failClosed();
  }

  private sendBody(payload: DesktopGeometry): void {
    const safe = controlCommandPayloads['stage.desktop.update'].parse(payload);
    const next = JSON.stringify(safe);
    if (next !== this.geometry) {
      this.sink.body(safe);
      this.geometry = next;
    }
  }

  private sendDnd(state: DndState): void {
    if (state === this.dnd) return;
    this.sink.bus('desktop.dnd', { state });
    this.observe({ kind: 'dnd_changed', dnd: state });
    this.dnd = state;
  }

  failClosed(): void {
    this.sendDnd('busy');
    this.sendBody({ platforms: [], dnd: 'busy' });
  }

  tick(): void {
    if (this.locked) return;
    const sample = desktopSampleSchema.parse(this.source.sample());
    const front = sample.windows.find((w) => w.hwnd === sample.foreground);
    // No title read can occur before a current grant, including for meeting detection.
    const titles = new Map<string, string>();
    if (this.settings.titleAuthorized?.() === true) {
      for (const w of sample.windows) {
        if (!w.visible || w.cloaked) continue;
        if (
          w.hwnd !== front?.hwnd &&
          !/^(ms-teams|teams|zoom|chrome|msedge|firefox)\.exe$/i.test(w.processName ?? '')
        )
          continue;
        if (this.settings.titleAuthorized?.() !== true) break;
        const title = this.source.readTitle(w.hwnd);
        if (title !== undefined) titles.set(w.hwnd, title);
      }
    }
    const meetingWindows = new Set(
      sample.windows
        .filter(
          (w) =>
            w.visible &&
            !w.cloaked &&
            (isMeeting(
              w,
              this.settings.titleAuthorized?.() === true ? titles.get(w.hwnd) : undefined,
            ) ||
              (this.meetingWindows.has(`${w.hwnd}|${w.processName ?? ''}`) &&
                (this.settings.titleAuthorized?.() !== true || !titles.has(w.hwnd)))),
        )
        .map((w) => `${w.hwnd}|${w.processName ?? ''}`),
    );
    this.meetingWindows = meetingWindows;
    const meeting = meetingWindows.size > 0;
    if (sample.foreground !== this.foreground) {
      this.observe({
        kind: 'foreground_changed',
        ...(front?.processName === undefined ? {} : { processName: front.processName }),
        ...(front && titles.has(front.hwnd) ? { title: titles.get(front.hwnd)! } : {}),
      });
      this.foreground = sample.foreground;
    }
    if (sample.idleMs !== null) {
      const idle = sample.idleMs >= this.settings.idleThresholdMs;
      this.sink.bus('desktop.idle', { idleMs: sample.idleMs });
      if (this.idle !== idle && (this.idle !== undefined || idle))
        this.observe({ kind: idle ? 'idle_started' : 'idle_ended', idleMs: sample.idleMs });
      this.idle = idle;
    }
    if (meeting !== this.meeting) {
      this.observe({ kind: meeting ? 'meeting_started' : 'meeting_ended' });
      this.meeting = meeting;
    }
    const dnd = detectDnd(sample, meeting, this.settings.quietHours?.() === true, false);
    this.sendDnd(dnd);
    this.sendBody({
      platforms: buildPlatforms(sample),
      ...(sample.caret ? { caret: sample.caret } : {}),
      dnd,
    });
  }
}

export function startSenseLoop(
  source: DesktopSource,
  engine: SenseEngine,
  onFault: () => void,
  options: { throttleMs?: number; pollMs?: number } = {},
): () => void {
  const throttleMs = options.throttleMs ?? 100,
    pollMs = options.pollMs ?? 1000;
  if (
    !Number.isFinite(throttleMs) ||
    throttleMs < 50 ||
    !Number.isFinite(pollMs) ||
    pollMs < throttleMs
  )
    throw new Error('Invalid sense cadence');
  let stopped = false,
    last = -Infinity;
  let pending: ReturnType<typeof setTimeout> | undefined;
  const run = () => {
    pending = undefined;
    if (stopped) return;
    last = performance.now();
    try {
      engine.tick();
    } catch {
      engine.failClosed();
      onFault();
    }
  };
  const invalidate = () => {
    if (stopped || pending !== undefined) return;
    pending = setTimeout(run, Math.max(0, throttleMs - (performance.now() - last)));
  };
  let unsubscribe: () => void;
  try {
    unsubscribe = source.subscribe(invalidate);
  } catch (error) {
    source.dispose();
    throw error;
  }
  const poll = setInterval(invalidate, pollMs);
  run();
  return () => {
    if (stopped) return;
    stopped = true;
    clearInterval(poll);
    if (pending !== undefined) clearTimeout(pending);
    try {
      unsubscribe();
    } finally {
      source.dispose();
    }
  };
}
