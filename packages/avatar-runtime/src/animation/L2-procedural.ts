import * as THREE from 'three';
import type { VRM } from '@pixiv/three-vrm';
import type { NemuProfile, GazeTarget } from '@nemu/contracts';
import type { GazeLimits, PostureLimits, TwoBoneIKSolution } from './types.js';
import { CriticallyDampedSpring, DelayBuffer, solveTwoBoneIK } from './math.js';
import { FramePoseOffsets } from './pose-offsets.js';

export { solveTwoBoneIK };

export interface ProceduralLayerOptions {
  vrm?: VRM | undefined;
  profile?: NemuProfile | undefined;
}

export class ProceduralLayer {
  private vrm: VRM | null = null;
  private profile: NemuProfile | null = null;
  private offsets = new FramePoseOffsets();

  // Modulation inputs
  private idleTempo = 1.0; // 0.5..1.5
  private posture = 0.0; // -1..1
  private gazeUserWeight = 0.5;
  private bodyTurnMaxDeg = 10;
  private motionAmplitude = 1;

  // Breathing state
  private breathingPhase = 0;
  private readonly breathingNominalPeriodSec = 4.0; // 3.5 - 4.5s
  private readonly chestBreathPitchRad = THREE.MathUtils.degToRad(1.2);
  private readonly upperChestBreathPitchRad = THREE.MathUtils.degToRad(0.8);

  // Posture limits
  private postureLimits: PostureLimits = {
    spinePitchMaxDeg: 5.0,
    chestPitchMaxDeg: 4.0,
    neckPitchMaxDeg: 5.0,
    shoulderRollMaxDeg: 4.0,
  };

  // Hips sway state
  private hipsSwayPhase = 0;
  private readonly hipsSwayPeriodSec = 10.0; // 8 - 14s

  // Gaze state
  private gazeLimits: GazeLimits = {
    yawDeg: 45,
    pitchUpDeg: 25,
    pitchDownDeg: 30,
  };
  private currentGazeTarget: THREE.Vector3 = new THREE.Vector3(0, 1.4, 2.0);

  // Delays and springs for Gaze
  // Eye: weight 1.0, delay 0
  private eyeYawSpring = new CriticallyDampedSpring(0, 18.0);
  private eyePitchSpring = new CriticallyDampedSpring(0, 18.0);

  // Head: weight 0.6, delay 120ms
  private headTargetBuffer = new DelayBuffer<{ yaw: number; pitch: number }>(1.0);
  private headYawSpring = new CriticallyDampedSpring(0, 10.0);
  private headPitchSpring = new CriticallyDampedSpring(0, 10.0);

  // Neck: weight 0.3, delay 200ms
  private neckTargetBuffer = new DelayBuffer<{ yaw: number; pitch: number }>(1.0);
  private neckYawSpring = new CriticallyDampedSpring(0, 8.0);
  private neckPitchSpring = new CriticallyDampedSpring(0, 8.0);

  // Cached humanoid bone nodes
  private bones = {
    hips: null as THREE.Object3D | null,
    spine: null as THREE.Object3D | null,
    chest: null as THREE.Object3D | null,
    upperChest: null as THREE.Object3D | null,
    neck: null as THREE.Object3D | null,
    head: null as THREE.Object3D | null,
    leftShoulder: null as THREE.Object3D | null,
    rightShoulder: null as THREE.Object3D | null,
    leftEye: null as THREE.Object3D | null,
    rightEye: null as THREE.Object3D | null,
  };

  constructor(options?: ProceduralLayerOptions) {
    if (options?.vrm) {
      this.bindVRM(options.vrm, options.profile);
    }
  }

  bindVRM(vrm: VRM, profile?: NemuProfile): void {
    this.restoreOffsets();
    this.vrm = vrm;
    if (profile) {
      this.setProfile(profile);
    }
    this.cacheBones();
    this.resetSprings();
  }

  setProfile(profile: NemuProfile): void {
    this.profile = profile;
    this.gazeLimits = {
      yawDeg: profile.gazeLimits.yawDeg,
      pitchUpDeg: profile.gazeLimits.pitchUpDeg,
      pitchDownDeg: profile.gazeLimits.pitchDownDeg,
    };
  }

  setPostureLimits(limits: Partial<PostureLimits>): void {
    this.postureLimits = { ...this.postureLimits, ...limits };
  }

  getPostureLimits(): PostureLimits {
    return { ...this.postureLimits };
  }

