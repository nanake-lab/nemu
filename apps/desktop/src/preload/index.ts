import { ipcRenderer, contextBridge } from 'electron';

// Expose port transfer to renderer frames
const forwardedChannels = [
  'nemu:channel:control',
  'nemu:channel:audio',
  'nemu:channel:mic',
  'nemu:channel:bus',
];

for (const channel of forwardedChannels) {
  ipcRenderer.on(channel, (event) => {
    window.postMessage({ type: channel }, '*', event.ports);
  });
}

// Listen to messages from renderer (e.g. stage.hitTest, stage.ready) and forward to main process
window.addEventListener('message', (event) => {
  const data = event.data;
  if (!data || typeof data !== 'object') return;

  if (data.type === 'stage.hitTest') {
    ipcRenderer.send('nemu:stage:hitTest', data.payload);
  } else if (
    data.type === 'stage.ready' ||
    data.type === 'stage.hit' ||
    data.type === 'stage.drag' ||
    data.type === 'stage.bounds' ||
    data.type === 'stage.frameStats' ||
    data.type === 'stage.actionResult' ||
    data.type === 'stage.error'
  ) {
    ipcRenderer.send('nemu:stage:event', { type: data.type, payload: data.payload });
  }
});

contextBridge.exposeInMainWorld('nemuDesktop', {
  isElectron: true,
  platform: process.platform,
});
