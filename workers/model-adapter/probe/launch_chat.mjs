// This entry is run only by an explicit `pnpm probe:chat` command.
import { existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const here = dirname(fileURLToPath(import.meta.url));
const python = process.env.NEMU_PROBE_PYTHON
  ?? resolve(here, '.venv', process.platform === 'win32' ? 'Scripts/python.exe' : 'bin/python');
if (!existsSync(python)) {
  throw new Error('Set NEMU_PROBE_PYTHON to the existing probe environment python executable.');
}
const result = spawnSync(python, [resolve(here, 'chat.py'), ...process.argv.slice(2)], {
  stdio: 'inherit',
  windowsHide: true,
  env: { ...process.env, HF_HUB_OFFLINE: '1', TRANSFORMERS_OFFLINE: '1' },
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
