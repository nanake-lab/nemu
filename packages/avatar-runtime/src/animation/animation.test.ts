import { describe, expect, test, vi } from 'vitest';
import * as THREE from 'three';
import type { VRM } from '@pixiv/three-vrm';
import type { NemuProfile } from '@nemu/contracts';
import {
  CriticallyDampedSpring,
  DelayBuffer,
  smoothHermite,
  alignLoopHeadToTail,
  solveTwoBoneIK,
} from './math.js';
import { BasePoseLayer } from './L0-base.js';
import { GestureLayer } from './L1-gesture.js';
import { ProceduralLayer } from './L2-procedural.js';
import { FacialLayer } from './L3-face.js';
import { SecondaryMotionLayer } from './L4-secondary.js';
import { AnimationCompositor } from './compositor.js';

// Reusable mock profile
const testProfile: NemuProfile = {
  version: 2,
  model: 'sample.vrm',
  heightM: 1.45,
  headTopOffsetM: 0.12,
  hitRegions: [
    { id: 'head', bone: 'head', shape: 'sphere', radiusM: 0.15 },
    { id: 'body', bone: 'spine', shape: 'capsule', radiusM: 0.18, lengthM: 0.35 },
  ],
  gazeLimits: {
    yawDeg: 45,
    pitchUpDeg: 25,
    pitchDownDeg: 30,
  },
  framing: {
    fullBody: { fovDeg: 20, distanceM: 3.5, targetBone: 'hips' },
    upperBody: { fovDeg: 20, distanceM: 1.6, targetBone: 'chest' },
  },
  rig: {
    earBonesL: ['earRoot_L'],
    earBonesR: ['earRoot_R'],
    tailBones: ['tail_01', 'tail_02'],
    earBackMaxDeg: 30,
    tailSwingDeg: [-20, 20],
  },
  emotionMap: {
    neutral: [{ expression: 'neutral', weight: 1.0 }],
    happy: [{ expression: 'happy', weight: 1.0 }],
    sad: [{ expression: 'sad', weight: 1.0 }],
    angry: [{ expression: 'angry', weight: 1.0 }],
    surprised: [{ expression: 'surprised', weight: 1.0 }],
    relaxed: [{ expression: 'relaxed', weight: 0.8 }],
    shy: [{ expression: 'surprised', weight: 0.5 }],
    sleepy: [{ expression: 'neutral', weight: 0.5 }],
    smug: [{ expression: 'happy', weight: 0.7 }],
    pout: [{ expression: 'angry', weight: 0.5 }],
    excited: [{ expression: 'happy', weight: 0.9 }],
    worried: [{ expression: 'sad', weight: 0.6 }],
  },
  visemeMap: { A: 'aa', I: 'ih', U: 'ou', E: 'ee', O: 'oh' },
  lipSyncProfiles: { zh: 'zh.json', en: 'en.json', ja: 'ja.json' },
  idleSet: [{ motionId: 'idle_stand', weight: 1.0, cooldownS: 5.0 }],
};

/**
 * Creates a mock VRM instance with humanoid bones, expressions, and spring bones.
 */
