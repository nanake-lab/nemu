import * as THREE from 'three';
import type { VRM } from '@pixiv/three-vrm';
import {
  loadAvatar,
  AvatarRenderer,
  ThreePointLighting,
  AvatarCamera,
  adaptOutlineWidth,
  FrameScheduler,
  HitTestManager,
  type FramingMode,
  AnimationCompositor,
  type AnimationLayerId,
} from '@nemu/avatar-runtime';
import type {
  NemuProfile,
  PhysicalState,
  Rect,
  Anchor,
  GazeTarget,
  StagePayloads,
  BodyReadoutV2,
  Emotion,
} from '@nemu/contracts';
import { ReadoutApplicator, StageReadoutInput } from '@nemu/body';

export type StageEventHandler = <T extends keyof StagePayloads>(
  type: T,
  payload: StagePayloads[T],
) => void;

export interface StageAppOptions {
  container: HTMLElement;
  deterministic?: boolean;
  onEvent?: StageEventHandler;
}

export class StageApp {
  readonly container: HTMLElement;
  readonly canvasContainer: HTMLDivElement;
  readonly canvas: HTMLCanvasElement;

  readonly scene: THREE.Scene;
  readonly renderer: AvatarRenderer;
  readonly lighting: ThreePointLighting;
  readonly cameraManager: AvatarCamera;
  readonly scheduler: FrameScheduler;
  readonly compositor: AnimationCompositor;
  readonly readoutApplicator: ReadoutApplicator;
  readonly readoutInput: StageReadoutInput;
  private hitTestManager: HitTestManager | null = null;

  private currentVrm: VRM | null = null;
  private currentProfile: NemuProfile | null = null;

  private physicalState: PhysicalState = 'idle';
  private canvasWidth = 420;
  private canvasHeight = 600;
  private placementX = 0;
  private placementY = 0;

  private lastBoundsReportTime = 0;
  private lastReportedBounds: Rect | null = null;
  private readonly isDeterministic: boolean;
  private readonly eventListeners: StageEventHandler[] = [];

  // Quality settings
  private quality = {
    fpsActive: 60,
    fpsIdle: 30,
    fpsSleep: 12,
    msaa: 0 as 0 | 4,
  };

  constructor(options: StageAppOptions) {
    this.container = options.container;
    this.isDeterministic = options.deterministic ?? false;
    if (options.onEvent) {
      this.eventListeners.push(options.onEvent);
    }

    // 1. DOM Hierarchy (Strategy B: Overlay working area + translated canvas container)
    this.canvasContainer = document.createElement('div');
    this.canvasContainer.className = 'stage-canvas-container';
    this.canvasContainer.style.width = `${this.canvasWidth}px`;
    this.canvasContainer.style.height = `${this.canvasHeight}px`;
    this.canvasContainer.style.position = 'absolute';
    this.canvasContainer.style.transform = 'translate3d(0px, 0px, 0px)';
    this.canvasContainer.style.willChange = 'transform';
    this.container.appendChild(this.canvasContainer);

    this.canvas = document.createElement('canvas');
    this.canvas.className = 'stage-canvas';
    this.canvas.setAttribute('role', 'img');
    this.canvas.setAttribute('aria-label', 'Stage placeholder');
    this.canvasContainer.appendChild(this.canvas);

    // 2. Three.js Scene Setup
    this.scene = new THREE.Scene();

    this.renderer = new AvatarRenderer({
      canvas: this.canvas,
      width: this.canvasWidth,
      height: this.canvasHeight,
      antialiasMode: 'defaultMSAA',
    });

    this.lighting = new ThreePointLighting();
    this.scene.add(this.lighting.group);

    this.cameraManager = new AvatarCamera(20.0, this.canvasWidth / this.canvasHeight);
    this.compositor = new AnimationCompositor({ deterministic: this.isDeterministic });
    this.readoutApplicator = new ReadoutApplicator({
      onReadoutUpdated: (readout) => { this.compositor.applyReadout(readout); },
    });
    this.readoutInput = new StageReadoutInput(this.readoutApplicator);

    // 3. WebGL2 Async PBO Hit Test
    this.hitTestManager = new HitTestManager(this.renderer.renderer, {
      alphaThreshold: 0.1,
      maxFrequencyHz: 30,
      onHitTestChange: (overCharacter) => {
        this.emit('stage.hitTest', { overCharacter });
      },
    });

    // 4. Mouse Move Event Listener for Hit Testing
    this.canvas.addEventListener('mousemove', (e) => {
      const rect = this.canvas.getBoundingClientRect();
      const canvasX = e.clientX - rect.left;
      const canvasY = e.clientY - rect.top;
      this.hitTestManager?.onMouseMove(canvasX, canvasY, this.canvasHeight);
    });

    // 5. Frame Scheduler
    this.scheduler = new FrameScheduler({
      targetFps: this.quality.fpsActive,
      deterministic: this.isDeterministic,
      onRender: (deltaSec) => this.renderFrame(deltaSec),
      onStats: (stats) => {
        this.emit('stage.frameStats', {
          fps: stats.fps,
          p95FrameMs: stats.p95FrameMs,
          state: this.physicalState,
        });
      },
    });

    // Handle window resize
    window.addEventListener('resize', () => this.onResize());
  }

