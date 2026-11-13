import { writeFileSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { ttsRequestSchema } from '../../packages/contracts/dist/tts.js';
const require = createRequire(new URL('../../packages/contracts/package.json', import.meta.url));
const { z } = require('zod');
const path = new URL('./src/nemu_voice_sidecar/tts-request.schema.json', import.meta.url);
const content = JSON.stringify(z.toJSONSchema(ttsRequestSchema), null, 2) + '\n';
if (process.argv.includes('--check')) {
  if (readFileSync(path, 'utf8') !== content)
    throw new Error('TTS schema drift; regenerate from contracts');
} else {
  writeFileSync(path, content);
}
