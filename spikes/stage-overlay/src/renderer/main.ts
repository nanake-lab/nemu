import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { VRM, VRMLoaderPlugin, type VRMHumanBoneName } from '@pixiv/three-vrm';
import type { SpikeApi, SpikeConfig, MetricSample } from '../preload/index';

declare global {
  interface Window {
    spikeApi: SpikeApi;
  }
}

// Elements
const wallpaperBg = document.getElementById('wallpaper-bg') as HTMLDivElement;
const container = document.getElementById('character-container') as HTMLDivElement;
const canvas = document.getElementById('stage-canvas') as HTMLCanvasElement;
const hud = document.getElementById('hud') as HTMLDivElement;
const badgeStrategy = document.getElementById('badge-strategy') as HTMLSpanElement;

const statFps = document.getElementById('stat-fps') as HTMLSpanElement;
const statFrameTime = document.getElementById('stat-frametime') as HTMLSpanElement;
const statCpu = document.getElementById('stat-cpu') as HTMLSpanElement;
const statGpu = document.getElementById('stat-gpu') as HTMLSpanElement;
const statAlpha = document.getElementById('stat-alpha') as HTMLSpanElement;
const statReadback = document.getElementById('stat-readback') as HTMLSpanElement;

const btnToggleStrategy = document.getElementById('btn-toggle-strategy') as HTMLButtonElement;
const btnToggleFps = document.getElementById('btn-toggle-fps') as HTMLButtonElement;
const btnToggleReadback = document.getElementById('btn-toggle-readback') as HTMLButtonElement;
const btnBgTrans = document.getElementById('btn-bg-trans') as HTMLButtonElement;
const btnBgWhite = document.getElementById('btn-bg-white') as HTMLButtonElement;
const btnBgBlack = document.getElementById('btn-bg-black') as HTMLButtonElement;
const btnBgColor = document.getElementById('btn-bg-color') as HTMLButtonElement;
const btnCapture = document.getElementById('btn-capture') as HTMLButtonElement;
const btnRunSmoke = document.getElementById('btn-run-smoke') as HTMLButtonElement;
const btnRunFullBench = document.getElementById('btn-run-full-bench') as HTMLButtonElement;

const benchProgress = document.getElementById('bench-progress') as HTMLDivElement;
const benchStatusText = document.getElementById('bench-status-text') as HTMLDivElement;
const benchTimerText = document.getElementById('bench-timer-text') as HTMLDivElement;

// State
let config: SpikeConfig = {
  strategy: 'B',
  mode: 'interactive',
  targetFps: 30,
  scaleFactor: 1.0,
  durationSeconds: 300,
  readPixelsMode: 'async-pbo',
  background: 'transparent',
  width: 2560,
  height: 1400,
  displayBounds: { x: 0, y: 0, width: 2560, height: 1440 },
  workArea: { x: 0, y: 0, width: 2560, height: 1400 },
};

let currentVrm: VRM | null = null;
let currentStrategy: 'A' | 'B' = 'B';
let targetFps: 30 | 60 = 30;
let readPixelsMode: 'async-pbo' | 'sync-readpixels' = 'async-pbo';

// Position in Strategy B
let canvasX = 1800;
let canvasY = 500;
let isDragging = false;
let dragStartX = 0;
let dragStartY = 0;
let initialCanvasX = 0;
let initialCanvasY = 0;
let lastScreenX = 0;
let lastScreenY = 0;

// Cursor and Alpha
let cursorClientX = -1;
let cursorClientY = -1;
let lastAlphaCheckTime = 0;
let currentAlpha = 0;
let isClickThrough = true;
let isCursorOverHud = false;

// Hardware metrics
let latestCpu = { totalCpu: 0, rendererCpu: 0, mainCpu: 0, gpuCpu: 0 };
let latestGpu = { utilization: 0, memoryUsedMb: 0 };

// Benchmark data recording
interface BenchSession {
  active: boolean;
  name: string;
  durationSec: number;
  startTimeMs: number;
  samples: MetricSample[];
  syncDurationsMs: number[];
  onComplete?: (() => void) | undefined;
}
let currentBench: BenchSession | null = null;

// Three.js setup
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(30, 600 / 800, 0.1, 20.0);
camera.position.set(0.0, 1.25, 2.5);
camera.lookAt(0.0, 1.1, 0.0);

const ambientLight = new THREE.AmbientLight(0xffffff, 1.2);
scene.add(ambientLight);

const dirLight = new THREE.DirectionalLight(0xffffff, 1.5);
dirLight.position.set(1.0, 2.0, 1.0);
scene.add(dirLight);

