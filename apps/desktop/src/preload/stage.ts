import { MicController } from '../../../../packages/voice/web/mic.js';
let microphone: MicController | undefined;
import { ipcRenderer, contextBridge } from 'electron';

let controlPort: MessagePort | null = null;
const pendingMessagesToPort: unknown[] = [];

// Whitelist of stage outgoing messages allowed to be forwarded to main via controlPort (CONTRACTS 2.2)
const ALLOWED_STAGE_OUTGOING_TYPES = new Set([
  'stage.ready',
  'stage.hitTest',
  'stage.hit',
  'stage.drag',
  'stage.bounds',
  'stage.frameStats',
  'stage.actionResult',
  'stage.error',
]);

// Listen for control channel port from main process
ipcRenderer.on('nemu:channel:control', (event) => {
  const [port] = event.ports;
  if (!port) return;

  if (controlPort) {
    controlPort.close();
  }
  controlPort = port;

  // Forward incoming control messages (e.g. stage.event, stage.* commands) to the stage window page
  controlPort.onmessage = (msgEvent) => {
    void microphone?.command(msgEvent.data).catch(() => {
      controlPort?.postMessage({
        v: 2,
        id: 'mic-error',
        type: 'stage.error',
        payload: {
          code: 'MIC_CAPTURE_FAILED',
          message: 'Microphone permission or device unavailable',
        },
      });
    });
    window.postMessage(msgEvent.data, '*');
  };

  // Flush any pending messages queued before port arrived
  while (pendingMessagesToPort.length > 0) {
    const item = pendingMessagesToPort.shift();
    controlPort.postMessage(item);
  }
});

// Also forward audio, mic, bus ports to the page if needed
for (const channel of ['nemu:channel:audio', 'nemu:channel:mic', 'nemu:channel:bus']) {
  ipcRenderer.on(channel, (event) => {
    if (channel === 'nemu:channel:mic' && event.ports[0]) {
      void microphone?.set(false);
      microphone = new MicController(event.ports[0]);
    } else window.postMessage({ type: channel }, '*', event.ports);
  });
}

// Intercept window.postMessage from stage page and bridge to controlPort
window.addEventListener('message', (event) => {
  if (event.source !== window) return;
  const data = event.data as { type?: unknown } | null;
  if (!data || typeof data !== 'object') return;

  const type = data.type;
  if (typeof type === 'string' && ALLOWED_STAGE_OUTGOING_TYPES.has(type)) {
    if (controlPort) {
      controlPort.postMessage(data);
    } else {
      pendingMessagesToPort.push(data);
    }
  }
});

contextBridge.exposeInMainWorld('nemuDesktop', {
  isElectron: true,
  platform: process.platform,
  role: 'stage',
  microphoneStatus: () => microphone?.status() ?? { active: false, frames: 0 },
});

window.addEventListener('beforeunload', () => {
  void microphone?.set(false);
});
