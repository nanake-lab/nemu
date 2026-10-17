import type { Lang, PerformanceCue } from '@nemu/contracts';
import type {
  SpeechBeginMessage,
  SpeechChunkMessage,
  SpeechDonePayload,
  SpeechEndMessage,
  SpeechProgressPayload,
  SpeechStartedPayload,
  SpeechStopMessage,
} from './types.js';

export interface QueuedSentence {
  outputId: string;
  chunkIndex: number;
  sentenceId: string;
  text: string;
  lang: Lang;
  cues: PerformanceCue[];
  sampleRate: number;
  chunks: Float32Array[];
  totalSamples: number;
  playedSamples: number;
  isEnded: boolean;
  isStarted: boolean;
  startedAt: number;
}

export interface AudioQueueOptions {
  onSpeechStarted?: ((event: SpeechStartedPayload, cues: PerformanceCue[]) => void) | undefined;
  onSpeechProgress?: ((event: SpeechProgressPayload) => void) | undefined;
  onSpeechDone?: ((event: SpeechDonePayload) => void) | undefined;
  onAudioStartMark?: ((timestamp: number) => void) | undefined;
  clock?: (() => number) | undefined;
}

export interface AudioStepResult {
  pcmFrame: Float32Array;
  gain: number;
  volumeDb: number;
  isSpeaking: boolean;
  activeSentence?: QueuedSentence | undefined;
  playedChars: number;
}

export class StageAudioQueue {
  private queue: QueuedSentence[] = [];
  private currentSentence: QueuedSentence | null = null;

  // Interruption fade-out state
  private isFadingOut = false;
  private fadeDurationSec = 0.05; // Max 50ms per acceptance
  private fadeElapsedSec = 0;
  private currentGain = 1.0;

  private readonly clock: () => number;
  private readonly onSpeechStarted?: ((event: SpeechStartedPayload, cues: PerformanceCue[]) => void) | undefined;
  private readonly onSpeechProgress?: ((event: SpeechProgressPayload) => void) | undefined;
  private readonly onSpeechDone?: ((event: SpeechDonePayload) => void) | undefined;
  private readonly onAudioStartMark?: ((timestamp: number) => void) | undefined;

  constructor(options?: AudioQueueOptions) {
    this.clock = options?.clock ?? (() => (typeof performance !== 'undefined' ? performance.now() : Date.now()));
    this.onSpeechStarted = options?.onSpeechStarted;
    this.onSpeechProgress = options?.onSpeechProgress;
    this.onSpeechDone = options?.onSpeechDone;
    this.onAudioStartMark = options?.onAudioStartMark;
  }

  handleSpeechBegin(msg: SpeechBeginMessage): void {
    const item: QueuedSentence = {
      outputId: msg.outputId,
      chunkIndex: msg.chunkIndex,
      sentenceId: msg.sentenceId,
      text: msg.text,
      lang: msg.lang,
      cues: msg.cues ?? [],
      sampleRate: msg.sampleRate || 16000,
      chunks: [],
      totalSamples: 0,
      playedSamples: 0,
      isEnded: false,
      isStarted: false,
      startedAt: 0,
    };
    this.queue.push(item);
  }

  handleSpeechChunk(msg: SpeechChunkMessage): void {
    const target = this.findSentence(msg.sentenceId);
    if (target) {
      target.chunks.push(msg.pcm);
      target.totalSamples += msg.pcm.length;
    }
  }

  handleSpeechEnd(msg: SpeechEndMessage): void {
    const target = this.findSentence(msg.sentenceId);
    if (target) {
      target.isEnded = true;
    }
  }

  handleSpeechStop(msg: SpeechStopMessage): void {
    const effectiveFadeMs = Math.min(Math.max(0, msg.fadeMs), 50);
    this.fadeDurationSec = Math.max(0.001, effectiveFadeMs / 1000);
    this.fadeElapsedSec = 0;
    this.isFadingOut = true;

    // Discard any queued sentences that have not yet started
    const notStarted = [...this.queue];
    this.queue = [];
    for (const pending of notStarted) {
      this.onSpeechDone?.({
        sentenceId: pending.sentenceId,
        interrupted: true,
        playedChars: 0,
      });
    }

    // If nothing currently playing, complete immediately
    if (!this.currentSentence) {
      this.isFadingOut = false;
      this.currentGain = 1.0;
    }
  }

  getCurrentGain(): number {
    return this.currentGain;
  }

  getCurrentSentence(): QueuedSentence | null {
    return this.currentSentence;
  }

  getQueueLength(): number {
    return this.queue.length;
  }

