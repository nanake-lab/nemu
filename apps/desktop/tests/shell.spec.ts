import { _electron as electron, expect, test } from '@playwright/test';
import { resolve, join } from 'node:path';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import type { ElectronApplication } from '@playwright/test';
import type { ProcessSupervisor } from '../src/supervisor/index.js';
import type { AppBus } from '../src/bus/index.js';
import type { StageWindowManager } from '../src/main/stage-window.js';
import type { PanelWindowManager } from '../src/main/panel-window.js';

test('stage window shell properties, channel routing, panel summoning and clean process termination', async () => {
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

    const stagePage = application.windows().find((w) => w.url().includes('/stage/'));
    const panelPage = application.windows().find((w) => w.url().includes('/panel/'));
    expect(stagePage).toBeDefined();
    expect(panelPage).toBeDefined();

    // 1. Verify stage window shell properties
    const stageProps = await application.evaluate(({ BrowserWindow }) => {
      const stage = BrowserWindow.getAllWindows().find((w) => w.getTitle() === 'Nemu stage');
      if (!stage) throw new Error('Stage window missing');

      const hwndBuf = stage.getNativeWindowHandle();
      const hwndNum = hwndBuf.length >= 8 && typeof hwndBuf.readBigInt64LE === 'function'
        ? hwndBuf.readBigInt64LE()
        : BigInt(hwndBuf.readInt32LE());
      const hwndHex = '0x' + hwndNum.toString(16);

      return {
        isAlwaysOnTop: stage.isAlwaysOnTop(),
        isDestroyed: stage.isDestroyed(),
        isFocusable: stage.isFocusable(),
        title: stage.getTitle(),
        hwndHex,
      };
    });

    expect(stageProps.isAlwaysOnTop).toBe(true);
    expect(stageProps.isFocusable).toBe(false);
    expect(stageProps.isDestroyed).toBe(false);

    // On Windows, verify Win32 WS_EX_TOOLWINDOW is set and WS_EX_APPWINDOW is absent (Alt+Tab and taskbar hiding)
    if (process.platform === 'win32') {
      const scriptPath = resolve('apps/desktop/tests/helpers/window-style.ps1');
      const styleJson = execFileSync('pwsh', ['-NoProfile', '-File', scriptPath, '-HwndHex', stageProps.hwndHex], {
        encoding: 'utf-8',
      });
      const style = JSON.parse(styleJson) as { WS_EX_TOOLWINDOW: boolean; WS_EX_APPWINDOW: boolean };
      expect(style.WS_EX_TOOLWINDOW).toBe(true);
      expect(style.WS_EX_APPWINDOW).toBe(false);
    }

    // 2. Control channel and event forwarding verification (CONTRACTS 2.3)
    await stagePage?.evaluate(() => {
      const win = window as unknown as { __FORWARDED_EVENTS__?: unknown[] };
      win.__FORWARDED_EVENTS__ = [];
      window.addEventListener('message', (event) => {
        const data = event.data as { type?: string; payload?: unknown } | null;
        if (data?.type === 'stage.event') {
          win.__FORWARDED_EVENTS__?.push(data.payload);
        }
      });
    });

    // Publish desktop.dnd event on main AppBus
    await application.evaluate(({ app }) => {
      const bus = (app as unknown as { __nemu_bus__?: AppBus }).__nemu_bus__;
      if (!bus) throw new Error('AppBus not found on app');
      bus.publish(
        bus.createEvent('main', 'desktop.dnd', {
          state: 'busy',
        }),
      );
    });

    // Stage window receives forwarded stage.event wrapping desktop.dnd
    await expect
      .poll(async () => {
        return await stagePage?.evaluate(() => {
          const win = window as unknown as { __FORWARDED_EVENTS__?: Array<{ event?: { type?: string } }> };
          return win.__FORWARDED_EVENTS__?.some((e) => e.event?.type === 'desktop.dnd');
        });
      })
      .toBe(true);

    // Verify stage.hitTest toggles stage window setIgnoreMouseEvents
    await stagePage?.evaluate(() => {
      window.postMessage({ type: 'stage.hitTest', payload: { overCharacter: true } }, '*');
    });

    await expect
      .poll(async () => {
        return await application?.evaluate(({ app }) => {
          const stageMgr = (app as unknown as { __nemu_stage_window__?: StageWindowManager }).__nemu_stage_window__;
          // In stage-window.ts, handleHitTest(true) sets isIgnored to false
          return (stageMgr as unknown as { isIgnored?: boolean })?.isIgnored;
        });
      })
      .toBe(false);

    await stagePage?.evaluate(() => {
      window.postMessage({ type: 'stage.hitTest', payload: { overCharacter: false } }, '*');
    });

    await expect
      .poll(async () => {
        return await application?.evaluate(({ app }) => {
          const stageMgr = (app as unknown as { __nemu_stage_window__?: StageWindowManager }).__nemu_stage_window__;
          return (stageMgr as unknown as { isIgnored?: boolean })?.isIgnored;
        });
      })
      .toBe(true);

    // 3. Panel window close and summon lifecycle
    await application.evaluate(({ BrowserWindow }) => {
      const panel = BrowserWindow.getAllWindows().find((w) => w.getTitle() === 'Nemu panel');
      if (!panel) throw new Error('Panel window missing');
      panel.close(); // Simulates user clicking 'X'
    });

    // Panel is hidden, but NOT destroyed
    const panelStatusAfterClose = await application.evaluate(({ BrowserWindow }) => {
      const panel = BrowserWindow.getAllWindows().find((w) => w.getTitle() === 'Nemu panel');
      return {
        exists: Boolean(panel),
        isDestroyed: panel ? panel.isDestroyed() : true,
        isVisible: panel ? panel.isVisible() : false,
      };
    });
    expect(panelStatusAfterClose.exists).toBe(true);
    expect(panelStatusAfterClose.isDestroyed).toBe(false);
    expect(panelStatusAfterClose.isVisible).toBe(false);

    // Re-summon panel window
    await application.evaluate(({ app }) => {
      const panelMgr = (app as unknown as { __nemu_panel_window__?: PanelWindowManager }).__nemu_panel_window__;
      panelMgr?.show();
    });

    const panelStatusAfterSummon = await application.evaluate(({ BrowserWindow }) => {
      const panel = BrowserWindow.getAllWindows().find((w) => w.getTitle() === 'Nemu panel');
      return panel?.isVisible();
    });
    expect(panelStatusAfterSummon).toBe(true);

    // 4. Supervised processes check and grandchild termination
    // Assert all default placeholder processes are running and restartCount is 0
    const supervisorStatus = await application.evaluate(({ app }) => {
      const sup = (app as unknown as { __nemu_supervisor__?: ProcessSupervisor }).__nemu_supervisor__;
      if (!sup) throw new Error('Supervisor missing');
      return sup.getStatus();
    });

    expect(Object.keys(supervisorStatus).length).toBe(7);
    for (const [name, proc] of Object.entries(supervisorStatus)) {
      expect(proc.state, `Process ${name} should be running`).toBe('running');
      expect(proc.restartCount, `Process ${name} should have 0 restarts`).toBe(0);
      expect(proc.pid, `Process ${name} should have valid PID`).toBeDefined();
    }

    // Spawn a supervised test process that launches a grandchild process
    let grandchildPid: number | undefined;
    const grandchildPidPath = join(scratch, 'grandchild.pid');

    const allChildPids = await application.evaluate(
      async ({ app }, { pidPath }) => {
        const sup = (app as unknown as { __nemu_supervisor__?: ProcessSupervisor }).__nemu_supervisor__;
        if (!sup) throw new Error('Supervisor missing');

        const grandchildScript = `
          const cp = require('node:child_process');
          const grandchild = cp.spawn(process.execPath, ['-e', 'setInterval(()=>{console.log("HEARTBEAT")},1000)'], {
            detached: true,
            stdio: 'ignore',
            env: { ...process.env, ELECTRON_RUN_AS_NODE: '1' }
          });
          require('node:fs').writeFileSync(process.env.GRANDCHILD_PID_FILE, String(grandchild.pid));
          setInterval(() => { console.log('HEARTBEAT'); }, 1000);
        `;

        const testProcess = sup.addProcess({
          name: 'tree-kill-test',
          isWorker: false,
          command: process.execPath,
          args: ['-e', grandchildScript],
          env: {
            ELECTRON_RUN_AS_NODE: '1',
            GRANDCHILD_PID_FILE: pidPath,
          },
        });

        await testProcess.start();
        return sup.getRunningPids();
      },
      { pidPath: grandchildPidPath },
    );

    // Wait for grandchild PID file to be written
    await expect
      .poll(async () => {
        try {
          const { readFileSync } = await import('node:fs');
          const raw = readFileSync(grandchildPidPath, 'utf-8').trim();
          const pid = Number.parseInt(raw, 10);
          if (!Number.isNaN(pid) && pid > 0) {
            grandchildPid = pid;
            return true;
          }
          return false;
        } catch {
          return false;
        }
      })
      .toBe(true);

    expect(grandchildPid).toBeDefined();
    const pidsToCheck = [...allChildPids, grandchildPid!];
    expect(pidsToCheck.length).toBeGreaterThanOrEqual(8);

    // Verify all processes are alive before shutdown
    for (const pid of pidsToCheck) {
      let isAlive = false;
      try {
        process.kill(pid, 0);
        isAlive = true;
      } catch {
        isAlive = false;
      }
      expect(isAlive, `PID ${pid} should be alive prior to shutdown`).toBe(true);
    }

    // Close application
    await application.close();
    application = undefined;

    // Allow graceful process tree cleanup
    await new Promise((r) => setTimeout(r, 2000));

    // Verify every single child and grandchild process is dead
    for (const pid of pidsToCheck) {
      let isAlive = false;
      try {
        process.kill(pid, 0);
        isAlive = true;
      } catch {
        isAlive = false;
      }
      expect(isAlive, `PID ${pid} must be terminated after shutdown`).toBe(false);
    }
  } finally {
    try {
      await application?.close();
    } finally {
      await rm(scratch, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
    }
  }
});
