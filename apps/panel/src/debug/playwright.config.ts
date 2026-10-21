import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './',
  testMatch: '**/*.e2e.spec.ts',
  timeout: 30_000,
  workers: 1,
});
