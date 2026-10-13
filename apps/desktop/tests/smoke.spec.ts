import { _electron as electron, expect, test } from '@playwright/test';
import { resolve, join } from 'node:path';
import { mkdir, mkdtemp, rm, access } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { PNG } from 'pngjs';
test('two visible isolated windows; transparent stage; no core storage', async () => {
  const scratch = await mkdtemp(join(tmpdir(), 'nemu-scaffold-'));
  const application = await electron.launch({
    args: [
      resolve('apps/desktop'),
      '--use-gl=angle',
      '--use-angle=swiftshader',
      '--enable-unsafe-swiftshader',
    ],
    env: { ...process.env, NEMU_SCAFFOLD_USER_DATA: scratch },
  });
  try {
    await expect.poll(() => application.windows().length).toBe(2);
    const stage = application.windows().find((page) => page.url().includes('/stage/'));
    const panel = application.windows().find((page) => page.url().includes('/panel/'));
    expect(stage).toBeDefined();
    expect(panel).toBeDefined();
    if (!stage || !panel) throw new Error('Missing windows');
    await expect(stage.getByRole('img', { name: 'Stage placeholder' })).toBeVisible();
    await expect(panel.getByRole('heading', { name: 'Nemu' })).toBeVisible();
    await panel.getByLabel('Message').fill('Draft only');
    await expect(panel.getByLabel('Message')).toHaveValue('Draft only');
    await expect(panel.getByRole('button', { name: 'Send' })).toBeDisabled();
    const windows = await application.evaluate(({ BrowserWindow, app }) => ({
      dataPath: app.getPath('userData'),
      windows: BrowserWindow.getAllWindows().map((window) => ({
        title: window.getTitle(),
        visible: window.isVisible(),
        background: window.getBackgroundColor(),
      })),
    }));
    expect(windows.dataPath).toContain('nemu-scaffold-');
    for (const window of windows.windows) {
      expect(window.visible).toBe(true);
    }
    // getBackgroundColor returns RGB; capture native window pixels to test alpha.
    await stage.bringToFront();
    await stage.evaluate(
      () =>
        new Promise<void>((done) =>
          requestAnimationFrame(() => requestAnimationFrame(() => done())),
        ),
    );
    const capture = await application.evaluate(async ({ BrowserWindow }) => {
      const window = BrowserWindow.getAllWindows().find((item) => item.getTitle() === 'Nemu stage');
      if (!window) throw new Error('Stage missing');
      return (await window.webContents.capturePage()).toPNG().toString('base64');
    });
    const nativePixels = PNG.sync.read(Buffer.from(capture, 'base64'));
    expect(nativePixels.data[3]).toBe(0);
    expect(await stage.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe(
      'rgba(0, 0, 0, 0)',
    );
    expect(await stage.evaluate(() => typeof Reflect.get(window, 'require'))).toBe('undefined');
    await mkdir('reports/P0-01/visual', { recursive: true });
    await stage.screenshot({
      path: 'reports/P0-01/visual/electron-stage.png',
      omitBackground: true,
    });
    await panel.screenshot({ path: 'reports/P0-01/visual/electron-panel.png' });
  } finally {
    await application.close();
    await rm(scratch, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
    await expect(access(scratch)).rejects.toThrow();
  }
});
