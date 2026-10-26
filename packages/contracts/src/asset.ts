import { z } from 'zod';
import { emotionSchema, hitRegionIdSchema, langSchema } from './vocabulary.js';
import { bodyReadoutMappingSchema } from './body-animation.js';

const vector3 = z.tuple([z.number(), z.number(), z.number()]);
// These contract fields are relative paths, not URLs or host-absolute paths.
const relativePath = z
  .string()
  .min(1)
  .refine((path) => !/^(?:[\\/]|[A-Za-z][A-Za-z\d+.-]*:)/u.test(path), 'Expected a relative path');
const framing = z.strictObject({
  fovDeg: z.number(),
  distanceM: z.number(),
  targetBone: z.string(),
  offsetM: vector3.optional(),
});
export const nemuProfileSchema = z.strictObject({
  version: z.literal(2),
  model: relativePath,
  heightM: z.number(),
  headTopOffsetM: z.number(),
  hitRegions: z.array(
    z
      .strictObject({
        id: hitRegionIdSchema,
        bone: z.string(),
        shape: z.enum(['sphere', 'capsule']),
        radiusM: z.number(),
        lengthM: z.number().optional(),
        offsetM: vector3.optional(),
      })
      .refine((region) => region.shape !== 'capsule' || region.lengthM !== undefined, {
        message: 'Capsule requires lengthM',
        path: ['lengthM'],
      }),
  ),
  gazeLimits: z.strictObject({
    yawDeg: z.number(),
    pitchUpDeg: z.number(),
    pitchDownDeg: z.number(),
  }),
  framing: z.strictObject({ fullBody: framing, upperBody: framing }),
  rig: z.strictObject({
    earBonesL: z.array(z.string()),
    earBonesR: z.array(z.string()),
    tailBones: z.array(z.string()),
    earBackMaxDeg: z.number(),
    tailSwingDeg: z.tuple([z.number(), z.number()]),
  }),
  emotionMap: z.record(
    emotionSchema,
    z.array(z.strictObject({ expression: z.string(), weight: z.number() })),
  ),
  visemeMap: z.strictObject({
    A: z.literal('aa'),
    I: z.literal('ih'),
    U: z.literal('ou'),
    E: z.literal('ee'),
    O: z.literal('oh'),
  }),
  lipSyncProfiles: z.record(langSchema, relativePath),
  idleSet: z.array(
    z.strictObject({ motionId: z.string(), weight: z.number(), cooldownS: z.number() }),
  ),
  bodyReadoutMapping: bodyReadoutMappingSchema.optional(),
});
export type NemuProfile = z.infer<typeof nemuProfileSchema>;
export const assetManifestEntrySchema = z.strictObject({
  path: z.string(),
  kind: z.enum(['model', 'motion', 'texture', 'voice', 'core', 'other']),
  source: z.string(),
  license: z.string(),
  redistributable: z.boolean(),
  author: z.string().optional(),
  notes: z.string().optional(),
});
export type AssetManifestEntry = z.infer<typeof assetManifestEntrySchema>;
