import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
const probe = resolve(process.env.NEMU_TTS_PROBE ?? 'workers/model-adapter/probe');
const python = resolve(
  probe,
  process.platform === 'win32' ? '.venv/Scripts/python.exe' : '.venv/bin/python',
);
if (!existsSync(python))
  throw new Error('Set NEMU_TTS_PROBE to the prepared P0-07 probe directory');
const result = spawnSync(
  python,
  ['services/voice-sidecar/benchmark.py', '--probe', probe, ...process.argv.slice(2)],
  {
    stdio: 'inherit',
    windowsHide: true,
  },
);
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
