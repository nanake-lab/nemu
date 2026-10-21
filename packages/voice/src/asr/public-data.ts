import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
if (process.env.CI) throw new Error('Dataset downloads are disabled in CI');
const result = spawnSync(
  'uv',
  ['run', fileURLToPath(new URL('./download-public.py', import.meta.url))],
  { stdio: 'inherit' },
);
process.exitCode = result.status ?? 1;
