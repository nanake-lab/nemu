import * as THREE from 'three';
import type { VRM } from '@pixiv/three-vrm';
import {
  loadAvatar,
  AvatarRenderer,
  ThreePointLighting,
  AvatarCamera,
  adaptOutlineWidth,
  AnimationCompositor,
} from '@nemu/avatar-runtime';
import type { NemuProfile, Emotion } from '@nemu/contracts';
import { TurntableController, type TurntablePreset } from './turntable.js';
import { LightingAndBackgroundManager, type LightingPreset, type BackgroundPreset } from './lighting-bg.js';
import { SpringBoneInspector } from './spring-bones.js';

export class ReviewApp {
  readonly canvas: HTMLCanvasElement;
  readonly container: HTMLElement;

  readonly scene: THREE.Scene;
  readonly renderer: AvatarRenderer;
  readonly lighting: ThreePointLighting;
  readonly lightBgManager: LightingAndBackgroundManager;
  readonly camera: THREE.PerspectiveCamera;
  readonly cameraManager: AvatarCamera;
  readonly turntable: TurntableController;
  readonly springInspector: SpringBoneInspector;
  readonly compositor: AnimationCompositor;

  private currentVrm: VRM | null = null;
  private currentProfile: NemuProfile | null = null;

  // State
  private activeExprMode: 'single' | 'emo' = 'single';
  private activeExprName = 'neutral';
  private exprWeight = 1.0;
  private isComparisonActive = false;
  private motionSpeed = 1.0;
  private isMotionPlaying = true;
  private isMotionLooping = true;

  // Animation timing
  private lastTime = performance.now();
  private animationFrameId: number | null = null;

  constructor(canvas: HTMLCanvasElement, container: HTMLElement) {
    this.canvas = canvas;
    this.container = container;

    // 1. Scene & Renderer
    this.scene = new THREE.Scene();
    this.renderer = new AvatarRenderer({
      canvas: this.canvas,
      width: canvas.clientWidth || 800,
      height: canvas.clientHeight || 900,
      antialiasMode: 'defaultMSAA',
    });

    // 2. Lighting & Background
    this.lighting = new ThreePointLighting();
    this.scene.add(this.lighting.group);
    this.lightBgManager = new LightingAndBackgroundManager(this.lighting, this.container);

    // 3. Camera & Turntable
    this.cameraManager = new AvatarCamera(20.0, this.canvas.width / this.canvas.height);
    this.camera = this.cameraManager.camera;
    this.turntable = new TurntableController({
      domElement: this.canvas,
      camera: this.camera,
      target: new THREE.Vector3(0, 1.0, 0),
      distance: 3.5,
    });

    // 4. Spring bone inspector & Compositor
    this.springInspector = new SpringBoneInspector();
    this.compositor = new AnimationCompositor();

    // 5. Window resize handling
    window.addEventListener('resize', () => this.onResize());
  }

  async loadModel(vrmUrl = '/sample.vrm', profileUrl = '/sample.profile.json'): Promise<void> {
    try {
      const { vrm, profile } = await loadAvatar(vrmUrl, profileUrl);

      if (this.currentVrm) {
        this.scene.remove(this.currentVrm.scene);
      }

      this.currentVrm = vrm;
      this.currentProfile = profile;
      this.scene.add(vrm.scene);

      // Bind subsystems
      this.compositor.bindAvatar(vrm, profile);
      this.springInspector.bindVRM(vrm);

      // Frame camera to avatar
      const targetPos = new THREE.Vector3(0, profile.heightM * 0.55, 0);
      this.turntable.setTarget(targetPos);
      this.turntable.setDistance(profile.heightM * 2.3);

      // Adapt outline
      adaptOutlineWidth(vrm, this.canvas.clientHeight || 900);

      // Update badge
      const badge = document.getElementById('model-badge');
      if (badge) {
        const vrmName = (vrm.meta && 'name' in vrm.meta ? (vrm.meta as { name?: string }).name : undefined) ?? 'Avatar';
        badge.textContent = `${vrmName} (VRM 1.0, H: ${profile.heightM}m)`;
      }

      // Initial expression grid population
      this.renderExpressionGrid();
      this.updateYamlExport();
      await this.loadMotionsManifest();
    } catch (err) {
      console.error('Failed to load avatar in ReviewApp:', err);
    }
  }

