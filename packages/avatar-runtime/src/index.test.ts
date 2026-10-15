import { describe, expect, test, vi } from 'vitest';
import * as THREE from 'three';
import {
  loadProfile,
  StageError,
  ThreePointLighting,
  AvatarCamera,
  adaptOutlineWidth,
  FrameScheduler,
} from './index.js';
import type { NemuProfile } from '@nemu/contracts';

const validProfile: NemuProfile = {
  version: 2,
  model: 'sample.vrm',
  heightM: 1.45,
  headTopOffsetM: 0.12,
  hitRegions: [
    {
      id: 'head',
      bone: 'head',
      shape: 'sphere',
      radiusM: 0.15,
      offsetM: [0, 0.05, 0],
    },
    {
      id: 'body',
      bone: 'spine',
      shape: 'capsule',
      radiusM: 0.18,
      lengthM: 0.35,
    },
  ],
  gazeLimits: {
    yawDeg: 45,
    pitchUpDeg: 25,
    pitchDownDeg: 30,
  },
  framing: {
    fullBody: {
      fovDeg: 20,
      distanceM: 3.5,
      targetBone: 'hips',
    },
    upperBody: {
      fovDeg: 20,
      distanceM: 1.6,
      targetBone: 'chest',
    },
  },
  rig: {
    earBonesL: [],
    earBonesR: [],
    tailBones: [],
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
  visemeMap: {
    A: 'aa',
    I: 'ih',
    U: 'ou',
    E: 'ee',
    O: 'oh',
  },
  lipSyncProfiles: {
    zh: 'lipsync/zh.json',
    en: 'lipsync/en.json',
    ja: 'lipsync/ja.json',
  },
  idleSet: [
    {
      motionId: 'idle_stand',
      weight: 1.0,
      cooldownS: 5.0,
    },
  ],
};

describe('avatar-runtime', () => {
  describe('Profile Loader', () => {
    test('successfully validates a conforming NemuProfile object', async () => {
      const parsed = await loadProfile(validProfile);
      expect(parsed.version).toBe(2);
      expect(parsed.model).toBe('sample.vrm');
    });

    test('successfully parses and validates JSON string profile', async () => {
      const jsonStr = JSON.stringify(validProfile);
      const parsed = await loadProfile(jsonStr);
      expect(parsed.heightM).toBe(1.45);
    });

    test('rejects profile with invalid schema', async () => {
      const invalid = { ...validProfile, version: 1 };
      await expect(loadProfile(invalid)).rejects.toThrow(StageError);
    });

    test('rejects capsule region missing lengthM', async () => {
      const invalid = {
        ...validProfile,
        hitRegions: [
          {
            id: 'body' as const,
            bone: 'spine',
            shape: 'capsule' as const,
            radiusM: 0.2,
          },
        ],
      };
      await expect(loadProfile(invalid)).rejects.toThrow(StageError);
    });
  });

  describe('Lighting', () => {
    test('creates default three-point lights and exposes modifiers', () => {
      const lighting = new ThreePointLighting();
      expect(lighting.group.children.length).toBe(3);

      lighting.setKeyLight(1.5, 0xff0000, [1, 2, 3]);
      expect(lighting.keyLight.intensity).toBe(1.5);
      expect(lighting.keyLight.position.x).toBe(1);

      lighting.setRimLight(0.9, 0x00ff00);
      expect(lighting.rimLight.intensity).toBe(0.9);

      lighting.setAmbientLight(0.3);
      expect(lighting.ambientLight.intensity).toBe(0.3);

      const config = lighting.getConfig();
      expect(config.keyLight.intensity).toBe(1.5);
      expect(config.ambientLight.intensity).toBe(0.3);

      lighting.dispose();
    });
  });

  describe('Camera & Framing', () => {
    test('computes bounds and updates framing modes', () => {
      const camera = new AvatarCamera(20.0, 1.0);
      camera.setAspect(1.5);
      expect(camera.camera.aspect).toBe(1.5);

      // Mock VRM
      const mockScene = new THREE.Group();
      const mesh = new THREE.Mesh(new THREE.BoxGeometry(0.5, 1.5, 0.3));
      mesh.position.set(0, 0.75, 0);
      mockScene.add(mesh);
      mesh.updateMatrixWorld(true);
      mockScene.updateMatrixWorld(true);

      const mockVrm = {
        scene: mockScene,
        humanoid: {
          getNormalizedBoneNode: vi.fn().mockReturnValue(null),
        },
      } as unknown as import('@pixiv/three-vrm').VRM;

      camera.applyFraming(mockVrm, validProfile, 'fullBody');
      expect(camera.getFramingMode()).toBe('fullBody');
      expect(camera.camera.position.z).toBeGreaterThan(0);

      const bounds = camera.computeScreenBounds(mockVrm, 600, 800);
      expect(bounds.w).toBeGreaterThan(0);
      expect(bounds.h).toBeGreaterThan(0);
      expect(bounds.x).toBeGreaterThanOrEqual(0);
      expect(bounds.y).toBeGreaterThanOrEqual(0);
    });

    test('clamps head gaze within profile limits', () => {
      const camera = new AvatarCamera();
      const headBone = new THREE.Object3D();
      headBone.position.set(0, 1.4, 0);

      const mockVrm = {
        scene: new THREE.Group(),
        humanoid: {
          getNormalizedBoneNode: vi.fn().mockImplementation((name: string) => {
            return name === 'head' ? headBone : null;
          }),
        },
      } as unknown as import('@pixiv/three-vrm').VRM;

      // Target far to the right (exceeding 45 deg limit)
      const targetPos = new THREE.Vector3(10, 1.4, 0);
      camera.applyHeadGaze(mockVrm, validProfile, targetPos);

      // Yaw should be clamped to 45 degrees in radians
      const maxRad = THREE.MathUtils.degToRad(validProfile.gazeLimits.yawDeg);
      expect(headBone.rotation.y).toBeCloseTo(maxRad, 2);
    });
  });

  describe('Outline Adaptation', () => {
    test('scales outline width factor proportionally across viewport heights', () => {
      const mockScene = new THREE.Group();
      const material = {
        outlineWidthFactor: 0.05,
        userData: {},
      };
      const mesh = new THREE.Mesh(
        new THREE.BufferGeometry(),
        material as unknown as THREE.Material,
      );
      mockScene.add(mesh);

      const mockVrm = { scene: mockScene } as import('@pixiv/three-vrm').VRM;

      // Height 300px (smaller canvas -> needs thicker factor to keep same pixel width)
      const count = adaptOutlineWidth(mockVrm, 300, { referenceHeightPx: 600 });
      expect(count).toBe(1);
      // scaleRatio = 600 / 300 = 2.0 -> outlineWidthFactor = 0.05 * 2 = 0.1
      expect(material.outlineWidthFactor).toBeCloseTo(0.1, 3);

      // Height 900px -> scaleRatio = 600 / 900 = 0.667 -> 0.05 * 0.667 = 0.0333
      adaptOutlineWidth(mockVrm, 900, { referenceHeightPx: 600 });
      expect(material.outlineWidthFactor).toBeCloseTo(0.05 * (600 / 900), 3);
    });
  });

  describe('Frame Scheduler', () => {
    test('supports target FPS changes and deterministic stepping', () => {
      const onRender = vi.fn();
      const onStats = vi.fn();

      const scheduler = new FrameScheduler({
        targetFps: 60,
        deterministic: true,
        onRender,
        onStats,
      });

      expect(scheduler.getTargetFps()).toBe(60);
      scheduler.setTargetFps(30);
      expect(scheduler.getTargetFps()).toBe(30);

      // Deterministic steps
      scheduler.step(16.67);
      expect(onRender).toHaveBeenCalledTimes(1);
      expect(onRender).toHaveBeenCalledWith(16.67 / 1000);

      scheduler.step(33.33);
      expect(onRender).toHaveBeenCalledTimes(2);

      const stats = scheduler.getRecentStats();
      expect(stats.renderedFrames).toBe(2);
      expect(typeof stats.p95FrameMs).toBe('number');
    });
  });
});
