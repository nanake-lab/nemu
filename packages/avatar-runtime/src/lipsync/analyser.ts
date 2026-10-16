import type { LipSyncProfile, VowelKey, VowelScores } from './types.js';

export interface AnalyserOptions {
  sampleRate?: number | undefined;
  sampleCount?: number | undefined;
  minVolumeDb?: number | undefined;
}

export interface FrameAnalysisResult {
  rms: number;
  volumeDb: number;
  mfcc: number[];
  vowelScores: VowelScores;
}

const VOWEL_KEYS: VowelKey[] = ['A', 'I', 'U', 'E', 'O'];

export class LipSyncAnalyser {
  private readonly sampleRate: number;
  private readonly sampleCount: number;
  private readonly minVolumeDb: number;

  private profile: LipSyncProfile;
  private readonly hannWindow: Float32Array;
  private readonly melFilters: Float32Array[];

  constructor(profile: LipSyncProfile, options?: AnalyserOptions) {
    this.profile = profile;
    this.sampleRate = options?.sampleRate ?? profile.targetSampleRate ?? 16000;
    this.sampleCount = options?.sampleCount ?? profile.sampleCount ?? 512;
    this.minVolumeDb = options?.minVolumeDb ?? -40;

    // Precompute Hann window
    this.hannWindow = new Float32Array(this.sampleCount);
    for (let i = 0; i < this.sampleCount; i++) {
      this.hannWindow[i] = 0.5 * (1 - Math.cos((2 * Math.PI * i) / (this.sampleCount - 1)));
    }

    // Precompute Mel Filterbank
    const numFilters = profile.melFilterBankChannels ?? 16;
    this.melFilters = this.buildMelFilterbank(numFilters, this.sampleCount, this.sampleRate);
  }

  setProfile(profile: LipSyncProfile): void {
    this.profile = profile;
  }

  getProfile(): LipSyncProfile {
    return this.profile;
  }

  /**
   * Analyzes a frame of PCM audio (typically 512 samples).
   */
  analyzeFrame(pcm: Float32Array): FrameAnalysisResult {
    // 1. Compute RMS & dB
    let sumSquares = 0;
    const len = Math.min(pcm.length, this.sampleCount);
    for (let i = 0; i < len; i++) {
      const s = pcm[i] ?? 0;
      sumSquares += s * s;
    }
    const rms = Math.sqrt(sumSquares / Math.max(1, len));
    const volumeDb = 20 * Math.log10(Math.max(1e-6, rms));

    if (volumeDb < this.minVolumeDb || rms < 1e-4) {
      return {
        rms,
        volumeDb,
        mfcc: new Array(12).fill(0),
        vowelScores: { A: 0, I: 0, U: 0, E: 0, O: 0 },
      };
    }

    // 2. Windowed FFT
    const real = new Float32Array(this.sampleCount);
    const imag = new Float32Array(this.sampleCount);
    for (let i = 0; i < len; i++) {
      real[i] = (pcm[i] ?? 0) * (this.hannWindow[i] ?? 0);
    }

    this.fft(real, imag);

    // Magnitude spectrum
    const halfSize = this.sampleCount >> 1;
    const spectrum = new Float32Array(halfSize);
    for (let i = 0; i < halfSize; i++) {
      const r = real[i] ?? 0;
      const im = imag[i] ?? 0;
      spectrum[i] = Math.sqrt(r * r + im * im);
    }

    // 3. Mel Filterbank energies
    const numFilters = this.melFilters.length;
    const filterEnergies = new Float32Array(numFilters);
    for (let m = 0; m < numFilters; m++) {
      const filter = this.melFilters[m]!;
      let energy = 0;
      for (let k = 0; k < halfSize; k++) {
        energy += (spectrum[k] ?? 0) * (filter[k] ?? 0);
      }
      filterEnergies[m] = Math.log(Math.max(1e-6, energy));
    }

    // 4. DCT-II for MFCC (12 coefficients)
    const mfccNum = 12;
    const mfcc = new Array<number>(mfccNum);
    for (let n = 0; n < mfccNum; n++) {
      let sum = 0;
      for (let m = 0; m < numFilters; m++) {
        sum += (filterEnergies[m] ?? 0) * Math.cos((Math.PI * n * (m + 0.5)) / numFilters);
      }
      mfcc[n] = sum;
    }

    // 5. Compare against profile calibrated MFCCs
    const vowelScores: VowelScores = { A: 0, I: 0, U: 0, E: 0, O: 0 };
    const rawScores: Record<VowelKey, number> = { A: 0, I: 0, U: 0, E: 0, O: 0 };
    let sumRaw = 0;

    for (const p of this.profile.mfccs) {
      const name = p.name as VowelKey;
      if (!VOWEL_KEYS.includes(name)) continue;

      const refMfcc = p.values ?? p.mfccCalibrationDataList?.[0]?.array;
      if (!refMfcc) continue;

      let distSq = 0;
      for (let i = 0; i < mfccNum; i++) {
        const diff = (mfcc[i] ?? 0) - (refMfcc[i] ?? 0);
        distSq += diff * diff;
      }
      const dist = Math.sqrt(distSq);

      // Score decays exponentially with distance
      const score = Math.exp(-dist / 8.0);
      rawScores[name] = Math.max(rawScores[name] ?? 0, score);
    }

    for (const k of VOWEL_KEYS) {
      sumRaw += rawScores[k] ?? 0;
    }

    // Volume scaling factor [0..1]
    const volScale = Math.min(1.0, Math.max(0, (volumeDb - this.minVolumeDb) / (-15 - this.minVolumeDb)));

    if (sumRaw > 0) {
      for (const k of VOWEL_KEYS) {
        vowelScores[k] = ((rawScores[k] ?? 0) / sumRaw) * volScale;
      }
    }

    return {
      rms,
      volumeDb,
      mfcc,
      vowelScores,
    };
  }

