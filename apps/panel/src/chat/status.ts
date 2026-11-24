import { z } from 'zod';

export const chatStatusSchema = z.object({
  reason: z.string().nullable(),
  canSend: z.boolean(),
  disconnected: z.boolean().optional(),
  rejection: z.object({ eventId: z.ulid(), reason: z.string() }).optional(),
});
export type ChatStatus = z.infer<typeof chatStatusSchema>;