const renderer = new THREE.WebGLRenderer({
  canvas,
  alpha: true,
  premultipliedAlpha: true,
  antialias: true,
  preserveDrawingBuffer: true,
  powerPreference: 'high-performance',
});
renderer.setSize(600, 800);
renderer.setPixelRatio(window.devicePixelRatio || 1);
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.setClearColor(0x000000, 0);

// WebGL2 PBO Setup for Asynchronous Alpha Readback
const gl = renderer.getContext() as WebGL2RenderingContext;
const pbo = gl.createBuffer();
gl.bindBuffer(gl.PIXEL_PACK_BUFFER, pbo);
gl.bufferData(gl.PIXEL_PACK_BUFFER, 4, gl.STREAM_READ); // 4 bytes for 1 RGBA pixel
gl.bindBuffer(gl.PIXEL_PACK_BUFFER, null);

const pboPixelBuffer = new Uint8Array(4);
const syncPixelBuffer = new Uint8Array(4);
let pendingSync: WebGLSync | null = null;

// Load VRM Model
const loader = new GLTFLoader();
loader.register((parser) => new VRMLoaderPlugin(parser));

loader.load(
  './VRM1_Constraint_Twist_Sample.vrm',
  (gltf) => {
    const vrm = gltf.userData.vrm as VRM;
    currentVrm = vrm;
    scene.add(vrm.scene);
    console.log('[VRM] Model loaded successfully');
  },
  undefined,
  (error) => {
    console.error('[VRM] Failed to load model:', error);
  },
);

// Alpha Readback Logic
function checkAlphaAtCursor(): void {
  if (isDragging) {
    updateClickThroughState(255);
    return;
  }
  if (isCursorOverHud) {
    updateClickThroughState(255);
    return;
  }
  if (cursorClientX < 0 || cursorClientY < 0) {
    updateClickThroughState(0);
    return;
  }

  const rect = canvas.getBoundingClientRect();
  if (
    cursorClientX < rect.left ||
    cursorClientX > rect.right ||
    cursorClientY < rect.top ||
    cursorClientY > rect.bottom
  ) {
    updateClickThroughState(0);
    return;
  }

  const dpr = window.devicePixelRatio || 1;
  const px = Math.floor((cursorClientX - rect.left) * dpr);
  const py = Math.floor((rect.bottom - cursorClientY) * dpr);

  if (px < 0 || px >= canvas.width || py < 0 || py >= canvas.height) {
    updateClickThroughState(0);
    return;
  }

  if (readPixelsMode === 'sync-readpixels') {
    const t0 = performance.now();
    gl.readPixels(px, py, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, syncPixelBuffer);
    const costMs = performance.now() - t0;
    if (currentBench?.active) {
      currentBench.syncDurationsMs.push(costMs);
    }
    const alpha = syncPixelBuffer[3] ?? 0;
    updateClickThroughState(alpha);
  } else {
    // WebGL2 Async PBO Readback
    if (pendingSync) {
      const status = gl.clientWaitSync(pendingSync, 0, 0);
      if (status === gl.ALREADY_SIGNALED || status === gl.CONDITION_SATISFIED) {
        gl.bindBuffer(gl.PIXEL_PACK_BUFFER, pbo);
        gl.getBufferSubData(gl.PIXEL_PACK_BUFFER, 0, pboPixelBuffer);
        gl.bindBuffer(gl.PIXEL_PACK_BUFFER, null);
        gl.deleteSync(pendingSync);
        pendingSync = null;
        const alpha = pboPixelBuffer[3] ?? 0;
        updateClickThroughState(alpha);
      }
    }

    if (!pendingSync) {
      gl.bindBuffer(gl.PIXEL_PACK_BUFFER, pbo);
      gl.readPixels(px, py, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, 0);
      gl.bindBuffer(gl.PIXEL_PACK_BUFFER, null);
      pendingSync = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
    }
  }
}

function updateClickThroughState(alpha: number): void {
  currentAlpha = alpha;
  const shouldHit = isDragging || isCursorOverHud || alpha > 10;
  const newClickThrough = !shouldHit;

  if (newClickThrough !== isClickThrough) {
    isClickThrough = newClickThrough;
    window.spikeApi.setIgnoreMouseEvents(isClickThrough, true);
  }

  statAlpha.textContent = `${currentAlpha} (${isClickThrough ? 'Click-Through' : 'Interactive Hit'})`;
}

// Mouse Event Handlers
hud.addEventListener('mouseenter', () => {
  isCursorOverHud = true;
  updateClickThroughState(255);
});
hud.addEventListener('mouseleave', () => {
  isCursorOverHud = false;
});

