import './style.css';
import { StageApp } from './stage-app.js';
import type { FramingMode } from '@nemu/avatar-runtime';

declare global {
  interface Window {
    __stageApp?: StageApp;
    __STAGE_READY__?: boolean;
    __LAST_STAGE_EVENT__?: { type: string; payload: unknown };
  }
}

const root = document.querySelector<HTMLDivElement>('#root');
if (!root) throw new Error('Missing stage root element #root');

const params = new URLSearchParams(window.location.search);
const isDeterministic =
  params.get('deterministic') === '1' || params.get('deterministic') === 'true';

// Apply background class
const bg = params.get('bg');
if (bg === 'white') {
  document.body.classList.add('bg-white');
} else if (bg === 'black') {
  document.body.classList.add('bg-black');
} else if (bg === 'gradient') {
  document.body.classList.add('bg-gradient');
}

const stageApp = new StageApp({
  container: root,
  deterministic: isDeterministic,
  onEvent: (type, payload) => {
    window.__LAST_STAGE_EVENT__ = { type, payload };
  },
});

window.__stageApp = stageApp;

// Handle commands from window message or parent process
window.addEventListener('message', (event) => {
  const data = event.data;
  if (!data || typeof data !== 'object') return;

  switch (data.type) {
    case 'stage.loadAvatar':
      if (data.payload?.vrmUrl && data.payload?.profileUrl) {
        stageApp.loadAvatar(data.payload.vrmUrl, data.payload.profileUrl);
      }
      break;
    case 'stage.setPlacement':
      if (data.payload?.anchor) {
        stageApp.setPlacement(
          data.payload.displayId ?? 0,
          data.payload.anchor,
          data.payload.transitionMs ?? 0,
        );
      }
      break;
    case 'stage.setQuality':
      if (data.payload) {
        stageApp.setQuality(
          data.payload.fpsActive ?? 60,
          data.payload.fpsIdle ?? 30,
          data.payload.fpsSleep ?? 12,
          data.payload.msaa ?? 0,
        );
      }
      break;
    case 'stage.gaze':
      if (data.payload?.target) {
        stageApp.gaze(data.payload.target);
      }
      break;
    case 'stage.setPhysicalState':
      if (data.payload?.state) {
        stageApp.setPhysicalState(data.payload.state);
      }
      break;
    case 'stage.readout':
      if (data.payload?.readout) {
        stageApp.applyReadout(data.payload.readout);
      }
      break;
    case 'stage.setLayerEnabled':
      if (data.payload?.layer && typeof data.payload.enabled === 'boolean') {
        stageApp.setLayerEnabled(data.payload.layer, data.payload.enabled);
      }
      break;
  }
});

// Initial avatar load from URL parameters or defaults
const modelUrl = params.get('model') || '/sample.vrm';
const profileUrl = params.get('profile') || '/sample.profile.json';
const framingParam = (params.get('framing') as FramingMode) || 'fullBody';
const exprParam = params.get('expr');
const layersParam = params.get('layers');

// Configure layer enablement before avatar loading
if (layersParam) {
  const activeLayers = new Set(layersParam.split(',').map((s) => s.trim().toUpperCase()));
  const allLayers = ['L0', 'L1', 'L2', 'L3', 'L4'] as const;
  for (const layer of allLayers) {
    stageApp.setLayerEnabled(layer, activeLayers.has(layer));
  }
} else if (isDeterministic && !params.get('motion')) {
  // In deterministic golden visual tests, keep L0..L4 idle so avatar remains in deterministic rest pose for golden match
  stageApp.setLayerEnabled('L0', false);
  stageApp.setLayerEnabled('L1', false);
  stageApp.setLayerEnabled('L2', false);
  stageApp.setLayerEnabled('L3', false);
  stageApp.setLayerEnabled('L4', false);
}

async function initializeStage(): Promise<void> {
  try {
    await stageApp.loadAvatar(modelUrl, profileUrl);

    // Apply framing mode
    stageApp.setFraming(framingParam);

    // Apply expression if requested in URL
    if (exprParam) {
      const parts = exprParam.split(':');
      const exprName = parts[0]!;
      const weight = parts[1] ? parseFloat(parts[1]) : 1.0;
      stageApp.setExpression(exprName, weight);
    }

    if (isDeterministic) {
      // Step twice to settle bone constraints and meshes deterministically
      stageApp.scheduler.step(16.67);
      stageApp.scheduler.step(16.67);
      window.__STAGE_READY__ = true;
    } else {
      stageApp.start();
      window.__STAGE_READY__ = true;
    }
  } catch (err) {
    console.error('Failed to initialize stage avatar:', err);
    window.__STAGE_READY__ = false;
  }
}

initializeStage();
