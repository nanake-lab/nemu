import { z } from 'zod';

export const langSchema = z.enum(['zh', 'en', 'ja']);
export type Lang = z.infer<typeof langSchema>;
export const emotionSchema = z.enum([
  'neutral',
  'happy',
  'angry',
  'sad',
  'relaxed',
  'surprised',
  'smug',
  'shy',
  'sleepy',
  'pout',
  'excited',
  'worried',
]);
export type Emotion = z.infer<typeof emotionSchema>;
export const dndStateSchema = z.enum([
  'none',
  'fullscreen',
  'presentation',
  'busy',
  'meeting',
  'quietHours',
]);
export type DndState = z.infer<typeof dndStateSchema>;
export const toolTierSchema = z.enum(['T0', 'T1', 'T2', 'T3']);
export type ToolTier = z.infer<typeof toolTierSchema>;
export const speechModeSchema = z.enum(['A_LOCAL_STRICT', 'B_REMOTE_PROJECTED', 'C_DRAFT_REWRITE']);
export type SpeechMode = z.infer<typeof speechModeSchema>;
export const lineageSchema = z.enum(['REHEARSAL', 'LIFE']);
export type Lineage = z.infer<typeof lineageSchema>;
export const hitRegionIdSchema = z.enum([
  'head',
  'ears',
  'tail',
  'body',
  'handL',
  'handR',
  'skirt',
]);
export type HitRegionId = z.infer<typeof hitRegionIdSchema>;