window.addEventListener('mousemove', (e) => {
  cursorClientX = e.clientX;
  cursorClientY = e.clientY;

  // Max 30 Hz throttling for alpha check
  const now = performance.now();
  if (now - lastAlphaCheckTime >= 33.3) {
    lastAlphaCheckTime = now;
    checkAlphaAtCursor();
  }

  // Dragging logic
  if (isDragging) {
    if (currentStrategy === 'B') {
      canvasX = initialCanvasX + (e.clientX - dragStartX);
      canvasY = initialCanvasY + (e.clientY - dragStartY);
      container.style.transform = `translate3d(${canvasX}px, ${canvasY}px, 0)`;
    } else {
      const deltaX = e.screenX - lastScreenX;
      const deltaY = e.screenY - lastScreenY;
      lastScreenX = e.screenX;
      lastScreenY = e.screenY;
      window.spikeApi.moveWindow(deltaX, deltaY);
    }
  }
});

container.addEventListener('mousedown', (e) => {
  if (e.button !== 0) return;
  if (currentAlpha > 10) {
    isDragging = true;
    container.classList.add('dragging');
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    initialCanvasX = canvasX;
    initialCanvasY = canvasY;
    lastScreenX = e.screenX;
    lastScreenY = e.screenY;
    window.spikeApi.setIgnoreMouseEvents(false, true);
  }
});

window.addEventListener('mouseup', () => {
  if (isDragging) {
    isDragging = false;
    container.classList.remove('dragging');
    checkAlphaAtCursor();
  }
});

// UI Strategy Update
function setStrategy(strat: 'A' | 'B'): void {
  currentStrategy = strat;
  badgeStrategy.textContent = `Strategy ${strat}`;
  badgeStrategy.className = `hud-badge badge-${strat.toLowerCase()}`;

  if (strat === 'A') {
    container.style.position = 'fixed';
    container.style.top = '0px';
    container.style.left = '0px';
    container.style.transform = 'none';
  } else {
    container.style.position = 'absolute';
    container.style.transform = `translate3d(${canvasX}px, ${canvasY}px, 0)`;
  }
}

// Background Switcher
function setBackground(bg: 'transparent' | 'white' | 'black' | 'colorful'): void {
  wallpaperBg.className = bg === 'transparent' ? '' : bg;
  btnBgTrans.className = bg === 'transparent' ? 'active' : '';
  btnBgWhite.className = bg === 'white' ? 'active' : '';
  btnBgBlack.className = bg === 'black' ? 'active' : '';
  btnBgColor.className = bg === 'colorful' ? 'active' : '';
}

// Render Loop Variables
let lastRenderTime = performance.now();
let lastFpsUpdateTime = performance.now();
let frameCount = 0;
let currentFps = 0;
let currentFrameTime = 0;

function animate(now: number): void {
  requestAnimationFrame(animate);

  const targetInterval = 1000 / targetFps;
  const elapsed = now - lastRenderTime;

  if (elapsed < targetInterval - 1.0) {
    return;
  }

  const frameStart = performance.now();
  const delta = (now - lastRenderTime) / 1000;
  lastRenderTime = now;

  // VRM animation update
  if (currentVrm) {
    if (targetFps === 30) {
      // Idle 30fps: gentle breathing swaying
      const t = now * 0.001;
      const spine = currentVrm.humanoid?.getNormalizedBoneNode('spine');
      if (spine) {
        spine.rotation.x = Math.sin(t * 1.5) * 0.02;
        spine.rotation.z = Math.cos(t * 0.8) * 0.01;
      }
      const head = currentVrm.humanoid?.getNormalizedBoneNode('head');
      if (head) {
        head.rotation.y = Math.sin(t * 0.5) * 0.04;
      }
    } else {
      // Active 60fps: lively swaying animation
      const t = now * 0.002;
      const spine = currentVrm.humanoid?.getNormalizedBoneNode('spine');
      if (spine) {
        spine.rotation.x = Math.sin(t * 2.5) * 0.06;
        spine.rotation.y = Math.sin(t * 1.8) * 0.08;
      }
      const head = currentVrm.humanoid?.getNormalizedBoneNode('head');
      if (head) {
        head.rotation.y = Math.cos(t * 2.0) * 0.1;
      }
      const leftUpperArm = currentVrm.humanoid?.getNormalizedBoneNode('leftUpperArm');
      if (leftUpperArm) {
        leftUpperArm.rotation.z = 1.0 + Math.sin(t * 3.0) * 0.2;
      }
    }
    currentVrm.update(delta);
  }

  // Render scene
  renderer.render(scene, camera);

  // Check pending async PBO status if waiting
  if (pendingSync && readPixelsMode === 'async-pbo') {
    const status = gl.clientWaitSync(pendingSync, 0, 0);
    if (status === gl.ALREADY_SIGNALED || status === gl.CONDITION_SATISFIED) {
      gl.bindBuffer(gl.PIXEL_PACK_BUFFER, pbo);
      gl.getBufferSubData(gl.PIXEL_PACK_BUFFER, 0, pboPixelBuffer);
      gl.bindBuffer(gl.PIXEL_PACK_BUFFER, null);
      gl.deleteSync(pendingSync);
      pendingSync = null;
      updateClickThroughState(pboPixelBuffer[3] ?? 0);
    }
  }

  currentFrameTime = performance.now() - frameStart;
  frameCount++;

  if (now - lastFpsUpdateTime >= 500) {
    currentFps = Math.round((frameCount * 1000) / (now - lastFpsUpdateTime));
    frameCount = 0;
    lastFpsUpdateTime = now;

    statFps.textContent = `${currentFps} / ${targetFps} fps`;
    statFrameTime.textContent = `${currentFrameTime.toFixed(2)} ms`;
  }

  // In active benchmark, record sample
  if (currentBench?.active) {
    const benchElapsedSec = (now - currentBench.startTimeMs) / 1000;
    const benchTotalSec = currentBench.durationSec;

    benchTimerText.textContent = `${benchElapsedSec.toFixed(1)}s / ${benchTotalSec}s`;

    if (benchElapsedSec >= benchTotalSec) {
      finishBenchmark();
    }
  }
}

