import { defineConfig, externalizeDepsPlugin } from 'electron-vite';
import { resolve } from 'node:path';

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin({ exclude: ['@nemu/contracts'] })],
  },
  preload: {
    build: { rollupOptions: { output: { format: 'cjs', entryFileNames: '[name].cjs' } } },
  },
  renderer: {
    root: resolve(import.meta.dirname, '..'),
    base: './',
    build: {
      rollupOptions: {
        input: {
          stage: resolve(import.meta.dirname, '../stage/index.html'),
          panel: resolve(import.meta.dirname, '../panel/index.html'),
        },
      },
    },
  },
});
