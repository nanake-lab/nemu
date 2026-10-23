import { appBusPayloads, type AppBusPayloads } from '@nemu/contracts';
import { z } from 'zod';

// Narrow the existing contract. Optional context fields are deliberately excluded.
export const metadataSchema = appBusPayloads['codex.session']
  .pick({ event: true, sessionId: true })
  .extend({ sessionId: z.string().regex(/^[a-zA-Z0-9_-]{1,128}$/) })
  .strict();
export type SessionMetadata = Pick<AppBusPayloads['codex.session'], 'event' | 'sessionId'>;
export const sessionEvents = appBusPayloads['codex.session'].shape.event.options;
export const maxFrameBytes = 1024;

export function projectHookInput(raw: unknown): SessionMetadata | undefined {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return;
  const fields = raw as Record<string, unknown>;
  const result = metadataSchema.safeParse({
    event: fields.hook_event_name,
    sessionId: fields.session_id,
  });
  return result.success ? result.data : undefined;
}