// Metric sampling timer (500ms)
setInterval(() => {
  statCpu.textContent = `${latestCpu.totalCpu.toFixed(1)}% (${latestCpu.mainCpu.toFixed(1)} / ${latestCpu.rendererCpu.toFixed(1)} / ${latestCpu.gpuCpu.toFixed(1)})`;
  statGpu.textContent = `${latestGpu.utilization.toFixed(0)}% / ${latestGpu.memoryUsedMb.toFixed(0)} MB`;

  if (currentBench?.active) {
    const elapsedSec = (performance.now() - currentBench.startTimeMs) / 1000;
    const sample: MetricSample = {
      timestamp: Date.now(),
      timeSec: parseFloat(elapsedSec.toFixed(2)),
      fps: currentFps,
      frameTimeMs: parseFloat(currentFrameTime.toFixed(2)),
      cpuPercentTotal: parseFloat(latestCpu.totalCpu.toFixed(2)),
      cpuPercentRenderer: parseFloat(latestCpu.rendererCpu.toFixed(2)),
      cpuPercentMain: parseFloat(latestCpu.mainCpu.toFixed(2)),
      cpuPercentGpu: parseFloat(latestCpu.gpuCpu.toFixed(2)),
      gpuUtilization: latestGpu.utilization,
      gpuMemoryUsedMb: latestGpu.memoryUsedMb,
      cursorAlpha: currentAlpha,
      isClickThrough,
    };
    currentBench.samples.push(sample);
  }
}, 500);

// Hardware sample listener
window.addEventListener('message', (event) => {
  if (event.data?.type === 'hardware-sample') {
    latestCpu = event.data.cpu;
    latestGpu = event.data.gpu;
  }
});

