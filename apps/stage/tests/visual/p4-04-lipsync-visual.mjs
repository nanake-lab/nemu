import { chromium } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = resolve(__dirname, '../../../../');
const screenshotsDir = resolve(projectRoot, 'reports/P4-04/screenshots');

await mkdir(screenshotsDir, { recursive: true });

console.log('--- 1. Starting Stage Server for P4-04 Viseme Visual Evidence ---');
const server = await createServer({
  root: resolve(projectRoot, 'apps/stage'),
  configFile: resolve(projectRoot, 'apps/stage/vite.config.ts'),
  server: { host: '127.0.0.1', port: 5176, strictPort: true },
});
await server.listen();

const browser = await chromium.launch({
  headless: true,
  args: ['--enable-gpu-rasterization', '--enable-zero-copy', '--ignore-gpu-blocklist', '--use-gl=angle'],
});

const backgrounds = ['white', 'black'];
const visemes = [
  { name: 'aa', weight: 0.8 },
  { name: 'ih', weight: 0.8 },
  { name: 'ou', weight: 0.8 },
  { name: 'ee', weight: 0.8 },
  { name: 'oh', weight: 0.8 },
  { name: 'neutral', weight: 0.0 },
];

try {
  const page = await browser.newPage({
    viewport: { width: 500, height: 650 },
    deviceScaleFactor: 1,
  });

  for (const bg of backgrounds) {
    for (const v of visemes) {
      const url = `http://127.0.0.1:5176/?deterministic=1&bg=${bg}&framing=upperBody&expr=${v.name}:${v.weight}`;
      await page.goto(url);
      await page.waitForFunction(() => window.__STAGE_READY__ === true, null, { timeout: 20000 });
      await page.waitForTimeout(150);

      const filename = `bg-${bg}_framing-upperBody_viseme-${v.name}-${v.weight}.png`;
      const dest = resolve(screenshotsDir, filename);
      await page.screenshot({ path: dest });
      console.log(`Captured: reports/P4-04/screenshots/${filename}`);
    }
  }

  console.log('All P4-04 viseme visual captures completed successfully!');
} finally {
  await browser.close();
  await server.close();
}
