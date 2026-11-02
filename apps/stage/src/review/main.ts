import './style.css';
import { mountReadoutControls } from './readout-controls.js';
import { ReviewApp } from './review-app.js';
import type { TurntablePreset } from './turntable.js';
import type { LightingPreset, BackgroundPreset } from './lighting-bg.js';

declare global {
  interface Window {
    __reviewApp?: ReviewApp;
    __REVIEW_READY__?: boolean;
  }
}

const canvas = document.getElementById('review-canvas') as HTMLCanvasElement;
const viewportContainer = document.getElementById('viewport-container') as HTMLElement;

if (!canvas || !viewportContainer) {
  throw new Error('Review DOM elements not found');
}

const app = new ReviewApp(canvas, viewportContainer);
window.__reviewApp = app;

// 1. Tab Navigation
const tabButtons = document.querySelectorAll('.tab-btn');
const tabContents = document.querySelectorAll('.tab-content');

tabButtons.forEach((btn) => {
  btn.addEventListener('click', () => {
    tabButtons.forEach((b) => b.classList.remove('active'));
    tabContents.forEach((c) => c.classList.remove('active'));

    btn.classList.add('active');
    const targetId = (btn as HTMLElement).dataset.tab;
    if (targetId) {
      document.getElementById(targetId)?.classList.add('active');
    }
  });
});

// 2. Turntable Presets
const presetButtons = document.querySelectorAll('[data-preset]');
presetButtons.forEach((btn) => {
  btn.addEventListener('click', () => {
    presetButtons.forEach((b) => b.classList.remove('active'));
    btn.classList.add('active');
    const preset = (btn as HTMLElement).dataset.preset as TurntablePreset;
    app.setTurntablePreset(preset);
  });
});

// 3. Auto-Rotate Button
const btnAutoRotate = document.getElementById('btn-auto-rotate');
btnAutoRotate?.addEventListener('click', () => {
  const isRotating = app.toggleAutoRotate();
  btnAutoRotate.classList.toggle('active', isRotating);
});

// 4. Lighting Presets
const lightButtons = document.querySelectorAll('[data-light]');
lightButtons.forEach((btn) => {
  btn.addEventListener('click', () => {
    lightButtons.forEach((b) => b.classList.remove('active'));
    btn.classList.add('active');
    const preset = (btn as HTMLElement).dataset.light as LightingPreset;
    app.setLightingPreset(preset);
  });
});

// 5. Background Presets
const bgButtons = document.querySelectorAll('[data-bg]');
bgButtons.forEach((btn) => {
  btn.addEventListener('click', () => {
    bgButtons.forEach((b) => b.classList.remove('active'));
    btn.classList.add('active');
    const preset = (btn as HTMLElement).dataset.bg as BackgroundPreset;
    app.setBackgroundPreset(preset);
  });
});

// 6. Comparison Mode Toggle
const btnToggleComparison = document.getElementById('btn-toggle-comparison');
btnToggleComparison?.addEventListener('click', () => {
  const isActive = app.toggleComparisonMode();
  btnToggleComparison.classList.toggle('active', isActive);
});

// 7. Expression Mode Toggles
const btnExprSingle = document.getElementById('btn-expr-mode-single');
const btnExprEmo = document.getElementById('btn-expr-mode-emo');

btnExprSingle?.addEventListener('click', () => {
  btnExprSingle.classList.add('active');
  btnExprEmo?.classList.remove('active');
  app.setExpressionMode('single');
});

btnExprEmo?.addEventListener('click', () => {
  btnExprEmo.classList.add('active');
  btnExprSingle?.classList.remove('active');
  app.setExpressionMode('emo');
});

// Expression Weight Slider
const sliderExprWeight = document.getElementById('slider-expr-weight') as HTMLInputElement | null;
const valExprWeight = document.getElementById('val-expr-weight');
sliderExprWeight?.addEventListener('input', () => {
  const weight = parseFloat(sliderExprWeight.value);
  if (valExprWeight) valExprWeight.textContent = weight.toFixed(2);
  app.setExpressionWeight(weight);
});

// 8. Body Readout v2 controls are mounted after profile loading.

