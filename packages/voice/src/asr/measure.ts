import { spawnSync } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { assertQuiet } from './quiet.js';
assertQuiet();
const evidence = resolve('reports/P4-01/measurement');
mkdirSync(evidence, { recursive: true });
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
