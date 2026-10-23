// Narrow native binding surface, verified against sherpa-onnx-node 1.13.8.
declare module 'sherpa-onnx-node' {
  export interface NativeStream {
    acceptWaveform(wave: { sampleRate: number; samples: Float32Array }): void;
  }
  export interface NativeResult {
    text: string;
    lang: string;
  }
  export class OfflineRecognizer {
    constructor(config: unknown);
    createStream(): NativeStream;
    decodeAsync(stream: NativeStream): Promise<NativeResult>;
  }
  export class Vad {
    constructor(config: unknown, bufferSeconds: number);
    acceptWaveform(samples: Float32Array): void;
    isDetected(): boolean;
    isEmpty(): boolean;
    front(copy?: boolean): { start: number; samples: Float32Array };
    pop(): void;
    flush(): void;
    reset(): void;
  }
  const native: {
    OfflineRecognizer: typeof OfflineRecognizer;
    Vad: typeof Vad;
    readWave: typeof readWave;
  };
  export default native;
  export function readWave(path: string): { sampleRate: number; samples: Float32Array };
}
