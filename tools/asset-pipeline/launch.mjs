import { spawnSync } from 'node:child_process';
import { join } from 'node:path';

const args = process.argv.slice(2).filter((arg) => arg !== '--');
const mode = args.shift();
if (mode === 'step') {
  const [step, input, output, ...rest] = args;
  if (!step || !input || !output) throw new Error('Usage: pnpm asset:step -- tail in.blend out.blend [-Config build.yaml] [-Output output-directory]');
  args.splice(0, args.length, '-Step', step, '-InputBlend', input, '-OutputBlend', output, ...rest);
}
const result = spawnSync('pwsh', ['-NoProfile', '-File', join(import.meta.dirname, 'run.ps1'), ...args], { stdio: 'inherit' });
if (result.error) console.error(result.error.message);
process.exitCode = result.status ?? 1;