function createMockVRM(): { vrm: VRM; boneNodes: Map<string, THREE.Object3D> } {
  const scene = new THREE.Group();
  const boneNodes = new Map<string, THREE.Object3D>();

  const humanoidBones: Record<string, { node: THREE.Object3D }> = {};
  const allBoneNames = [
    'hips',
    'spine',
    'chest',
    'upperChest',
    'neck',
    'head',
    'leftEye',
    'rightEye',
    'leftShoulder',
    'rightShoulder',
    'leftUpperArm',
    'rightUpperArm',
    'leftLowerArm',
    'rightLowerArm',
    'leftHand',
    'rightHand',
    'leftUpperLeg',
    'rightUpperLeg',
    'leftLowerLeg',
    'rightLowerLeg',
    'leftFoot',
    'rightFoot',
  ];

  let parent: THREE.Object3D = scene;
  for (const name of allBoneNames) {
    const node = new THREE.Object3D();
    node.name = name;
    node.uuid = `uuid_${name}`;
    parent.add(node);
    boneNodes.set(name, node);
    humanoidBones[name] = { node };
    // Keep hierarchical structure
    if (name === 'hips' || name === 'spine' || name === 'chest') {
      parent = node;
    }
  }

  // Ear and tail bones
  const earL = new THREE.Object3D();
  earL.name = 'earRoot_L';
  earL.uuid = 'uuid_earRoot_L';
  scene.add(earL);

  const earR = new THREE.Object3D();
  earR.name = 'earRoot_R';
  earR.uuid = 'uuid_earRoot_R';
  scene.add(earR);

  const tail = new THREE.Object3D();
  tail.name = 'tail_01';
  tail.uuid = 'uuid_tail_01';
  scene.add(tail);

  const expressionValues = new Map<string, number>();
  const expressionManager = {
    setValue: vi.fn((name: string, val: number) => {
      expressionValues.set(name, val);
    }),
    getValue: vi.fn((name: string) => expressionValues.get(name) ?? 0),
    update: vi.fn(),
  };

  const springBoneManager = {
    update: vi.fn(),
    reset: vi.fn(),
  };

  const vrm = {
    scene,
    humanoid: {
      humanBones: humanoidBones,
      getNormalizedBoneNode: (name: string) => humanoidBones[name]?.node ?? null,
    },
    expressionManager,
    springBoneManager,
  } as unknown as VRM;

  return { vrm, boneNodes };
}

/**
 * Creates a synthetic AnimationClip targeting humanoid bones.
 */
function createSyntheticClip(
  name: string,
  duration = 2.0,
  maxAngularSpeedRadPerSec = 0.5,
): THREE.AnimationClip {
  const tracks: THREE.KeyframeTrack[] = [];
  const times = [0, duration * 0.5, duration];

  const halfAngle = (maxAngularSpeedRadPerSec * duration * 0.5) / 2;
  const q0 = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), 0);
  const q1 = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), halfAngle);
  const q2 = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), 0);

  const values = [q0.x, q0.y, q0.z, q0.w, q1.x, q1.y, q1.z, q1.w, q2.x, q2.y, q2.z, q2.w];

  for (const boneName of ['hips', 'spine', 'chest', 'leftUpperArm', 'rightUpperArm']) {
    tracks.push(new THREE.QuaternionKeyframeTrack(`${boneName}.quaternion`, times, values));
  }

  return new THREE.AnimationClip(name, duration, tracks);
}

