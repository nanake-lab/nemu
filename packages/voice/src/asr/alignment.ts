import { spawn } from 'node:child_process';
import { mkdir, open } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { assertQuiet } from './quiet.js';
if (process.env.CI) throw new Error('Independent alignment is local-only; disabled in CI');
assertQuiet();
const project = fileURLToPath(new URL('./alignment/', import.meta.url));
const cache = join(process.env.LOCALAPPDATA ?? '', 'Nemu', 'models', 'evaluation');
const data =
  process.env.NEMU_ASR_DATA ?? join(process.env.LOCALAPPDATA ?? '', 'Nemu', 'asr-evaluation');
const logs = join(data, 'alignment-logs');
await mkdir(logs, { recursive: true });
const log = await open(join(logs, `${new Date().toISOString().replace(/[:.]/g, '-')}.log`), 'wx');
const args = [
  'run',
  '--project',
  project,
  '--locked',
  '--',
  'python',
  join(project, process.argv[2] === 'download' ? 'align_user.py' : 'context_references.py'),
];
if (process.argv[2] === 'download') args.push('--download');
const child = spawn('uv', args, {
  windowsHide: true,
  env: {
    ...process.env,
    CUDA_VISIBLE_DEVICES: '',
    UV_PROJECT_ENVIRONMENT: join(cache, 'alignment-venv'),
    HF_HUB_DISABLE_TELEMETRY: '1',
  },
  stdio: ['ignore', 'pipe', 'pipe'],
});
const writes: Promise<unknown>[] = [];
child.stdout.on('data', (chunk: Buffer) => {
  process.stdout.write(chunk);
  writes.push(log.write(chunk));
});
child.stderr.on('data', (chunk: Buffer) => {
  process.stderr.write(chunk);
  writes.push(log.write(chunk));
});
try {
  process.exitCode = await new Promise<number>((resolve, reject) => {
    child.on('error', reject);
    child.on('close', (code) => resolve(code ?? 1));
  });
} finally {
  await Promise.all(writes);
  await log.close();
}
