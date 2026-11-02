// Run the unchanged visual suite, changing only its historical report destination.
import { readFile, writeFile, unlink } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';

const source = await readFile('apps/stage/tests/visual/runner.mjs', 'utf8');
const marker = "const directory = 'reports/P1-01/visual';";
if (!source.includes(marker)) throw new Error('Visual runner report declaration changed; inspect before running');
const target = resolve('apps/stage/src/review/.p306-visual-runner.mjs');
await writeFile(target, source.replace(marker, "const directory = 'reports/P3-06/visual/golden';"));
try {
  const code = await new Promise((resolveExit, reject) => {
    const child = spawn(process.execPath, [target], { stdio: 'inherit', windowsHide: true });
    child.on('error', reject); child.on('exit', resolveExit);
  });
  process.exitCode = typeof code === 'number' ? code : 1;
} finally {
  await unlink(target);
}
