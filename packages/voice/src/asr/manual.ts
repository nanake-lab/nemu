import { spawn } from 'node:child_process';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { evaluationDirectory, evaluationEnvironment } from './cache.js';

if (process.env.CI)
  throw new Error('Human audio annotation is local-only; CI does not access recordings');
const mode = process.argv[2];
if (mode !== 'prepare' && mode !== 'import' && mode !== 'rescore')
  throw new Error('Expected prepare, import or rescore');
if (mode === 'import' && !process.argv[3]) throw new Error('Pass the final annotation JSON path');
const python = join(
  evaluationDirectory(),
  'alignment-venv',
  ...(process.platform === 'win32' ? ['Scripts', 'python.exe'] : ['bin', 'python']),
);
const script = fileURLToPath(
  new URL(
    mode === 'prepare'
      ? './alignment/manual_page.py'
      : mode === 'rescore'
        ? './alignment/rescore_annotation.py'
        : './alignment/manual_import.py',
    import.meta.url,
  ),
);
const child = spawn(python, [script, ...(mode === 'import' ? [process.argv[3]!] : [])], {
  windowsHide: true,
  stdio: 'inherit',
  env: evaluationEnvironment(),
});
process.exitCode = await new Promise<number>((resolve, reject) => {
  child.on('error', reject);
  child.on('close', (code) => resolve(code ?? 1));
});
