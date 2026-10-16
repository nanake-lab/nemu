import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import type { VRM } from '@pixiv/three-vrm';
import {
  VRMAnimationLoaderPlugin,
  createVRMAnimationClip,
  type VRMAnimation,
} from '@pixiv/three-vrm-animation';
import type { BasePoseOptions } from './types.js';
import { alignLoopHeadToTail, smoothHermite, computeAngularVelocity } from './math.js';

interface CrossFadeTransition {
  fromAction: THREE.AnimationAction;
  toAction: THREE.AnimationAction;
  durationSec: number;
  elapsedSec: number;
}

export class BasePoseLayer {
  private vrm: VRM | null = null;
  private mixer: THREE.AnimationMixer | null = null;

  private currentAction: THREE.AnimationAction | null = null;
  private currentClip: THREE.AnimationClip | null = null;
  private transition: CrossFadeTransition | null = null;

  // Angular velocity tracking
  private previousBoneRotations = new Map<string, THREE.Quaternion>();
  private maxObservedAngularVelocity = 0;
  private transitionMaxAngularVelocity = 0;

  constructor(vrm?: VRM) {
    if (vrm) {
      this.bindVRM(vrm);
    }
  }

  bindVRM(vrm: VRM): void {
    this.dispose();
    this.vrm = vrm;
    this.mixer = new THREE.AnimationMixer(vrm.scene);
  }

  /**
   * Loads a VRMA asset using GLTFLoader + VRMAnimationLoaderPlugin.
   */
  async loadVRMA(source: string | ArrayBuffer): Promise<VRMAnimation> {
    const loader = new GLTFLoader();
    loader.register((parser) => new VRMAnimationLoaderPlugin(parser));

    return new Promise<VRMAnimation>((resolve, reject) => {
      const onLoad = (gltf: unknown) => {
        const vrmAnimation = (gltf as { userData: { vrmAnimation?: VRMAnimation } }).userData
          .vrmAnimation;
        if (!vrmAnimation) {
          reject(new Error('Loaded GLTF asset does not contain VRMAnimation data'));
          return;
        }
        resolve(vrmAnimation);
      };

      const onError = (error: unknown) => {
        reject(new Error(`Failed to load VRMA: ${(error as Error).message || String(error)}`));
      };

      if (typeof source === 'string') {
        loader.load(source, onLoad, undefined, onError);
      } else {
        loader.parse(source, '', onLoad, onError);
      }
    });
  }

  /**
   * Creates an AnimationClip from a VRMAnimation bound to the current VRM.
   */
  createClip(vrmAnimation: VRMAnimation): THREE.AnimationClip {
    if (!this.vrm) {
      throw new Error('BasePoseLayer: Cannot create clip without bound VRM');
    }
    return createVRMAnimationClip(vrmAnimation, this.vrm);
  }

  /**
   * Plays a base pose clip with cross-fading.
   * Cross-fade duration: default 250 ms, minimum 150 ms.
   * Auto-aligns loop head-to-tail for looping clips.
   */
  playClip(clip: THREE.AnimationClip, options?: BasePoseOptions): THREE.AnimationAction {
    if (!this.mixer) {
      throw new Error('BasePoseLayer: Cannot play clip without bound VRM');
    }

    const isLoop = options?.loop ?? true;
    if (isLoop) {
      alignLoopHeadToTail(clip);
    }

    // Enforce default 250 ms, minimum 150 ms
    const requestedMs = options?.crossFadeDurationMs ?? 250;
    const durationMs = Math.max(150, requestedMs);
    const durationSec = durationMs / 1000;

    const nextAction = this.mixer.clipAction(clip);
    nextAction.setLoop(isLoop ? THREE.LoopRepeat : THREE.LoopOnce, Infinity);
    nextAction.clampWhenFinished = !isLoop;

    if (!this.currentAction) {
      // First action: play immediately at full weight
      nextAction.reset();
      nextAction.setEffectiveWeight(1.0);
      nextAction.play();
      this.currentAction = nextAction;
      this.currentClip = clip;
      return nextAction;
    }

    if (this.currentAction === nextAction) {
      // Already playing this action
      return nextAction;
    }

    // Set up cross-fade transition
    nextAction.reset();
    nextAction.setEffectiveWeight(0.0);
    nextAction.play();

    this.transition = {
      fromAction: this.currentAction,
      toAction: nextAction,
      durationSec,
      elapsedSec: 0,
    };

    this.currentAction = nextAction;
    this.currentClip = clip;
    return nextAction;
  }

  stop(fadeOutDurationMs = 250): void {
    if (!this.currentAction) return;
    const durationSec = Math.max(0.15, fadeOutDurationMs / 1000);
    this.currentAction.fadeOut(durationSec);
    this.currentAction = null;
    this.currentClip = null;
    this.transition = null;
  }

  update(deltaSec: number): void {
    if (!this.mixer || !this.vrm) return;

    // Update cross-fade transition with Hermite smoothstep
    if (this.transition) {
      this.transition.elapsedSec += deltaSec;
      const progress = Math.min(1.0, this.transition.elapsedSec / this.transition.durationSec);
      const weight = smoothHermite(progress);

      this.transition.fromAction.setEffectiveWeight(1.0 - weight);
      this.transition.toAction.setEffectiveWeight(weight);

      if (progress >= 1.0) {
        this.transition.fromAction.stop();
        this.transition.toAction.setEffectiveWeight(1.0);
        this.transition = null;
      }
    }

    // Step mixer
    this.mixer.update(deltaSec);

    // Track bone angular velocities
    this.trackBoneAngularVelocities(deltaSec);
  }

  private trackBoneAngularVelocities(deltaSec: number): void {
    if (!this.vrm?.humanoid || deltaSec <= 0) return;

    const humanoid = this.vrm.humanoid;
    let maxVelThisFrame = 0;

    for (const boneName of Object.values(humanoid.humanBones)) {
      const node = boneName.node;
      if (!node) continue;

      const currentRot = node.quaternion.clone();
      const prevRot = this.previousBoneRotations.get(node.uuid);

      if (prevRot) {
        const vel = computeAngularVelocity(prevRot, currentRot, deltaSec);
        if (vel > maxVelThisFrame) {
          maxVelThisFrame = vel;
        }
      }
      this.previousBoneRotations.set(node.uuid, currentRot);
    }

    if (maxVelThisFrame > this.maxObservedAngularVelocity) {
      this.maxObservedAngularVelocity = maxVelThisFrame;
    }
    if (this.transition && maxVelThisFrame > this.transitionMaxAngularVelocity) {
      this.transitionMaxAngularVelocity = maxVelThisFrame;
    }
  }

  getCurrentClip(): THREE.AnimationClip | null {
    return this.currentClip;
  }

  isCrossFading(): boolean {
    return this.transition !== null;
  }

  getMaxObservedAngularVelocity(): number {
    return this.maxObservedAngularVelocity;
  }

  getTransitionMaxAngularVelocity(): number {
    return this.transitionMaxAngularVelocity;
  }

  resetVelocityTracking(): void {
    this.maxObservedAngularVelocity = 0;
    this.transitionMaxAngularVelocity = 0;
    this.previousBoneRotations.clear();
  }

  dispose(): void {
    if (this.mixer) {
      this.mixer.stopAllAction();
      this.mixer.uncacheRoot(this.mixer.getRoot());
      this.mixer = null;
    }
    this.currentAction = null;
    this.currentClip = null;
    this.transition = null;
    this.previousBoneRotations.clear();
  }
}
