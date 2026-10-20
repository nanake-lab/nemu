import { ipcRenderer, contextBridge } from 'electron';

// Panel receives the bus port only
ipcRenderer.on('nemu:channel:bus', (event) => {
  window.postMessage({ type: 'nemu:channel:bus' }, '*', event.ports);
});

contextBridge.exposeInMainWorld('nemuDesktop', {
  isElectron: true,
  platform: process.platform,
  role: 'panel',
});
