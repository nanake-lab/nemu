import type { Lang, PerformanceCue } from '@nemu/contracts';
import type { VisemeWeights } from '../animation/types.js';

export interface SpeechBeginMessage {
  outputId: string;
  chunkIndex: number;
  sentenceId: string;
  text: string;
  lang: Lang;
  cues: PerformanceCue[];
  sampleRate: number;
}

export interface SpeechChunkMessage {
  sentenceId: string;
  pcm: Float32Array;
}

export interface SpeechEndMessage {
  sentenceId: string;
}

export interface SpeechStopMessage {
  fadeMs: number;
}

export interface SpeechStartedPayload {
  sentenceId: string;
  at: number;
}

export interface SpeechProgressPayload {
  sentenceId: string;
  playedMs: number;
}

export interface SpeechDonePayload {
  sentenceId: string;
  interrupted: boolean;
  playedChars: number;
}

export type VowelKey = 'A' | 'I' | 'U' | 'E' | 'O';
export type VowelScores = Record<VowelKey, number>;

export interface VisemeMappingConfig {
  A: 'aa';
  I: 'ih';
  U: 'ou';
  E: 'ee';
  O: 'oh';
}

export interface ProfileMFCC {
  name: string;
  mfccCalibrationDataList: Array<{ array: number[] }>;
  values?: number[] | undefined;
}

export interface LipSyncProfile {
  targetSampleRate: number;
  sampleCount: number;
  melFilterBankChannels: number;
  compareMethod: 0 | 1 | 2; // 0: L1_NORM, 1: L2_NORM, 2: COSINE_SIMILARITY
  mfccNum: number;
  mfccDataCount: number;
  useStandardization: boolean;
  mfccs: ProfileMFCC[];
  means?: number[] | undefined;
  stdDevs?: number[] | undefined;
}

export interface LipSyncRuntimeOptions {
  sampleRate?: number | undefined;
  visemeMap?: VisemeMappingConfig | undefined;
  minVolumeDb?: number | undefined; // Volume threshold below which mouth closes (default: -40 dB)
  attackTimeMs?: number | undefined; // Viseme opening transition time (default: 40 ms)
  decayTimeMs?: number | undefined; // Viseme closing transition time (default: 80 ms)
  profiles?: Partial<Record<Lang, LipSyncProfile>> | undefined;
  onSpeechStarted?: ((event: SpeechStartedPayload, cues: PerformanceCue[]) => void) | undefined;
  onSpeechProgress?: ((event: SpeechProgressPayload) => void) | undefined;
  onSpeechDone?: ((event: SpeechDonePayload) => void) | undefined;
  onVisemesUpdated?: ((visemes: VisemeWeights) => void) | undefined;
  onAudioStartMark?: ((timestamp: number) => void) | undefined;
}
