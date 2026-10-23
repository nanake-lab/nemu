/** Candidate for offline diagnosis. The production provider does not use this gate. */
export interface WindowDetector {
  acceptWaveform(samples: Float32Array): void;
  isDetected(): boolean;
  isEmpty(): boolean;
  pop(): void;
  reset(): void;
}
/** Requires effective native zero-sample delays, window/shift=512 at 16 kHz.
 * The C API uses numeric 0 as a default sentinel; callers use 1e-6 seconds (0 samples).
 * Native onset then confirms two positive windows before isDetected can become true.
 * Count that confirmed window pair once; subsequent hysteresis-positive windows add 32 ms.
 * A negative window clears the run, so endpoint hangover cannot satisfy the 200 ms gate.
 */
export class BargeWindowGate {
  private pending = new Float32Array(0);
  private needed = 576;
  private detected = false;
  private confirmedSamples = 0;
  constructor(private readonly detector: WindowDetector) {}
  feed(samples: Float32Array): boolean {
    const joined = new Float32Array(this.pending.length + samples.length);
    joined.set(this.pending);
    joined.set(samples, this.pending.length);
    let offset = 0;
    while (joined.length - offset >= this.needed) {
      this.detector.acceptWaveform(joined.slice(offset, offset + this.needed));
      offset += this.needed;
      // The first 576 samples provide one complete window for both v4/v5.
      // Thereafter 512 samples advance exactly one native classification window.
      this.needed = 512;
      const next = this.detector.isDetected();
      this.confirmedSamples = next ? (this.detected ? this.confirmedSamples + 512 : 1024) : 0;
      this.detected = next;
      while (!this.detector.isEmpty()) this.detector.pop();
    }
    this.pending = joined.slice(offset);
    return this.confirmedSamples >= 3200;
  }
  reset(): void {
    this.detector.reset();
    this.pending = new Float32Array(0);
    this.needed = 576;
    this.detected = false;
    this.confirmedSamples = 0;
  }
}
