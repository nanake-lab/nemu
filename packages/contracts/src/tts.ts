import { z } from 'zod';
import { langSchema } from './vocabulary.js';

/** CONTRACTS.md 5.4; Base style selects reference audio, never an instruction. */
export const ttsRequestSchema = z
  .object({
    text: z.string().min(1),
    lang: langSchema,
    style: z.string(),
    voiceId: z.string(),
    sampleRate: z.literal(24000),
  })
  .strict();
export type TtsRequest = z.infer<typeof ttsRequestSchema>;

export interface TtsProvider {
  id: string;
  synthesize(req: TtsRequest, signal: AbortSignal): AsyncIterable<Float32Array>;
  warmup?(): Promise<void>;
}
