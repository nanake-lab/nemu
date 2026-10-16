import type * as THREE from 'three';
import type { VRM } from '@pixiv/three-vrm';
import type { NemuProfile, Emotion } from '@nemu/contracts';

export type AnimationLayerId = 'L0' | 'L1' | 'L2' | 'L3' | 'L4';

export interface VisemeWeights {
  aa?: number | undefined;
  ih?: number | undefined;
  ou?: number | undefined;
  ee?: number | undefined;
  oh?: number | undefined;
}

export interface GazeLimits {
  yawDeg: number;
  pitchUpDeg: number;
  pitchDownDeg: number;
}

export interface PostureLimits {
  spinePitchMaxDeg: number;
  chestPitchMaxDeg: number;
  neckPitchMaxDeg: number;
  shoulderRollMaxDeg: number;
}

export interface TwoBoneIKParams {
  rootPos: THREE.Vector3;
  midPos: THREE.Vector3;
  endPos: THREE.Vector3;
  targetPos: THREE.Vector3;
  poleVector: THREE.Vector3;
}

export interface TwoBoneIKSolution {
  rootRotation: THREE.Quaternion;
  midRotation: THREE.Quaternion;
}

export interface BasePoseOptions {
  loop?: boolean | undefined;
  crossFadeDurationMs?: number | undefined;
}

export interface GestureOptions {
  fadeInMs?: number | undefined;
  fadeOutMs?: number | undefined;
  loop?: boolean | undefined;
}

export interface FacialPerformanceCue {
  emotion: Emotion;
  weight?: number | undefined;
  durationMs?: number | undefined;
}

export interface PerformanceOverride {
  earPose?: number | undefined;
  tailAmplitude?: number | undefined;
  durationMs?: number | undefined;
}

export interface AnimationCompositorOptions {
  vrm?: VRM | undefined;
  profile?: NemuProfile | undefined;
  deterministic?: boolean | undefined;
  prng?: (() => number) | undefined;
}
