import { describe, expect, test, vi, beforeEach } from 'vitest';
import * as THREE from 'three';
import type { VRM } from '@pixiv/three-vrm';
import { TurntableController } from './turntable.js';
import { LightingAndBackgroundManager } from './lighting-bg.js';
import { SpringBoneInspector } from './spring-bones.js';
import { ThreePointLighting } from '@nemu/avatar-runtime';

describe('P1-03 Review Bench Utilities', () => {
  describe('TurntableController', () => {
    let camera: THREE.PerspectiveCamera;
    let mockDomElement: HTMLElement;
    let turntable: TurntableController;

    beforeEach(() => {
      camera = new THREE.PerspectiveCamera(20, 1.0, 0.1, 100);
      mockDomElement = {
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        setPointerCapture: vi.fn(),
        releasePointerCapture: vi.fn(),
      } as unknown as HTMLElement;

      turntable = new TurntableController({
        camera,
        domElement: mockDomElement,
        target: new THREE.Vector3(0, 1.0, 0),
        distance: 2.5,
      });
    });

    test('sets preset angles correctly', () => {
      turntable.setPreset('front', false);
      expect(turntable.getAzimuthDeg()).toBeCloseTo(0, 1);

      turntable.setPreset('side', false);
      expect(turntable.getAzimuthDeg()).toBeCloseTo(90, 1);

      turntable.setPreset('back', false);
      expect(turntable.getAzimuthDeg()).toBeCloseTo(180, 1);

      turntable.setPreset('threeQuarter', false);
      expect(turntable.getAzimuthDeg()).toBeCloseTo(45, 1);
    });

    test('updates azimuth during auto-rotation', () => {
      turntable.setPreset('front', false);
      turntable.setAutoRotate(true, 1.0); // 1.0 rad/s

      expect(turntable.isAutoRotating()).toBe(true);
      turntable.update(1.0); // 1 second

      // 1 radian approx 57.3 degrees
      expect(turntable.getAzimuthDeg()).toBeCloseTo(THREE.MathUtils.radToDeg(1.0), 1);
    });

    test('clamps distance between min and max bounds', () => {
      turntable.setDistance(0.1); // below min 0.5
      expect(turntable.getDistance()).toBe(0.5);

      turntable.setDistance(20.0); // above max 10.0
      expect(turntable.getDistance()).toBe(10.0);
    });
  });

  describe('LightingAndBackgroundManager', () => {
    let lighting: ThreePointLighting;
    let mockContainer: HTMLElement;
    let manager: LightingAndBackgroundManager;
    let classSet: Set<string>;

    beforeEach(() => {
      lighting = new ThreePointLighting();
      classSet = new Set<string>();
      mockContainer = {
        classList: {
          add: (c: string) => classSet.add(c),
          remove: (...cs: string[]) => cs.forEach((c) => classSet.delete(c)),
          contains: (c: string) => classSet.has(c),
        },
      } as unknown as HTMLElement;

      manager = new LightingAndBackgroundManager(lighting, mockContainer);
    });

    test('switches lighting presets and configures light intensities', () => {
      manager.applyLighting('studio');
      expect(manager.getCurrentLighting()).toBe('studio');
      expect(lighting.keyLight.intensity).toBe(1.5);

      manager.applyLighting('dim');
      expect(manager.getCurrentLighting()).toBe('dim');
      expect(lighting.keyLight.intensity).toBe(0.5);

      manager.applyLighting('dramatic');
      expect(manager.getCurrentLighting()).toBe('dramatic');
      expect(lighting.rimLight.intensity).toBe(1.6);
    });

    test('switches background classes on container', () => {
      manager.applyBackground('white');
      expect(mockContainer.classList.contains('bg-white')).toBe(true);

      manager.applyBackground('black');
      expect(mockContainer.classList.contains('bg-black')).toBe(true);
      expect(mockContainer.classList.contains('bg-white')).toBe(false);

      manager.applyBackground('gradient');
      expect(mockContainer.classList.contains('bg-gradient')).toBe(true);
      expect(mockContainer.classList.contains('bg-black')).toBe(false);
    });
  });

  describe('SpringBoneInspector', () => {
    test('adjusts spring parameters and generates build.yaml snippet', () => {
      const inspector = new SpringBoneInspector();

      const mockJoint = {
        stiffnessForce: 1.0,
        dragForce: 0.4,
        gravityPower: 1.0,
        gravityDir: { y: -1.0 },
      };

      const mockVrm = {
        springBoneManager: {
          joints: [mockJoint],
          reset: vi.fn(),
        },
      } as unknown as VRM;

      inspector.bindVRM(mockVrm);

      // Tune parameters
      inspector.setParameters({
        stiffnessMultiplier: 1.5,
        dragForce: 0.6,
        gravityPower: 1.2,
      });

      expect(mockJoint.stiffnessForce).toBeCloseTo(1.5, 2);
      expect(mockJoint.dragForce).toBeCloseTo(0.6, 2);
      expect(mockJoint.gravityPower).toBeCloseTo(1.2, 2);

      // Verify YAML export snippet
      const yaml = inspector.exportBuildYamlSnippet();
      expect(yaml).toContain('stiffnessMultiplier: 1.50');
      expect(yaml).toContain('dragForce: 0.60');
      expect(yaml).toContain('gravityPower: 1.20');
      expect(yaml).toContain('gravityDir: [0.0, -1.00, 0.0]');
    });
  });
});