  private buildMelFilterbank(numFilters: number, fftSize: number, sampleRate: number): Float32Array[] {
    const halfSize = fftSize >> 1;
    const minMel = this.hzToMel(100);
    const maxMel = this.hzToMel(Math.min(8000, sampleRate / 2));

    const melPoints = new Float32Array(numFilters + 2);
    const binPoints = new Int32Array(numFilters + 2);

    for (let i = 0; i < numFilters + 2; i++) {
      melPoints[i] = minMel + (i * (maxMel - minMel)) / (numFilters + 1);
      const hz = this.melToHz(melPoints[i] ?? 0);
      binPoints[i] = Math.floor(((fftSize + 1) * hz) / sampleRate);
    }

    const filters: Float32Array[] = [];
    for (let m = 1; m <= numFilters; m++) {
      const filter = new Float32Array(halfSize);
      const left = binPoints[m - 1] ?? 0;
      const center = binPoints[m] ?? 0;
      const right = binPoints[m + 1] ?? 0;

      for (let k = left; k < center && k < halfSize; k++) {
        filter[k] = (k - left) / Math.max(1, center - left);
      }
      for (let k = center; k < right && k < halfSize; k++) {
        filter[k] = (right - k) / Math.max(1, right - center);
      }
      filters.push(filter);
    }

    return filters;
  }

  private hzToMel(hz: number): number {
    return 2595 * Math.log10(1 + hz / 700);
  }

  private melToHz(mel: number): number {
    return 700 * (Math.pow(10, mel / 2595) - 1);
  }

  /**
   * In-place Radix-2 Cooley-Tukey FFT.
   */
  private fft(real: Float32Array, imag: Float32Array): void {
    const n = real.length;
    let j = 0;
    for (let i = 0; i < n - 1; i++) {
      if (i < j) {
        const tr = real[i]!;
        real[i] = real[j]!;
        real[j] = tr;
        const ti = imag[i]!;
        imag[i] = imag[j]!;
        imag[j] = ti;
      }
      let k = n >> 1;
      while (k <= j) {
        j -= k;
        k >>= 1;
      }
      j += k;
    }

    for (let len = 2; len <= n; len <<= 1) {
      const halfLen = len >> 1;
      const angle = (-2 * Math.PI) / len;
      const wStepR = Math.cos(angle);
      const wStepI = Math.sin(angle);

      for (let i = 0; i < n; i += len) {
        let wr = 1;
        let wi = 0;
        for (let k = 0; k < halfLen; k++) {
          const pos = i + k;
          const matchPos = pos + halfLen;
          const uR = real[pos]!;
          const uI = imag[pos]!;
          const vR = (real[matchPos] ?? 0) * wr - (imag[matchPos] ?? 0) * wi;
          const vI = (real[matchPos] ?? 0) * wi + (imag[matchPos] ?? 0) * wr;

          real[pos] = uR + vR;
          imag[pos] = uI + vI;
          real[matchPos] = uR - vR;
          imag[matchPos] = uI - vI;

          const nextWr = wr * wStepR - wi * wStepI;
          wi = wr * wStepI + wi * wStepR;
          wr = nextWr;
        }
      }
    }
  }
}
