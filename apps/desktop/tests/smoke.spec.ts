import { _electron as electron, expect, test } from '@playwright/test';
import { resolve, join } from 'node:path';
import { mkdir, mkdtemp, rm, access } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { PNG } from 'pngjs';
import type { ElectronApplication } from '@playwright/test';
import { spawn } from 'node:child_process';
import { appendFileSync } from 'node:fs';
test('two visible isolated windows; transparent stage; no core storage', async () => {
  const scratch = await mkdtemp(join(tmpdir(), 'nemu-scaffold-'));
  let application: ElectronApplication | undefined;
  const diagnostic = (event: string, detail: Record<string, unknown> = {}) => {
    if (process.env.NEMU_EVIDENCE_DIR) {
      appendFileSync(
        join(process.env.NEMU_EVIDENCE_DIR, 'playwright-events.jsonl'),
        JSON.stringify({ at: new Date().toISOString(), event, ...detail }) + '\n',
      );
    }
  };
  try {
    diagnostic('launch-begin');
    application = await electron.launch({
      args: [
        resolve('apps/desktop'),
        '--use-gl=angle',
        '--use-angle=swiftshader',
        '--enable-unsafe-swiftshader',
      ],
      env: { ...process.env, NEMU_SCAFFOLD_USER_DATA: scratch },
    });
    const launched = application;
    diagnostic('launch-resolved', {
      pid: launched.process().pid,
      observedWindows: launched.windows().length,
    });
    launched.on('window', () =>
      diagnostic('window-observed', { observedWindows: launched.windows().length }),
    );
    launched.on('close', () => diagnostic('application-closed'));
    try {
      await expect.poll(() => launched.windows().length).toBe(2);
    } finally {
      diagnostic('window-wait-ended');
      if (process.env.NEMU_EVIDENCE_DIR && process.platform === 'win32') {
        // OS collection starts only after the original assertion settles, including failure.
        const child = spawn(
          'pwsh',
          [
            '-NoProfile',
            '-File',
            resolve('tools/scaffold/windows-evidence.ps1'),
            '-Mode',
            'Snapshot',
            '-Label',
            'electron-after-window-wait',
          ],
          { stdio: 'inherit', windowsHide: true, timeout: 10_000 },
        );
        await new Promise<void>((done) => {
          child.on('exit', (code, signal) => {
            diagnostic('snapshot-exit', { code, signal });
            done();
          });
          child.on('error', (error) => {
            diagnostic('snapshot-error', { message: error.message });
            done();
          });
        });
      }
    }
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
    const visualDirectory = join(process.env.NEMU_EVIDENCE_DIR ?? 'reports/P0-01', 'visual');
    await mkdir(visualDirectory, { recursive: true });
    await stage.screenshot({
      path: join(visualDirectory, 'electron-stage.png'),
      omitBackground: true,
    });
    await panel.screenshot({ path: join(visualDirectory, 'electron-panel.png') });
  } finally {
    try {
      await application?.close();
    } finally {
      await rm(scratch, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
      await expect(access(scratch)).rejects.toThrow();
    }
  }
});
