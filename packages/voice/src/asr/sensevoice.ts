import { randomUUID } from 'node:crypto';
import { join } from 'node:path';
import sherpa from 'sherpa-onnx-node';
import type { OfflineRecognizer } from 'sherpa-onnx-node';
const { OfflineRecognizer: NativeRecognizer, Vad } = sherpa;
import type { Lang } from '@nemu/contracts';
import {
  normalizeLanguage,
  type AsrProvider,
  type AsrSession,
  type AsrResult,
} from './provider.js';
export interface AsrHooks {
  onBargeIn?(): void;
  onMarks?(marks: {
    vadEnd: number;
    asrDone: number;
    durationMs: number;
    utteranceKey: string;
  }): void;
  onMappedLanguage?(raw: string, mapped: Lang): void;
  onError?(reason: string): void;
}
export const epochNow = () => performance.timeOrigin + performance.now();
export class SenseVoiceProvider implements AsrProvider {
  readonly id = 'sensevoice-int8';
  private recognizers = new Map<string, OfflineRecognizer>();
  private directory: string;
  private hooks: AsrHooks;
  private playing = new Set<string>();
  constructor(directory: string, hooks: AsrHooks = {}) {
    this.directory = directory;
    this.hooks = hooks;
    this.getRecognizer('auto');
  }
  private getRecognizer(language: 'auto' | Lang): OfflineRecognizer {
    const existing = this.recognizers.get(language);
    if (existing) return existing;
    const recognizer = new NativeRecognizer({
      featConfig: { sampleRate: 16000, featureDim: 80 },
      modelConfig: {
        senseVoice: {
          model: join(this.directory, 'model.int8.onnx'),
          language,
          useInverseTextNormalization: 1,
        },
        tokens: join(this.directory, 'tokens.txt'),
        numThreads: 2,
        provider: 'cpu',
        debug: 0,
      },
    });
    this.recognizers.set(language, recognizer);
    return recognizer;
  }
  resetPlayback(): void {
    this.playing.clear();
  }
  setPlayback(id: string, active: boolean): void {
    if (active) this.playing.add(id);
    else this.playing.delete(id);
  }
  async recognize(samples: Float32Array, language: 'auto' | Lang = 'auto'): Promise<AsrResult> {
    const recognizer = this.getRecognizer(language);
    const stream = recognizer.createStream();
    stream.acceptWaveform({ sampleRate: 16000, samples });
    const raw = await recognizer.decodeAsync(stream);
    const normalized = normalizeLanguage(raw.lang, raw.text);
    if (normalized.mapped) this.hooks.onMappedLanguage?.(raw.lang, normalized.lang);
    return { text: raw.text.replace(/<\|[^>]*\|>/g, '').trim(), lang: normalized.lang };
  }
  start(options: { lang: 'auto' | Lang; mode: 'pushToTalk' | 'continuous' }): AsrSession {
    const vad = new Vad(
      {
        sileroVad: {
          model: join(this.directory, 'silero_vad.onnx'),
          threshold: 0.5,
          minSilenceDuration: 0.2,
          minSpeechDuration: 0.032,
          windowSize: 512,
          maxSpeechDuration: 20,
        },
        sampleRate: 16000,
        numThreads: 1,
        provider: 'cpu',
        debug: 0,
      },
      30,
    );
    const bargeVad = new Vad(
      {
        sileroVad: {
          model: join(this.directory, 'silero_vad.onnx'),
          threshold: 0.5,
          minSilenceDuration: 0.032,
          minSpeechDuration: 0.2,
          windowSize: 512,
          maxSpeechDuration: 20,
        },
        sampleRate: 16000,
        numThreads: 1,
        provider: 'cpu',
        debug: 0,
      },
      30,
    );
    const vadListeners: Array<Parameters<AsrSession['onVad']>[0]> = [];
    const resultListeners: Array<(r: AsrResult) => void> = [];
    let closed = false,
      detected = false,
      bargeSent = false,
      playbackWasActive = false,
      inFlight = 0;
    const emitVad = (state: 'speechStart' | 'speechEnd') => vadListeners.forEach((cb) => cb(state));
    const drain = () => {
      while (!vad.isEmpty()) {
        const samples = new Float32Array(vad.front(false).samples);
        vad.pop();
        const vadEnd = epochNow();
        const utteranceKey = randomUUID();
        if (inFlight >= 2) {
          this.hooks.onError?.('ASR_QUEUE_FULL');
          continue;
        }
        inFlight++;
        void this.recognize(samples, options.lang)
          .then((result) => {
            const metadata = {
              vadEnd,
              asrDone: epochNow(),
              durationMs: samples.length / 16,
              utteranceKey,
            };
            this.hooks.onMarks?.(metadata);
            if (result.text) resultListeners.forEach((cb) => cb({ ...result, metadata }));
          })
          .catch(() => this.hooks.onError?.('ASR_DECODE_FAILED'))
          .finally(() => {
            inFlight--;
          });
      }
    };
    return {
      feed: (pcm) => {
        if (closed) return;
        if (pcm.length !== 320) throw new Error('Expected 20 ms / 320 samples');
        const samples = Float32Array.from(pcm, (v) => v / 32768);
        vad.acceptWaveform(samples);
        const playbackActive = this.playing.size > 0;
        if (playbackActive) {
          bargeVad.acceptWaveform(samples);
          while (!bargeVad.isEmpty()) bargeVad.pop();
        } else if (playbackWasActive) bargeVad.reset();
        playbackWasActive = playbackActive;
        const next = vad.isDetected();
        if (next && !detected) {
          bargeSent = false;
          emitVad('speechStart');
        }
        if (!bargeSent && playbackActive && bargeVad.isDetected()) {
          bargeSent = true;
          this.hooks.onBargeIn?.();
        }
        if (!next && detected) {
          emitVad('speechEnd');
        }
        detected = next;
        drain();
      },
      onVad: (cb) => {
        vadListeners.push(cb);
      },
      onResult: (cb) => {
        resultListeners.push(cb);
      },
      close: () => {
        if (closed) return;
        closed = true;
        vad.flush();
        if (detected) emitVad('speechEnd');
        drain();
        vad.reset();
        bargeVad.reset();
      },
    };
  }
}
