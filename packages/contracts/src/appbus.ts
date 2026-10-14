import { z } from 'zod';
import { performanceCueSchema } from './performance.js';
import { mirroredStagePayloads } from './stage-events.js';
import {
  dndStateSchema,
  langSchema,
  lineageSchema,
  speechModeSchema,
  toolTierSchema,
} from './vocabulary.js';

export const desktopObservationKindSchema = z.enum([
  'foreground_changed',
  'idle_started',
  'idle_ended',
  'dnd_changed',
  'session_locked',
  'session_unlocked',
  'meeting_started',
  'meeting_ended',
]);
export type DesktopObservationKind = z.infer<typeof desktopObservationKindSchema>;
export const queryKindSchema = z.enum(['memory', 'actions', 'run_history']);
export type QueryKind = z.infer<typeof queryKindSchema>;
export const queryStatusSchema = z.enum([
  'ok',
  'no_match',
  'index_lagging',
  'denied',
  'object_missing',
  'budget_exhausted',
]);
export type QueryStatus = z.infer<typeof queryStatusSchema>;
export const queryFilterSchema = z.strictObject({
  text: z.string().optional(),
  from: z.string().optional(),
  to: z.string().optional(),
  kinds: z.array(z.string()).optional(),
  includeRetracted: z.boolean().optional(),
});
export type QueryFilter = z.infer<typeof queryFilterSchema>;
export const queryItemSchema = z.strictObject({
  ref: z.string(),
  at: z.string(),
  kind: z.string(),
  summary: z.string(),
  source: z.string(),
  retracted: z.boolean(),
});
export type QueryItem = z.infer<typeof queryItemSchema>;
export const latencyMarksSchema = z.strictObject({
  vadEnd: z.number().optional(),
  asrDone: z.number().optional(),
  received: z.number().optional(),
  committed: z.number().optional(),
  firstChunk: z.number().optional(),
  ttsFirstChunk: z.number().optional(),
  audioStart: z.number().optional(),
});
export type LatencyMarks = z.infer<typeof latencyMarksSchema>;
export const appBusPayloads = {
  'voice.vad': z.strictObject({ state: z.enum(['speechStart', 'speechEnd']) }),
  'voice.bargeIn': z.strictObject({}),
  'voice.recognized': z.strictObject({
    utteranceKey: z.string(),
    text: z.string(),
    lang: langSchema,
    durationMs: z.number(),
    confidence: z.number().optional(),
  }),
  'output.state': z.strictObject({
    outputId: z.string(),
    state: z.enum(['streaming', 'speaking', 'done', 'cancelled']),
  }),
  'output.chunk': z.strictObject({
    outputId: z.string(),
    chunkIndex: z.number(),
    text: z.string(),
    cues: z.array(performanceCueSchema),
  }),
  'core.runState': z.strictObject({
    states: z.array(z.string()),
    speechMode: speechModeSchema,
    strict: z.boolean(),
    lineage: lineageSchema,
    bornAt: z.string().optional(),
    autonomy: z.enum(['active', 'paused']),
    reasons: z.array(z.string()),
  }),
  'approval.request': z.strictObject({
    approvalId: z.string(),
    actionId: z.string(),
    tier: toolTierSchema,
    title: z.string(),
    detail: z.string(),
    expiresAt: z.number(),
  }),
  'approval.decision': z.strictObject({
    approvalId: z.string(),
    decision: z.enum(['allowOnce', 'allowAlways', 'deny']),
  }),
  'chat.send': z.strictObject({ text: z.string() }),
  'memory.retract': z.strictObject({
    targetRefs: z.array(z.string()),
    note: z.string().optional(),
  }),
  'core.query': z.strictObject({
    queryId: z.string(),
    kind: queryKindSchema,
    filter: queryFilterSchema.optional(),
    limit: z.number(),
  }),
  'core.queryResult': z.strictObject({
    queryId: z.string(),
    status: queryStatusSchema,
    items: z.array(queryItemSchema),
  }),
  'desktop.dnd': z.strictObject({ state: dndStateSchema }),
  'desktop.idle': z.strictObject({ idleMs: z.number() }),
  'desktop.observation': z.strictObject({
    kind: desktopObservationKindSchema,
    processName: z.string().optional(),
    title: z.string().optional(),
    dnd: dndStateSchema.optional(),
    idleMs: z.number().optional(),
  }),
  'codex.session': z.strictObject({
    event: z.enum(['SessionStart', 'PermissionRequest', 'Stop']),
    sessionId: z.string(),
    cwdLabel: z.string().optional(),
    summary: z.string().optional(),
  }),
  'metrics.latency': z.strictObject({ utteranceKey: z.string(), marks: latencyMarksSchema }),
  ...mirroredStagePayloads,
} as const;
export type AppBusPayloads = {
  [K in keyof typeof appBusPayloads]: z.infer<(typeof appBusPayloads)[K]>;
};
export interface AppBusEvent<T extends string = keyof AppBusPayloads, P = unknown> {
  v: 2;
  id: string;
  ts: number;
  source: 'main' | 'stage' | 'panel' | 'voice';
  type: T;
  payload: P;
  corr?: string;
}
export type ApplicationEvent = {
  [K in keyof AppBusPayloads]: AppBusEvent<K, AppBusPayloads[K]>;
}[keyof AppBusPayloads];
function eventSchema<K extends keyof AppBusPayloads>(type: K) {
  return z.strictObject({
    v: z.literal(2),
    id: z.ulid(),
    ts: z.number(),
    source: z.enum(['main', 'stage', 'panel', 'voice']),
    type: z.literal(type),
    payload: appBusPayloads[type],
    corr: z.string().optional(),
  });
}
export const appBusEventSchema = z.union(
  (Object.keys(appBusPayloads) as Array<keyof AppBusPayloads>).map(eventSchema),
) as z.ZodType<ApplicationEvent>;
export const forwardedAppBusTypes = [
  'voice.vad',
  'voice.bargeIn',
  'output.state',
  'core.runState',
  'desktop.dnd',
  'desktop.idle',
] as const;
export type ForwardedAppBusEvent = Extract<
  ApplicationEvent,
  { type: (typeof forwardedAppBusTypes)[number] }
>;
export const forwardedAppBusEventSchema = z.union(
  forwardedAppBusTypes.map(eventSchema),
) as z.ZodType<ForwardedAppBusEvent>;