  start(): void {
    if (this.animationFrameId !== null) return;
    this.lastTime = performance.now();
    const loop = (now: number) => {
      const deltaSec = Math.min(0.1, (now - this.lastTime) / 1000);
      this.lastTime = now;

      this.update(deltaSec);
      this.render();

      this.animationFrameId = requestAnimationFrame(loop);
    };
    this.animationFrameId = requestAnimationFrame(loop);
  }

  stop(): void {
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
  }

  private update(deltaSec: number): void {
    // 1. Update turntable
    this.turntable.update(deltaSec);

    // 2. Update compositor with speed multiplier
    if (this.isMotionPlaying) {
      this.compositor.update(deltaSec * this.motionSpeed);
    }

    // 3. Fallback spring bones update if L4 is off
    if (!this.compositor.isLayerEnabled('L4')) {
      this.currentVrm?.springBoneManager?.update(deltaSec);
    }
  }

  private render(): void {
    this.renderer.render(this.scene, this.camera);
  }

  private onResize(): void {
    const width = this.canvas.clientWidth || 800;
    const height = this.canvas.clientHeight || 900;
    this.renderer.setSize(width, height);
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();

    if (this.currentVrm) {
      adaptOutlineWidth(this.currentVrm, height);
    }
  }

  // --- Controls & UI Bindings ---

  setTurntablePreset(preset: TurntablePreset): void {
    this.turntable.setPreset(preset, true);
  }

  toggleAutoRotate(): boolean {
    const nextState = !this.turntable.isAutoRotating();
    this.turntable.setAutoRotate(nextState);
    return nextState;
  }

  setLightingPreset(preset: LightingPreset): void {
    this.lightBgManager.applyLighting(preset);
  }

  setBackgroundPreset(preset: BackgroundPreset): void {
    this.lightBgManager.applyBackground(preset);
  }

  toggleComparisonMode(): boolean {
    this.isComparisonActive = !this.isComparisonActive;
    const panel = document.getElementById('comparison-panel');
    if (panel) {
      panel.classList.toggle('active', this.isComparisonActive);
    }
    this.onResize();
    return this.isComparisonActive;
  }

  setExpressionMode(mode: 'single' | 'emo'): void {
    this.activeExprMode = mode;
    this.renderExpressionGrid();
  }

  setExpressionWeight(weight: number): void {
    this.exprWeight = weight;
    this.applyActiveExpression();
  }

  selectExpression(name: string): void {
    this.activeExprName = name;
    this.applyActiveExpression();
    this.highlightActiveExprCard();
  }

  private applyActiveExpression(): void {
    if (!this.currentVrm) return;
    const manager = this.currentVrm.expressionManager;
    if (!manager) return;

    if (this.activeExprMode === 'single') {
      // Clear standard expressions and apply single expression
      const allPresets = [
        'neutral', 'happy', 'angry', 'sad', 'relaxed', 'surprised',
        'blink', 'blinkLeft', 'blinkRight', 'aa', 'ih', 'ou', 'ee', 'oh',
        'nemuSmug', 'nemuCatMouth', 'nemuSparkle', 'nemuBlush',
        'nemuTeary', 'nemuSleepy', 'nemuPout', 'nemuAngryMark',
      ];
      for (const p of allPresets) {
        manager.setValue(p, 0);
      }
      manager.setValue(this.activeExprName, this.exprWeight);
      manager.update();
    } else {
      // Performance emo expanded through emotionMap
      this.compositor.setPerformanceEmotion({
        emotion: this.activeExprName as Emotion,
        weight: this.exprWeight,
      });
    }
  }