// IPC Config Initialization
async function init(): Promise<void> {
  config = await window.spikeApi.getConfig();
  console.log('[Spike] Initial configuration:', config);

  setStrategy(config.strategy);
  targetFps = config.targetFps;
  readPixelsMode = config.readPixelsMode;
  setBackground(config.background);

  statReadback.textContent =
    readPixelsMode === 'async-pbo' ? 'Async PBO (30Hz)' : 'Sync readPixels (Stall)';
  statFps.textContent = `-- / ${targetFps} fps`;

  // Position Strategy B canvas
  canvasX = config.workArea.width - 660;
  canvasY = config.workArea.height - 860;
  if (config.strategy === 'B') {
    container.style.transform = `translate3d(${canvasX}px, ${canvasY}px, 0)`;
  }

  // Setup UI event listeners
  btnToggleStrategy.addEventListener('click', async () => {
    const nextStrat = currentStrategy === 'A' ? 'B' : 'A';
    await window.spikeApi.switchStrategy(nextStrat);
    setStrategy(nextStrat);
  });

  btnToggleFps.addEventListener('click', () => {
    targetFps = targetFps === 30 ? 60 : 30;
    statFps.textContent = `-- / ${targetFps} fps`;
  });

  btnToggleReadback.addEventListener('click', () => {
    readPixelsMode = readPixelsMode === 'async-pbo' ? 'sync-readpixels' : 'async-pbo';
    statReadback.textContent =
      readPixelsMode === 'async-pbo' ? 'Async PBO (30Hz)' : 'Sync readPixels (Stall)';
  });

  btnBgTrans.addEventListener('click', () => setBackground('transparent'));
  btnBgWhite.addEventListener('click', () => setBackground('white'));
  btnBgBlack.addEventListener('click', () => setBackground('black'));
  btnBgColor.addEventListener('click', () => setBackground('colorful'));

  btnCapture.addEventListener('click', async () => {
    hud.classList.add('hidden');
    await new Promise((r) => setTimeout(r, 200));
    const filename = `screenshot_${currentStrategy}_${targetFps}fps_${Date.now()}.png`;
    const path = await window.spikeApi.captureScreenshot(filename);
    hud.classList.remove('hidden');
    alert(`Screenshot saved:\n${path}`);
  });

  btnRunSmoke.addEventListener('click', () => {
    startBenchmark(`smoke_${currentStrategy}_${targetFps}fps`, 10);
  });

  btnRunFullBench.addEventListener('click', () => {
    startBenchmark(`bench_${currentStrategy}_${targetFps}fps`, 300);
  });

  // Listen for config updates
  window.spikeApi.onConfigUpdated((newConf) => {
    config = newConf;
    setStrategy(newConf.strategy);
  });

  // Start loop
  requestAnimationFrame(animate);

  // Dispatch automated test from CLI
  if (config.runTest === 'clickthrough') {
    console.log('[Spike] Scheduled Click-Through Accuracy Test...');
    setTimeout(runClickThroughAccuracyTest, 1500);
  } else if (config.runTest === 'readpixels') {
    console.log('[Spike] Scheduled ReadPixels Comparison Test...');
    setTimeout(runReadPixelsComparisonTest, 1500);
  } else if (config.runTest === 'drag') {
    console.log('[Spike] Scheduled Drag Tearing Test...');
    setTimeout(runDragTest, 1500);
  } else if (config.runTest === 'screenshots') {
    console.log('[Spike] Scheduled Screenshot Suite...');
    setTimeout(runScreenshotSuite, 1500);
  } else if (config.mode === 'benchmark') {
    console.log('[Spike] Auto-starting benchmark from CLI...');
    setTimeout(() => {
      const benchName =
        config.testName ??
        `${config.strategy.toLowerCase()}_${config.targetFps === 30 ? 'idle' : 'active'}_${Math.round(config.scaleFactor * 100)}`;
      startBenchmark(benchName, config.durationSeconds);
    }, 1500);
  }
}

