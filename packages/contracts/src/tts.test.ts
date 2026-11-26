import { expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { z } from 'zod';
import { ttsRequestSchema } from './tts.js';

it('implements CONTRACTS 5.4 without accepting reference paths or different sample rates', () => {
  const request = {
    text: 'Hello.',
    lang: 'en',
    style: 'neutral',
    voiceId: 'test',
    sampleRate: 24000,
  };
  expect(ttsRequestSchema.parse(request)).toEqual(request);
  expect(ttsRequestSchema.safeParse({ ...request, sampleRate: 16000 }).success).toBe(false);
  expect(ttsRequestSchema.safeParse({ ...request, ref_audio: 'human.wav' }).success).toBe(false);
});

it('keeps the Python boundary schema generated from the shared contract', () => {
  const path = new URL(
    '../../../services/voice-sidecar/src/nemu_voice_sidecar/tts-request.schema.json',
    import.meta.url,
  );
  expect(JSON.parse(readFileSync(path, 'utf8'))).toEqual(z.toJSONSchema(ttsRequestSchema));
});
