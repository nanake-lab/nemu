import { z } from 'zod';

/** ADR 0022: animation-only candidate values, never genesis calibration. */
export const bodyReadoutMappingSchema = z.strictObject({
  version: z.literal('body-animation-v2-candidate-1'),
  calibration: z.literal('pending'),
  facingBodyYawDeg: z.number().min(0).max(30),
  postureScale: z.number().min(-1).max(1),
  earPoseScale: z.number().min(-1).max(1),
  tailTensionScale: z.number().min(-0.5).max(0.5),
  energyTempoScale: z.number().min(0).max(0.5),
  energyAmplitudeScale: z.number().min(0).max(1),
});
export type BodyReadoutMapping = z.infer<typeof bodyReadoutMappingSchema>;

export const CANDIDATE_BODY_READOUT_MAPPING: Readonly<BodyReadoutMapping> = Object.freeze({
  version: 'body-animation-v2-candidate-1',
  calibration: 'pending',
  facingBodyYawDeg: 10,
  postureScale: -0.5,
  earPoseScale: -0.5,
  tailTensionScale: -0.25,
  energyTempoScale: 0.5,
  energyAmplitudeScale: 0.5,
});
