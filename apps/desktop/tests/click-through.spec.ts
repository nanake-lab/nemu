import { _electron as electron, expect, test } from '@playwright/test';
import { resolve, join } from 'node:path';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import type { ElectronApplication } from '@playwright/test';
import type { AppBus } from '../src/bus/index.js';
import type { StageWindowManager } from '../src/main/stage-window.js';

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
    const layout = await application.evaluate(async ({ BrowserWindow, screen }) => {
      const primary = screen.getPrimaryDisplay();
      const scaleFactor = primary.scaleFactor || 1;

      // Position stage window at top-left of primary workArea
      const stage = BrowserWindow.getAllWindows().find((w) => w.getTitle() === 'Nemu stage');
      if (!stage) throw new Error('Stage window missing');

      const originX = primary.workArea.x + 100;
      const originY = primary.workArea.y + 100;

      stage.setBounds({
        x: originX,
        y: originY,
        width: 400,
        height: 400,
      });

      // Create counter window directly beneath stage window
      const counterWindow = new BrowserWindow({
        x: originX,
        y: originY,
        width: 400,
        height: 400,
        title: 'Click Counter Window',
        frame: false,
        show: true,
      });

      await counterWindow.loadURL(
        'data:text/html;charset=utf-8,' +
          encodeURIComponent(`
            <!DOCTYPE html>
            <html>
              <head><title>Counter</title></head>
              <body style="margin:0; width:100vw; height:100vh; background:#fff; user-select:none;">
                <div id="target" style="width:100%; height:100%; display:flex; align-items:center; justify-content:center; font-size:24px;">
                  Click Target
                </div>
                <script>
                  window.__EVENTS__ = [];
                  window.__CLICK_COUNT__ = 0;
                  for (const ev of ['mousedown', 'mouseup', 'click', 'pointerdown', 'pointerup']) {
                    window.addEventListener(ev, (e) => {
                      window.__EVENTS__.push(e.type);
                      if (ev === 'click') window.__CLICK_COUNT__++;
                    });
                  }
                </script>
              </body>
            </html>
          `),
      );

      counterWindow.show();

      // Re-assert stage is on top of counter window
      stage.setAlwaysOnTop(true, 'screen-saver');

      return {
        originX,
        originY,
        scaleFactor,
      };
    });

    await expect.poll(() => application?.windows().length).toBe(3);
    const counterPage = application.windows().find((w) => w.url().includes('data:text/html'));
    const stagePage = application.windows().find((w) => w.url().includes('/stage/'));
    expect(counterPage).toBeDefined();
    expect(stagePage).toBeDefined();

    // 2. Setup interactive character region and hit testing inside the stage page
    await stagePage?.evaluate(() => {
      // Inject character test element into stage document
      const char = document.createElement('div');
      char.id = 'nemu-character-hit-box';
      char.style.position = 'absolute';
      char.style.left = '200px';
      char.style.top = '200px';
      char.style.width = '150px';
      char.style.height = '150px';
      char.style.background = 'rgba(255, 0, 0, 0.8)';
      char.style.pointerEvents = 'auto';
      document.body.appendChild(char);

      // Perform real hit-testing on cursor movements over the stage page
      window.addEventListener('mousemove', (e) => {
        const target = document.elementFromPoint(e.clientX, e.clientY);
        const overChar = target === char || char.contains(target as Node);
        window.postMessage({ type: 'stage.hitTest', payload: { overCharacter: overChar } }, '*');
      });

      // Emits stage.hit when clicked
      char.addEventListener('click', () => {
        window.postMessage(
          {
            v: 2,
            type: 'stage.hit',
            payload: { region: 'body', kind: 'click' },
          },
          '*',
        );
      });
    });

    // Track stage.hit events on main AppBus
    await application.evaluate(({ app }) => {
      const bus = (app as unknown as { __nemu_bus__?: AppBus }).__nemu_bus__;
      const appState = app as unknown as Record<string, unknown>;
      appState.__nemu_hit_events_count__ = 0;
      bus?.subscribe('stage.hit', () => {
        (appState.__nemu_hit_events_count__ as number)++;
      });
    });

    const clickScriptPath = resolve('apps/desktop/tests/helpers/click-mouse.ps1');

    function sendNativeClicks(dipX: number, dipY: number, count: number): void {
      const physX = Math.round(dipX * layout.scaleFactor);
      const physY = Math.round(dipY * layout.scaleFactor);

      if (process.platform === 'win32') {
        execFileSync(
          'pwsh',
          [
            '-NoProfile',
            '-File',
            clickScriptPath,
            '-X',
            String(physX),
            '-Y',
            String(physY),
            '-Count',
            String(count),
            '-DelayMs',
            '10',
          ],
          { encoding: 'utf-8' },
        );
      } else {
        // Fallback for non-Windows (e.g. Linux / xvfb with xdotool)
        try {
          execFileSync('xdotool', ['mousemove', String(physX), String(physY), 'click', '--repeat', String(count), '1']);
        } catch {
          // If xdotool is unavailable in non-win environment
        }
      }
    }

    // 3. Phase 1: 50 clicks on transparent area
    // Point (layout.originX + 50, layout.originY + 50) is empty transparent stage area
    // First trigger hitTest via mouse move to set stage into click-through mode
    await stagePage?.evaluate(() => {
      window.dispatchEvent(new MouseEvent('mousemove', { clientX: 50, clientY: 50, bubbles: true }));
      window.postMessage({ type: 'stage.hitTest', payload: { overCharacter: false } }, '*');
    });

    // Wait for stageWindowManager to confirm click-through state
    await expect
      .poll(async () => {
        return await application?.evaluate(({ app }) => {
          const stageMgr = (app as unknown as { __nemu_stage_window__?: StageWindowManager }).__nemu_stage_window__;
          return (stageMgr as unknown as { isIgnored?: boolean })?.isIgnored;
        });
      })
      .toBe(true);

    // Send 50 OS mouse clicks through transparent stage area to the counter window below
    sendNativeClicks(layout.originX + 50, layout.originY + 50, 50);

    // Assert counter window received all 50 clicks
    await expect
      .poll(async () => {
        return await counterPage?.evaluate(() => {
          const win = window as unknown as { __CLICK_COUNT__?: number };
          return win.__CLICK_COUNT__;
        });
      })
      .toBe(50);

    // Assert stage received 0 hits
    const initialHits = await application.evaluate(({ app }) => {
      return (app as unknown as Record<string, unknown>).__nemu_hit_events_count__ as number;
    });
    expect(initialHits).toBe(0);

    // 4. Phase 2: 50 clicks on character pixels
    // Point (layout.originX + 250, layout.originY + 250) is over the character hit box (at 200..350)
    await stagePage?.evaluate(() => {
      window.dispatchEvent(new MouseEvent('mousemove', { clientX: 250, clientY: 250, bubbles: true }));
      window.postMessage({ type: 'stage.hitTest', payload: { overCharacter: true } }, '*');
    });

    // Wait for stageWindowManager to confirm character capture mode (isIgnored = false)
    await expect
      .poll(async () => {
        return await application?.evaluate(({ app }) => {
          const stageMgr = (app as unknown as { __nemu_stage_window__?: StageWindowManager }).__nemu_stage_window__;
          return (stageMgr as unknown as { isIgnored?: boolean })?.isIgnored;
        });
      })
      .toBe(false);

    // Send 50 OS mouse clicks on the character
    sendNativeClicks(layout.originX + 250, layout.originY + 250, 50);

    // Assert counter window did NOT receive any additional clicks (100% blocked/captured by stage)
    const finalCounterClicks = await counterPage?.evaluate(() => {
      const win = window as unknown as { __CLICK_COUNT__?: number };
      return win.__CLICK_COUNT__;
    });
    expect(finalCounterClicks).toBe(50);

    // Assert stage captured all 50 clicks and published stage.hit to AppBus
    await expect
      .poll(async () => {
        return await application?.evaluate(({ app }) => {
          return (app as unknown as Record<string, unknown>).__nemu_hit_events_count__ as number;
        });
      })
      .toBe(50);
  } finally {
    try {
      await application?.close();
    } finally {
      await rm(scratch, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
    }
  }
});
