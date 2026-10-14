import { defineConfig } from 'vitest/config';
export default defineConfig({
  test: {
    include: ['packages/contracts/src/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      include: ['packages/contracts/src/**/*.ts'],
      exclude: ['**/*.test.ts', '**/generated/**'],
      reportsDirectory: 'reports/P0-02/coverage',
      reporter: ['text', 'json-summary'],
      thresholds: { 'packages/contracts/src/performance.ts': { lines: 90 } },
    },
  },
});
