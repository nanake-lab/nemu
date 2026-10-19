import { _electron as electron, expect, test } from '@playwright/test';
import { resolve, join } from 'node:path';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import type { ElectronApplication } from '@playwright/test';

test('stage window shell properties and clean process termination', async () => {
  const scratch = await mkdtemp(join(tmpdir(), 'nemu-scaffold-shell-'));
  let application: ElectronApplication | undefined;

  try {
    application = await electron.launch({
      args: [
        resolve('apps/desktop'),
        '--use-gl=angle',
        '--use-angle=swiftshader',
        '--enable-unsafe-swiftshader',
      ],
      env: { ...process.env, NEMU_SCAFFOLD_USER_DATA: scratch },
    });

    await expect.poll(() => application?.windows().length).toBe(2);

    // Verify window shell properties
    const windowProps = await application.evaluate(({ BrowserWindow }) => {
      const windows = BrowserWindow.getAllWindows();
      const stage = windows.find((w) => w.getTitle() === 'Nemu stage');
      if (!stage) throw new Error('Stage window missing');

      return {
        isAlwaysOnTop: stage.isAlwaysOnTop(),
        isDestroyed: stage.isDestroyed(),
        isFocusable: stage.isFocusable(),
        bounds: stage.getBounds(),
        title: stage.getTitle(),
      };
    });

    expect(windowProps.isAlwaysOnTop).toBe(true);
    expect(windowProps.isFocusable).toBe(false);
    expect(windowProps.isDestroyed).toBe(false);
    expect(windowProps.isFocusable).toBe(false);

    // Get supervised process PIDs
    const supervisedPids = await application.evaluate(({ app }) => {
      const metrics = app.getAppMetrics();
      return metrics.map((m) => m.pid);
    });
    expect(supervisedPids.length).toBeGreaterThan(0);

    // Close application
    await application.close();
    application = undefined;

    // Verify all processes have terminated and no lingering processes remain
    await new Promise((r) => setTimeout(r, 1000));
    for (const pid of supervisedPids) {
      let isAlive = false;
      try {
        process.kill(pid, 0);
        isAlive = true;
      } catch {
        isAlive = false;
      }
      expect(isAlive).toBe(false);
    }
  } finally {
    try {
      await application?.close();
    } finally {
      await rm(scratch, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
    }
  }
});
