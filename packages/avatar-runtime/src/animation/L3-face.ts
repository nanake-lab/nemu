import type { VRM } from '@pixiv/three-vrm';
import type { NemuProfile, Emotion } from '@nemu/contracts';
import type { VisemeWeights, FacialPerformanceCue } from './types.js';
import { smoothHermite } from './math.js';

export interface FacialLayerOptions {
  vrm?: VRM | undefined;
  profile?: NemuProfile | undefined;
  prng?: (() => number) | undefined;
  speechExpressionLimit?: number | undefined; // default 0.6
  speakingRecoveryMs?: number | undefined; // default 300ms
}

type BlinkPhase = 'open' | 'closing' | 'opening' | 'double_pause';

export class FacialLayer {
  private vrm: VRM | null = null;
  private profile: NemuProfile | null = null;

  // PRNG (defaults to Math.random)
  private readonly prng: () => number;

  // Performance cue (emo) is the only expression input.
  private appliedExpressionNames = new Set<string>();
  private activeCue: FacialPerformanceCue | null = null;
  private cueWeight = 0;
  private cueDecaying = false;
  private cueDecayElapsedSec = 0;
  private readonly cueDecayDurationSec = 0.35; // ~350ms return to neutral

  // Speech & Lip-sync channel
  private visemes: VisemeWeights = { aa: 0, ih: 0, ou: 0, ee: 0, oh: 0 };
  private isSpeaking = false;
  private speechExpressionLimit = 0.6; // Configurable upper bound during speech
  private currentExpressionCeiling = 1.0;
  private speakingRecoveryMs = 300;
  private recoveryElapsedSec = Infinity; // Initially fully recovered

  // Blink controller
  private blinkTimer = 0;
  private nextBlinkInterval = 3.5; // 2 - 6s
  private isDoubleBlink = false;
  private blinkPhase: BlinkPhase = 'open';
  private blinkPhaseElapsed = 0;
  private blinkWeight = 0;

  // Blink timing constants
  private readonly blinkCloseDuration = 0.06; // 60ms
  private readonly blinkOpenDuration = 0.08; // 80ms
  private readonly doubleBlinkPause = 0.07; // 70ms between blinks

  constructor(options?: FacialLayerOptions) {
    this.prng = options?.prng ?? Math.random;
    if (options?.speechExpressionLimit !== undefined) {
      this.speechExpressionLimit = options.speechExpressionLimit;
    }
    if (options?.speakingRecoveryMs !== undefined) {
      this.speakingRecoveryMs = options.speakingRecoveryMs;
    }
    if (options?.vrm) {
      this.bindVRM(options.vrm, options.profile);
    }
    this.scheduleNextBlink();
  }

  bindVRM(vrm: VRM, profile?: NemuProfile): void {
    this.vrm = vrm;
    if (profile) {
      this.profile = profile;
    }
  }

  setProfile(profile: NemuProfile): void {
    this.profile = profile;
  }

  setSpeechExpressionLimit(limit: number): void {
    this.speechExpressionLimit = Math.max(0, Math.min(1.0, limit));
  }

  getSpeechExpressionLimit(): number {
    return this.speechExpressionLimit;
  }

  /**
   * Sets performance cue (emo) expanded via profile.emotionMap.
   */
  setPerformanceCue(cue: FacialPerformanceCue): void {
    this.activeCue = cue;
    this.cueWeight = cue.weight ?? 1.0;
    this.cueDecaying = false;
    this.cueDecayElapsedSec = 0;
  }

  /**
   * Triggered at sentence end: decays cue back to neutral along a fixed curve.
   */
  onSentenceEnd(): void {
    if (this.activeCue) {
      this.cueDecaying = true;
      this.cueDecayElapsedSec = 0;
    }
  }

  /**
   * Updates lip-sync viseme weights.
   */
  setVisemes(visemes: VisemeWeights): void {
    this.visemes = { ...visemes };
    const hasActiveViseme = Object.values(visemes).some((v) => v !== undefined && v > 0.01);
    this.setIsSpeaking(hasActiveViseme);
  }

