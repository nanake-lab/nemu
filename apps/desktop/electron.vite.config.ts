import { workletSource } from '../../packages/voice/web/mic-worklet.js';
import { defineConfig, externalizeDepsPlugin } from 'electron-vite';
import { resolve } from 'node:path';

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin({ exclude: ['@nemu/contracts', '@nemu/voice', '@nemu/desktop-sense'] })],
    build: {
      rollupOptions: {
        external: ['sherpa-onnx-node', 'koffi'],
        input: {
          index: resolve(import.meta.dirname, 'src/main/index.ts'),
          voice: resolve(import.meta.dirname, 'src/voice/utility.ts'),
        },
      },
    },
  },
  preload: {
    plugins: [externalizeDepsPlugin({ exclude: ['@nemu/contracts', 'zod'] })],
    build: {
      rollupOptions: {
        input: {
          stage: resolve(import.meta.dirname, 'src/preload/stage.ts'),
          panel: resolve(import.meta.dirname, 'src/preload/panel.ts'),
        },
        output: { format: 'cjs', entryFileNames: '[name].cjs' },
      },
    },
  },
  renderer: {
    plugins: [
      {
        name: 'nemu-mic-worklet',
        configureServer(server) {
          server.middlewares.use('/stage/asr-mic-worklet.js', (_request, response) => {
            response.setHeader('Content-Type', 'text/javascript');
            response.end(workletSource());
          });
        },
        generateBundle() {
          this.emitFile({
            type: 'asset',
            fileName: 'stage/asr-mic-worklet.js',
            source: workletSource(),
          });
        },
      },
    ],
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