  /**
   * Advances the audio playback by deltaSec seconds and extracts a 512-sample PCM analysis frame.
   */
  processStep(deltaSec: number, frameSize = 512): AudioStepResult {
    // 1. Advance fade-out if active
    if (this.isFadingOut) {
      this.fadeElapsedSec += deltaSec;
      const progress = Math.min(1.0, this.fadeElapsedSec / this.fadeDurationSec);
      this.currentGain = Math.max(0, 1.0 - progress);

      if (progress >= 1.0) {
        // Interruption fade-out finished
        this.currentGain = 0;
        this.isFadingOut = false;
        if (this.currentSentence) {
          const playedChars = this.computePlayedChars(this.currentSentence);
          const doneSentenceId = this.currentSentence.sentenceId;
          this.currentSentence = null;
          this.currentGain = 1.0;
          this.onSpeechDone?.({
            sentenceId: doneSentenceId,
            interrupted: true,
            playedChars,
          });
        }
        return {
          pcmFrame: new Float32Array(frameSize),
          gain: 0,
          volumeDb: -100,
          isSpeaking: false,
          playedChars: 0,
        };
      }
    }

    // 2. Select next sentence if needed
    if (!this.currentSentence && this.queue.length > 0) {
      const next = this.queue.shift()!;
      this.currentSentence = next;
    }

    if (!this.currentSentence) {
      return {
        pcmFrame: new Float32Array(frameSize),
        gain: this.currentGain,
        volumeDb: -100,
        isSpeaking: false,
        playedChars: 0,
      };
    }

    const s = this.currentSentence;

    // 3. Mark start if not started
    if (!s.isStarted) {
      s.isStarted = true;
      s.startedAt = this.clock();
      this.onAudioStartMark?.(s.startedAt);
      this.onSpeechStarted?.({ sentenceId: s.sentenceId, at: s.startedAt }, s.cues);
    }

    // 4. Calculate samples to advance
    const sampleRate = s.sampleRate;
    const samplesToAdvance = Math.max(1, Math.round(sampleRate * deltaSec));
    const startSample = s.playedSamples;
    const endSample = startSample + samplesToAdvance;
    s.playedSamples = endSample;

    // 5. Gather PCM frame for analyser (centered around current playback position)
    const pcmFrame = new Float32Array(frameSize);
    this.extractPcmRange(s, startSample, frameSize, pcmFrame);

    // Apply gain
    let sumSquares = 0;
    for (let i = 0; i < frameSize; i++) {
      const val = (pcmFrame[i] ?? 0) * this.currentGain;
      pcmFrame[i] = val;
      sumSquares += val * val;
    }
    const rms = Math.sqrt(sumSquares / frameSize);
    const volumeDb = 20 * Math.log10(Math.max(1e-6, rms));

    // 6. Report progress
    const playedMs = Math.round((s.playedSamples / sampleRate) * 1000);
    this.onSpeechProgress?.({ sentenceId: s.sentenceId, playedMs });

    // 7. Check if sentence finished normally
    const playedChars = this.computePlayedChars(s);
    if (s.isEnded && s.playedSamples >= s.totalSamples) {
      const finishedSentenceId = s.sentenceId;
      const textLen = s.text.length;
      this.currentSentence = null;
      this.onSpeechDone?.({
        sentenceId: finishedSentenceId,
        interrupted: false,
        playedChars: textLen,
      });
    }

    return {
      pcmFrame,
      gain: this.currentGain,
      volumeDb,
      isSpeaking: true,
      activeSentence: s,
      playedChars,
    };
  }

  private extractPcmRange(
    s: QueuedSentence,
    startSample: number,
    count: number,
    out: Float32Array,
  ): void {
    let outIdx = 0;
    let currentOffset = 0;

    for (const chunk of s.chunks) {
      const chunkLen = chunk.length;
      if (currentOffset + chunkLen <= startSample) {
        currentOffset += chunkLen;
        continue;
      }

      const chunkStart = Math.max(0, startSample - currentOffset);
      const available = chunkLen - chunkStart;
      const toCopy = Math.min(count - outIdx, available);

      for (let i = 0; i < toCopy; i++) {
        out[outIdx++] = chunk[chunkStart + i] ?? 0;
      }

      currentOffset += chunkLen;
      if (outIdx >= count) break;
    }

    // Fill remaining with silence if any
    while (outIdx < count) {
      out[outIdx++] = 0;
    }
  }

  private computePlayedChars(s: QueuedSentence): number {
    if (s.totalSamples <= 0 || s.text.length === 0) return 0;
    const ratio = Math.min(1.0, Math.max(0, s.playedSamples / s.totalSamples));
    return Math.min(s.text.length, Math.round(s.text.length * ratio));
  }

  private findSentence(sentenceId: string): QueuedSentence | null {
    if (this.currentSentence && this.currentSentence.sentenceId === sentenceId) {
      return this.currentSentence;
    }
    return this.queue.find((q) => q.sentenceId === sentenceId) ?? null;
  }

  reset(): void {
    this.queue = [];
    this.currentSentence = null;
    this.isFadingOut = false;
    this.currentGain = 1.0;
  }
}
