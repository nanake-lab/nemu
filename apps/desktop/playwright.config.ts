import { defineConfig } from '@playwright/test';
import { resolve } from 'node:path';
export default defineConfig({
  testDir: './tests',
  timeout: 30_000,
  workers: 1,
  reporter: [
    ['list'],
    [
      'json',
      {
        outputFile: process.env.NEMU_EVIDENCE_DIR
          ? resolve(process.env.NEMU_EVIDENCE_DIR, 'e2e-results.json')
          : resolve(import.meta.dirname, '../../reports/P0-01/e2e-results.json'),
      },
    ],
  ],
  outputDir: process.env.NEMU_EVIDENCE_DIR
    ? resolve(process.env.NEMU_EVIDENCE_DIR, 'test-results/electron')
    : 'test-results/electron',
});
