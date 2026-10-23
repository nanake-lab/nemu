import { createServer } from 'vite';
import { chromium } from '@playwright/test';
import { createRequire } from 'node:module';
import { resolve, dirname } from 'node:path';
import { mkdir, writeFile } from 'node:fs/promises';
import { PNG } from 'pngjs';
import { execFileSync } from 'node:child_process';
export function assertQuiet() {
  execFileSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-Command',
      '$busy=@(Get-Process blender,electron,Runner.Worker -ErrorAction SilentlyContinue);if($busy.Count){$busy|Select-Object ProcessName,Id|ConvertTo-Json -Compress;exit 1};exit 0',
    ],
    { encoding: 'utf8' },
  );
}
const req = createRequire(resolve('apps/stage/package.json'));
const three = resolve(dirname(req.resolve('three')), '..');
const vrm = resolve(dirname(req.resolve('@pixiv/three-vrm')), 'three-vrm.module.js');
const aliases = [
  { find: 'three/addons', replacement: three + '/examples/jsm' },
  { find: 'three/examples', replacement: three + '/examples' },
  { find: 'three', replacement: three + '/build/three.module.js' },
  { find: '@pixiv/three-vrm', replacement: vrm },
  ...['avatar-runtime', 'body', 'contracts', 'log'].map((n) => ({
    find: '@nemu/' + n,
    replacement: resolve('packages', n, 'src/index.ts'),
  })),
];
export async function serverAndBrowser() {
  assertQuiet();
  const server = await createServer({
    root: process.cwd(),
    cacheDir: 'D:/Dev/cache/P2-06/vite',
    configFile: false,
    optimizeDeps: { noDiscovery: true, include: [] },
    resolve: { alias: aliases },
    server: { host: '127.0.0.1', port: 5188, strictPort: true },
  });
  await server.listen();
  const browser = await chromium.launch({
    args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
  });
  return { server, browser };
}
export function diff(a, b) {
  const x = PNG.sync.read(a),
    y = PNG.sync.read(b);
  if (x.width !== y.width || x.height !== y.height) throw Error('Image dimensions differ');
  let pixels = 0,
    max = 0;
  for (let i = 0; i < x.data.length; i += 4) {
    let different = false;
    for (let c = 0; c < 4; c++) {
      const d = Math.abs(x.data[i + c] - y.data[i + c]);
      if (d) different = true;
      max = Math.max(max, d);
    }
    if (different) pixels++;
  }
  return { differentPixels: pixels, maxChannelDifference: max };
}
const output = resolve('reports/P2-06/skin-audit');
export async function audit() {
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
    const cases = [
      { id: 'legacy-no-tone', light: 'legacy', tone: 'none' },
      { id: 'legacy-linear', light: 'legacy', tone: 'linear' },
      { id: 'stage-standard', light: 'stage' },
      { id: 'review-standard', light: 'review' },
      { id: 'stage-source', model: 'sourceModel', light: 'stage' },
      { id: 'stage-source-no-matcap', model: 'sourceModel', light: 'stage', mode: 'noMatcap' },
      { id: 'stage-standard-no-matcap', light: 'stage', mode: 'noMatcap' },
      { id: 'source-base-unlit', model: 'sourceModel', light: 'stage', mode: 'unlit' },
    ];
    const images = new Map(),
      reports = [];
    for (const c of cases) {
      const config = { background: 'white', framing: 'face', tone: 'linear', ...c };
      const report = await page.evaluate((c) => window.skinHarness.show(c), config);
      await page.locator('#label').evaluate((e) => (e.style.visibility = 'hidden'));
      const canvas = page.locator('canvas');
      const first = await canvas.screenshot();
      await page.evaluate(() => window.skinHarness.render());
      const second = await canvas.screenshot();
      if (!first.equals(second)) throw Error('Non-deterministic ' + c.id);
      await writeFile(resolve(output, c.id + '.png'), first);
      images.set(c.id, first);
      reports.push({ id: c.id, config, report });
      console.log(c.id);
    }
    await page.reload();
    await page.waitForFunction(() => window.__SKIN_READY__);
    await page.evaluate(() => window.skinHarness.show());
    const swatches = await page.evaluate(() => window.skinHarness.swatches());
    if (swatches.some((s) => s.actual.some((c, i) => Math.abs(c - s.expected[i]) > 1)))
      throw Error('Color swatch conversion failed');
    const comparisons = {
      toneMappingSameLight: diff(images.get('legacy-no-tone'), images.get('legacy-linear')),
      legacyVsStage: diff(images.get('legacy-linear'), images.get('stage-standard')),
      stageVsReview: diff(images.get('stage-standard'), images.get('review-standard')),
    };
    // Read the real stage/review entry points, not only this diagnostic harness.
    const apps = [];
    await page.route('**/sample.vrm', (route) =>
      route.continue({
        url: 'http://127.0.0.1:5188/content/_candidates/P2-06/models/standard.vrm',
      }),
    );
    await page.route('**/sample.profile.json', (route) =>
      route.continue({
        url: 'http://127.0.0.1:5188/content/_candidates/P2-06/models/standard.profile.json',
      }),
    );
    for (const kind of ['stage', 'review']) {
      const url =
        kind === 'stage'
          ? 'http://127.0.0.1:5188/apps/stage/index.html?deterministic=1&model=/content/_candidates/P2-06/models/standard.vrm&profile=/content/_candidates/P2-06/models/standard.profile.json&framing=upperBody&bg=white'
          : 'http://127.0.0.1:5188/apps/stage/review.html';
      await page.goto(url);
      await page.waitForFunction(
        (k) => (k === 'stage' ? window.__STAGE_READY__ : window.__REVIEW_READY__),
        kind,
        { timeout: 60000 },
      );
      const runtime = await page.evaluate(async (k) => {
        const app = k === 'stage' ? window.__stageApp : window.__reviewApp;
        if (k === 'review') {
          app.stop();
          await app.loadModel(
            '/content/_candidates/P2-06/models/standard.vrm',
            '/content/_candidates/P2-06/models/standard.profile.json',
          );
          app.stop();
        }
        const r = app.renderer.renderer;
        const materials = [];
        app.scene.traverse((o) => {
          if (o.isMesh)
            for (const m of Array.isArray(o.material) ? o.material : [o.material])
              if (m.name.includes('_SKIN') && !materials.some((x) => x.name === m.name))
                materials.push({
                  name: m.name,
                  toneMapped: m.toneMapped,
                  map: m.map?.colorSpace,
                  shade: m.shadeMultiplyTexture?.colorSpace,
                  normal: m.normalMap?.colorSpace,
                  matcap: m.matcapTexture?.colorSpace,
                  emissive: m.emissiveMap?.colorSpace,
                });
        });
        return {
          toneMapping: r.toneMapping,
          exposure: r.toneMappingExposure,
          outputColorSpace: r.outputColorSpace,
          lights: app.lighting.getConfig(),
          materials,
        };
      }, kind);
      apps.push({ kind, runtime });
    }
    assertQuiet();
    if (errors.length) throw Error(errors.join('\n'));
    await writeFile(
      resolve(output, 'runtime-audit.json'),
      JSON.stringify(
        {
          versions: { three: '0.174.0', threeVrm: '3.5.5' },
          renderer: 'SwiftShader; direct defaultMSAA',
          cases: reports,
          swatches,
          comparisons,
          actualApps: apps,
          errors,
        },
        null,
        2,
      ),
    );
    console.log(JSON.stringify({ swatches, comparisons, actualApps: apps }, null, 2));
  } finally {
    await browser.close();
    await server.close();
  }
}
