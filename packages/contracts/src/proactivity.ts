import { z } from 'zod';
import {
  resourceEnumSchema,
  resourceInputSchema,
  resourceToWire,
} from './generated/echo/browser.js';
import type { ProactivityBoundariesResource } from './generated/echo/resources.js';

// Authority fields, limits, enum spellings and JSON map projection all come from terminal.proto.
export const proactiveKindSchema = resourceEnumSchema('ProactiveKind');
export const proactivityBoundariesSchema = resourceInputSchema('ProactivityBoundaries');
export type ProactivityBoundaries = ProactivityBoundariesResource;
export function proactivityToWire(value: ProactivityBoundaries) {
  return resourceToWire('ProactivityBoundaries', value);
}

export const socialActionKindSchema = z.enum(['social.message', 'social.commit']);
/** ADR 0011 / V2-18: uncertain commitments are classified as T3 by P6-14. */
export function socialPermission(kind: z.infer<typeof socialActionKindSchema>) {
  return kind === 'social.message'
    ? ({ tier: 'T2', trustAllowed: true } as const)
    : ({ tier: 'T3', trustAllowed: false } as const);
}
