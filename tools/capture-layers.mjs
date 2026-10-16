import { chromium } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const outputDir = 'reports/P3-01/layers';
await mkdir(outputDir, { recursive: true });

const server = await createServer({
  root: resolve('apps/stage'),
  configFile: false,
  server: { host: '127.0.0.1', port: 5174, strictPort: true },
});
await server.listen();

console.log('Capturing animation layer screenshots for P3-01 report...');
const browser = await chromium.launch({
  args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
});

try {
  const page = await browser.newPage({
    viewport: { width: 420, height: 600 },
    deviceScaleFactor: 1,
  });

  const views = [
    {
      name: 'layer_none_rest_pose',
      url: 'http://127.0.0.1:5174/?deterministic=1&bg=gradient&framing=upperBody&layers=',
    },
    {
      name: 'layer_L2_procedural',
      url: 'http://127.0.0.1:5174/?deterministic=1&bg=gradient&framing=upperBody&layers=L2',
    },
    {
      name: 'layer_L3_face_happy',
      url: 'http://127.0.0.1:5174/?deterministic=1&bg=gradient&framing=upperBody&expr=happy&layers=L3',
    },
    {
      name: 'layer_L3_face_surprised',
      url: 'http://127.0.0.1:5174/?deterministic=1&bg=gradient&framing=upperBody&expr=surprised&layers=L3',
    },
    {
      name: 'layer_L4_secondary',
      url: 'http://127.0.0.1:5174/?deterministic=1&bg=gradient&framing=upperBody&layers=L4',
    },
    {
      name: 'layers_all_composite',
      url: 'http://127.0.0.1:5174/?deterministic=1&bg=gradient&framing=upperBody&expr=happy&layers=L0,L1,L2,L3,L4',
    },
  ];

  for (const view of views) {
    await page.goto(view.url);
    await page.waitForFunction(() => window.__STAGE_READY__ === true, null, { timeout: 15000 });
    await page.waitForTimeout(300);

    const screenshot = await page.screenshot();
    const dest = resolve(outputDir, `${view.name}.png`);
    await writeFile(dest, screenshot);
    console.log(`Saved layer screenshot: ${dest}`);
  }
} finally {
  await browser.close();
  await server.close();
}
