import type { VRM } from '@pixiv/three-vrm';
import type { NemuProfile, BodyReadout, GazeTarget } from '@nemu/contracts';
import type {
  AnimationLayerId,
  AnimationCompositorOptions,
  VisemeWeights,
  FacialPerformanceCue,
  PerformanceOverride,
} from './types.js';
import { BasePoseLayer } from './L0-base.js';
import { GestureLayer } from './L1-gesture.js';
import { ProceduralLayer } from './L2-procedural.js';
import { FacialLayer } from './L3-face.js';
import { SecondaryMotionLayer } from './L4-secondary.js';

export class AnimationCompositor {
  readonly l0: BasePoseLayer;
  readonly l1: GestureLayer;
  readonly l2: ProceduralLayer;
  readonly l3: FacialLayer;
  readonly l4: SecondaryMotionLayer;

  private vrm: VRM | null = null;
  private profile: NemuProfile | null = null;

  // Layer enable toggles (all enabled by default)
  private layerEnabled: Record<AnimationLayerId, boolean> = {
    L0: true,
    L1: true,
    L2: true,
    L3: true,
    L4: true,
  };

  constructor(options?: AnimationCompositorOptions) {
    this.l0 = new BasePoseLayer(options?.vrm);
    this.l1 = new GestureLayer(options?.vrm);
    this.l2 = new ProceduralLayer({ vrm: options?.vrm, profile: options?.profile });
    this.l3 = new FacialLayer({
      vrm: options?.vrm,
      profile: options?.profile,
      prng: options?.prng,
    });
    this.l4 = new SecondaryMotionLayer({ vrm: options?.vrm, profile: options?.profile });

    if (options?.vrm) {
      this.bindAvatar(options.vrm, options.profile);
    }
  }

  bindAvatar(vrm: VRM, profile?: NemuProfile): void {
    this.vrm = vrm;
    if (profile) {
      this.profile = profile;
    }

    this.l0.bindVRM(vrm);
    this.l1.bindVRM(vrm);
    this.l2.bindVRM(vrm, profile);
    this.l3.bindVRM(vrm, profile);
    this.l4.bindVRM(vrm, profile);
  }

  setLayerEnabled(layer: AnimationLayerId, enabled: boolean): void {
    this.layerEnabled[layer] = enabled;
  }

  isLayerEnabled(layer: AnimationLayerId): boolean {
    return this.layerEnabled[layer];
  }

  /**
   * Applies BodyReadout parameters across layers L2, L3, and L4.
   */
  applyReadout(readout: BodyReadout): void {
    // L2: idleTempo, posture, gazeSociability
    this.l2.setReadoutParameters({
      idleTempo: readout.idleTempo,
      posture: readout.posture,
      gazeSociability: readout.gazeSociability,
    });

    // L3: expressionBaseline (0..0.4)
    this.l3.setExpressionBaseline(readout.expressionBaseline);

    // L4: earPose, tailAmplitude
    this.l4.setReadoutParameters({
      earPose: readout.earPose,
      tailAmplitude: readout.tailAmplitude,
    });
  }

  setGaze(target: GazeTarget): void {
    this.l2.setGazeTarget(target);
  }

  setVisemes(visemes: VisemeWeights): void {
    this.l3.setVisemes(visemes);
  }

  setPerformanceEmotion(cue: FacialPerformanceCue): void {
    this.l3.setPerformanceCue(cue);
  }

  onSentenceEnd(): void {
    this.l3.onSentenceEnd();
  }

  setRigOverride(override: PerformanceOverride): void {
    if (override.earPose !== undefined) {
      this.l4.setEarOverride(override.earPose, (override.durationMs ?? 1000) / 1000);
    }
    if (override.tailAmplitude !== undefined) {
      this.l4.setTailOverride(override.tailAmplitude, (override.durationMs ?? 1000) / 1000);
    }
  }

  /**
   * Per-frame animation composition.
   * Execution order: L0 -> L1 -> L2 -> L3 -> L4
   */
  update(deltaSec: number): void {
    if (!this.vrm || deltaSec <= 0) return;

    // 1. L0 Base Pose
    if (this.layerEnabled.L0) {
      this.l0.update(deltaSec);
    }

    // 2. L1 Gesture Overlay (masked to upper body)
    if (this.layerEnabled.L1) {
      this.l1.update(deltaSec);
    }

    // 3. L2 Procedural Body (posture, breathing, sway, gaze, IK)
    if (this.layerEnabled.L2) {
      this.l2.update(deltaSec);
    }

    // 4. L3 Facial Expression (baseline + emo cue, blink, lip-sync clamp)
    if (this.layerEnabled.L3) {
      this.l3.update(deltaSec);
    }

    // 5. L4 Secondary Motion (ear/tail offsets + spring bones)
    if (this.layerEnabled.L4) {
      this.l4.update(deltaSec);
    }
  }

  dispose(): void {
    this.l0.dispose();
    this.l1.dispose();
    this.l2.dispose();
    this.l3.dispose();
    this.l4.dispose();
    this.vrm = null;
    this.profile = null;
  }
}