async function runClickThroughAccuracyTest(): Promise<void> {
  while (!currentVrm) {
    await new Promise((r) => setTimeout(r, 100));
  }
  await new Promise((r) => setTimeout(r, 500));

  console.log('[Test] Running Click-Through Accuracy Test...');
  const results: Array<{
    id: number;
    x: number;
    y: number;
    zone: string;
    alpha: number;
    expectedHit: boolean;
    actualHit: boolean;
    correct: boolean;
  }> = [];

  renderer.render(scene, camera);

  const transparentPoints = [
    { x: 10, y: 10, zone: 'Top-Left Corner' },
    { x: 100, y: 10, zone: 'Top-Left Margin' },
    { x: 500, y: 10, zone: 'Top-Right Margin' },
    { x: 590, y: 10, zone: 'Top-Right Corner' },
    { x: 10, y: 400, zone: 'Mid-Left Margin' },
    { x: 590, y: 400, zone: 'Mid-Right Margin' },
    { x: 10, y: 790, zone: 'Bottom-Left Corner' },
    { x: 590, y: 790, zone: 'Bottom-Right Corner' },
    { x: 50, y: 200, zone: 'Upper-Left Void' },
    { x: 550, y: 200, zone: 'Upper-Right Void' },
    { x: 50, y: 600, zone: 'Lower-Left Void' },
    { x: 550, y: 600, zone: 'Lower-Right Void' },
    { x: 200, y: 50, zone: 'Top Space' },
    { x: 400, y: 50, zone: 'Top Space' },
    { x: 100, y: 700, zone: 'Bottom Left' },
    { x: 500, y: 700, zone: 'Bottom Right' },
  ];

  const boneNames: Array<{ bone: VRMHumanBoneName; name: string }> = [
    { bone: 'head', name: 'Head' },
    { bone: 'neck', name: 'Neck' },
    { bone: 'chest', name: 'Chest' },
    { bone: 'spine', name: 'Spine' },
    { bone: 'hips', name: 'Hips' },
    { bone: 'leftUpperArm', name: 'Left Upper Arm' },
    { bone: 'rightUpperArm', name: 'Right Upper Arm' },
    { bone: 'leftLowerArm', name: 'Left Forearm' },
    { bone: 'rightLowerArm', name: 'Right Forearm' },
    { bone: 'leftUpperLeg', name: 'Left Thigh' },
    { bone: 'rightUpperLeg', name: 'Right Thigh' },
    { bone: 'leftLowerLeg', name: 'Left Calf' },
    { bone: 'rightLowerLeg', name: 'Right Calf' },
    { bone: 'leftFoot', name: 'Left Foot' },
    { bone: 'rightFoot', name: 'Right Foot' },
    { bone: 'upperChest', name: 'Upper Chest' },
  ];

  const characterPoints: Array<{ x: number; y: number; zone: string }> = [];
  for (const b of boneNames) {
    const node = currentVrm?.humanoid?.getNormalizedBoneNode(b.bone);
    if (node) {
      const v = new THREE.Vector3();
      node.getWorldPosition(v);
      v.project(camera);
      const x = Math.round((v.x * 0.5 + 0.5) * 600);
      const y = Math.round((-v.y * 0.5 + 0.5) * 800);
      if (x >= 0 && x <= 600 && y >= 0 && y <= 800) {
        characterPoints.push({ x, y, zone: b.name });
      }
    }
  }

  const allPoints = [
    ...transparentPoints.map((p) => ({ ...p, expectedHit: false })),
    ...characterPoints.map((p) => ({ ...p, expectedHit: true })),
  ];

  const dpr = window.devicePixelRatio || 1;
  const tempBuf = new Uint8Array(4);

  let id = 1;
  let passedCount = 0;

  for (const pt of allPoints) {
    const px = Math.floor(pt.x * dpr);
    const py = Math.floor((800 - pt.y) * dpr);
    gl.readPixels(px, py, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, tempBuf);
    const alpha = tempBuf[3] ?? 0;

    const actualHit = alpha > 10;
    const correct = actualHit === pt.expectedHit;
    if (correct) passedCount++;

    results.push({
      id: id++,
      x: pt.x,
      y: pt.y,
      zone: pt.zone,
      alpha,
      expectedHit: pt.expectedHit,
      actualHit,
      correct,
    });
  }

  const accuracy = (passedCount / results.length) * 100;
  console.log(
    `[Test] Click-Through Accuracy: ${accuracy.toFixed(1)}% (${passedCount}/${results.length})`,
  );

  let csv = 'point_id,x,y,zone,alpha,expected_hit,actual_hit,correct\n';
  for (const r of results) {
    csv += `${r.id},${r.x},${r.y},"${r.zone}",${r.alpha},${r.expectedHit ? 1 : 0},${r.actualHit ? 1 : 0},${r.correct ? 1 : 0}\n`;
  }

  await window.spikeApi.saveData('reports/P0-03/data/clickthrough_accuracy.csv', csv);
  window.spikeApi.notifyBenchmarkComplete({
    test: 'clickthrough_accuracy',
    totalPoints: results.length,
    passedCount,
    accuracyPct: accuracy,
  });
}

