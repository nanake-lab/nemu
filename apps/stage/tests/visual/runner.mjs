import { chromium } from '@playwright/test';
import { createServer } from 'vite';
import { PNG } from 'pngjs';
import pixelmatch from 'pixelmatch';
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';

const candidatesMode = process.argv.includes('--update-candidates');
const updateGoldenMode = process.argv.includes('--update-golden');
const directory = 'reports/P1-01/visual';
const goldenDir = resolve('apps/stage/tests/visual/__golden__');

await mkdir(directory, { recursive: true });
await mkdir(goldenDir, { recursive: true });

const server = await createServer({
  root: resolve('apps/stage'),
  configFile: false,
  server: { host: '127.0.0.1', port: 5174, strictPort: true },
});
await server.listen();

const browser = await chromium.launch({
  args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
});

const backgrounds = ['white', 'black', 'gradient'];
const framings = ['fullBody', 'upperBody'];
const expressions = ['neutral', 'happy', 'surprised'];

const results = [];
let totalFailures = 0;

try {
  const page = await browser.newPage({
    viewport: { width: 420, height: 600 },
    deviceScaleFactor: 1,
  });

  for (const bg of backgrounds) {
    for (const framing of framings) {
      for (const expr of expressions) {
        const file = `deterministic-1_bg-${bg}_framing-${framing}_expr-${expr}.png`;
        const url = `http://127.0.0.1:5174/?deterministic=1&bg=${bg}&framing=${framing}&expr=${expr}`;

        await page.goto(url);
        // Wait for stage to report ready
        await page.waitForFunction(() => window.__STAGE_READY__ === true, null, { timeout: 15000 });
        // Allow WebGL buffer swap
        await page.waitForTimeout(100);

        const firstScreenshot = await page.screenshot();
        const secondScreenshot = await page.screenshot();

        // 1. Verify frame determinism
        assert.deepEqual(
          secondScreenshot,
          firstScreenshot,
          `Frame must be deterministic for ${file}`,
        );

        const baselinePath = resolve(goldenDir, file);

        if (candidatesMode) {
          await writeFile(resolve(directory, file), firstScreenshot);
          results.push({ file, status: 'candidate', deterministic: true });
          console.log(`[CANDIDATE] Saved candidate: ${file}`);
        } else if (updateGoldenMode || !existsSync(baselinePath)) {
          // Initialize or update baseline
          await writeFile(baselinePath, firstScreenshot);
          await writeFile(resolve(directory, file), firstScreenshot);
          results.push({ file, status: 'golden_initialized', deterministic: true });
          console.log(`[GOLDEN] Initialized golden baseline: ${file}`);
        } else {
          const expected = PNG.sync.read(await readFile(baselinePath));
          const actual = PNG.sync.read(firstScreenshot);

          assert.equal(actual.width, expected.width, `${file}: width mismatch`);
          assert.equal(actual.height, expected.height, `${file}: height mismatch`);

          const diff = new PNG({ width: actual.width, height: actual.height });
          // P1-01 acceptance criteria: pixelmatch threshold = 0.1, diff ratio <= 0.1%
          const diffPixels = pixelmatch(
            actual.data,
            expected.data,
            diff.data,
            actual.width,
            actual.height,
            { threshold: 0.1 },
          );

          const totalPixels = actual.width * actual.height;
          const diffRatio = diffPixels / totalPixels;

          const passed = diffRatio <= 0.001; // <= 0.1%
          if (!passed) {
            totalFailures++;
            const diffFile = resolve(directory, `diff_${file}`);
            await writeFile(diffFile, PNG.sync.write(diff));
            console.error(
              `[FAILED] ${file}: ${diffPixels} diff pixels (${(diffRatio * 100).toFixed(3)}% > 0.1%)`,
            );
          } else {
            console.log(
              `[PASSED] ${file}: ${diffPixels} diff pixels (${(diffRatio * 100).toFixed(4)}% <= 0.1%)`,
            );
          }

          results.push({
            file,
            status: passed ? 'passed' : 'failed',
            diffPixels,
            diffRatio,
            deterministic: true,
          });
        }
      }
    }
  }

  const resultFilePath = resolve(directory, candidatesMode ? 'candidates.json' : 'comparison.json');
  await writeFile(resultFilePath, JSON.stringify(results, null, 2));

  if (totalFailures > 0) {
    throw new Error(`Visual regression failed with ${totalFailures} mismatching cases.`);
  }

  console.log(`All ${results.length} visual regression cases completed successfully!`);
} finally {
  await browser.close();
  await server.close();
}
