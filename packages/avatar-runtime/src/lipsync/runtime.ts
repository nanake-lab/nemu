import type { Lang } from '@nemu/contracts';
import type { FacialLayer } from '../animation/L3-face.js';
import type { AnimationCompositor } from '../animation/compositor.js';
import type { VisemeWeights } from '../animation/types.js';
import { LipSyncAnalyser } from './analyser.js';
import { StageAudioQueue } from './audio-queue.js';
import { getProfileForLang } from './profiles.js';
import { VisemeSmoother } from './smoother.js';
import type {
  LipSyncProfile,
  LipSyncRuntimeOptions,
  SpeechBeginMessage,
  SpeechChunkMessage,
  SpeechEndMessage,
  SpeechStopMessage,
  VowelScores,
} from './types.js';

export class LipSyncRuntime {
  private readonly queue: StageAudioQueue;
  private readonly smoother: VisemeSmoother;
  private analyser: LipSyncAnalyser;

  private currentLang: Lang = 'zh';
  private customProfiles: Partial<Record<Lang, LipSyncProfile>> = {};
  private currentVolumeDb = -100;
  private currentAperture = 0;

  private facialLayer: FacialLayer | null = null;
  private compositor: AnimationCompositor | null = null;

  private readonly onVisemesUpdated?: ((visemes: VisemeWeights) => void) | undefined;

  constructor(options?: LipSyncRuntimeOptions | undefined) {
    this.customProfiles = options?.profiles ?? {};
    const initialProfile = getProfileForLang(this.currentLang, this.customProfiles);

    this.analyser = new LipSyncAnalyser(initialProfile, {
      sampleRate: options?.sampleRate,
      minVolumeDb: options?.minVolumeDb,
    });

    this.smoother = new VisemeSmoother({
      attackTimeMs: options?.attackTimeMs,
      decayTimeMs: options?.decayTimeMs,
      minVolumeDb: options?.minVolumeDb,
      visemeMap: options?.visemeMap,
    });

    this.queue = new StageAudioQueue({
      onSpeechStarted: options?.onSpeechStarted,
      onSpeechProgress: options?.onSpeechProgress,
      onSpeechDone: options?.onSpeechDone,
      onAudioStartMark: options?.onAudioStartMark,
    });

    this.onVisemesUpdated = options?.onVisemesUpdated;
  }

  bindFacialLayer(facialLayer: FacialLayer): void {
    this.facialLayer = facialLayer;
  }

  bindCompositor(compositor: AnimationCompositor): void {
    this.compositor = compositor;
  }

  setCustomProfiles(profiles: Partial<Record<Lang, LipSyncProfile>>): void {
    this.customProfiles = { ...this.customProfiles, ...profiles };
    this.switchLanguage(this.currentLang);
  }

  switchLanguage(lang: Lang): void {
    this.currentLang = lang;
    const profile = getProfileForLang(lang, this.customProfiles);
    this.analyser.setProfile(profile);
  }

  getCurrentLanguage(): Lang {
    return this.currentLang;
  }

  handleSpeechBegin(msg: SpeechBeginMessage): void {
    if (msg.lang && msg.lang !== this.currentLang) {
      this.switchLanguage(msg.lang);
    }
    this.queue.handleSpeechBegin(msg);
  }

  handleSpeechChunk(msg: SpeechChunkMessage): void {
    this.queue.handleSpeechChunk(msg);
  }

  handleSpeechEnd(msg: SpeechEndMessage): void {
    this.queue.handleSpeechEnd(msg);
  }

  handleSpeechStop(msg: SpeechStopMessage): void {
    this.queue.handleSpeechStop(msg);
  }

  /**
   * Main per-frame update for lip-sync and audio processing.
   */
  update(deltaSec: number): VisemeWeights {
    // 1. Process audio queue step
    const step = this.queue.processStep(deltaSec);
    this.currentVolumeDb = step.volumeDb;

    let vowelScores: VowelScores = { A: 0, I: 0, U: 0, E: 0, O: 0 };
    if (step.isSpeaking && step.gain > 0) {
      // 2. Extract MFCC and phoneme scores
      const analysis = this.analyser.analyzeFrame(step.pcmFrame);
      vowelScores = analysis.vowelScores;
    }

    // 3. Smooth visemes with asymmetric attack/decay and volume threshold gating
    const visemes = this.smoother.update(vowelScores, this.currentVolumeDb, deltaSec);
    this.currentAperture = this.smoother.getMouthAperture();

    // 4. Connect to L3 facial layer / animation compositor
    if (this.compositor) {
      this.compositor.setVisemes(visemes);
    } else if (this.facialLayer) {
      this.facialLayer.setVisemes(visemes);
    }

    // 5. Notify callback
    this.onVisemesUpdated?.(visemes);

    return visemes;
  }

  getVisemes(): VisemeWeights {
    return this.smoother.getVisemes();
  }

  getVowels(): VowelScores {
    return this.smoother.getVowels();
  }

  getMouthAperture(): number {
    return this.currentAperture;
  }

  getAudioVolumeDb(): number {
    return this.currentVolumeDb;
  }

  getCurrentGain(): number {
    return this.queue.getCurrentGain();
  }

  getQueueLength(): number {
    return this.queue.getQueueLength();
  }

  reset(): void {
    this.queue.reset();
    this.smoother.reset();
    this.currentVolumeDb = -100;
    this.currentAperture = 0;
  }
}