async function runReadPixelsComparisonTest(): Promise<void> {
  while (!currentVrm) {
    await new Promise((r) => setTimeout(r, 100));
  }
  await new Promise((r) => setTimeout(r, 500));

  console.log('[Test] Running Synchronous readPixels vs Asynchronous PBO Benchmark...');
  const dpr = window.devicePixelRatio || 1;
  const px = Math.floor(300 * dpr);
  const py = Math.floor(400 * dpr);

  const samples: Array<{
    frame: number;
    mode: 'async-pbo' | 'sync-readpixels';
    frameTimeMs: number;
    readCostMs: number;
  }> = [];

  const testFrames = 500;

  // Phase 1: Async PBO
  readPixelsMode = 'async-pbo';
  for (let i = 0; i < testFrames; i++) {
    const t0 = performance.now();
    renderer.render(scene, camera);

    const readStart = performance.now();
    gl.bindBuffer(gl.PIXEL_PACK_BUFFER, pbo);
    gl.readPixels(px, py, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, 0);
    gl.bindBuffer(gl.PIXEL_PACK_BUFFER, null);
    const sync = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
    const readCost = performance.now() - readStart;

    await new Promise((r) => requestAnimationFrame(r));

    if (sync) {
      gl.clientWaitSync(sync, 0, 0);
      gl.deleteSync(sync);
    }
    const frameTime = performance.now() - t0;

    samples.push({
      frame: i + 1,
      mode: 'async-pbo',
      frameTimeMs: parseFloat(frameTime.toFixed(3)),
      readCostMs: parseFloat(readCost.toFixed(3)),
    });
  }

  // Phase 2: Sync readPixels
  readPixelsMode = 'sync-readpixels';
  const syncBuf = new Uint8Array(4);
  for (let i = 0; i < testFrames; i++) {
    const t0 = performance.now();
    renderer.render(scene, camera);

    const readStart = performance.now();
    gl.readPixels(px, py, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, syncBuf);
    const readCost = performance.now() - readStart;

    await new Promise((r) => requestAnimationFrame(r));
    const frameTime = performance.now() - t0;

    samples.push({
      frame: testFrames + i + 1,
      mode: 'sync-readpixels',
      frameTimeMs: parseFloat(frameTime.toFixed(3)),
      readCostMs: parseFloat(readCost.toFixed(3)),
    });
  }

  const asyncFrames = samples.filter((s) => s.mode === 'async-pbo');
  const syncFrames = samples.filter((s) => s.mode === 'sync-readpixels');

  const asyncAvgFrame = asyncFrames.reduce((acc, s) => acc + s.frameTimeMs, 0) / asyncFrames.length;
  const syncAvgFrame = syncFrames.reduce((acc, s) => acc + s.frameTimeMs, 0) / syncFrames.length;
  const syncAvgCost = syncFrames.reduce((acc, s) => acc + s.readCostMs, 0) / syncFrames.length;
  const asyncAvgCost = asyncFrames.reduce((acc, s) => acc + s.readCostMs, 0) / asyncFrames.length;

  console.log(
    `[Test] Async PBO Frame Time: ${asyncAvgFrame.toFixed(2)}ms (dispatch: ${asyncAvgCost.toFixed(3)}ms)`,
  );
  console.log(
    `[Test] Sync readPixels Frame Time: ${syncAvgFrame.toFixed(2)}ms (stall cost: ${syncAvgCost.toFixed(3)}ms)`,
  );

  let csv = 'frame,mode,frame_time_ms,read_cost_ms\n';
  for (const s of samples) {
    csv += `${s.frame},${s.mode},${s.frameTimeMs},${s.readCostMs}\n`;
  }

  await window.spikeApi.saveData('reports/P0-03/data/readpixels_sync_vs_async.csv', csv);
  window.spikeApi.notifyBenchmarkComplete({
    test: 'readpixels_sync_vs_async',
    asyncAvgFrameMs: asyncAvgFrame,
    syncAvgFrameMs: syncAvgFrame,
    syncAvgCostMs: syncAvgCost,
    stallIncreaseMs: syncAvgFrame - asyncAvgFrame,
  });
}

async function runDragTest(): Promise<void> {
  while (!currentVrm) {
    await new Promise((r) => setTimeout(r, 100));
  }
  await new Promise((r) => setTimeout(r, 500));

  console.log(`[Test] Running Drag Test for Strategy ${currentStrategy}...`);
  const frames = 300; // 5 seconds at 60fps
  const samples: Array<{ frame: number; frameTimeMs: number; deltaX: number; deltaY: number }> = [];

  const startX = canvasX;
  const startY = canvasY;
  let prevX = startX;
  let prevY = startY;

  for (let i = 0; i < frames; i++) {
    const t0 = performance.now();
    const t = i / 30; // frequency
    const targetX = Math.round(startX + Math.sin(t * 3) * 100);
    const targetY = Math.round(startY + Math.cos(t * 2) * 50);
    const dx = targetX - prevX;
    const dy = targetY - prevY;
    prevX = targetX;
    prevY = targetY;

    if (currentStrategy === 'B') {
      canvasX = targetX;
      canvasY = targetY;
      container.style.transform = `translate3d(${canvasX}px, ${canvasY}px, 0)`;
    } else {
      window.spikeApi.moveWindow(dx, dy);
    }

    await new Promise((r) => requestAnimationFrame(r));
    const frameTime = performance.now() - t0;
    samples.push({
      frame: i + 1,
      frameTimeMs: parseFloat(frameTime.toFixed(2)),
      deltaX: dx,
      deltaY: dy,
    });
  }

  const avgFrame = samples.reduce((acc, s) => acc + s.frameTimeMs, 0) / samples.length;
  const hitches = samples.filter((s) => s.frameTimeMs > 25).length;
  console.log(
    `[Test] Drag Test (${currentStrategy}): Avg Frame Time = ${avgFrame.toFixed(2)}ms, Hitches (>25ms) = ${hitches}`,
  );

  let csv = 'frame,frame_time_ms,delta_x,delta_y\n';
  for (const s of samples) {
    csv += `${s.frame},${s.frameTimeMs},${s.deltaX},${s.deltaY}\n`;
  }
  await window.spikeApi.saveData(
    `reports/P0-03/data/drag_${currentStrategy.toLowerCase()}_${Math.round(config.scaleFactor * 100)}.csv`,
    csv,
  );

  window.spikeApi.notifyBenchmarkComplete({
    test: 'drag_test',
    strategy: currentStrategy,
    avgFrameMs: avgFrame,
    hitchesCount: hitches,
  });
}