// 9. Motion Player Controls
const btnMotionPlay = document.getElementById('btn-motion-play');
const btnMotionPause = document.getElementById('btn-motion-pause');
const btnMotionLoop = document.getElementById('btn-motion-loop');

btnMotionPlay?.addEventListener('click', () => {
  btnMotionPlay.classList.add('active');
  btnMotionPause?.classList.remove('active');
  app.setMotionPlaying(true);
});

btnMotionPause?.addEventListener('click', () => {
  btnMotionPause.classList.add('active');
  btnMotionPlay?.classList.remove('active');
  app.setMotionPlaying(false);
});

let isLoop = true;
btnMotionLoop?.addEventListener('click', () => {
  isLoop = !isLoop;
  btnMotionLoop.textContent = `🔁 循环: ${isLoop ? '开' : '关'}`;
  btnMotionLoop.classList.toggle('active', isLoop);
  app.setMotionLooping(isLoop);
});

// Motion Selection
const selectMotion = document.getElementById('select-motion') as HTMLSelectElement | null;
selectMotion?.addEventListener('change', () => {
  if (selectMotion.value) {
    app.selectMotion(selectMotion.value);
  }
});

// Motion Speed
const speedButtons = document.querySelectorAll('[data-speed]');
const valMotionSpeed = document.getElementById('val-motion-speed');
speedButtons.forEach((btn) => {
  btn.addEventListener('click', () => {
    speedButtons.forEach((b) => b.classList.remove('active'));
    btn.classList.add('active');
    const speed = parseFloat((btn as HTMLElement).dataset.speed ?? '1.0');
    if (valMotionSpeed) valMotionSpeed.textContent = `${speed}x`;
    app.setMotionSpeed(speed);
  });
});

// 10. Spring Bones Controls
function setupSpringSlider(sliderId: string, valId: string, paramName: string) {
  const slider = document.getElementById(sliderId) as HTMLInputElement | null;
  const valSpan = document.getElementById(valId);
  slider?.addEventListener('input', () => {
    const val = parseFloat(slider.value);
    if (valSpan) valSpan.textContent = val.toFixed(2);
    app.updateSpringParameters({ [paramName]: val });
  });
}

setupSpringSlider('slider-spring-stiff', 'val-spring-stiff', 'stiffnessMultiplier');
setupSpringSlider('slider-spring-drag', 'val-spring-drag', 'dragForce');
setupSpringSlider('slider-spring-grav', 'val-spring-grav', 'gravityPower');

document.getElementById('btn-spring-reset')?.addEventListener('click', () => {
  app.resetSpringSimulation();
});

document.getElementById('btn-copy-yaml')?.addEventListener('click', () => {
  const area = document.getElementById('yaml-export-area') as HTMLTextAreaElement | null;
  if (area) {
    navigator.clipboard.writeText(area.value);
    alert('已复制 build.yaml 配置代码到剪贴板！');
  }
});

// 11. Screenshot
document.getElementById('btn-screenshot')?.addEventListener('click', () => {
  app.captureScreenshot();
});

// 12. Save Review Log
async function handleSaveReview() {
  const msgEl = document.getElementById('save-status-msg');
  try {
    const filePath = await app.saveReviewLog();
    if (msgEl) {
      msgEl.textContent = `✅ 评审记录已保存至 ${filePath}`;
      msgEl.style.color = 'var(--success)';
    }
  } catch (err) {
    if (msgEl) {
      msgEl.textContent = `❌ 保存失败: ${(err as Error).message}`;
      msgEl.style.color = 'var(--danger)';
    }
  }
}

document.getElementById('btn-save-review-top')?.addEventListener('click', handleSaveReview);
document.getElementById('btn-save-review-log')?.addEventListener('click', handleSaveReview);

// Initialize model and start loop
app
  .loadModel(new URLSearchParams(location.search).get('model') ?? '/sample.vrm',
    new URLSearchParams(location.search).get('profile') ?? '/sample.profile.json')
  .then(() => {
    mountReadoutControls(app);
    app.start();
    window.__REVIEW_READY__ = true;
  })
  .catch((err) => {
    console.error('Failed to initialize review app:', err);
  });
