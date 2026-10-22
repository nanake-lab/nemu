import { z } from 'zod';

export const proactiveKindSchema = z.enum([
  'speak', 'motion', 'goTo', 'sit', 'sleep', 'wake', 'peek', 'gaze', 'helper.ask',
]);
export const proactivityBoundariesSchema = z.strictObject({
  version: z.string().min(1),
  allowedKinds: z.array(proactiveKindSchema).refine(v => new Set(v).size === v.length),
  cooldownS: z.partialRecord(proactiveKindSchema, z.number().nonnegative().finite()),
  gates: z.array(z.enum(['dnd', 'quietHours', 'budget', 'mode', 'minInterval']))
    .refine(v => new Set(v).size === v.length),
  budgets: z.strictObject({
    minIntervalS: z.number().nonnegative().finite(),
    speakPerHour: z.number().int().nonnegative(),
    quietDayFactor: z.number().min(0).max(1),
    autonomousHelperPerDay: z.number().int().nonnegative(),
  }),
});
export type ProactivityBoundaries = z.infer<typeof proactivityBoundariesSchema>;

export const socialActionKindSchema = z.enum(['social.message', 'social.commit']);
/** ADR 0011 / V2-18: uncertain commitments are classified as T3 by P6-14. */
export function socialPermission(kind: z.infer<typeof socialActionKindSchema>) {
  return kind === 'social.message'
    ? { tier: 'T2', trustAllowed: true } as const
    : { tier: 'T3', trustAllowed: false } as const;
}
