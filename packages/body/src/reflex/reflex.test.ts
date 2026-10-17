import { describe, it, expect, vi } from 'vitest';
import { ReflexManager } from './reflex-manager.js';

describe('ReflexManager', () => {
  it('triggers nod and user gaze within <= 300ms following speechEnd (完成标准第 8 项)', () => {
    let mockTime = 1000;
    const gazeFn = vi.fn();
    const gestureFn = vi.fn();
    const latencySamples: number[] = [];

    const reflex = new ReflexManager({
      speechEndResponseMs: 180, // Target 180ms
      clock: () => mockTime,
      onGaze: gazeFn,
      onGesture: gestureFn,
      onLatencySample: (_, elapsed) => latencySamples.push(elapsed),
    });

    // Event: voice.vad speechEnd at t = 1000
    reflex.handleSpeechEnd(1000);

    // At t = 1100 (100ms elapsed) -> not yet triggered
    mockTime = 1100;
    reflex.update(0.1);
    expect(gazeFn).not.toHaveBeenCalled();
    expect(gestureFn).not.toHaveBeenCalled();

    // At t = 1200 (200ms elapsed, > 180ms scheduled target) -> triggers
    mockTime = 1200;
    reflex.update(0.1);

    expect(gazeFn).toHaveBeenCalledWith('user', 2500);
    expect(gestureFn).toHaveBeenCalledWith('nod', 0.8);
    expect(latencySamples.length).toBe(1);

    // Critical Acceptance Requirement: response latency <= 300 ms
    const measuredLatency = latencySamples[0]!;
    expect(measuredLatency).toBeLessThanOrEqual(300);
    expect(measuredLatency).toBe(200);
  });

  it('triggers ear flick and scratch gesture when receiving pat interaction', () => {
    const earFlickFn = vi.fn();
    const gestureFn = vi.fn();

    const reflex = new ReflexManager({
      onEarFlick: earFlickFn,
      onGesture: gestureFn,
    });

    reflex.handleHit({ region: 'head', kind: 'pat' });
    expect(earFlickFn).toHaveBeenCalled();
    expect(gestureFn).toHaveBeenCalledWith('fidget_ear_scratch', 0.8);

    reflex.handleHit({ region: 'ears', kind: 'click' });
    expect(earFlickFn).toHaveBeenCalledTimes(2);
  });

  it('triggers react_surprised on high velocity fling release (> 1200 px/s)', () => {
    const gestureFn = vi.fn();

    const reflex = new ReflexManager({
      flingVelocityThresholdPx: 1200,
      onGesture: gestureFn,
    });

    // Slow release: vx = 200, vy = 200 -> speed ~282 px/s
    reflex.handleDrag({ phase: 'end', vx: 200, vy: 200 });
    expect(gestureFn).not.toHaveBeenCalled();

    // Fast fling: vx = 1000, vy = 1000 -> speed ~1414 px/s >= 1200 px/s
    reflex.handleDrag({ phase: 'end', vx: 1000, vy: 1000 });
    expect(gestureFn).toHaveBeenCalledWith('react_surprised', 1.0);
  });

  it('adjusts gaze behavior: cursor when idle, user when speaking', () => {
    const gazeFn = vi.fn();

    const reflex = new ReflexManager({
      onGaze: gazeFn,
    });

    reflex.handleStateChange('speaking');
    expect(gazeFn).toHaveBeenCalledWith('user');

    reflex.handleStateChange('idle');
    expect(gazeFn).toHaveBeenCalledWith('cursor');
  });
});
