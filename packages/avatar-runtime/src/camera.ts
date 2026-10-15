import * as THREE from 'three';
import type { VRM, VRMHumanBoneName } from '@pixiv/three-vrm';
import type { NemuProfile, Rect } from '@nemu/contracts';

export type FramingMode = 'fullBody' | 'upperBody';

export class AvatarCamera {
  readonly camera: THREE.PerspectiveCamera;
  private currentFraming: FramingMode = 'fullBody';
  private targetPosition: THREE.Vector3 = new THREE.Vector3();
  private lookTarget: THREE.Vector3 = new THREE.Vector3();

  constructor(fov = 20.0, aspect = 1.0, near = 0.1, far = 20.0) {
    this.camera = new THREE.PerspectiveCamera(fov, aspect, near, far);
    this.camera.position.set(0, 1.0, 3.5);
  }

  setAspect(aspect: number): void {
    this.camera.aspect = aspect;
    this.camera.updateProjectionMatrix();
  }

  /**
   * Applies framing configuration based on NemuProfile.
   */
  applyFraming(vrm: VRM, profile: NemuProfile, mode: FramingMode): void {
    this.currentFraming = mode;
    const framingConfig = profile.framing[mode];
    this.camera.fov = framingConfig.fovDeg;

    const targetBoneName = framingConfig.targetBone as VRMHumanBoneName;
    const boneNode = vrm.humanoid?.getNormalizedBoneNode(targetBoneName);

    const boneWorldPos = new THREE.Vector3();
    if (boneNode) {
      boneNode.getWorldPosition(boneWorldPos);
    } else {
      // Fallback: estimate from avatar height
      boneWorldPos.set(0, mode === 'fullBody' ? profile.heightM * 0.5 : profile.heightM * 0.8, 0);
    }

    if (framingConfig.offsetM) {
      boneWorldPos.add(new THREE.Vector3(...framingConfig.offsetM));
    }

    this.lookTarget.copy(boneWorldPos);
    this.targetPosition.set(
      boneWorldPos.x,
      boneWorldPos.y,
      boneWorldPos.z + framingConfig.distanceM,
    );

    this.camera.position.copy(this.targetPosition);
    this.camera.lookAt(this.lookTarget);
    this.camera.updateProjectionMatrix();
  }

  getFramingMode(): FramingMode {
    return this.currentFraming;
  }

  /**
   * Calculates the 2D screen bounding box of the avatar in canvas coordinates (pixels).
   */
  computeScreenBounds(vrm: VRM, canvasWidth: number, canvasHeight: number): Rect {
    const bbox = new THREE.Box3().setFromObject(vrm.scene);
    if (bbox.isEmpty()) {
      return { x: 0, y: 0, w: canvasWidth, h: canvasHeight };
    }

    const corners = [
      new THREE.Vector3(bbox.min.x, bbox.min.y, bbox.min.z),
      new THREE.Vector3(bbox.min.x, bbox.min.y, bbox.max.z),
      new THREE.Vector3(bbox.min.x, bbox.max.y, bbox.min.z),
      new THREE.Vector3(bbox.min.x, bbox.max.y, bbox.max.z),
      new THREE.Vector3(bbox.max.x, bbox.min.y, bbox.min.z),
      new THREE.Vector3(bbox.max.x, bbox.min.y, bbox.max.z),
      new THREE.Vector3(bbox.max.x, bbox.max.y, bbox.min.z),
      new THREE.Vector3(bbox.max.x, bbox.max.y, bbox.max.z),
    ];

    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    for (const corner of corners) {
      corner.project(this.camera);
      // NDC (-1 to 1) to screen pixel coordinates
      const x = ((corner.x + 1) / 2) * canvasWidth;
      const y = ((-corner.y + 1) / 2) * canvasHeight;

      if (x < minX) minX = x;
      if (x > maxX) maxX = x;
      if (y < minY) minY = y;
      if (y > maxY) maxY = y;
    }

    // Clamp to canvas boundaries
    const clampedMinX = Math.max(0, Math.floor(minX));
    const clampedMinY = Math.max(0, Math.floor(minY));
    const clampedMaxX = Math.min(canvasWidth, Math.ceil(maxX));
    const clampedMaxY = Math.min(canvasHeight, Math.ceil(maxY));

    return {
      x: clampedMinX,
      y: clampedMinY,
      w: Math.max(1, clampedMaxX - clampedMinX),
      h: Math.max(1, clampedMaxY - clampedMinY),
    };
  }

  /**
   * Applies head gaze direction towards target coordinates in screen or world space.
   */
  applyHeadGaze(vrm: VRM, profile: NemuProfile, targetWorldPos: THREE.Vector3): void {
    const headNode = vrm.humanoid?.getNormalizedBoneNode('head');
    if (!headNode) return;

    const headWorldPos = new THREE.Vector3();
    headNode.getWorldPosition(headWorldPos);

    const dir = new THREE.Vector3().subVectors(targetWorldPos, headWorldPos).normalize();
    // Compute yaw and pitch
    const yaw = Math.atan2(dir.x, dir.z);
    const pitch = -Math.asin(THREE.MathUtils.clamp(dir.y, -1, 1));

    const yawDeg = THREE.MathUtils.radToDeg(yaw);
    const pitchDeg = THREE.MathUtils.radToDeg(pitch);

    const clampedYawDeg = THREE.MathUtils.clamp(
      yawDeg,
      -profile.gazeLimits.yawDeg,
      profile.gazeLimits.yawDeg,
    );
    const clampedPitchDeg = THREE.MathUtils.clamp(
      pitchDeg,
      -profile.gazeLimits.pitchDownDeg,
      profile.gazeLimits.pitchUpDeg,
    );

    headNode.rotation.y = THREE.MathUtils.degToRad(clampedYawDeg);
    headNode.rotation.x = THREE.MathUtils.degToRad(clampedPitchDeg);
  }
}
