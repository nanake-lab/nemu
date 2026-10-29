import { z } from 'zod';
import {
  anchorSchema,
  bodyActionSchema,
  gazeTargetSchema,
  physicalStateSchema,
  platformSchema,
  rectSchema,
} from './geometry.js';
import { forwardedAppBusEventSchema } from './appbus.js';
import { performanceCueSchema } from './performance.js';
import { bodyReadoutSchema, bodyReadoutV2Schema } from './readout.js';
import { mirroredStagePayloads } from './stage-events.js';
import { dndStateSchema, langSchema } from './vocabulary.js';
export * from './geometry.js';

export const controlCommandPayloads = {
  'stage.loadAvatar': z.strictObject({ vrmUrl: z.string(), profileUrl: z.string() }),
  'stage.setPlacement': z.strictObject({
    displayId: z.number(),
    anchor: anchorSchema,
    transitionMs: z.number().optional(),
  }),
  'stage.setPhysicalState': z.strictObject({ state: physicalStateSchema, reason: z.string() }),
  'stage.perform': z.strictObject({ cue: performanceCueSchema }),
  'stage.action': z.strictObject({ actionId: z.string(), action: bodyActionSchema }),
  'stage.readout': z.strictObject({ readout: bodyReadoutSchema }),
  'stage.readout.v2': z.strictObject({ readout: bodyReadoutV2Schema }),
  'stage.event': z.strictObject({ event: forwardedAppBusEventSchema }),
  'stage.gaze': z.strictObject({ target: gazeTargetSchema, holdMs: z.number().optional() }),
  'stage.setQuality': z.strictObject({
    fpsActive: z.number(),
    fpsIdle: z.number(),
    fpsSleep: z.number(),
    msaa: z.union([z.literal(0), z.literal(4)]),
  }),
  'stage.mic.set': z.strictObject({
    enabled: z.boolean(),
    mode: z.enum(['pushToTalk', 'continuous']),
  }),
  'stage.desktop.update': z.strictObject({
    platforms: z.array(platformSchema),
    caret: rectSchema.optional(),
    dnd: dndStateSchema,
  }),
} as const;
export const controlEventPayloads = {
  ...mirroredStagePayloads,
  'stage.hitTest': z.strictObject({ overCharacter: z.boolean() }),
  'stage.bounds': z.strictObject({ screenRect: rectSchema }),
  'stage.actionResult': z.strictObject({
    actionId: z.string(),
    status: z.enum(['succeeded', 'failed', 'cancelled']),
    detail: z.string().optional(),
  }),
} as const;
export const audioCommandPayloads = {
  'stage.speech.begin': z.strictObject({
    outputId: z.string(),
    chunkIndex: z.number(),
    sentenceId: z.string(),
    text: z.string(),
    lang: langSchema,
    cues: z.array(performanceCueSchema),
    sampleRate: z.number(),
  }),
  'stage.speech.chunk': z.strictObject({ sentenceId: z.string(), pcm: z.instanceof(Float32Array) }),
  'stage.speech.end': z.strictObject({ sentenceId: z.string() }),
  'stage.speech.stop': z.strictObject({ fadeMs: z.number() }),
} as const;
export const audioEventPayloads = {
  'stage.speech.started': z.strictObject({ sentenceId: z.string(), at: z.number() }),
  'stage.speech.progress': z.strictObject({ sentenceId: z.string(), playedMs: z.number() }),
  'stage.speech.done': z.strictObject({
    sentenceId: z.string(),
    interrupted: z.boolean(),
    playedChars: z.number(),
  }),
} as const;
export const stagePayloads = {
  ...controlCommandPayloads,
  ...controlEventPayloads,
  ...audioCommandPayloads,
  ...audioEventPayloads,
};
export type StagePayloads = {
  [K in keyof typeof stagePayloads]: z.infer<(typeof stagePayloads)[K]>;
};
export interface StageMessage<T extends string = keyof StagePayloads, P = unknown> {
  v: 2;
  id: string;
  type: T;
  payload: P;
}
type Messages<K extends keyof StagePayloads> = { [T in K]: StageMessage<T, StagePayloads[T]> }[K];
export type ControlCommand = Messages<keyof typeof controlCommandPayloads>;
export type ControlEvent = Messages<keyof typeof controlEventPayloads>;
export type AudioCommand = Messages<keyof typeof audioCommandPayloads>;
export type AudioEvent = Messages<keyof typeof audioEventPayloads>;
export type ControlMessage = ControlCommand | ControlEvent;
export type AudioMessage = AudioCommand | AudioEvent;
export type StageWireMessage = ControlMessage | AudioMessage;

function messages<K extends keyof StagePayloads>(
  payloads: Record<K, z.ZodType>,
): z.ZodType<Messages<K>> {
  return z.union(
    (Object.keys(payloads) as K[]).map((type) =>
      z.strictObject({
        v: z.literal(2),
        id: z.string(),
        type: z.literal(type),
        payload: payloads[type],
        // Keys and payloads come from the same catalogue; Object.keys erases that correlation.
      }),
    ),
  ) as unknown as z.ZodType<Messages<K>>;
}
export const controlCommandSchema = messages(controlCommandPayloads);
export const controlEventSchema = messages(controlEventPayloads);
export const audioCommandSchema = messages(audioCommandPayloads);
export const audioEventSchema = messages(audioEventPayloads);
export const controlMessageSchema = z.union([controlCommandSchema, controlEventSchema]);
export const audioMessageSchema = z.union([audioCommandSchema, audioEventSchema]);
export const stageMessageSchema = z.union([controlMessageSchema, audioMessageSchema]);
export const microphoneMessageSchema = z.strictObject({
  v: z.literal(2),
  seq: z.number().int().nonnegative(),
  pcm: z
    .instanceof(Int16Array)
    .refine((pcm) => pcm.length === 320, '20 ms at 16 kHz is 320 mono samples'),
});
export type MicrophoneMessage = z.infer<typeof microphoneMessageSchema>;