  setIsSpeaking(speaking: boolean): void {
    if (this.isSpeaking !== speaking) {
      this.isSpeaking = speaking;
      if (!speaking) {
        // Speech ended: start recovery
        this.recoveryElapsedSec = 0;
      }
    }
  }

  getIsSpeaking(): boolean {
    return this.isSpeaking;
  }

  getCurrentExpressionCeiling(): number {
    return this.currentExpressionCeiling;
  }

  /**
   * Schedules the next blink interval randomly in [2.0, 6.0]s, with 15% double-blink.
   */
  scheduleNextBlink(): number {
    // Uniform random in [2.0, 6.0] seconds
    const interval = 2.0 + this.prng() * 4.0;
    this.nextBlinkInterval = interval;
    this.blinkTimer = 0;
    this.isDoubleBlink = this.prng() < 0.15;
    this.blinkPhase = 'open';
    this.blinkPhaseElapsed = 0;
    this.blinkWeight = 0;
    return interval;
  }

  getNextBlinkInterval(): number {
    return this.nextBlinkInterval;
  }

  getIsDoubleBlink(): boolean {
    return this.isDoubleBlink;
  }

  getBlinkWeight(): number {
    return this.blinkWeight;
  }

  /**
   * Main per-frame update for L3 Facial layer.
   */
  update(deltaSec: number): void {
    if (!this.vrm || deltaSec <= 0) return;

    // 1. Update speech expression ceiling
    this.updateSpeechClamp(deltaSec);

    // 2. Update performance cue decay
    this.updatePerformanceCue(deltaSec);

    // 3. Compute active performance expression weights
    const expressionWeights = this.computeExpressionWeights();

    // 4. Update blink controller (inhibited when surprised)
    const isSurprised = (expressionWeights.get('surprised') ?? 0) > 0.5;
    this.updateBlink(deltaSec, isSurprised);

    // 5. Apply all expressions to VRM
    this.applyExpressionsToVRM(expressionWeights);
  }

  private updateSpeechClamp(deltaSec: number): void {
    if (this.isSpeaking) {
      // Immediately clamp to speech expression limit (e.g. 0.6)
      this.currentExpressionCeiling = this.speechExpressionLimit;
      this.recoveryElapsedSec = 0;
    } else {
      // Recover back to 1.0 over speakingRecoveryMs
      const recoverySec = Math.max(0.001, this.speakingRecoveryMs / 1000);
      this.recoveryElapsedSec += deltaSec;
      const progress = Math.min(1.0, this.recoveryElapsedSec / recoverySec);
      this.currentExpressionCeiling =
        this.speechExpressionLimit + (1.0 - this.speechExpressionLimit) * smoothHermite(progress);
    }
  }

  private updatePerformanceCue(deltaSec: number): void {
    if (!this.activeCue) return;

    if (this.cueDecaying) {
      this.cueDecayElapsedSec += deltaSec;
      const progress = Math.min(1.0, this.cueDecayElapsedSec / this.cueDecayDurationSec);
      const remaining = 1.0 - smoothHermite(progress);
      this.cueWeight = (this.activeCue.weight ?? 1.0) * remaining;

      if (progress >= 1.0) {
        this.activeCue = null;
        this.cueWeight = 0;
        this.cueDecaying = false;
      }
    }
  }

  private computeExpressionWeights(): Map<string, number> {
    const result = new Map<string, number>();

    // Performance cue only; no body readout route into L3.
    if (this.activeCue && this.cueWeight > 0) {
      this.addEmotionToWeights(this.activeCue.emotion, this.cueWeight, result);
    }

    // 3. Clamp non-viseme, non-blink expressions by currentExpressionCeiling
    for (const [name, weight] of result.entries()) {
      if (name !== 'blink' && name !== 'blinkLeft' && name !== 'blinkRight') {
        result.set(name, Math.min(weight, this.currentExpressionCeiling));
      }
    }

    return result;
  }

