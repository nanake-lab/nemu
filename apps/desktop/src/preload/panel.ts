import { ipcRenderer, contextBridge } from 'electron';
import {
  chatIpc,
  type ChatPanelChannel,
  type ChatStatus,
} from '../../../panel/src/chat/channel.js';

// Panel receives the bus port only.
ipcRenderer.on('nemu:channel:bus', (event) => {
  window.postMessage({ type: 'nemu:channel:bus' }, '*', event.ports);
});

contextBridge.exposeInMainWorld('nemuDesktop', {
  isElectron: true,
  platform: process.platform,
  role: 'panel',
});

const chat: ChatPanelChannel = {
  onStatus(listener) {
    const receive = (_: Electron.IpcRendererEvent, status: ChatStatus) => listener(status);
    ipcRenderer.on(chatIpc.status, receive);
    ipcRenderer.send(chatIpc.getStatus);
    return () => {
      ipcRenderer.removeListener(chatIpc.status, receive);
    };
  },
  cancel(message) {
    ipcRenderer.send(chatIpc.cancel, message);
  },
};
contextBridge.exposeInMainWorld('nemuChat', chat);
