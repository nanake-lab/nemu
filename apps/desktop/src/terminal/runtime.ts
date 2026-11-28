import { ipcMain, type BrowserWindow, type IpcMainEvent } from 'electron';
import { join } from 'node:path';
import { ChatOutbox, TerminalClient } from '@nemu/terminal';
import { chatIpc, type ChatStatus } from '../../../panel/src/chat/channel.js';
import type { AppBus } from '../bus/bus.js';
import { bindTerminal } from './bridge.js';

export function startChat(bus: AppBus, panel: BrowserWindow, userData: string, isolated: boolean) {
  let status: ChatStatus = {
    canSend: false,
    reason: 'Echo Core unavailable in the isolated desktop shell',
  };
  let client: TerminalClient | undefined;
  let bridge: ReturnType<typeof bindTerminal> | undefined;
  const sendStatus = (next = status) => {
    if (!panel.isDestroyed()) panel.webContents.send(chatIpc.status, next);
  };
  const setStatus = (reason: string | null, disconnected = false, rejectedEventId?: string) => {
    if (reason !== null && rejectedEventId !== undefined) {
      sendStatus({ ...status, rejection: { eventId: rejectedEventId, reason } });
      return;
    }
    status = { reason, canSend: Boolean(client), disconnected };
    if (disconnected) bridge?.disconnected();
    sendStatus();
  };
  const fromPanel = (event: IpcMainEvent) =>
    !panel.isDestroyed() &&
    event.sender === panel.webContents &&
    event.senderFrame === panel.webContents.mainFrame;
  const getStatus = (event: IpcMainEvent) => {
    if (fromPanel(event)) sendStatus();
  };
  const cancel = (event: IpcMainEvent, payload: unknown) => {
    if (fromPanel(event)) void bridge?.cancel(payload);
  };
  ipcMain.on(chatIpc.getStatus, getStatus);
  ipcMain.on(chatIpc.cancel, cancel);

  if (!isolated) {
    try {
      client = new TerminalClient(new ChatOutbox(join(userData, 'chat-outbox')), setStatus);
      bridge = bindTerminal(bus, client, setStatus);
      client.start(bridge);
    } catch {
      setStatus('Could not open the terminal outbox');
    }
  }

  return () => {
    client?.stop();
    bridge?.close();
    ipcMain.removeListener(chatIpc.getStatus, getStatus);
    ipcMain.removeListener(chatIpc.cancel, cancel);
  };
}
