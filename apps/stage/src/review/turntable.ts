import * as THREE from 'three';

export type TurntablePreset = 'front' | 'side' | 'back' | 'threeQuarter';

export interface TurntableOptions {
  domElement: HTMLElement;
  camera: THREE.PerspectiveCamera;
  target?: THREE.Vector3;
  distance?: number;
}

export class TurntableController {
  private readonly camera: THREE.PerspectiveCamera;
  private readonly domElement: HTMLElement;
  private target: THREE.Vector3;

  private azimuth = 0;   // Radians around Y axis (0 = front)
  private elevation = 0; // Radians above horizontal plane
  private distance = 2.5;

  private autoRotate = false;
  private autoRotateSpeedRadPerSec = 0.5; // ~0.5 rad/s

  private isDragging = false;
  private previousMouseX = 0;
  private previousMouseY = 0;

  // Animation interpolation
  private targetAzimuth: number | null = null;
  private targetElevation: number | null = null;

  constructor(options: TurntableOptions) {
    this.camera = options.camera;
    this.domElement = options.domElement;
    this.target = options.target?.clone() ?? new THREE.Vector3(0, 1.0, 0);
    this.distance = options.distance ?? 2.5;

    this.setupEventListeners();
    this.updateCameraTransform();
  }

  setTarget(target: THREE.Vector3): void {
    this.target.copy(target);
    this.updateCameraTransform();
  }

  setDistance(distance: number): void {
    this.distance = Math.max(0.5, Math.min(10.0, distance));
    this.updateCameraTransform();
  }

  setAutoRotate(enabled: boolean, speedRadPerSec = 0.5): void {
    this.autoRotate = enabled;
    this.autoRotateSpeedRadPerSec = speedRadPerSec;
  }

  isAutoRotating(): boolean {
    return this.autoRotate;
  }

  setPreset(preset: TurntablePreset, smooth = true): void {
    let desiredAzimuth = 0;
    switch (preset) {
      case 'front':
        desiredAzimuth = 0;
        break;
      case 'threeQuarter':
        desiredAzimuth = Math.PI / 4; // 45 deg
        break;
      case 'side':
        desiredAzimuth = Math.PI / 2; // 90 deg
        break;
      case 'back':
        desiredAzimuth = Math.PI; // 180 deg
        break;
    }

    if (!smooth) {
      this.azimuth = desiredAzimuth;
      this.elevation = 0;
      this.targetAzimuth = null;
      this.targetElevation = null;
      this.updateCameraTransform();
    } else {
      this.targetAzimuth = desiredAzimuth;
      this.targetElevation = 0;
    }
  }

  update(deltaSec: number): void {
    if (this.autoRotate && !this.isDragging) {
      this.azimuth = (this.azimuth + this.autoRotateSpeedRadPerSec * deltaSec) % (Math.PI * 2);
      this.updateCameraTransform();
    } else if (this.targetAzimuth !== null && this.targetElevation !== null) {
      // Smoothly slerp/lerp to target preset
      const speed = 8.0 * deltaSec;
      let diffAz = (this.targetAzimuth - this.azimuth) % (Math.PI * 2);
      if (diffAz > Math.PI) diffAz -= Math.PI * 2;
      if (diffAz < -Math.PI) diffAz += Math.PI * 2;

      this.azimuth += diffAz * Math.min(1.0, speed);
      this.elevation += (this.targetElevation - this.elevation) * Math.min(1.0, speed);

      if (Math.abs(diffAz) < 0.001 && Math.abs(this.targetElevation - this.elevation) < 0.001) {
        this.azimuth = this.targetAzimuth;
        this.elevation = this.targetElevation;
        this.targetAzimuth = null;
        this.targetElevation = null;
      }
      this.updateCameraTransform();
    }
  }

  private updateCameraTransform(): void {
    // Spherical to Cartesian relative to target
    const cosEl = Math.cos(this.elevation);
    const sinEl = Math.sin(this.elevation);
    const sinAz = Math.sin(this.azimuth);
    const cosAz = Math.cos(this.azimuth);

    const x = this.target.x + this.distance * cosEl * sinAz;
    const y = this.target.y + this.distance * sinEl;
    const z = this.target.z + this.distance * cosEl * cosAz;

    this.camera.position.set(x, y, z);
    this.camera.lookAt(this.target);
  }

  private setupEventListeners(): void {
    this.domElement.addEventListener('pointerdown', (e) => {
      this.isDragging = true;
      this.targetAzimuth = null;
      this.targetElevation = null;
      this.previousMouseX = e.clientX;
      this.previousMouseY = e.clientY;
      this.domElement.setPointerCapture(e.pointerId);
    });

    this.domElement.addEventListener('pointermove', (e) => {
      if (!this.isDragging) return;

      const deltaX = e.clientX - this.previousMouseX;
      const deltaY = e.clientY - this.previousMouseY;
      this.previousMouseX = e.clientX;
      this.previousMouseY = e.clientY;

      // Rotate azimuth and elevation
      this.azimuth -= deltaX * 0.008;
      this.elevation += deltaY * 0.008;

      // Clamp elevation [-80 deg, +80 deg]
      const maxEl = (80 * Math.PI) / 180;
      this.elevation = Math.max(-maxEl, Math.min(maxEl, this.elevation));

      this.updateCameraTransform();
    });

    const stopDragging = (e: PointerEvent) => {
      if (this.isDragging) {
        this.isDragging = false;
        try {
          this.domElement.releasePointerCapture(e.pointerId);
        } catch {
          // Ignore if pointer capture already lost
        }
      }
    };

    this.domElement.addEventListener('pointerup', stopDragging);
    this.domElement.addEventListener('pointercancel', stopDragging);

    // Mouse wheel for dolly zoom
    this.domElement.addEventListener('wheel', (e) => {
      e.preventDefault();
      this.distance += e.deltaY * 0.002;
      this.distance = Math.max(0.6, Math.min(6.0, this.distance));
      this.updateCameraTransform();
    }, { passive: false });
  }

  getAzimuthDeg(): number {
    const deg = THREE.MathUtils.radToDeg(this.azimuth) % 360;
    return deg < 0 ? deg + 360 : deg;
  }

  getDistance(): number {
    return this.distance;
  }
}
