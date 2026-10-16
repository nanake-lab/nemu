import type { VisemeWeights } from '../animation/types.js';
import type { VisemeMappingConfig, VowelKey, VowelScores } from './types.js';

export interface VisemeSmootherOptions {
  attackTimeMs?: number | undefined;
  decayTimeMs?: number | undefined;
  minVolumeDb?: number | undefined;
  visemeMap?: VisemeMappingConfig | undefined;
}

const DEFAULT_VISEME_MAP: VisemeMappingConfig = {
  A: 'aa',
  I: 'ih',
  U: 'ou',
  E: 'ee',
  O: 'oh',
};

const VOWEL_KEYS: VowelKey[] = ['A', 'I', 'U', 'E', 'O'];

export class VisemeSmoother {
  private readonly attackTimeSec: number;
  private readonly decayTimeSec: number;
  private readonly minVolumeDb: number;
  private readonly visemeMap: VisemeMappingConfig;

  private currentVowels: Record<VowelKey, number> = {
    A: 0,
    I: 0,
    U: 0,
    E: 0,
    O: 0,
  };

  private currentVisemes: VisemeWeights = {
    aa: 0,
    ih: 0,
    ou: 0,
    ee: 0,
    oh: 0,
  };

  constructor(options?: VisemeSmootherOptions | undefined) {
    this.attackTimeSec = Math.max(0.001, (options?.attackTimeMs ?? 40) / 1000);
    this.decayTimeSec = Math.max(0.001, (options?.decayTimeMs ?? 80) / 1000);
    this.minVolumeDb = options?.minVolumeDb ?? -40;
    this.visemeMap = options?.visemeMap ?? DEFAULT_VISEME_MAP;
  }

  /**
   * Updates smoothed viseme weights given raw vowel scores, current volume (dB), and time step (seconds).
   */
  update(rawScores: Partial<VowelScores>, volumeDb: number, deltaSec: number): VisemeWeights {
    const isSilenced = volumeDb < this.minVolumeDb;

    for (const key of VOWEL_KEYS) {
      const rawTarget = isSilenced ? 0 : (rawScores[key] ?? 0);
      const target = Math.max(0, Math.min(1.0, rawTarget));
      const current = this.currentVowels[key];

      let nextVal: number;
      if (deltaSec > 0) {
        if (target > current) {
          // Attack (rising / opening)
          const alpha = 1 - Math.exp(-deltaSec / this.attackTimeSec);
          nextVal = current + (target - current) * alpha;
        } else {
          // Decay (falling / closing)
          const alpha = 1 - Math.exp(-deltaSec / this.decayTimeSec);
          nextVal = current + (target - current) * alpha;
        }
      } else {
        nextVal = target;
      }

      this.currentVowels[key] = Math.max(0, Math.min(1.0, nextVal));
    }

    // Map to viseme weights
    this.currentVisemes = {
      aa: this.currentVowels[this.invertKey(this.visemeMap.A)],
      ih: this.currentVowels[this.invertKey(this.visemeMap.I)],
      ou: this.currentVowels[this.invertKey(this.visemeMap.U)],
      ee: this.currentVowels[this.invertKey(this.visemeMap.E)],
      oh: this.currentVowels[this.invertKey(this.visemeMap.O)],
    };

    return { ...this.currentVisemes };
  }

  private invertKey(target: 'aa' | 'ih' | 'ou' | 'ee' | 'oh'): VowelKey {
    switch (target) {
      case 'aa': return 'A';
      case 'ih': return 'I';
      case 'ou': return 'U';
      case 'ee': return 'E';
      case 'oh': return 'O';
    }
  }

  getVisemes(): VisemeWeights {
    return { ...this.currentVisemes };
  }

  getVowels(): VowelScores {
    return { ...this.currentVowels };
  }

  /**
   * Calculates overall mouth opening aperture in [0, 1].
   * Used for cross-correlation deviation testing and mouth openness tracking.
   */
  getMouthAperture(): number {
    const { aa, ih, ou, ee, oh } = this.currentVisemes;
    const aperture =
      (aa ?? 0) * 1.0 +
      (oh ?? 0) * 0.75 +
      (ee ?? 0) * 0.55 +
      (ih ?? 0) * 0.35 +
      (ou ?? 0) * 0.25;
    return Math.max(0, Math.min(1.0, aperture));
  }

  reset(): void {
    this.currentVowels = { A: 0, I: 0, U: 0, E: 0, O: 0 };
    this.currentVisemes = { aa: 0, ih: 0, ou: 0, ee: 0, oh: 0 };
  }
}
