import { defineConfig } from '@playwright/test';
import { resolve } from 'node:path';
export default defineConfig({
  testDir: './tests',
  timeout: 30_000,
  workers: 1,
  reporter: [
    ['list'],
    ['json', { outputFile: resolve(import.meta.dirname, '../../reports/P0-01/e2e-results.json') }],
  ],
  outputDir: 'test-results/electron',
});