  setReadoutParameters(params: {
    idleTempo?: number;
    posture?: number;
    gazeUserWeight?: number;
    bodyTurnMaxDeg?: number;
    motionAmplitude?: number;
  }): void {
    if (params.idleTempo !== undefined) {
      this.idleTempo = Math.max(0.5, Math.min(1.5, params.idleTempo));
    }
    if (params.posture !== undefined) {
      this.posture = Math.max(-1.0, Math.min(1.0, params.posture));
    }
    if (params.gazeUserWeight !== undefined) {
      this.gazeUserWeight = Math.max(0, Math.min(1, params.gazeUserWeight));
    }
    if (params.bodyTurnMaxDeg !== undefined) this.bodyTurnMaxDeg = params.bodyTurnMaxDeg;
    if (params.motionAmplitude !== undefined) this.motionAmplitude = params.motionAmplitude;
  }

  setGazeTarget(target: GazeTarget): void {
    const avatarHeight = this.profile?.heightM ?? 1.45;

    if (target === 'user') {
      this.currentGazeTarget.set(0, avatarHeight * 0.95, 2.5);
    } else if (target === 'away') {
      this.currentGazeTarget.set(1.5, avatarHeight * 0.7, 2.0);
    } else if (typeof target === 'object' && 'x' in target && 'y' in target) {
      // Screen space approximation (normalized -1..1 coordinates)
      this.currentGazeTarget.set(target.x * 2.0, avatarHeight + target.y * 0.5, 2.0);
    } else {
      this.currentGazeTarget.set(0, avatarHeight * 0.95, 2.0);
    }
  }

  setGazeWorldPosition(x: number, y: number, z: number): void {
    this.currentGazeTarget.set(x, y, z);
  }

  private cacheBones(): void {
    if (!this.vrm?.humanoid) return;
    const humanoid = this.vrm.humanoid;

    this.bones.hips = humanoid.getNormalizedBoneNode('hips');
    this.bones.spine = humanoid.getNormalizedBoneNode('spine');
    this.bones.chest = humanoid.getNormalizedBoneNode('chest');
    this.bones.upperChest = humanoid.getNormalizedBoneNode('upperChest');
    this.bones.neck = humanoid.getNormalizedBoneNode('neck');
    this.bones.head = humanoid.getNormalizedBoneNode('head');
    this.bones.leftShoulder = humanoid.getNormalizedBoneNode('leftShoulder');
    this.bones.rightShoulder = humanoid.getNormalizedBoneNode('rightShoulder');
    this.bones.leftEye = humanoid.getNormalizedBoneNode('leftEye');
    this.bones.rightEye = humanoid.getNormalizedBoneNode('rightEye');
  }

  private resetSprings(): void {
    this.eyeYawSpring.reset(0);
    this.eyePitchSpring.reset(0);
    this.headYawSpring.reset(0);
    this.headPitchSpring.reset(0);
    this.neckYawSpring.reset(0);
    this.neckPitchSpring.reset(0);
    this.headTargetBuffer.reset();
    this.neckTargetBuffer.reset();
  }

  /**
   * Main per-frame update for L2 Procedural layer.
   */
  update(deltaSec: number): void {
    if (!this.vrm || deltaSec <= 0) return;
    this.restoreOffsets();
    this.offsets.capture(Object.values(this.bones));

    // 1. Posture offset (static offset layered under breathing and sway)
    this.applyPostureOffset();

    // 2. Breathing oscillation (chest and upperChest, scaled by idleTempo)
    this.applyBreathing(deltaSec);

    // 3. Hips sway oscillation (slow 8-14s period)
    this.applyHipsSway(deltaSec);

    // 4. Gaze tracking (Eye weight 1.0 / 0 delay, Head 0.6 / 120ms, Neck 0.3 / 200ms)
    this.applyGaze(deltaSec);
    this.offsets.finish();
  }

  restoreOffsets(): void {
    this.offsets.restore();
  }

  private applyPostureOffset(): void {
    if (Math.abs(this.posture) < 0.0001) return;

    // Posture value -1..1
    // Negative = withdrawn/hunched: spine flexes forward (+X), shoulders roll inward
    // Positive = upright/open: spine extends (-X), shoulders pull back
    const factor = this.posture;

    const spinePitch = factor * THREE.MathUtils.degToRad(this.postureLimits.spinePitchMaxDeg);
    const chestPitch = factor * THREE.MathUtils.degToRad(this.postureLimits.chestPitchMaxDeg);
    const neckPitch = factor * THREE.MathUtils.degToRad(this.postureLimits.neckPitchMaxDeg);
    const shoulderRoll = factor * THREE.MathUtils.degToRad(this.postureLimits.shoulderRollMaxDeg);

    if (this.bones.spine) {
      const qOffset = new THREE.Quaternion().setFromAxisAngle(
        new THREE.Vector3(1, 0, 0),
        -spinePitch,
      );
      this.bones.spine.quaternion.multiply(qOffset);
    }
    if (this.bones.chest) {
      const qOffset = new THREE.Quaternion().setFromAxisAngle(
        new THREE.Vector3(1, 0, 0),
        -chestPitch,
      );
      this.bones.chest.quaternion.multiply(qOffset);
    }
    if (this.bones.neck) {
      const qOffset = new THREE.Quaternion().setFromAxisAngle(
        new THREE.Vector3(1, 0, 0),
        neckPitch * 0.5,
      );
      this.bones.neck.quaternion.multiply(qOffset);
    }
    if (this.bones.leftShoulder) {
      const qOffset = new THREE.Quaternion().setFromAxisAngle(
        new THREE.Vector3(0, 1, 0),
        -shoulderRoll,
      );
      this.bones.leftShoulder.quaternion.multiply(qOffset);
    }
    if (this.bones.rightShoulder) {
      const qOffset = new THREE.Quaternion().setFromAxisAngle(
        new THREE.Vector3(0, 1, 0),
        shoulderRoll,
      );
      this.bones.rightShoulder.quaternion.multiply(qOffset);
    }
  }

