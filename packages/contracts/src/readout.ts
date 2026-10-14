import { z } from 'zod';
import { emotionSchema } from './vocabulary.js';

export const bodyReadoutSchema = z.strictObject({
  version: z.string(),
  expressionBaseline: z.partialRecord(emotionSchema, z.number().min(0).max(0.4)),
  idleTempo: z.number().min(0.5).max(1.5),
  earPose: z.number().min(-1).max(1),
  tailAmplitude: z.number().min(0).max(1),
  gazeSociability: z.number().min(0).max(1),
  posture: z.number().min(-1).max(1),
});
export type BodyReadout = z.infer<typeof bodyReadoutSchema>;