  private renderExpressionGrid(): void {
    const grid = document.getElementById('expr-grid');
    if (!grid) return;
    grid.innerHTML = '';

    const list =
      this.activeExprMode === 'single'
        ? [
            'neutral', 'happy', 'relaxed', 'sad', 'angry', 'surprised',
            'blink', 'aa', 'ih', 'ou', 'ee', 'oh',
            'nemuSmug', 'nemuCatMouth', 'nemuSparkle', 'nemuBlush',
            'nemuTeary', 'nemuSleepy', 'nemuPout', 'nemuAngryMark',
          ]
        : [
            'neutral', 'happy', 'sad', 'angry', 'surprised', 'relaxed',
            'shy', 'sleepy', 'smug', 'pout', 'excited', 'worried',
          ];

    for (const name of list) {
      const card = document.createElement('div');
      card.className = `expr-card ${name === this.activeExprName ? 'active' : ''}`;
      card.dataset.expr = name;
      card.textContent = name;
      card.addEventListener('click', () => this.selectExpression(name));
      grid.appendChild(card);
    }
  }

  private highlightActiveExprCard(): void {
    const cards = document.querySelectorAll('.expr-card');
    cards.forEach((card) => {
      const el = card as HTMLElement;
      if (el.dataset.expr === this.activeExprName) {
        el.classList.add('active');
      } else {
        el.classList.remove('active');
      }
    });
  }

  applyBodyReadout(params: {
    posture?: number;
    idleTempo?: number;
    earPose?: number;
    tailAmplitude?: number;
    gazeSociability?: number;
    expressionBaseline?: number;
  }): void {
    this.compositor.applyReadout({
      version: '1.0',
      posture: params.posture ?? 0,
      idleTempo: params.idleTempo ?? 1.0,
      earPose: params.earPose ?? 0,
      tailAmplitude: params.tailAmplitude ?? 0.5,
      gazeSociability: params.gazeSociability ?? 0.5,
      expressionBaseline: {
        relaxed: params.expressionBaseline ?? 0,
      },
    });
  }

  updateSpringParameters(params: {
    stiffnessMultiplier?: number;
    dragForce?: number;
    gravityPower?: number;
  }): void {
    this.springInspector.setParameters(params);
    this.updateYamlExport();
  }

  resetSpringSimulation(): void {
    this.springInspector.resetSimulation();
  }

  private updateYamlExport(): void {
    const area = document.getElementById('yaml-export-area') as HTMLTextAreaElement | null;
    if (area) {
      area.value = this.springInspector.exportBuildYamlSnippet();
    }
  }

  setMotionPlaying(playing: boolean): void {
    this.isMotionPlaying = playing;
  }

  setMotionLooping(looping: boolean): void {
    this.isMotionLooping = looping;
  }

  setMotionSpeed(speed: number): void {
    this.motionSpeed = speed;
  }

  async loadMotionsManifest(): Promise<Array<{ id: string; name: string }>> {
    try {
      const res = await fetch('/motions/manifest.json');
      if (res.ok) {
        const data = (await res.json()) as Array<{ id: string; name: string }>;
        if (Array.isArray(data) && data.length > 0) {
          const select = document.getElementById('select-motion') as HTMLSelectElement | null;
          if (select) {
            select.innerHTML = '';
            for (const item of data) {
              const opt = document.createElement('option');
              opt.value = item.id;
              opt.textContent = item.name ?? item.id;
              select.appendChild(opt);
            }
          }
          return data;
        }
      }
    } catch (e) {
      console.warn('Failed to load motions/manifest.json:', e);
    }
    return [];
  }

