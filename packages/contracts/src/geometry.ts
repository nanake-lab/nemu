import { z } from 'zod';
import { requestableMotionIdSchema } from './motions.js';

export const anchorSchema = z.discriminatedUnion('kind', [
  z.strictObject({ kind: z.literal('free'), x: z.number(), y: z.number() }),
  z.strictObject({ kind: z.literal('windowEdge'), hwnd: z.string(), offsetX: z.number() }),
  z.strictObject({
    kind: z.literal('screenEdge'),
    side: z.enum(['left', 'right', 'bottom']),
    along: z.number(),
  }),
  z.strictObject({ kind: z.literal('taskbar'), along: z.number() }),
]);
export type Anchor = z.infer<typeof anchorSchema>;
export const gazeTargetSchema = z.union([
  z.enum(['user', 'cursor', 'away', 'window']),
  z.strictObject({ x: z.number(), y: z.number() }),
]);
export type GazeTarget = z.infer<typeof gazeTargetSchema>;
export const platformSchema = z.strictObject({
  id: z.string(),
  hwnd: z.string(),
  x1: z.number(),
  x2: z.number(),
  y: z.number(),
  z: z.number(),
});
export type Platform = z.infer<typeof platformSchema>;
export const rectSchema = z.strictObject({
  x: z.number(),
  y: z.number(),
  w: z.number(),
  h: z.number(),
});
export type Rect = z.infer<typeof rectSchema>;
export const physicalStateSchema = z.enum([
  'idle',
  'listening',
  'speaking',
  'sitting',
  'walking',
  'dragged',
  'falling',
  'sleeping',
  'peeking',
  'hidden',
]);
export type PhysicalState = z.infer<typeof physicalStateSchema>;
export const bodyActionSchema = z.discriminatedUnion('kind', [
  z.strictObject({ kind: z.literal('motion'), motionId: requestableMotionIdSchema }),
  z.strictObject({ kind: z.literal('goTo'), anchor: anchorSchema }),
  z.strictObject({ kind: z.literal('sit'), platformId: z.string() }),
  z.strictObject({ kind: z.literal('sleep') }),
  z.strictObject({ kind: z.literal('wake') }),
  z.strictObject({ kind: z.literal('peek'), side: z.enum(['left', 'right']) }),
  z.strictObject({
    kind: z.literal('gaze'),
    target: gazeTargetSchema,
    holdMs: z.number().optional(),
  }),
]);
export type BodyAction = z.infer<typeof bodyActionSchema>;
