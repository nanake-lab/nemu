import { fileURLToPath } from 'node:url';
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  testMatch: 'chat.spec.ts',
  workers: 1,
  use: { baseURL: 'http://127.0.0.1:5175' },
  webServer: {
    cwd: fileURLToPath(new URL('../../../', import.meta.url)),
    command: 'pnpm dev:panel',
    url: 'http://127.0.0.1:5175',
  },
});
