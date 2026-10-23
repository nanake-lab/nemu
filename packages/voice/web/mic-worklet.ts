/** Stateful area resampler; fractional phase survives 128-frame worklet quanta. */
export class PcmFramer {
  private ratio: number;
  private remaining: number;
  private sum = 0;
  private frame = new Int16Array(320);
  private index = 0;
  constructor(inputRate: number) {
    if (!Number.isFinite(inputRate) || inputRate < 16000)
      throw new Error('Input rate must be >= 16 kHz');
    this.ratio = inputRate / 16000;
    this.remaining = this.ratio;
  }
  push(input: Float32Array): Int16Array[] {
    const frames: Int16Array[] = [];
    for (const value of input) {
      let weight = 1;
      while (weight > 1e-9) {
        const used = Math.min(weight, this.remaining);
        this.sum += Math.max(-1, Math.min(1, value)) * used;
        this.remaining -= used;
        weight -= used;
        if (this.remaining < 1e-9) {
          const v = this.sum / this.ratio;
          this.frame[this.index++] = Math.round(v * (v < 0 ? 32768 : 32767));
          this.sum = 0;
          this.remaining = this.ratio;
          if (this.index === 320) {
            frames.push(this.frame);
            this.frame = new Int16Array(320);
            this.index = 0;
          }
        }
      }
    }
    return frames;
  }
}
// Bundlers strip TS before toString(). The worklet has no imports or Node access.
export function workletSource(): string {
  return `const PcmFramer = ${PcmFramer.toString()};\nregisterProcessor('nemu-mic', class extends AudioWorkletProcessor {
    constructor() { super(); this.framer = new PcmFramer(sampleRate); }
    process(inputs) {
      const input = inputs[0]?.[0];
      if (input) for (const pcm of this.framer.push(input)) this.port.postMessage(pcm, [pcm.buffer]);
      return true;
    }
  });`;
}