async function runScreenshotSuite(): Promise<void> {
  while (!currentVrm) {
    await new Promise((r) => setTimeout(r, 100));
  }
  await new Promise((r) => setTimeout(r, 600));

  console.log(
    `[Test] Running Screenshot Suite for Strategy ${currentStrategy} at scale ${config.scaleFactor}...`,
  );
  hud.classList.add('hidden');

  const bgs: Array<'white' | 'black' | 'colorful' | 'transparent'> = [
    'white',
    'black',
    'colorful',
    'transparent',
  ];
  const scaleTag = Math.round(config.scaleFactor * 100);

  for (const bg of bgs) {
    setBackground(bg);
    // Render multiple frames to ensure clean presentation
    for (let f = 0; f < 10; f++) {
      await new Promise((r) => requestAnimationFrame(r));
    }
    const filename = `strat_${currentStrategy.toLowerCase()}_${bg === 'colorful' ? 'color' : bg}_${scaleTag}.png`;
    await window.spikeApi.captureScreenshot(filename);
  }

  hud.classList.remove('hidden');
  console.log(`[Test] Screenshot Suite completed.`);
  window.spikeApi.notifyBenchmarkComplete({
    test: 'screenshot_suite',
    strategy: currentStrategy,
    scale: config.scaleFactor,
  });
}

function startBenchmark(name: string, durationSec: number, onComplete?: () => void): void {
  benchProgress.style.display = 'block';
  benchStatusText.textContent = `Running: ${name} (${durationSec}s)`;
  benchTimerText.textContent = `0s / ${durationSec}s`;

  currentBench = {
    active: true,
    name,
    durationSec,
    startTimeMs: performance.now(),
    samples: [],
    syncDurationsMs: [],
    onComplete,
  };
}

async function finishBenchmark(): Promise<void> {
  if (!currentBench) return;
  const bench = currentBench;
  bench.active = false;
  benchStatusText.textContent = `Saving ${bench.name}...`;

  let csv =
    'timestamp,time_sec,fps,frame_time_ms,cpu_total_pct,cpu_renderer_pct,cpu_main_pct,cpu_gpu_pct,gpu_util_pct,gpu_mem_mb,cursor_alpha,click_through\n';
  for (const s of bench.samples) {
    csv += `${s.timestamp},${s.timeSec},${s.fps},${s.frameTimeMs},${s.cpuPercentTotal},${s.cpuPercentRenderer},${s.cpuPercentMain},${s.cpuPercentGpu},${s.gpuUtilization},${s.gpuMemoryUsedMb},${s.cursorAlpha},${s.isClickThrough ? 1 : 0}\n`;
  }

  const filename = `reports/P0-03/data/${bench.name}.csv`;
  await window.spikeApi.saveData(filename, csv);

  const count = bench.samples.length;
  const avgCpu =
    count > 0 ? bench.samples.reduce((acc, s) => acc + s.cpuPercentTotal, 0) / count : 0;
  const avgGpu =
    count > 0 ? bench.samples.reduce((acc, s) => acc + s.gpuUtilization, 0) / count : 0;
  const avgFps = count > 0 ? bench.samples.reduce((acc, s) => acc + s.fps, 0) / count : 0;

  console.log(
    `[Benchmark] ${bench.name} finished: Avg CPU=${avgCpu.toFixed(2)}%, Avg GPU=${avgGpu.toFixed(2)}%, Avg FPS=${avgFps.toFixed(1)}`,
  );

  benchStatusText.textContent = `Completed ${bench.name}! (Avg CPU: ${avgCpu.toFixed(1)}%, GPU: ${avgGpu.toFixed(1)}%)`;
  benchProgress.style.display = 'none';

  if (bench.onComplete) {
    bench.onComplete();
  }

  window.spikeApi.notifyBenchmarkComplete({
    name: bench.name,
    durationSec: bench.durationSec,
    samplesCount: count,
    avgCpu,
    avgGpu,
    avgFps,
  });

  currentBench = null;
}

init();