  addEventListener(handler: StageEventHandler): void {
    this.eventListeners.push(handler);
  }

  removeEventListener(handler: StageEventHandler): void {
    const idx = this.eventListeners.indexOf(handler);
    if (idx !== -1) this.eventListeners.splice(idx, 1);
  }

  emit<T extends keyof StagePayloads>(type: T, payload: StagePayloads[T]): void {
    for (const listener of this.eventListeners) {
      try {
        listener(type, payload);
      } catch (err) {
        console.error(`Error in stage event listener for ${String(type)}:`, err);
      }
    }

    // Also dispatch as DOM CustomEvent / window message
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('nemu:stage', { detail: { type, payload } }));
      window.postMessage({ v: 2, type, payload }, '*');
    }
  }

  async loadAvatar(vrmUrl: string, profileUrl: string): Promise<void> {
    try {
      const { vrm, profile } = await loadAvatar(vrmUrl, profileUrl);

      // Remove existing VRM if present
      if (this.currentVrm) {
        this.scene.remove(this.currentVrm.scene);
        this.currentVrm = null;
      }

      this.currentVrm = vrm;
      this.currentProfile = profile;
      this.scene.add(vrm.scene);

      // Bind avatar to animation compositor
      this.compositor.bindAvatar(vrm, profile);

      // Apply default framing
      this.cameraManager.applyFraming(vrm, profile, 'fullBody');

      // Adapt outline width to current canvas height
      adaptOutlineWidth(vrm, this.canvasHeight);

      // In deterministic mode, disable spring bone random perturbation
      if (this.isDeterministic) {
        vrm.springBoneManager?.reset();
      }

      // Initial frame render
      this.renderFrame(0);

      // Compute initial bounds and emit stage.ready
      const localBounds = this.cameraManager.computeScreenBounds(
        vrm,
        this.canvasWidth,
        this.canvasHeight,
      );
      const screenRect: Rect = {
        x: this.placementX + localBounds.x,
        y: this.placementY + localBounds.y,
        w: localBounds.w,
        h: localBounds.h,
      };

      const meta = vrm.meta as { name?: string } | undefined;
      this.emit('stage.ready', {
        avatarName: meta?.name || 'Nemu',
        boundsPx: screenRect,
      });

      this.lastReportedBounds = screenRect;
    } catch (err) {
      const code = (err as { code?: string }).code || 'AVATAR_LOAD_FAILED';
      const message = (err as Error).message || String(err);
      this.emit('stage.error', { code, message });
      throw err;
    }
  }

  setPlacement(displayId: number, anchor: Anchor, transitionMs = 0): void {
    if (anchor.kind === 'free') {
      this.placementX = anchor.x;
      this.placementY = anchor.y;

      if (transitionMs > 0) {
        this.canvasContainer.style.transition = `transform ${transitionMs}ms ease-out`;
      } else {
        this.canvasContainer.style.transition = 'none';
      }

      this.canvasContainer.style.transform = `translate3d(${anchor.x}px, ${anchor.y}px, 0px)`;
      this.checkAndReportBounds(true);
    }
  }

  setQuality(fpsActive: number, fpsIdle: number, fpsSleep: number, msaa: 0 | 4): void {
    this.quality = { fpsActive, fpsIdle, fpsSleep, msaa };
    this.renderer.setAntialiasMode(msaa === 4 ? 'renderTarget4x' : 'defaultMSAA');

    // Update scheduler rate depending on state
    const targetFps =
      this.physicalState === 'sleeping'
        ? fpsSleep
        : this.physicalState === 'idle'
          ? fpsIdle
          : fpsActive;
    this.scheduler.setTargetFps(targetFps);
  }

  setFraming(mode: FramingMode): void {
    if (!this.currentVrm || !this.currentProfile) return;
    this.cameraManager.applyFraming(this.currentVrm, this.currentProfile, mode);
    this.checkAndReportBounds(true);
  }

  setExpression(expressionName: string, weight = 1.0): void {
    if (!this.currentVrm) return;
    this.compositor.setPerformanceEmotion({ emotion: expressionName as Emotion, weight });
    const manager = this.currentVrm.expressionManager;
    if (!manager) return;

    // Reset common standard expressions
    const commonExpressions = ['happy', 'angry', 'sad', 'relaxed', 'surprised', 'neutral'];
    for (const exp of commonExpressions) {
      manager.setValue(exp, 0.0);
    }

    manager.setValue(expressionName, weight);
    manager.update();
  }

  gaze(target: GazeTarget): void {
    this.compositor.setGaze(target);
    if (!this.currentVrm || !this.currentProfile) return;

    let targetWorldPos: THREE.Vector3;
    if (target === 'user') {
      targetWorldPos = new THREE.Vector3(0, this.currentProfile.heightM * 0.9, 3.0);
    } else if (target === 'away') {
      targetWorldPos = new THREE.Vector3(2.0, this.currentProfile.heightM * 0.7, 2.5);
    } else if (typeof target === 'object' && 'x' in target && 'y' in target) {
      // Screen coordinates to world space approximation
      const ndcX = (target.x / window.innerWidth) * 2 - 1;
      const ndcY = -(target.y / window.innerHeight) * 2 + 1;
      targetWorldPos = new THREE.Vector3(
        ndcX * 2.0,
        this.currentProfile.heightM * 0.9 + ndcY * 0.5,
        2.0,
      );
    } else {
      targetWorldPos = new THREE.Vector3(0, this.currentProfile.heightM * 0.9, 2.0);
    }

    this.cameraManager.applyHeadGaze(this.currentVrm, this.currentProfile, targetWorldPos);
  }

  applyReadout(readout: BodyReadoutV2): void {
    this.readoutApplicator.applyReadout(readout);
  }

  setLayerEnabled(layer: AnimationLayerId, enabled: boolean): void {
    this.compositor.setLayerEnabled(layer, enabled);
  }

  isLayerEnabled(layer: AnimationLayerId): boolean {
    return this.compositor.isLayerEnabled(layer);
  }

  setPhysicalState(state: PhysicalState): void {
    this.physicalState = state;
    const targetFps =
      state === 'sleeping'
        ? this.quality.fpsSleep
        : state === 'idle'
          ? this.quality.fpsIdle
          : this.quality.fpsActive;
    this.scheduler.setTargetFps(targetFps);
  }

  getPhysicalState(): PhysicalState {
    return this.physicalState;
  }

  private renderFrame(deltaSec: number): void {
    if (this.currentVrm) {
      // In deterministic mode, freeze time step
      const stepDelta = this.isDeterministic ? 0.01667 : deltaSec;
      this.readoutApplicator.update(stepDelta);
      this.compositor.update(stepDelta);
      if (!this.compositor.isLayerEnabled('L3')) {
        this.currentVrm.expressionManager?.update();
      }
      if (!this.compositor.isLayerEnabled('L4')) {
        this.currentVrm.springBoneManager?.update(stepDelta);
      }
      this.currentVrm.nodeConstraintManager?.update();
    }

    // Render 3D Scene
    this.renderer.render(this.scene, this.cameraManager.camera);

    // Poll async WebGL2 PBO readback fence
    this.hitTestManager?.pollFence();

    // Check bounds report
    this.checkAndReportBounds(false);
  }

  private checkAndReportBounds(force = false): void {
    if (!this.currentVrm) return;
    const now = performance.now();
    // Throttle to 20 Hz (50ms interval) unless forced
    if (!force && now - this.lastBoundsReportTime < 50) return;

    const localBounds = this.cameraManager.computeScreenBounds(
      this.currentVrm,
      this.canvasWidth,
      this.canvasHeight,
    );
    const screenRect: Rect = {
      x: this.placementX + localBounds.x,
      y: this.placementY + localBounds.y,
      w: localBounds.w,
      h: localBounds.h,
    };

    if (
      !this.lastReportedBounds ||
      Math.abs(this.lastReportedBounds.x - screenRect.x) > 1 ||
      Math.abs(this.lastReportedBounds.y - screenRect.y) > 1 ||
      Math.abs(this.lastReportedBounds.w - screenRect.w) > 1 ||
      Math.abs(this.lastReportedBounds.h - screenRect.h) > 1
    ) {
      this.lastReportedBounds = screenRect;
      this.emit('stage.bounds', { screenRect });
      this.lastBoundsReportTime = now;
    }
  }

  private onResize(): void {
    // Stage container occupies full window; canvas remains fixed or adapts to avatar
    this.cameraManager.setAspect(this.canvasWidth / this.canvasHeight);
  }

  start(): void {
    this.scheduler.start();
  }

  stop(): void {
    this.scheduler.stop();
  }

  dispose(): void {
    this.stop();
    this.compositor.dispose();
    this.hitTestManager?.dispose();
    this.renderer.dispose();
    this.lighting.dispose();
  }
}
