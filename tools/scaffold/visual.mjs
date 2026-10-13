import { chromium } from '@playwright/test';
import { createServer } from 'vite';
import { PNG } from 'pngjs';
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const candidates = process.argv.includes('--update-candidates');
const directory = 'reports/P0-01/visual';
await mkdir(directory, { recursive: true });
const server = await createServer({
  root: resolve('apps/stage'),
  configFile: false,
  server: { host: '127.0.0.1', port: 5174, strictPort: true },
});
await server.listen();
const browser = await chromium.launch({
  args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
});
const results = [];
try {
  const page = await browser.newPage({
    viewport: { width: 420, height: 600 },
    deviceScaleFactor: 1,
  });
  for (const bg of ['white', 'black']) {
    const file = `deterministic-1_bg-${bg}_framing-fullBody.png`;
    await page.goto(`http://127.0.0.1:5174/?deterministic=1&bg=${bg}&framing=fullBody`);
    await page.getByRole('img', { name: 'Stage placeholder' }).waitFor();
    const first = await page.screenshot();
    const second = await page.screenshot();
    assert.deepEqual(second, first, 'Scaffold frame must be deterministic');
    const baseline = resolve('apps/stage/tests/visual/__golden__', file);
    if (candidates) {
      await writeFile(resolve(directory, file), first);
      results.push({ file, status: 'candidate', deterministic: true });
    } else {
      const expected = PNG.sync.read(await readFile(baseline));
      const actual = PNG.sync.read(first);
      assert.equal(actual.width, expected.width);
      assert.equal(actual.height, expected.height);
      assert.deepEqual(actual.data, expected.data, file);
      results.push({ file, status: 'passed' });
    }
  }
  await writeFile(
    resolve(directory, candidates ? 'candidates.json' : 'comparison.json'),
    JSON.stringify(results, null, 2),
  );
} finally {
  await browser.close();
  await server.close();
}
