import { chromium } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const outputDir = 'reports/P1-01/screenshots';
await mkdir(outputDir, { recursive: true });

const server = await createServer({
  root: resolve('apps/stage'),
  configFile: false,
  server: { host: '127.0.0.1', port: 5174, strictPort: true },
});
await server.listen();

console.log('Capturing real GPU stage screenshots for report...');
const browser = await chromium.launch({
  headless: false,
  args: ['--enable-gpu-rasterization', '--enable-zero-copy', '--ignore-gpu-blocklist'],
});

try {
  const page = await browser.newPage({
    viewport: { width: 420, height: 600 },
    deviceScaleFactor: 1,
  });

  const views = [
    {
      name: 'fullBody_neutral_transparent',
      url: 'http://127.0.0.1:5174/?framing=fullBody&expr=neutral',
    },
    {
      name: 'fullBody_happy_white',
      url: 'http://127.0.0.1:5174/?bg=white&framing=fullBody&expr=happy',
    },
    {
      name: 'upperBody_surprised_black',
      url: 'http://127.0.0.1:5174/?bg=black&framing=upperBody&expr=surprised',
    },
    {
      name: 'upperBody_happy_gradient',
      url: 'http://127.0.0.1:5174/?bg=gradient&framing=upperBody&expr=happy',
    },
  ];

  for (const view of views) {
    await page.goto(view.url);
    await page.waitForFunction(() => window.__STAGE_READY__ === true, null, { timeout: 15000 });
    await page.waitForTimeout(500);

    const screenshot = await page.screenshot();
    const dest = resolve(outputDir, `${view.name}.png`);
    await writeFile(dest, screenshot);
    console.log(`Saved screenshot: ${dest}`);
  }
} finally {
  await browser.close();
  await server.close();
}