  private applyBreathing(deltaSec: number): void {
    // Breathing frequency scaled by idleTempo
    const breathFreq = (1.0 / this.breathingNominalPeriodSec) * this.idleTempo;
    this.breathingPhase =
      (this.breathingPhase + deltaSec * breathFreq * Math.PI * 2) % (Math.PI * 2);

    const breathSin = Math.sin(this.breathingPhase);

    // Rotate chest and upperChest slightly along local X axis (pitch)
    if (this.bones.chest) {
      const chestAngle = breathSin * this.chestBreathPitchRad * this.motionAmplitude;
      const qBreath = new THREE.Quaternion().setFromAxisAngle(
        new THREE.Vector3(1, 0, 0),
        chestAngle,
      );
      this.bones.chest.quaternion.multiply(qBreath);
    }

    if (this.bones.upperChest) {
      const upperChestAngle =
        Math.sin(this.breathingPhase - 0.2) * this.upperChestBreathPitchRad * this.motionAmplitude;
      const qBreath = new THREE.Quaternion().setFromAxisAngle(
        new THREE.Vector3(1, 0, 0),
        upperChestAngle,
      );
      this.bones.upperChest.quaternion.multiply(qBreath);
    }
  }

  private applyHipsSway(deltaSec: number): void {
    const swayFreq = this.idleTempo / this.hipsSwayPeriodSec;
    this.hipsSwayPhase = (this.hipsSwayPhase + deltaSec * swayFreq * Math.PI * 2) % (Math.PI * 2);

    if (this.bones.hips) {
      // Subtle lateral sway in position
      const swayX = Math.sin(this.hipsSwayPhase) * 0.005 * this.motionAmplitude;
      const swayZ = Math.sin(this.hipsSwayPhase * 2.0) * 0.002 * this.motionAmplitude;
      this.bones.hips.position.x += swayX;
      this.bones.hips.position.z += swayZ;

      // Subtle roll
      const swayRoll =
        Math.cos(this.hipsSwayPhase) * THREE.MathUtils.degToRad(0.4) * this.motionAmplitude;
      const qSway = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 0, 1), swayRoll);
      this.bones.hips.quaternion.multiply(qSway);
    }
  }

  private applyGaze(deltaSec: number): void {
    if (!this.bones.head) return;

    // 1. Calculate yaw and pitch towards target
    const headWorldPos = new THREE.Vector3();
    this.bones.head.getWorldPosition(headWorldPos);

    // Blend the external gaze target toward the user without changing action selection.
    const userTarget = new THREE.Vector3(0, (this.profile?.heightM ?? 1.45) * 0.95, 2.5);
    const target = this.currentGazeTarget.clone().lerp(userTarget, this.gazeUserWeight);
    const toTarget = new THREE.Vector3().subVectors(target, headWorldPos);
    const dist = toTarget.length();
    if (dist < 0.001) return;

    // Yaw: angle in horizontal plane (X-Z)
    const rawYaw = Math.atan2(toTarget.x, toTarget.z);
    // Pitch: angle in vertical plane
    const rawPitch = Math.atan2(
      toTarget.y,
      Math.sqrt(toTarget.x * toTarget.x + toTarget.z * toTarget.z),
    );

    // 2. Clamp angles to profile gaze limits
    const maxYawRad = THREE.MathUtils.degToRad(this.gazeLimits.yawDeg);
    const maxPitchUpRad = THREE.MathUtils.degToRad(this.gazeLimits.pitchUpDeg);
    const maxPitchDownRad = THREE.MathUtils.degToRad(this.gazeLimits.pitchDownDeg);

    const clampedYaw = Math.max(-maxYawRad, Math.min(maxYawRad, rawYaw));
    const clampedPitch = Math.max(-maxPitchDownRad, Math.min(maxPitchUpRad, rawPitch));
    const bodyYaw =
      Math.max(
        -THREE.MathUtils.degToRad(this.bodyTurnMaxDeg),
        Math.min(THREE.MathUtils.degToRad(this.bodyTurnMaxDeg), rawYaw),
      ) *
      (1 - this.gazeUserWeight);
    this.bones.spine?.quaternion.multiply(
      new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), bodyYaw),
    );

    // 3. Eye: weight 1.0, delay 0
    const eyeTargetYaw = clampedYaw * 1.0;
    const eyeTargetPitch = clampedPitch * 1.0;
    const smoothEyeYaw = this.eyeYawSpring.update(eyeTargetYaw, deltaSec);
    const smoothEyePitch = this.eyePitchSpring.update(eyeTargetPitch, deltaSec);

    // 4. Head: weight 0.6, delay 120ms
    const headWeight = 0.6;
    this.headTargetBuffer.push(
      { yaw: clampedYaw * headWeight, pitch: clampedPitch * headWeight },
      deltaSec,
    );
    const delayedHeadTarget = this.headTargetBuffer.getDelayed(0.12) ?? {
      yaw: clampedYaw * headWeight,
      pitch: clampedPitch * headWeight,
    };
    const smoothHeadYaw = this.headYawSpring.update(delayedHeadTarget.yaw, deltaSec);
    const smoothHeadPitch = this.headPitchSpring.update(delayedHeadTarget.pitch, deltaSec);

    // 5. Neck: weight 0.3, delay 200ms
    const neckWeight = 0.3;
    this.neckTargetBuffer.push(
      { yaw: clampedYaw * neckWeight, pitch: clampedPitch * neckWeight },
      deltaSec,
    );
    const delayedNeckTarget = this.neckTargetBuffer.getDelayed(0.2) ?? {
      yaw: clampedYaw * neckWeight,
      pitch: clampedPitch * neckWeight,
    };
    const smoothNeckYaw = this.neckYawSpring.update(delayedNeckTarget.yaw, deltaSec);
    const smoothNeckPitch = this.neckPitchSpring.update(delayedNeckTarget.pitch, deltaSec);

    // 6. Apply to bones
    // Neck
    if (this.bones.neck) {
      const qNeck = new THREE.Quaternion().setFromEuler(
        new THREE.Euler(-smoothNeckPitch, smoothNeckYaw, 0, 'YXZ'),
      );
      this.bones.neck.quaternion.multiply(qNeck);
    }

    // Head
    if (this.bones.head) {
      const qHead = new THREE.Quaternion().setFromEuler(
        new THREE.Euler(-smoothHeadPitch, smoothHeadYaw, 0, 'YXZ'),
      );
      this.bones.head.quaternion.multiply(qHead);
    }

    // Eyes: apply residual eye angles relative to head/neck
    const remainingEyeYaw = smoothEyeYaw - smoothHeadYaw - smoothNeckYaw;
    const remainingEyePitch = smoothEyePitch - smoothHeadPitch - smoothNeckPitch;

    if (this.bones.leftEye) {
      const qEye = new THREE.Quaternion().setFromEuler(
        new THREE.Euler(-remainingEyePitch, remainingEyeYaw, 0, 'YXZ'),
      );
      this.bones.leftEye.quaternion.multiply(qEye);
    }
    if (this.bones.rightEye) {
      const qEye = new THREE.Quaternion().setFromEuler(
        new THREE.Euler(-remainingEyePitch, remainingEyeYaw, 0, 'YXZ'),
      );
      this.bones.rightEye.quaternion.multiply(qEye);
    }
  }

  /**
   * Applies Two-Bone IK to a humanoid limb (e.g. leg or arm).
   */
  applyTwoBoneIK(
    rootBone: THREE.Object3D,
    midBone: THREE.Object3D,
    endBone: THREE.Object3D,
    targetWorldPos: THREE.Vector3,
    poleWorldPos: THREE.Vector3,
  ): TwoBoneIKSolution {
    const rootPos = new THREE.Vector3();
    const midPos = new THREE.Vector3();
    const endPos = new THREE.Vector3();

    rootBone.getWorldPosition(rootPos);
    midBone.getWorldPosition(midPos);
    endBone.getWorldPosition(endPos);

    const solution = solveTwoBoneIK({
      rootPos,
      midPos,
      endPos,
      targetPos: targetWorldPos,
      poleVector: poleWorldPos,
    });

    rootBone.quaternion.multiply(solution.rootRotation);
    midBone.quaternion.multiply(solution.midRotation);

    return solution;
  }

  // Getters for testing
  getBreathingPhase(): number {
    return this.breathingPhase;
  }

  getGazeAngles(): { eyeYaw: number; headYaw: number; neckYaw: number } {
    return {
      eyeYaw: this.eyeYawSpring.getValue(),
      headYaw: this.headYawSpring.getValue(),
      neckYaw: this.neckYawSpring.getValue(),
    };
  }

  dispose(): void {
    this.restoreOffsets();
    this.resetSprings();
  }
}
