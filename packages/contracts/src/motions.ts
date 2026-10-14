import { z } from 'zod';

// CONTRACTS 1.8: the catalogue is the single source for both allowlists.
export const motionGroups = {
  idle: ['idle_stand', 'idle_relaxed', 'idle_hands_behind', 'idle_look_around'],
  fidget: [
    'fidget_stretch',
    'fidget_yawn',
    'fidget_tail_play',
    'fidget_ear_scratch',
    'fidget_hum',
    'fidget_sway',
  ],
  gesture: ['wave', 'nod', 'shake_head', 'think', 'explain', 'cheer'],
  reaction: ['react_surprised', 'react_happy_hop', 'react_pout', 'react_sigh'],
  desktop: ['sit_edge_loop', 'sit_to_stand', 'walk', 'dangle'],
  state: ['sleep_loop', 'wake_up', 'working_loop', 'peek_loop', 'fall_loop', 'land'],
} as const;
export const motionIdSchema = z.enum([
  ...motionGroups.idle,
  ...motionGroups.fidget,
  ...motionGroups.gesture,
  ...motionGroups.reaction,
  ...motionGroups.desktop,
  ...motionGroups.state,
]);
export type MotionId = z.infer<typeof motionIdSchema>;
export const performableMotionIdSchema = z.enum([
  ...motionGroups.gesture,
  ...motionGroups.reaction,
]);
export const requestableMotionIdSchema = z.enum([
  ...motionGroups.fidget,
  ...motionGroups.gesture,
  ...motionGroups.reaction,
]);
export const motionSlots = Object.entries(motionGroups).flatMap(([category, ids]) =>
  ids.map((id) =>
    Object.freeze({
      id,
      category,
      performable: performableMotionIdSchema.safeParse(id).success,
      requestable: requestableMotionIdSchema.safeParse(id).success,
    }),
  ),
);
