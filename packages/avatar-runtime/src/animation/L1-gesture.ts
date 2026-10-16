import * as THREE from 'three';
import type { VRM } from '@pixiv/three-vrm';
import type { GestureOptions } from './types.js';
import { smoothHermite } from './math.js';

export const UPPER_BODY_BONES = new Set<string>([
  'spine',
  'chest',
  'upperChest',
  'neck',
  'head',
  'leftEye',
  'rightEye',
  'jaw',
  'leftShoulder',
  'rightShoulder',
  'leftUpperArm',
  'rightUpperArm',
  'leftLowerArm',
  'rightLowerArm',
  'leftHand',
  'rightHand',
  'leftThumbMetacarpal',
  'leftThumbProximal',
  'leftThumbDistal',
  'leftIndexProximal',
  'leftIndexIntermediate',
  'leftIndexDistal',
  'leftMiddleProximal',
  'leftMiddleIntermediate',
  'leftMiddleDistal',
  'leftRingProximal',
  'leftRingIntermediate',
  'leftRingDistal',
  'leftLittleProximal',
  'leftLittleIntermediate',
  'leftLittleDistal',
  'rightThumbMetacarpal',
  'rightThumbProximal',
  'rightThumbDistal',
  'rightIndexProximal',
  'rightIndexIntermediate',
  'rightIndexDistal',
  'rightMiddleProximal',
  'rightMiddleIntermediate',
  'rightMiddleDistal',
  'rightRingProximal',
  'rightRingIntermediate',
  'rightRingDistal',
  'rightLittleProximal',
  'rightLittleIntermediate',
  'rightLittleDistal',
]);

type FadeState = 'idle' | 'fadingIn' | 'playing' | 'fadingOut';

export class GestureLayer {
  private vrm: VRM | null = null;
  private mixer: THREE.AnimationMixer | null = null;
  private currentAction: THREE.AnimationAction | null = null;
  private currentClip: THREE.AnimationClip | null = null;

  private state: FadeState = 'idle';
  private currentWeight = 0;
  private fadeElapsedSec = 0;

  // Fade configuration (per spec: 120ms fade-in, 200ms fade-out)
  private readonly fadeInDurationSec = 0.12;
  private readonly fadeOutDurationSec = 0.2;

  // Cached bone nodes
  private upperBodyNodes = new Map<string, THREE.Object3D>();
  private lowerBodyNodes = new Map<string, THREE.Object3D>();

  constructor(vrm?: VRM) {
    if (vrm) {
      this.bindVRM(vrm);
    }
  }

  bindVRM(vrm: VRM): void {
    this.dispose();
    this.vrm = vrm;
    this.mixer = new THREE.AnimationMixer(vrm.scene);

    this.cacheBoneNodes();
  }

  private cacheBoneNodes(): void {
    this.upperBodyNodes.clear();
    this.lowerBodyNodes.clear();

    if (!this.vrm?.humanoid) return;

    for (const [boneName, humanBone] of Object.entries(this.vrm.humanoid.humanBones)) {
      if (!humanBone?.node) continue;
      if (UPPER_BODY_BONES.has(boneName)) {
        this.upperBodyNodes.set(boneName, humanBone.node);
      } else {
        this.lowerBodyNodes.set(boneName, humanBone.node);
      }
    }
  }

  /**
   * Filters an AnimationClip to retain only upper-body tracks (from spine upwards).
   */
  filterUpperBodyClip(clip: THREE.AnimationClip): THREE.AnimationClip {
    const upperTracks: THREE.KeyframeTrack[] = [];

    for (const track of clip.tracks) {
      const trackNodeName = track.name.split('.')[0]!;
      // Match against known upper body bone node names
      let isUpper = false;
      for (const [boneName, node] of this.upperBodyNodes.entries()) {
        if (trackNodeName === node.name || trackNodeName.includes(boneName)) {
          isUpper = true;
          break;
        }
      }
      // If humanoid bone name directly matches
      if (UPPER_BODY_BONES.has(trackNodeName)) {
        isUpper = true;
      }

      if (isUpper) {
        upperTracks.push(track);
      }
    }

    return new THREE.AnimationClip(
      `upper_${clip.name}`,
      clip.duration,
      upperTracks,
      clip.blendMode,
    );
  }

