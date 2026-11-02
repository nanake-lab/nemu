import { z } from 'zod';
import { rectSchema, type AppBusPayloads, type StagePayloads } from '@nemu/contracts';

const bounds = rectSchema.refine((r) => r.w > 0 && r.h > 0);
export const windowSampleSchema = z.strictObject({
  hwnd: z.string().min(1),
  processName: z.string().optional(),
  windowClass: z.string(),
  bounds,
  visible: z.boolean(),
  minimized: z.boolean(),
  cloaked: z.boolean(),
  z: z.number().int().nonnegative(),
});
export type WindowSample = z.infer<typeof windowSampleSchema>;
export const desktopSampleSchema = z.strictObject({
  windows: z.array(windowSampleSchema),
  monitors: z.array(
    z.strictObject({
      id: z.string(),
      bounds,
      workArea: bounds,
      dpiX: z.number().positive(),
      dpiY: z.number().positive(),
    }),
  ),
  taskbars: z.array(z.strictObject({ hwnd: z.string(), bounds })),
  foreground: z.string().nullable(),
  caret: rectSchema.optional(),
  idleMs: z.number().int().nonnegative().nullable(),
  notificationState: z.number().int().min(1).max(7).nullable(),
});
export type DesktopSample = z.infer<typeof desktopSampleSchema>;
export type DesktopGeometry = StagePayloads['stage.desktop.update'];
export type DesktopObservation = AppBusPayloads['desktop.observation'];

/** In-process adapter only. Titles are deliberately absent from the sample DTO. */
export interface DesktopSource {
  sample(): DesktopSample;
  readTitle(hwnd: string): string | undefined;
  subscribe(invalidate: () => void): () => void;
  dispose(): void;
}

export interface SenseSettings {
  /** Must query the current T1 decision, including revocation; defaults to denied. */
  titleAuthorized?: () => boolean;
  quietHours?: () => boolean;
  idleThresholdMs: number;
}
