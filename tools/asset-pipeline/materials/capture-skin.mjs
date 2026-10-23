import { writeFile, mkdir, readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { serverAndBrowser, assertQuiet } from './audit-capture.mjs';
const output = resolve('reports/P2-06/skin-audit/visual');
await mkdir(output, { recursive: true });
const { server, browser } = await serverAndBrowser();
const errors = [];
try {
  const page = await browser.newPage({
    viewport: { width: 440, height: 500 },
    deviceScaleFactor: 1,
  });
  page.on('pageerror', (e) => errors.push(String(e)));
  await page.goto('http://127.0.0.1:5188/tools/asset-pipeline/materials/skin-harness.html');
  await page.waitForFunction(() => window.__SKIN_READY__);
  const results = [];
  for (const framing of ['face', 'upperBody'])
    for (const light of ['left', 'right'])
      for (const background of ['white', 'black', 'gradient'])
        for (const variant of ['control', 'A', 'B', 'C']) {
          const config = { variant, light, background, framing, tone: 'linear' };
          const params = new URLSearchParams({ deterministic: '1', ...config });
          const url = `http://127.0.0.1:5188/tools/asset-pipeline/materials/skin-harness.html?${params}`;
          const runtime = await page.evaluate(
            async ({ config, url }) => {
              history.replaceState(null, '', url);
              return await window.skinHarness.show(config);
            },
            { config, url },
          );
          const name = [...params].map(([k, v]) => `${k}-${v}`).join('_') + '.png';
          const first = await page.locator('canvas').screenshot();
          await page.evaluate(() => window.skinHarness.render());
          const second = await page.locator('canvas').screenshot();
          if (!first.equals(second)) throw Error('Non-deterministic ' + name);
          await writeFile(resolve(output, name), first);
          results.push({ file: name, config, url, deterministic: true, runtime });
          console.log(name);
        }
  assertQuiet();
  if (errors.length) throw Error(errors.join('\n'));
  const integrity = JSON.parse(
    await readFile('reports/P2-06/skin-audit/candidate-integrity.json', 'utf8'),
  );
  await writeFile(
    resolve(output, 'screenshots.json'),
    JSON.stringify(
      {
        renderer:
          'Shared production AvatarRenderer, fixed stage default lights mirrored left/right, SwiftShader, LinearToneMapping exposure=1',
        baseModelSha256: integrity[0].baseModelSha256,
        results,
        errors,
      },
      null,
      2,
    ),
  );
} finally {
  await browser.close();
  await server.close();
}
