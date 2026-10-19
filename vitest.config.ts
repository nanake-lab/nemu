import { defineConfig } from 'vitest/config';
export default defineConfig({
  test: {
    include: [
      'apps/panel/src/legacy/**/*.test.tsx',
      'apps/stage/src/**/*.test.ts',
      'apps/desktop/src/**/*.test.ts',
      'packages/*/src/**/*.test.ts',
      'executors/codex-bridge/src/**/*.test.ts',
      'tools/scaffold/*.test.ts',
    ],
  },
});
