import { z } from 'zod';
import { physicalStateSchema, rectSchema } from './geometry.js';
import { hitRegionIdSchema } from './vocabulary.js';

// Reused verbatim by the stage wire messages and the application bus mirrors.
export const mirroredStagePayloads = {
  'stage.ready': z.strictObject({ avatarName: z.string(), boundsPx: rectSchema }),
  'stage.hit': z.strictObject({
    region: hitRegionIdSchema,
    kind: z.enum(['click', 'doubleClick', 'hoverStart', 'hoverEnd', 'pat']),
  }),
  'stage.drag': z.strictObject({
    phase: z.enum(['start', 'move', 'end']),
    screenX: z.number(),
    screenY: z.number(),
    vx: z.number(),
    vy: z.number(),
  }),
  'stage.frameStats': z.strictObject({
    fps: z.number(),
    p95FrameMs: z.number(),
    state: physicalStateSchema,
  }),
  'stage.error': z.strictObject({ code: z.string(), message: z.string() }),
} as const;
