import { evaluationEnvironment } from './cache.js';
import { mkdirSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
if (process.env.CI) throw new Error('Dataset downloads are disabled in CI');
mkdirSync(evaluationEnvironment().TEMP!, { recursive: true });
const result = spawnSync(
  'uv',
  ['run', fileURLToPath(new URL('./download-public.py', import.meta.url))],
  { stdio: 'inherit', env: evaluationEnvironment() },
);
process.exitCode = result.status ?? 1;
