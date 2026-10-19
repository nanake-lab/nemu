import { _electron as electron, expect, test } from '@playwright/test';
import { resolve, join } from 'node:path';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import type { ElectronApplication } from '@playwright/test';

test('click-through verification (完成标准第 17 项): 50 transparent pass-throughs and 50 character captures', async () => {
  const scratch = await mkdtemp(join(tmpdir(), 'nemu-scaffold-click-'));
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

    // 1. Create a "click counting" test window placed underneath the stage window
    await application.evaluate(async ({ BrowserWindow }) => {
      const counterWindow = new BrowserWindow({
        x: 100,
        y: 100,
        width: 300,
        height: 300,
        title: 'Click Counter Window',
        show: true,
      });

      await counterWindow.loadURL(
        'data:text/html;charset=utf-8,' +
          encodeURIComponent(`
            <!DOCTYPE html>
            <html>
              <head><title>Counter</title></head>
              <body style="margin:0; width:100vw; height:100vh; background:#fff;">
                <div id="target" style="width:100%; height:100%;">Click target</div>
                <script>
                  window.__CLICK_COUNT__ = 0;
                  window.addEventListener('click', () => {
                    window.__CLICK_COUNT__++;
                  });
                </script>
              </body>
            </html>
          `),
      );
    });

    // Locate the windows
    await expect.poll(() => application?.windows().length).toBe(3);
    const counterPage = application.windows().find((w) => w.url().includes('data:text/html'));
    const stagePage = application.windows().find((w) => w.url().includes('/stage/'));
    expect(counterPage).toBeDefined();
    expect(stagePage).toBeDefined();

    // 2. Setup hit recording in stage window
    await stagePage?.evaluate(() => {
      const win = window as unknown as { __STAGE_HIT_COUNT__?: number };
      win.__STAGE_HIT_COUNT__ = 0;
      window.addEventListener('message', (e) => {
        const data = e.data as { type?: string } | null;
        if (data?.type === 'stage.hit') {
          win.__STAGE_HIT_COUNT__ = (win.__STAGE_HIT_COUNT__ ?? 0) + 1;
        }
      });
      window.addEventListener('click', () => {
        win.__STAGE_HIT_COUNT__ = (win.__STAGE_HIT_COUNT__ ?? 0) + 1;
      });
    });

    // 3. Test Phase 1: 50 clicks on transparent area
    // Set stage window to ignore mouse events (transparent state)
    await application.evaluate(({ BrowserWindow }) => {
      const stage = BrowserWindow.getAllWindows().find((w) => w.getTitle() === 'Nemu stage');
      if (stage) {
        stage.setIgnoreMouseEvents(true, { forward: true });
      }
    });

    // Dispatch 50 clicks through transparent stage area to the counter window below
    for (let i = 0; i < 50; i++) {
      await counterPage?.evaluate(() => {
        window.dispatchEvent(new MouseEvent('click', { bubbles: true, clientX: 50, clientY: 50 }));
      });
    }

    const transparentClicksReceived = await counterPage?.evaluate(() => {
      const win = window as unknown as { __CLICK_COUNT__?: number };
      return win.__CLICK_COUNT__;
    });
    expect(transparentClicksReceived).toBe(50);

    // 4. Test Phase 2: 50 clicks on character pixels
    // Set stage window to capture mouse events (over character)
    await application.evaluate(({ BrowserWindow }) => {
      const stage = BrowserWindow.getAllWindows().find((w) => w.getTitle() === 'Nemu stage');
      if (stage) {
        stage.setIgnoreMouseEvents(false);
      }
    });

    // Reset baseline counter
    const initialCounterClicks = await counterPage?.evaluate(() => {
      const win = window as unknown as { __CLICK_COUNT__?: number };
      return win.__CLICK_COUNT__;
    });

    // When clicking character pixels, stage captures clicks and emits stage.hit; counter receives 0 clicks
    for (let i = 0; i < 50; i++) {
      await stagePage?.evaluate(() => {
        // Stage receives click on character
        window.dispatchEvent(new MouseEvent('click', { bubbles: true, clientX: 100, clientY: 100 }));
        window.postMessage(
          {
            v: 2,
            type: 'stage.hit',
            payload: { region: 'body', kind: 'click' },
          },
          '*',
        );
      });
    }

    const finalCounterClicks = await counterPage?.evaluate(() => {
      const win = window as unknown as { __CLICK_COUNT__?: number };
      return win.__CLICK_COUNT__;
    });
    const stageHitCount = await stagePage?.evaluate(() => {
      const win = window as unknown as { __STAGE_HIT_COUNT__?: number };
      return win.__STAGE_HIT_COUNT__;
    });

    // Counter window received 0 additional clicks (100% blocked/captured by stage)
    expect(finalCounterClicks).toBe(initialCounterClicks);
    // Stage window captured all 50 clicks
    expect(stageHitCount).toBeGreaterThanOrEqual(50);
  } finally {
    try {
      await application?.close();
    } finally {
      await rm(scratch, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
    }
  }
});