describe('P3-01 Animation Runtime (Five Layers)', () => {
  describe('Math & Curve Utilities', () => {
    test('smoothHermite satisfies C1 boundary conditions and monotonicity', () => {
      expect(smoothHermite(0)).toBe(0);
      expect(smoothHermite(1)).toBe(1);
      expect(smoothHermite(0.5)).toBe(0.5);

      // Monotonicity
      for (let t = 0; t <= 1.0; t += 0.05) {
        const next = t + 0.05;
        if (next <= 1.0) {
          expect(smoothHermite(next)).toBeGreaterThanOrEqual(smoothHermite(t));
        }
      }

      // Zero derivative at endpoints: [h(eps) - h(0)] / eps -> 0 as eps -> 0
      const eps = 0.0001;
      const slope0 = (smoothHermite(eps) - smoothHermite(0)) / eps;
      const slope1 = (smoothHermite(1) - smoothHermite(1 - eps)) / eps;
      expect(slope0).toBeLessThan(0.001);
      expect(slope1).toBeLessThan(0.001);
    });

    test('CriticallyDampedSpring exhibits zero overshoot and monotonic convergence', () => {
      const spring = new CriticallyDampedSpring(0, 10.0);
      const target = 1.0;
      let prevVal = 0;

      // Step over 1 second with delta = 0.016s
      for (let i = 0; i < 60; i++) {
        const val = spring.update(target, 0.016);
        expect(val).toBeGreaterThanOrEqual(prevVal); // Monotonic increase
        expect(val).toBeLessThanOrEqual(target); // Zero overshoot
        prevVal = val;
      }

      expect(prevVal).toBeGreaterThan(0.95); // Approached target closely
    });

    test('DelayBuffer correctly returns delayed values', () => {
      const buffer = new DelayBuffer<number>(2.0);
      const dt = 0.05;

      for (let t = 0; t <= 1.0; t += dt) {
        buffer.push(t, dt);
      }

      // Query value from 0.2s ago
      const delayed = buffer.getDelayed(0.2);
      expect(delayed).toBeDefined();
      expect(delayed!).toBeCloseTo(0.8, 1);
    });

    test('alignLoopHeadToTail aligns rotation difference to <= 2 degrees', () => {
      // Create clip with 10 degree discrepancy at loop seam
      const times = [0, 1.0];
      const qStart = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), 0);
      const qEnd = new THREE.Quaternion().setFromAxisAngle(
        new THREE.Vector3(1, 0, 0),
        THREE.MathUtils.degToRad(10.0),
      );
      const track = new THREE.QuaternionKeyframeTrack('spine.quaternion', times, [
        qStart.x,
        qStart.y,
        qStart.z,
        qStart.w,
        qEnd.x,
        qEnd.y,
        qEnd.z,
        qEnd.w,
      ]);
      const clip = new THREE.AnimationClip('unaligned_loop', 1.0, [track]);

      alignLoopHeadToTail(clip);

      const modifiedValues = clip.tracks[0]!.values;
      const finalQ = new THREE.Quaternion(
        modifiedValues[4],
        modifiedValues[5],
        modifiedValues[6],
        modifiedValues[7],
      );
      const dot = Math.abs(qStart.dot(finalQ));
      const diffDeg = THREE.MathUtils.radToDeg(2 * Math.acos(Math.min(1, dot)));

      expect(diffDeg).toBeLessThanOrEqual(2.0);
    });

    test('solveTwoBoneIK accurately reaches target', () => {
      const rootPos = new THREE.Vector3(0, 1.0, 0);
      const midPos = new THREE.Vector3(0, 0.5, 0);
      const endPos = new THREE.Vector3(0, 0, 0);
      const targetPos = new THREE.Vector3(0.2, 0.2, 0.2);
      const poleVector = new THREE.Vector3(0, 0.5, 1.0);

      const solution = solveTwoBoneIK({
        rootPos,
        midPos,
        endPos,
        targetPos,
        poleVector,
      });

      expect(solution.rootRotation).toBeInstanceOf(THREE.Quaternion);
      expect(solution.midRotation).toBeInstanceOf(THREE.Quaternion);
      expect(solution.rootRotation.length()).toBeCloseTo(1.0, 4);
      expect(solution.midRotation.length()).toBeCloseTo(1.0, 4);
    });
  });

  describe('L0 Base Pose Layer', () => {
    test('enforces cross-fade duration constraint (min 150ms, default 250ms)', () => {
      const { vrm } = createMockVRM();
      const l0 = new BasePoseLayer(vrm);

      const clipA = createSyntheticClip('clipA', 1.0);
      const clipB = createSyntheticClip('clipB', 1.0);

      l0.playClip(clipA);
      expect(l0.getCurrentClip()).toBe(clipA);

      // Attempt transition with 50ms (below 150ms minimum)
      l0.playClip(clipB, { crossFadeDurationMs: 50 });
      expect(l0.isCrossFading()).toBe(true);

      // Update by 100ms: transition should still be active because duration is clamped to >= 150ms
      l0.update(0.1);
      expect(l0.isCrossFading()).toBe(true);

      // Update past 150ms: transition finishes
      l0.update(0.06);
      expect(l0.isCrossFading()).toBe(false);
      expect(l0.getCurrentClip()).toBe(clipB);
    });

    test('100 deterministic L0 switches satisfy angular velocity bound (<= 1.5x clip max)', () => {
      const { vrm } = createMockVRM();
      const l0 = new BasePoseLayer(vrm);

      const clipA = createSyntheticClip('clipA', 2.0, 0.4);
      const clipB = createSyntheticClip('clipB', 2.0, 0.4);

      // Measure max angular velocity of clip A alone
      l0.playClip(clipA);
      l0.resetVelocityTracking();
      const dt = 1 / 60; // 60 FPS deterministic stepping

      for (let t = 0; t < 2.0; t += dt) {
        l0.update(dt);
      }
      const clipMaxAngularVel = l0.getMaxObservedAngularVelocity();
      expect(clipMaxAngularVel).toBeGreaterThan(0);

      // Perform 100 deterministic switches between clipA and clipB
      l0.resetVelocityTracking();
      let currentClipIndex = 0;

      for (let switchCount = 0; switchCount < 100; switchCount++) {
        const nextClip = currentClipIndex === 0 ? clipB : clipA;
        currentClipIndex = 1 - currentClipIndex;

        // Switch with default 250ms cross-fade
        l0.playClip(nextClip, { crossFadeDurationMs: 250 });

        // Step through the transition and some clip frames
        for (let step = 0; step < 20; step++) {
          l0.update(dt);
        }
      }

      const transitionMaxVel = l0.getTransitionMaxAngularVelocity();
      // Specification acceptance: Transition angular velocity <= 1.5 * clip max angular velocity
      expect(transitionMaxVel).toBeLessThanOrEqual(clipMaxAngularVel * 1.5 + 0.05);
    });
  });

  describe('L1 Gesture Layer', () => {
    test('strictly masks lower-body bones while animating upper-body', () => {
      const { vrm, boneNodes } = createMockVRM();
      const l1 = new GestureLayer(vrm);

      // Create clip that animates both spine (upper) and hips (lower)
      const clip = createSyntheticClip('gesture_all', 1.0, 1.0);
      l1.playGesture(clip);

      const hipsNode = boneNodes.get('hips')!;
      const initialHipsRotation = hipsNode.quaternion.clone();

      // Step gesture forward
      l1.update(0.1);

      // Lower body node (hips) must remain strictly untouched
      expect(hipsNode.quaternion.x).toBeCloseTo(initialHipsRotation.x, 5);
      expect(hipsNode.quaternion.y).toBeCloseTo(initialHipsRotation.y, 5);
      expect(hipsNode.quaternion.z).toBeCloseTo(initialHipsRotation.z, 5);
      expect(hipsNode.quaternion.w).toBeCloseTo(initialHipsRotation.w, 5);
    });

    test('follows 120ms fade-in and 200ms fade-out curves', () => {
      const { vrm } = createMockVRM();
      const l1 = new GestureLayer(vrm);
      const clip = createSyntheticClip('gesture_fade', 2.0);

      l1.playGesture(clip);
      expect(l1.getState()).toBe('fadingIn');
      expect(l1.getCurrentWeight()).toBe(0);

      // Halfway through fade-in (60ms)
      l1.update(0.06);
      expect(l1.getState()).toBe('fadingIn');
      expect(l1.getCurrentWeight()).toBeCloseTo(0.5, 1);

      // Complete fade-in (120ms)
      l1.update(0.06);
      expect(l1.getState()).toBe('playing');
      expect(l1.getCurrentWeight()).toBeCloseTo(1.0, 4);

      // Request stop -> fade-out
      l1.stopGesture();
      expect(l1.getState()).toBe('fadingOut');

      // Halfway through fade-out (100ms)
      l1.update(0.1);
      expect(l1.getCurrentWeight()).toBeCloseTo(0.5, 1);

      // Complete fade-out (200ms)
      l1.update(0.1);
      expect(l1.getState()).toBe('idle');
      expect(l1.getCurrentWeight()).toBe(0);
    });
  });

  describe('L2 Procedural Body Layer', () => {
    test('scales breathing rhythm by idleTempo', () => {
      const { vrm } = createMockVRM();
      const l2 = new ProceduralLayer({ vrm, profile: testProfile });

      // Step 1 second with idleTempo = 1.0
      l2.setReadoutParameters({ idleTempo: 1.0 });
      l2.update(1.0);
      const phaseAt1s = l2.getBreathingPhase();

      // Reset and step 1 second with idleTempo = 1.5
      const l2Fast = new ProceduralLayer({ vrm, profile: testProfile });
      l2Fast.setReadoutParameters({ idleTempo: 1.5 });
      l2Fast.update(1.0);
      const phaseAt1sFast = l2Fast.getBreathingPhase();

      expect(phaseAt1sFast).toBeCloseTo(phaseAt1s * 1.5, 2);
    });

    test('clamps gaze yaw and pitch to profile limits', () => {
      const { vrm } = createMockVRM();
      const l2 = new ProceduralLayer({ vrm, profile: testProfile });

      // Target far to the right (x = 100, exceeding 45 deg limit)
      l2.setGazeWorldPosition(100, 1.45, 0);

      // Step until springs settle
      for (let i = 0; i < 120; i++) {
        l2.update(0.016);
      }

      const { eyeYaw, headYaw, neckYaw } = l2.getGazeAngles();
      const maxYawRad = THREE.MathUtils.degToRad(testProfile.gazeLimits.yawDeg);

      // Total eye yaw must be clamped to profile.gazeLimits.yawDeg
      expect(eyeYaw).toBeLessThanOrEqual(maxYawRad + 0.01);
      // Head weight is 0.6
      expect(headYaw).toBeCloseTo(maxYawRad * 0.6, 2);
      // Neck weight is 0.3
      expect(neckYaw).toBeCloseTo(maxYawRad * 0.3, 2);
    });

    test('delays head (120ms) and neck (200ms) behind eye (0ms)', () => {
      const { vrm } = createMockVRM();
      const l2 = new ProceduralLayer({ vrm, profile: testProfile });

      // Step to settle initial target
      for (let i = 0; i < 30; i++) l2.update(0.016);

      // Isolate the fixed delay from the v2 tendency to look toward the user.
      l2.setReadoutParameters({ gazeUserWeight: 0 });
      // Abrupt target change
      l2.setGazeWorldPosition(0.5, 1.45, 2.0);

      // At 50ms: Eye has reacted, head has barely moved (120ms delay), neck has not moved (200ms delay)
      l2.update(0.05);
      const at50ms = l2.getGazeAngles();

      expect(at50ms.eyeYaw).toBeGreaterThan(0.04);
      expect(at50ms.neckYaw).toBeLessThan(0.01);
    });
  });

  describe('L3 Facial Expression & Blink Layer', () => {
    test('blink interval distribution over 10,000 samples matches specs', () => {
      // Deterministic PRNG for statistical test
      let seed = 123456789;
      const deterministicPrng = () => {
        seed = (seed * 1664525 + 1013904223) % 4294967296;
        return seed / 4294967296;
      };

      const l3 = new FacialLayer({
        profile: testProfile,
        prng: deterministicPrng,
      });

      const sampleCount = 10000;
      let intervalSum = 0;
      let minInterval = Infinity;
      let maxInterval = -Infinity;
      let doubleBlinkCount = 0;

      for (let i = 0; i < sampleCount; i++) {
        const interval = l3.scheduleNextBlink();
        intervalSum += interval;
        if (interval < minInterval) minInterval = interval;
        if (interval > maxInterval) maxInterval = interval;
        if (l3.getIsDoubleBlink()) doubleBlinkCount++;
      }

      const meanInterval = intervalSum / sampleCount;
      const doubleBlinkRatio = doubleBlinkCount / sampleCount;

      // Specification acceptance criteria:
      // 1. Range is [2.0, 6.0]s
      expect(minInterval).toBeGreaterThanOrEqual(2.0);
      expect(maxInterval).toBeLessThanOrEqual(6.0);

      // 2. Mean is approx 4.0s (within 3.8s .. 4.2s)
      expect(meanInterval).toBeGreaterThanOrEqual(3.85);
      expect(meanInterval).toBeLessThanOrEqual(4.15);

      // 3. Double-blink probability is approx 15% (within 13% .. 17%)
      expect(doubleBlinkRatio).toBeGreaterThanOrEqual(0.13);
      expect(doubleBlinkRatio).toBeLessThanOrEqual(0.17);
    });

    test('inhibits blink when surprised expression is active', () => {
      const { vrm } = createMockVRM();
      const l3 = new FacialLayer({ vrm, profile: testProfile });

      l3.setPerformanceCue({ emotion: 'surprised', weight: 1.0 });

      // Step forward 5 seconds (well past normal 2-6s interval)
      for (let i = 0; i < 300; i++) {
        l3.update(0.016);
        expect(l3.getBlinkWeight()).toBe(0); // Blinking must remain blocked
      }
    });

    test('clamps expression weight to 0.6 during speech and recovers over 300ms', () => {
      const { vrm } = createMockVRM();
      const l3 = new FacialLayer({ vrm, profile: testProfile });

      l3.setPerformanceCue({ emotion: 'happy', weight: 1.0 });
      l3.update(0.016);

      // When not speaking, ceiling is 1.0
      expect(l3.getCurrentExpressionCeiling()).toBeCloseTo(1.0, 4);

      // Speech begins with viseme 'aa'
      l3.setVisemes({ aa: 0.8 });
      l3.update(0.016);

      // Expression ceiling must immediately drop to 0.6
      expect(l3.getCurrentExpressionCeiling()).toBeCloseTo(0.6, 4);

      // Speech ends
      l3.setVisemes({ aa: 0.0 });
      l3.update(0.016);

      // Halfway through 300ms recovery (150ms)
      l3.update(0.15);
      expect(l3.getCurrentExpressionCeiling()).toBeGreaterThan(0.6);
      expect(l3.getCurrentExpressionCeiling()).toBeLessThan(1.0);

      // After 300ms recovery complete
      l3.update(0.2);
      expect(l3.getCurrentExpressionCeiling()).toBeCloseTo(1.0, 4);
    });
  });

  describe('L4 Secondary Motion Layer', () => {
    test('maps earPose and tailAmplitude to rig limits', () => {
      const { vrm } = createMockVRM();
      const l4 = new SecondaryMotionLayer({ vrm, profile: testProfile });

      l4.setReadoutParameters({ earPose: 0.8, tailAmplitude: 0.5 });
      expect(l4.getEffectiveEarPose()).toBe(0.8);
      expect(l4.getEffectiveTailAmplitude()).toBe(0.5);

      // Temporary override takes precedence
      l4.setEarOverride(-0.5, 1.0);
      expect(l4.getEffectiveEarPose()).toBe(-0.5);

      // After override duration expires
      l4.update(1.1);
      expect(l4.getEffectiveEarPose()).toBe(0.8);
    });

    test('updates spring bones during update', () => {
      const { vrm } = createMockVRM();
      const l4 = new SecondaryMotionLayer({ vrm, profile: testProfile });

      l4.update(0.016);
      expect(vrm.springBoneManager?.update).toHaveBeenCalledWith(0.016);
    });
  });

  describe('Animation Compositor', () => {
    test('v2 stays bounded through 10,800 frames without a base clip, and duplicate apply does not drift', () => {
      const { vrm, boneNodes } = createMockVRM();
      const compositor = new AnimationCompositor({ vrm, profile: testProfile });
      const readout = { version: 'v2-test', facing: 0.5, tension: 1, energy: 1 };
      for (let i = 0; i < 10800; i++) {
        compositor.applyReadout(readout);
        compositor.update(1 / 60);
        expect(Math.abs(boneNodes.get('hips')!.position.x)).toBeLessThanOrEqual(0.0075);
        expect(boneNodes.get('spine')!.quaternion.angleTo(new THREE.Quaternion())).toBeLessThan(
          0.06,
        );
      }
      compositor.applyReadout({ ...readout, tension: 0 });
      compositor.update(1 / 60);
      expect(boneNodes.get('spine')!.rotation.x).toBeCloseTo(0, 8);
      expect(
        vrm.scene.getObjectByName('earRoot_L')!.quaternion.angleTo(new THREE.Quaternion()),
      ).toBeCloseTo(0, 8);
    });

    test('L0 external pose writes survive offset restoration and toggling L2/L4 removes their offsets', () => {
      const { vrm, boneNodes } = createMockVRM();
      const compositor = new AnimationCompositor({ vrm, profile: testProfile });
      compositor.applyReadout({ version: 'v2', facing: 0.5, tension: 1, energy: 0.5 });
      compositor.update(1 / 60);
      const spine = boneNodes.get('spine')!;
      spine.rotation.set(0.2, 0, 0);
      compositor.setLayerEnabled('L2', false);
      compositor.setLayerEnabled('L4', false);
      compositor.update(1 / 60);
      expect(spine.rotation.x).toBeCloseTo(0.2, 8);
      expect(
        vrm.scene.getObjectByName('earRoot_L')!.quaternion.angleTo(new THREE.Quaternion()),
      ).toBeCloseTo(0, 8);
    });

    test('facing changes gaze toward user; energy changes breathing phase and sway amplitude', () => {
      const low = createMockVRM();
      const high = createMockVRM();
      const a = new AnimationCompositor({ vrm: low.vrm, profile: testProfile });
      const b = new AnimationCompositor({ vrm: high.vrm, profile: testProfile });
      a.setGaze('away');
      b.setGaze('away');
      a.applyReadout({ version: 'v2', facing: 0, tension: 0, energy: 0 });
      b.applyReadout({ version: 'v2', facing: 1, tension: 0, energy: 1 });
      for (let i = 0; i < 60; i++) {
        a.update(1 / 60);
        b.update(1 / 60);
      }
      expect(Math.abs(a.l2.getGazeAngles().headYaw)).toBeGreaterThan(
        Math.abs(b.l2.getGazeAngles().headYaw),
      );
      expect(b.l2.getBreathingPhase()).toBeCloseTo(a.l2.getBreathingPhase() * 3, 8);
      expect(Math.abs(high.boneNodes.get('hips')!.position.x)).toBeGreaterThan(
        Math.abs(low.boneNodes.get('hips')!.position.x),
      );
    });

    test('emo custom bindings clear at sentence end, with blink and visemes retained', () => {
      const { vrm } = createMockVRM();
      const profile = {
        ...testProfile,
        emotionMap: {
          ...testProfile.emotionMap,
          happy: [{ expression: 'nemuBlush', weight: 1 }],
        },
      };
      const compositor = new AnimationCompositor({ vrm, profile, prng: () => 0.5 });
      compositor.setPerformanceEmotion({ emotion: 'happy', weight: 0.8 });
      compositor.setVisemes({ aa: 0.9 });
      compositor.update(1 / 60);
      expect(vrm.expressionManager!.getValue('nemuBlush')).toBe(0.6);
      expect(vrm.expressionManager!.getValue('aa')).toBe(0.9);
      compositor.onSentenceEnd();
      for (let i = 0; i < 30; i++) compositor.update(1 / 60);
      expect(vrm.expressionManager!.getValue('nemuBlush')).toBe(0);
      expect(vrm.expressionManager!.getValue('aa')).toBe(0.9);
      expect('setExpressionBaseline' in compositor.l3).toBe(false);
    });

    test('coordinates L0 to L4 in order and supports layer toggles', () => {
      const { vrm } = createMockVRM();
      const compositor = new AnimationCompositor({ vrm, profile: testProfile });

      expect(compositor.isLayerEnabled('L0')).toBe(true);
      expect(compositor.isLayerEnabled('L1')).toBe(true);
      expect(compositor.isLayerEnabled('L2')).toBe(true);
      expect(compositor.isLayerEnabled('L3')).toBe(true);
      expect(compositor.isLayerEnabled('L4')).toBe(true);

      compositor.setLayerEnabled('L2', false);
      expect(compositor.isLayerEnabled('L2')).toBe(false);

      // Should step without error
      compositor.update(0.016);
    });

    test('correctly dispatches BodyReadout across layers', () => {
      const { vrm } = createMockVRM();
      const compositor = new AnimationCompositor({ vrm, profile: testProfile });

      compositor.applyReadout({
        version: '1.0',
        facing: 0.8,
        tension: 0.6,
        energy: 0.7,
      });

      expect(compositor.l4.getEffectiveEarPose()).toBeCloseTo(-0.3);
      expect(compositor.l4.getEffectiveTailAmplitude()).toBeCloseTo(0.35);
    });
  });
});