  selectMotion(motionId: string): void {
    if (!this.currentVrm) return;
    if (motionId === 'idle_stand') {
      this.compositor.applyReadout({
        version: '1.0',
        posture: 0,
        idleTempo: 1.0,
        earPose: 0,
        tailAmplitude: 0.5,
        gazeSociability: 0.5,
        expressionBaseline: { relaxed: 0 },
      });
    } else if (motionId === 'idle_relaxed') {
      this.compositor.applyReadout({
        version: '1.0',
        posture: -0.4,
        idleTempo: 0.7,
        earPose: 0.3,
        tailAmplitude: 0.3,
        gazeSociability: 0.7,
        expressionBaseline: { relaxed: 0.3 },
      });
    } else if (motionId === 'wave') {
      const rightUpperArm = this.currentVrm.humanoid.getNormalizedBoneNode('rightUpperArm');
      if (rightUpperArm) {
        rightUpperArm.rotation.set(0, 0, -1.2);
      }
    } else if (motionId === 'nod') {
      this.compositor.setGaze({
        x: 0,
        y: -0.3,
      });
      setTimeout(() => {
        this.compositor.setGaze({
          x: 0,
          y: 0,
        });
      }, 500);
    } else if (motionId === 'fidget_stretch') {
      const spine = this.currentVrm.humanoid.getNormalizedBoneNode('spine');
      if (spine) {
        spine.rotation.set(-0.2, 0, 0);
      }
    }
  }

  /**
   * Captures PNG screenshot with informative filename.
   */
  async captureScreenshot(): Promise<string> {
    this.render();
    const dataUrl = this.canvas.toDataURL('image/png');

    const angle = Math.round(this.turntable.getAzimuthDeg());
    const expr = this.activeExprName;
    const bg = this.lightBgManager.getCurrentBackground();
    const filename = `nemu_review_deg-${angle}_expr-${expr}_bg-${bg}.png`;

    const link = document.createElement('a');
    link.download = filename;
    link.href = dataUrl;
    link.click();

    return filename;
  }

  /**
   * Generates and saves Markdown review log.
   */
  async saveReviewLog(): Promise<string> {
    const today = new Date().toISOString().split('T')[0]!;
    const vrmMeta = this.currentVrm?.meta;
    const modelName = (vrmMeta && 'name' in vrmMeta ? (vrmMeta as { name?: string }).name : undefined) ?? 'Nemu 3D Model';

    const items = document.querySelectorAll('.review-item');
    const sections: string[] = [];

    items.forEach((item) => {
      const header = item.querySelector('.review-item-header span')?.textContent?.trim() ?? '项目';
      const checked = item.querySelector('input[type="radio"]:checked') as HTMLInputElement | null;
      const status = checked?.value === 'pass' ? '✅ 通过' : '⚠️ 有问题';
      const notes = item.querySelector('textarea')?.value.trim() ?? '无备注';

      sections.push(`### ${header}\n- **评审结论**: ${status}\n- **备注意见**: ${notes || '（无特定问题）'}\n`);
    });

    const markdown = `# 角色资产审美与动态评审记录

- **评审日期**: ${today}
- **模型资产**: ${modelName}
- **评审台版本**: P1-03 (Review Bench)

---

## 评审项目

${sections.join('\n')}

---

## 弹簧骨参数建议配置 (build.yaml 片段)

\`\`\`yaml
${this.springInspector.exportBuildYamlSnippet()}
\`\`\`
`;

    // POST to local dev server /api/save-review
    try {
      const res = await fetch('/api/save-review', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ date: today, markdown }),
      });
      const data = await res.json();
      return (data as { file?: string }).file ?? `reports/review-${today}.md`;
    } catch {
      // Fallback: create client-side download
      const blob = new Blob([markdown], { type: 'text/markdown' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.download = `review-${today}.md`;
      a.href = url;
      a.click();
      return `review-${today}.md`;
    }
  }

  dispose(): void {
    this.stop();
    this.compositor.dispose();
    this.renderer.dispose();
    this.lighting.dispose();
  }
}
