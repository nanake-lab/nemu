import { expect, it } from 'vitest';
import { BargeWindowGate, type WindowDetector } from './barge-window-gate.js';
class Detector implements WindowDetector {
  calls: Float32Array[] = [];
  constructor(private readonly decisions: boolean[]) {}
  acceptWaveform(samples: Float32Array): void {
    this.calls.push(samples);
  }
  isDetected(): boolean {
    return this.decisions[this.calls.length - 1] ?? false;
  }
  isEmpty(): boolean {
    return true;
  }
  pop(): void {}
  reset(): void {
    this.calls = [];
  }
}
it('cannot qualify before 200 ms and observes seven confirmed windows by the 240 ms packet', () => {
  const detector = new Detector([false, true, true, true, true, true, true]);
  const gate = new BargeWindowGate(detector);
  for (let i = 0; i < 11; i++) expect(gate.feed(new Float32Array(320))).toBe(false);
  expect(gate.feed(new Float32Array(320))).toBe(true);
  expect(detector.calls.map((x) => x.length)).toEqual([576, 512, 512, 512, 512, 512, 512]);
});
it('does not count endpoint silence after a short burst toward sustained speech', () => {
  const gate = new BargeWindowGate(
    new Detector([false, true, false, false, false, false, false, false]),
  );
  for (let i = 0; i < 20; i++) expect(gate.feed(new Float32Array(320))).toBe(false);
});
it('does not combine distinct short bursts', () => {
  const gate = new BargeWindowGate(
    new Detector([false, true, true, false, false, true, true, false, false]),
  );
  for (let i = 0; i < 16; i++) expect(gate.feed(new Float32Array(320))).toBe(false);
});
it('reset discards both a confirmed run and buffered samples', () => {
  const detector = new Detector([false, true, true, true, true, true, true]);
  const gate = new BargeWindowGate(detector);
  for (let i = 0; i < 12; i++) gate.feed(new Float32Array(320));
  gate.reset();
  expect(gate.feed(new Float32Array(512))).toBe(false);
  expect(detector.calls).toHaveLength(0);
});
