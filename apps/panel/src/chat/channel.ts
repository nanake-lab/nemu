import type { EchoCore } from '@nemu/contracts';
import { z } from 'zod';

// Local Electron plumbing for OutputCancel and terminal operational status.
// Stream text, run state and approvals continue to use the application bus.
export const chatIpc = {
  status: 'nemu:chat:status',
  getStatus: 'nemu:chat:get-status',
  cancel: 'nemu:chat:cancel',
} as const;

export const chatStatusSchema = z.object({
  reason: z.string().nullable(),
  canSend: z.boolean(),
  disconnected: z.boolean().optional(),
  rejection: z.object({ eventId: z.ulid(), reason: z.string() }).optional(),
});
export type ChatStatus = z.infer<typeof chatStatusSchema>;

export interface ChatPanelChannel {
  onStatus(listener: (status: ChatStatus) => void): () => void;
  cancel(message: EchoCore.OutputCancel): void;
}

declare global {
  interface Window {
    nemuChat?: ChatPanelChannel;
  }
}
