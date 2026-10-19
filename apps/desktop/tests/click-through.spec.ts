import { _electron as electron, expect, test } from '@playwright/test';
import { resolve, join } from 'node:path';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import type { ElectronApplication } from '@playwright/test';
import type { AppBus } from '../src/bus/index.js';
import type { StageWindowManager } from '../src/main/stage-window.js';

test('click-through verification (完成标准第 17 项): 50 transparent pass-throughs and 50 character captures', async () => {
  test.skip(process.platform !== 'win32', 'Click-through via Win32 SendInput and low-level mouse hooks is Windows-specific');

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

      const transparentDIP = { x: originX + 50, y: originY + 50 };
      const characterDIP = { x: originX + 275, y: originY + 275 };

      const transparentPhys = typeof (screen as unknown as { dipToScreenPoint?: (p: { x: number; y: number }) => { x: number; y: number } }).dipToScreenPoint === 'function'
        ? (screen as unknown as { dipToScreenPoint: (p: { x: number; y: number }) => { x: number; y: number } }).dipToScreenPoint(transparentDIP)
        : { x: Math.round(transparentDIP.x * scaleFactor), y: Math.round(transparentDIP.y * scaleFactor) };

      const characterPhys = typeof (screen as unknown as { dipToScreenPoint?: (p: { x: number; y: number }) => { x: number; y: number } }).dipToScreenPoint === 'function'
        ? (screen as unknown as { dipToScreenPoint: (p: { x: number; y: number }) => { x: number; y: number } }).dipToScreenPoint(characterDIP)
        : { x: Math.round(characterDIP.x * scaleFactor), y: Math.round(characterDIP.y * scaleFactor) };

      const stageHwnd = stage.getNativeWindowHandle();
      const stageHwndHex = '0x' + (stageHwnd.length >= 8 ? stageHwnd.readBigInt64LE() : BigInt(stageHwnd.readInt32LE())).toString(16);

      const counterHwnd = counterWindow.getNativeWindowHandle();
      const counterHwndHex = '0x' + (counterHwnd.length >= 8 ? counterHwnd.readBigInt64LE() : BigInt(counterHwnd.readInt32LE())).toString(16);

      console.log(`[Stage HWND]: ${stageHwndHex}, bounds:`, stage.getBounds());
      console.log(`[Counter HWND]: ${counterHwndHex}, bounds:`, counterWindow.getBounds());

      return {
        originX,
        originY,
        scaleFactor,
        transparentPhys,
        characterPhys,
        stageHwndHex,
      };
    });

    await expect.poll(() => application?.windows().length).toBe(3);
    const counterPage = application.windows().find((w) => w.url().includes('data:text/html'));
    const stagePage = application.windows().find((w) => w.url().includes('/stage/'));
    expect(counterPage).toBeDefined();
    expect(stagePage).toBeDefined();

    stagePage?.on('console', (msg) => {
      // Ignore resource 404s from unbuilt mock assets in test environment
      if (!msg.text().includes('Failed to load resource') && !msg.text().includes('Failed to initialize stage avatar')) {
        // debug logging if needed
      }
    });

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

    function sendNativeMouseMove(physX: number, physY: number): void {
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
            '-MoveOnly',
            '-StageHwndHex',
            layout.stageHwndHex,
          ],
          { encoding: 'utf-8' },
        );
      }
    }

    function sendNativeClicks(physX: number, physY: number, count: number): void {
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
            '-StageHwndHex',
            layout.stageHwndHex,
          ],
          { encoding: 'utf-8' },
        );
      }
    }

    // 3. Phase 1: 50 clicks on transparent area
    // Move cursor natively via Win32 SendInput to transparent area
    sendNativeMouseMove(layout.transparentPhys.x, layout.transparentPhys.y);

    // Wait for stageWindowManager to confirm click-through state (isIgnored = true)
    // triggered genuinely via WH_MOUSE_LL hook forwarding cursor move to Chromium
    await expect
      .poll(async () => {
        return await application?.evaluate(({ app }) => {
          const stageMgr = (app as unknown as { __nemu_stage_window__?: StageWindowManager }).__nemu_stage_window__;
          return (stageMgr as unknown as { isIgnored?: boolean })?.isIgnored;
        });
      })
      .toBe(true);

    // Send 50 OS mouse clicks through transparent stage area to the counter window below
    sendNativeClicks(layout.transparentPhys.x, layout.transparentPhys.y, 50);

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
    // Move cursor natively via Win32 SendInput onto the character hit box (200..350)
    sendNativeMouseMove(layout.characterPhys.x, layout.characterPhys.y);

    // Wait for stageWindowManager to confirm character capture mode (isIgnored = false)
    // triggered genuinely via WH_MOUSE_LL hook forwarding cursor move to Chromium
    await expect
      .poll(async () => {
        return await application?.evaluate(({ app }) => {
          const stageMgr = (app as unknown as { __nemu_stage_window__?: StageWindowManager }).__nemu_stage_window__;
          return (stageMgr as unknown as { isIgnored?: boolean })?.isIgnored;
        });
      })
      .toBe(false);

    // Send 50 OS mouse clicks on the character
    sendNativeClicks(layout.characterPhys.x, layout.characterPhys.y, 50);

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