  /**
   * Plays a gesture clip on the upper body.
   * Fades in over 120 ms, fades out over 200 ms.
   */
  playGesture(clip: THREE.AnimationClip, options?: GestureOptions): THREE.AnimationAction {
    if (!this.mixer || !this.vrm) {
      throw new Error('GestureLayer: Cannot play gesture without bound VRM');
    }

    const filteredClip = this.filterUpperBodyClip(clip);
    const isLoop = options?.loop ?? false;

    if (this.currentAction) {
      this.currentAction.stop();
    }

    const action = this.mixer.clipAction(filteredClip);
    action.setLoop(isLoop ? THREE.LoopRepeat : THREE.LoopOnce, isLoop ? Infinity : 1);
    action.clampWhenFinished = !isLoop;
    action.reset();
    action.play();

    this.currentAction = action;
    this.currentClip = filteredClip;
    this.state = 'fadingIn';
    this.fadeElapsedSec = 0;
    this.currentWeight = 0;

    return action;
  }

  stopGesture(): void {
    if (this.state === 'idle' || this.state === 'fadingOut') return;
    this.state = 'fadingOut';
    this.fadeElapsedSec = 0;
  }

  /**
   * Updates gesture layer and blends upper-body bones onto L0 base pose.
   */
  update(deltaSec: number): void {
    if (!this.mixer || !this.currentAction || this.state === 'idle') return;

    // 1. Update fade weights
    if (this.state === 'fadingIn') {
      this.fadeElapsedSec += deltaSec;
      const progress = Math.min(1.0, this.fadeElapsedSec / this.fadeInDurationSec);
      this.currentWeight = smoothHermite(progress);
      if (progress >= 1.0) {
        this.state = 'playing';
        this.currentWeight = 1.0;
      }
    } else if (this.state === 'fadingOut') {
      this.fadeElapsedSec += deltaSec;
      const progress = Math.min(1.0, this.fadeElapsedSec / this.fadeOutDurationSec);
      this.currentWeight = 1.0 - smoothHermite(progress);
      if (progress >= 1.0) {
        this.state = 'idle';
        this.currentWeight = 0;
        this.currentAction.stop();
        this.currentAction = null;
        this.currentClip = null;
        return;
      }
    } else if (this.state === 'playing') {
      // Check if non-looping action reached its end
      if (this.currentClip && !this.currentAction.loop && this.currentAction.time >= this.currentClip.duration - this.fadeOutDurationSec) {
        this.stopGesture();
      }
    }

    if (this.currentWeight <= 0) return;

    // 2. Snapshot base rotations of upper body nodes before mixer step
    const baseRotations = new Map<string, THREE.Quaternion>();
    for (const [name, node] of this.upperBodyNodes.entries()) {
      baseRotations.set(name, node.quaternion.clone());
    }

    // Snapshot lower body nodes to ensure lower body is strictly masked out
    const lowerRotations = new Map<string, THREE.Quaternion>();
    for (const [name, node] of this.lowerBodyNodes.entries()) {
      lowerRotations.set(name, node.quaternion.clone());
    }

    // 3. Step gesture mixer
    this.mixer.update(deltaSec);

    // 4. Blend upper body nodes: q = slerp(q_base, q_gesture, currentWeight)
    for (const [name, node] of this.upperBodyNodes.entries()) {
      const qBase = baseRotations.get(name);
      if (!qBase) continue;
      const qGesture = node.quaternion.clone();
      node.quaternion.copy(qBase).slerp(qGesture, this.currentWeight);
    }

    // 5. Restore lower body nodes (strictly untouched)
    for (const [name, node] of this.lowerBodyNodes.entries()) {
      const qLower = lowerRotations.get(name);
      if (qLower) {
        node.quaternion.copy(qLower);
      }
    }
  }

  getCurrentWeight(): number {
    return this.currentWeight;
  }

  getState(): FadeState {
    return this.state;
  }

  dispose(): void {
    if (this.mixer) {
      this.mixer.stopAllAction();
      this.mixer.uncacheRoot(this.mixer.getRoot());
      this.mixer = null;
    }
    this.currentAction = null;
    this.currentClip = null;
    this.state = 'idle';
    this.currentWeight = 0;
    this.upperBodyNodes.clear();
    this.lowerBodyNodes.clear();
  }
}
