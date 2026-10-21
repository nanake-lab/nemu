import { spawnSync } from 'node:child_process';
import { mkdirSync, existsSync, copyFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { assertQuiet } from './quiet.js';
assertQuiet();
const evidence = resolve(
  `reports/P4-01/measurement-${new Date().toISOString().replace(/[:.]/g, '-')}`,
);
mkdirSync(evidence, { recursive: true });
if (existsSync('reports/P4-01/idle-cpu.json'))
  copyFileSync('reports/P4-01/idle-cpu.json', resolve(evidence, 'previous-idle-cpu.json'));

const child = spawnSync(
  process.execPath,
  [
    'node_modules/@playwright/test/cli.js',
    'test',
    '--config',
    'apps/desktop/playwright.config.ts',
    'apps/desktop/tests/voice-idle.spec.ts',
  ],
  { env: { ...process.env, NEMU_ASR_MEASURE: '1', NEMU_EVIDENCE_DIR: evidence }, stdio: 'inherit' },
);
process.exitCode = child.status ?? 1;
