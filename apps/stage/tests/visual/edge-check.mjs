import { chromium } from '@playwright/test';
import { createServer } from 'vite';
import { PNG } from 'pngjs';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const outputDir = 'reports/P1-01/edge-check';
await mkdir(outputDir, { recursive: true });

const server = await createServer({
  root: resolve('apps/stage'),
  configFile: false,
  server: { host: '127.0.0.1', port: 5174, strictPort: true },
});
await server.listen();

const browser = await chromium.launch({
  args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
});

async function capture(page, url) {
  await page.goto(url);
  await page.waitForFunction(() => window.__STAGE_READY__ === true, null, { timeout: 15000 });
  await page.waitForTimeout(100);
  const buf = await page.screenshot({ omitBackground: true });
  return PNG.sync.read(buf);
}

try {
  const page = await browser.newPage({
    viewport: { width: 420, height: 600 },
    deviceScaleFactor: 1,
  });

  console.log('Capturing stage renders for edge check...');

  // 1. Transparent render to extract character alpha mask
  const transparentPng = await capture(
    page,
    'http://127.0.0.1:5174/?deterministic=1&framing=upperBody',
  );

  // 2. White background render
  const whitePng = await capture(
    page,
    'http://127.0.0.1:5174/?deterministic=1&bg=white&framing=upperBody',
  );

  // 3. Black background render
  const blackPng = await capture(
    page,
    'http://127.0.0.1:5174/?deterministic=1&bg=black&framing=upperBody',
  );

  // 4. Gradient background render
  const gradientPng = await capture(
    page,
    'http://127.0.0.1:5174/?deterministic=1&bg=gradient&framing=upperBody',
  );

  // 5. Clean gradient background without character for exact reference
  await page.evaluate(() => {
    if (window.__stageApp?.scene) {
      // Hide character temporarily
      window.__stageApp.scene.traverse((obj) => {
        if (obj.isMesh) obj.visible = false;
      });
      window.__stageApp.scheduler.step(16.67);
    }
  });
  const gradientRefBuffer = await page.screenshot();
  const gradientRefPng = PNG.sync.read(gradientRefBuffer);

  const { width, height } = transparentPng;

  // Build character silhouette mask (pixels with non-zero alpha)
  const isCharacter = new Uint8Array(width * height);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const idx = (y * width + x) * 4;
      const alpha = transparentPng.data[idx + 3];
      if (alpha > 5) {
        isCharacter[y * width + x] = 1;
      }
    }
  }

  // Find 2 px outer ring (background pixels within 2 px Euclidean distance of character)
  const isOuterRing = new Uint8Array(width * height);
  let ringPixelCount = 0;
  const maxDistSq = 4; // distance <= 2px => dist^2 <= 4

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const pIdx = y * width + x;
      if (isCharacter[pIdx] === 1) continue; // Must be background

      let nearChar = false;
      for (let dy = -2; dy <= 2 && !nearChar; dy++) {
        for (let dx = -2; dx <= 2 && !nearChar; dx++) {
          if (dx * dx + dy * dy > maxDistSq) continue;
          const nx = x + dx;
          const ny = y + dy;
          if (nx >= 0 && nx < width && ny >= 0 && ny < height) {
            if (isCharacter[ny * width + nx] === 1) {
              nearChar = true;
            }
          }
        }
      }

      if (nearChar) {
        isOuterRing[pIdx] = 1;
        ringPixelCount++;
      }
    }
  }

  console.log(`Identified ${ringPixelCount} pixels in the 2px outer silhouette ring.`);

  // Calculate background color deviations in the 2px ring
  let maxWhiteDelta = 0;
  let maxBlackDelta = 0;
  let maxGradientDelta = 0;

  // Visualization PNG of the outer ring and errors
  const ringVisPng = new PNG({ width, height });

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const pIdx = y * width + x;
      const idx = pIdx * 4;

      if (isOuterRing[pIdx] === 1) {
        // White background delta: expected (255, 255, 255)
        const wr = whitePng.data[idx];
        const wg = whitePng.data[idx + 1];
        const wb = whitePng.data[idx + 2];
        const dWhite = Math.max(Math.abs(wr - 255), Math.abs(wg - 255), Math.abs(wb - 255));
        if (dWhite > maxWhiteDelta) maxWhiteDelta = dWhite;

        // Black background delta: expected (0, 0, 0)
        const br = blackPng.data[idx];
        const bg = blackPng.data[idx + 1];
        const bb = blackPng.data[idx + 2];
        const dBlack = Math.max(br, bg, bb);
        if (dBlack > maxBlackDelta) maxBlackDelta = dBlack;

        // Gradient background delta: compared against gradientRef
        const gr = gradientPng.data[idx];
        const gg = gradientPng.data[idx + 1];
        const gb = gradientPng.data[idx + 2];
        const refr = gradientRefPng.data[idx];
        const refg = gradientRefPng.data[idx + 1];
        const refb = gradientRefPng.data[idx + 2];
        const dGrad = Math.max(Math.abs(gr - refr), Math.abs(gg - refg), Math.abs(gb - refb));
        if (dGrad > maxGradientDelta) maxGradientDelta = dGrad;

        // Visual highlight: green for outer ring
        ringVisPng.data[idx] = 0;
        ringVisPng.data[idx + 1] = 255;
        ringVisPng.data[idx + 2] = 0;
        ringVisPng.data[idx + 3] = 255;
      } else if (isCharacter[pIdx] === 1) {
        // Blue for character interior
        ringVisPng.data[idx] = 100;
        ringVisPng.data[idx + 1] = 150;
        ringVisPng.data[idx + 2] = 255;
        ringVisPng.data[idx + 3] = 180;
      } else {
        // Transparent
        ringVisPng.data[idx + 3] = 0;
      }
    }
  }

  const overallMaxDelta = Math.max(maxWhiteDelta, maxBlackDelta, maxGradientDelta);
  const maxAllowed = 8; // <= 8 / 255
  const passed = overallMaxDelta <= maxAllowed;

  const report = {
    ringPixelCount,
    maxWhiteDelta: `${maxWhiteDelta} / 255`,
    maxBlackDelta: `${maxBlackDelta} / 255`,
    maxGradientDelta: `${maxGradientDelta} / 255`,
    overallMaxDelta: `${overallMaxDelta} / 255`,
    threshold: `${maxAllowed} / 255`,
    passed,
  };

  await writeFile(resolve(outputDir, 'edge_ring_vis.png'), PNG.sync.write(ringVisPng));
  await writeFile(resolve(outputDir, 'edge_check_results.json'), JSON.stringify(report, null, 2));

  console.log('\n=== Edge Check Results ===');
  console.log(`Max White Delta:    ${maxWhiteDelta} / 255`);
  console.log(`Max Black Delta:    ${maxBlackDelta} / 255`);
  console.log(`Max Gradient Delta: ${maxGradientDelta} / 255`);
  console.log(`Overall Max Delta:  ${overallMaxDelta} / 255 (Limit <= ${maxAllowed}/255)`);
  console.log(`Status:             ${passed ? 'PASSED' : 'FAILED'}\n`);

  if (!passed) {
    throw new Error(
      `Edge check failed: overall max delta ${overallMaxDelta} exceeds allowed limit ${maxAllowed}/255`,
    );
  }
} finally {
  await browser.close();
  await server.close();
}
