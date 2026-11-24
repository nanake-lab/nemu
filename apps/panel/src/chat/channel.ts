import type { EchoCore } from '@nemu/contracts';
import type { ChatStatus } from './status.js';

export type { ChatStatus } from './status.js';

// Local Electron plumbing for OutputCancel and terminal operational status.
// Stream text, run state and approvals continue to use the application bus.
// Keep runtime imports out of this module so the sandboxed preload doesn't pull in shared chunks.
export const chatIpc = {
  status: 'nemu:chat:status',
  getStatus: 'nemu:chat:get-status',
  cancel: 'nemu:chat:cancel',
} as const;

export interface ChatPanelChannel {
  onStatus(listener: (status: ChatStatus) => void): () => void;
  cancel(message: EchoCore.OutputCancel): void;
}

declare global {
  interface Window {
    nemuChat?: ChatPanelChannel;
  }
}
