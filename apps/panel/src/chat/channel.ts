import type { EchoCore } from '@nemu/contracts';

// Local Electron plumbing for OutputCancel and terminal operational status.
// Stream text, run state and approvals continue to use the application bus.
export const chatIpc = {
  status: 'nemu:chat:status',
  getStatus: 'nemu:chat:get-status',
  cancel: 'nemu:chat:cancel',
} as const;

export interface ChatStatus {
  reason: string | null;
  canSend: boolean;
  disconnected?: boolean;
}

export interface ChatPanelChannel {
  onStatus(listener: (status: ChatStatus) => void): () => void;
  cancel(message: EchoCore.OutputCancel): void;
}

declare global {
  interface Window {
    nemuChat?: ChatPanelChannel;
  }
}
