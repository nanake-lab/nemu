import { defineConfig } from 'vitest/config';
export default defineConfig({
  test: {
    include: [
      'apps/panel/src/legacy/**/*.test.tsx',
      'packages/*/src/**/*.test.ts',
      'executors/codex-bridge/src/**/*.test.ts',
      'tools/scaffold/*.test.ts',
    ],
  },
});