  private addEmotionToWeights(
    emotion: Emotion,
    weight: number,
    targetMap: Map<string, number>,
  ): void {
    if (this.profile?.emotionMap?.[emotion]) {
      const mappings = this.profile.emotionMap[emotion];
      for (const mapItem of mappings) {
        const current = targetMap.get(mapItem.expression) ?? 0;
        targetMap.set(mapItem.expression, Math.max(current, mapItem.weight * weight));
      }
    } else {
      // Direct fallback
      const current = targetMap.get(emotion) ?? 0;
      targetMap.set(emotion, Math.max(current, weight));
    }
  }

  private updateBlink(deltaSec: number, isSurprised: boolean): void {
    if (isSurprised) {
      // Inhibit blink: hold eyes open, pause/reset blink timer
      this.blinkWeight = 0;
      this.blinkPhase = 'open';
      this.blinkPhaseElapsed = 0;
      return;
    }

    switch (this.blinkPhase) {
      case 'open':
        this.blinkTimer += deltaSec;
        this.blinkWeight = 0;
        if (this.blinkTimer >= this.nextBlinkInterval) {
          this.blinkPhase = 'closing';
          this.blinkPhaseElapsed = 0;
        }
        break;

      case 'closing':
        this.blinkPhaseElapsed += deltaSec;
        this.blinkWeight = Math.min(1.0, this.blinkPhaseElapsed / this.blinkCloseDuration);
        if (this.blinkPhaseElapsed >= this.blinkCloseDuration) {
          this.blinkPhase = 'opening';
          this.blinkPhaseElapsed = 0;
        }
        break;

      case 'opening':
        this.blinkPhaseElapsed += deltaSec;
        this.blinkWeight = Math.max(0, 1.0 - this.blinkPhaseElapsed / this.blinkOpenDuration);
        if (this.blinkPhaseElapsed >= this.blinkOpenDuration) {
          if (this.isDoubleBlink) {
            this.blinkPhase = 'double_pause';
            this.blinkPhaseElapsed = 0;
            this.isDoubleBlink = false; // completed first of two
          } else {
            this.scheduleNextBlink();
          }
        }
        break;

      case 'double_pause':
        this.blinkPhaseElapsed += deltaSec;
        this.blinkWeight = 0;
        if (this.blinkPhaseElapsed >= this.doubleBlinkPause) {
          this.blinkPhase = 'closing';
          this.blinkPhaseElapsed = 0;
        }
        break;
    }
  }

  private applyExpressionsToVRM(expressionWeights: Map<string, number>): void {
    const manager = this.vrm?.expressionManager;
    if (!manager) return;

    // 1. Reset standard emotion expressions
    const standardPresets = ['neutral', 'happy', 'angry', 'sad', 'relaxed', 'surprised'];
    for (const exp of standardPresets) {
      manager.setValue(exp, 0);
    }
    for (const name of this.appliedExpressionNames) manager.setValue(name, 0);
    this.appliedExpressionNames = new Set(expressionWeights.keys());

    // 2. Set blended expressions
    for (const [name, weight] of expressionWeights.entries()) {
      manager.setValue(name, weight);
    }

    // 3. Set blink
    manager.setValue('blink', this.blinkWeight);

    // 4. Set lip-sync visemes (unclamped)
    if (this.visemes.aa !== undefined) manager.setValue('aa', this.visemes.aa);
    if (this.visemes.ih !== undefined) manager.setValue('ih', this.visemes.ih);
    if (this.visemes.ou !== undefined) manager.setValue('ou', this.visemes.ou);
    if (this.visemes.ee !== undefined) manager.setValue('ee', this.visemes.ee);
    if (this.visemes.oh !== undefined) manager.setValue('oh', this.visemes.oh);

    manager.update();
  }

  dispose(): void {
    for (const name of this.appliedExpressionNames) this.vrm?.expressionManager?.setValue(name, 0);
    this.appliedExpressionNames.clear();
    this.activeCue = null;
  }
}
