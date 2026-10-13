import { spawn } from 'node:child_process';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const scratch = await mkdtemp(join(tmpdir(), 'nemu-scaffold-'));
const entry = join(require.resolve('electron-vite/package.json'), '..', 'bin', 'electron-vite.js');
const child = spawn(process.execPath, [entry, 'dev'], {
  stdio: 'inherit',
  env: { ...process.env, NEMU_SCAFFOLD_USER_DATA: scratch },
});
process.on('SIGINT', () => child.kill('SIGINT'));
process.on('SIGTERM', () => child.kill('SIGTERM'));
try {
  process.exitCode = await new Promise((resolve, reject) => {
    child.on('error', reject);
    child.on('exit', (code) => resolve(code ?? 1));
  });
} finally {
  await rm(scratch, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
}
