import * as THREE from 'three';
import type { VRM } from '@pixiv/three-vrm';
import type { NemuProfile } from '@nemu/contracts';

export interface SecondaryMotionLayerOptions {
  vrm?: VRM | undefined;
  profile?: NemuProfile | undefined;
}

interface TemporaryOverride {
  value: number;
  durationSec: number;
  elapsedSec: number;
}

export class SecondaryMotionLayer {
  private vrm: VRM | null = null;
  private profile: NemuProfile | null = null;

  // Readout inputs
  private earPose = 0.0;        // -1..1
  private tailAmplitude = 0.5;  // 0..1

  // Temporary performance cue overrides
  private earOverride: TemporaryOverride | null = null;
  private tailOverride: TemporaryOverride | null = null;

  // Tail oscillation phase
  private tailPhase = 0;
  private readonly tailFreqHz = 0.8; // ~0.8 Hz tail wagging cycle

  // Cached bone nodes
  private earRootLeft: THREE.Object3D | null = null;
  private earRootRight: THREE.Object3D | null = null;
  private tailRoot: THREE.Object3D | null = null;

  constructor(options?: SecondaryMotionLayerOptions) {
    if (options?.vrm) {
      this.bindVRM(options.vrm, options.profile);
    }
  }

  bindVRM(vrm: VRM, profile?: NemuProfile): void {
    this.vrm = vrm;
    if (profile) {
      this.setProfile(profile);
    }
    this.cacheRigBones();
  }

  setProfile(profile: NemuProfile): void {
    this.profile = profile;
    this.cacheRigBones();
  }

  setReadoutParameters(params: { earPose?: number; tailAmplitude?: number }): void {
    if (params.earPose !== undefined) {
      this.earPose = Math.max(-1.0, Math.min(1.0, params.earPose));
    }
    if (params.tailAmplitude !== undefined) {
      this.tailAmplitude = Math.max(0, Math.min(1.0, params.tailAmplitude));
    }
  }

  /**
   * Sets temporary performance cue override for ear pose.
   */
  setEarOverride(pose: number, durationSec = 1.0): void {
    this.earOverride = {
      value: Math.max(-1.0, Math.min(1.0, pose)),
      durationSec,
      elapsedSec: 0,
    };
  }

  /**
   * Sets temporary performance cue override for tail swing amplitude.
   */
  setTailOverride(amplitude: number, durationSec = 1.0): void {
    this.tailOverride = {
      value: Math.max(0, Math.min(1.0, amplitude)),
      durationSec,
      elapsedSec: 0,
    };
  }

  private cacheRigBones(): void {
    if (!this.vrm || !this.profile) return;

    this.earRootLeft = null;
    this.earRootRight = null;
    this.tailRoot = null;

    const rig = this.profile.rig;
    const scene = this.vrm.scene;

    if (rig.earBonesL.length > 0) {
      this.earRootLeft = scene.getObjectByName(rig.earBonesL[0]!) ?? null;
    }
    if (rig.earBonesR.length > 0) {
      this.earRootRight = scene.getObjectByName(rig.earBonesR[0]!) ?? null;
    }
    if (rig.tailBones.length > 0) {
      this.tailRoot = scene.getObjectByName(rig.tailBones[0]!) ?? null;
    }
  }

  /**
   * Main per-frame update for L4 Secondary Motion layer.
   * Offsets are applied to root bones BEFORE spring bone update.
   */
  update(deltaSec: number): void {
    if (!this.vrm || deltaSec <= 0) return;

    // 1. Advance temporary overrides
    this.updateOverrides(deltaSec);

    // 2. Apply ear pose offset to ear root bones
    this.applyEarPose();

    // 3. Apply tail oscillation to tail root bone
    this.applyTailSwing(deltaSec);

    // 4. Update three-vrm spring bones
    this.vrm.springBoneManager?.update(deltaSec);
  }

  private updateOverrides(deltaSec: number): void {
    if (this.earOverride) {
      this.earOverride.elapsedSec += deltaSec;
      if (this.earOverride.elapsedSec >= this.earOverride.durationSec) {
        this.earOverride = null;
      }
    }
    if (this.tailOverride) {
      this.tailOverride.elapsedSec += deltaSec;
      if (this.tailOverride.elapsedSec >= this.tailOverride.durationSec) {
        this.tailOverride = null;
      }
    }
  }

  private applyEarPose(): void {
    const effectivePose = this.earOverride ? this.earOverride.value : this.earPose;
    const maxDeg = this.profile?.rig.earBackMaxDeg ?? 30;
    // Ear pitch rotation: flattening backward when negative, perked up when positive
    const pitchRad = effectivePose * THREE.MathUtils.degToRad(maxDeg);

    if (this.earRootLeft) {
      const qOffset = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), pitchRad);
      this.earRootLeft.quaternion.multiply(qOffset);
    }
    if (this.earRootRight) {
      const qOffset = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), pitchRad);
      this.earRootRight.quaternion.multiply(qOffset);
    }
  }

  private applyTailSwing(deltaSec: number): void {
    const effectiveAmp = this.tailOverride ? this.tailOverride.value : this.tailAmplitude;
    if (effectiveAmp <= 0 && !this.tailRoot) return;

    this.tailPhase = (this.tailPhase + deltaSec * this.tailFreqHz * Math.PI * 2) % (Math.PI * 2);

    const swingDeg = this.profile?.rig.tailSwingDeg ?? [-20, 20];
    const swingX = THREE.MathUtils.degToRad(swingDeg[0]) * effectiveAmp * Math.sin(this.tailPhase);
    const swingY = THREE.MathUtils.degToRad(swingDeg[1]) * effectiveAmp * Math.cos(this.tailPhase);

    if (this.tailRoot) {
      const qTail = new THREE.Quaternion().setFromEuler(new THREE.Euler(swingX, swingY, 0, 'YXZ'));
      this.tailRoot.quaternion.multiply(qTail);
    }
  }

  getEffectiveEarPose(): number {
    return this.earOverride ? this.earOverride.value : this.earPose;
  }

  getEffectiveTailAmplitude(): number {
    return this.tailOverride ? this.tailOverride.value : this.tailAmplitude;
  }

  dispose(): void {
    this.earOverride = null;
    this.tailOverride = null;
    this.earRootLeft = null;
    this.earRootRight = null;
    this.tailRoot = null;
  }
}
