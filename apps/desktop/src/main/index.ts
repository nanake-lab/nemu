import { app, session } from 'electron';
import { join, resolve, relative, isAbsolute, basename } from 'node:path';
import { tmpdir } from 'node:os';
import { installDiagnostics } from './diagnostics.js';
import { AppBus } from '../bus/index.js';
import { SettingsStore } from '../settings/index.js';
import { ProcessSupervisor } from '../supervisor/index.js';
import { StageWindowManager } from './stage-window.js';
import { PanelWindowManager } from './panel-window.js';
import { DisplayManager } from './display.js';
import { ChannelManager } from './channels.js';
import { TrayManager } from './tray.js';
import { ShortcutManager } from './shortcuts.js';
import { DesktopLogger } from './logger.js';

app.setName('Nemu');

// The scaffold never opens Nemu's rehearsal or life storage.
const scratch = process.env.NEMU_SCAFFOLD_USER_DATA;
if (scratch) {
  if (
    !isAbsolute(scratch) ||
    !basename(scratch).startsWith('nemu-scaffold-') ||
    relative(resolve(tmpdir()), resolve(scratch)).startsWith('..')
  ) {
    throw new Error('Start through pnpm dev or the smoke test with an isolated temporary directory.');
  }
  app.setPath('userData', scratch);
} else if (!app.isPackaged) {
  throw new Error('Start through pnpm dev or the smoke test with an isolated temporary directory.');
}

installDiagnostics();

// Single instance lock: terminate immediately if lock is not acquired
const gotSingleInstanceLock = app.requestSingleInstanceLock();
if (!gotSingleInstanceLock) {
  app.exit(0);
} else {
  let stageWindowManager: StageWindowManager | null = null;
  let panelWindowManager: PanelWindowManager | null = null;
  let displayManager: DisplayManager | null = null;
  let channelManager: ChannelManager | null = null;
  let trayManager: TrayManager | null = null;
  let shortcutManager: ShortcutManager | null = null;
  let supervisor: ProcessSupervisor | null = null;
  let logger: DesktopLogger | null = null;
  let appBus: AppBus | null = null;
  let settingsStore: SettingsStore | null = null;

  async function bootstrap(): Promise<void> {
    logger = new DesktopLogger();
    logger.info('Starting Nemu desktop shell...');

    settingsStore = new SettingsStore();
    const settings = settingsStore.get();

    appBus = new AppBus({
      ringCapacity: 5000,
      enableValidation: process.env.NODE_ENV !== 'production',
    });

    const stagePreloadPath = join(import.meta.dirname, '../preload/stage.cjs');
    const panelPreloadPath = join(import.meta.dirname, '../preload/panel.cjs');

    stageWindowManager = new StageWindowManager({ preloadPath: stagePreloadPath });
    panelWindowManager = new PanelWindowManager({ preloadPath: panelPreloadPath });

    displayManager = new DisplayManager({
      onDisplayMigrated: (displayId, bounds) => {
        logger?.info('Stage overlay bounds updated', { displayId, bounds });
      },
    });

    const stageWin = stageWindowManager.create();
    const panelWin = panelWindowManager.create();

    displayManager.bindStageWindow(stageWin);

    channelManager = new ChannelManager({
      appBus,
      stageWindowManager,
      displayManager,
    });

    channelManager.establishChannels(stageWin, panelWin);

    trayManager = new TrayManager({
      stageWindowManager,
      panelWindowManager,
      appBus,
      settingsStore,
    });
    trayManager.create();

    shortcutManager = new ShortcutManager({
      settings,
      panelWindowManager,
      channelManager,
    });
    shortcutManager.registerAll();

    // Configure auto-launch setting
    try {
      app.setLoginItemSettings({ openAtLogin: settings.general.openAtLogin });
    } catch (err) {
      logger.warn('Failed to set login item settings', { error: String(err) });
    }

    // Load renderers
    const devUrl = process.env.ELECTRON_RENDERER_URL;
    if (devUrl) {
      await Promise.all([
        stageWin.loadURL(`${devUrl}/stage/index.html`),
        panelWin.loadURL(`${devUrl}/panel/index.html`),
      ]);
    } else {
      await Promise.all([
        stageWin.loadFile(join(import.meta.dirname, '../renderer/stage/index.html')),
        panelWin.loadFile(join(import.meta.dirname, '../renderer/panel/index.html')),
      ]);
    }

    stageWin.show();
    panelWin.show();

    // Initialize and start process supervisor
    supervisor = new ProcessSupervisor({
      settings,
      logDir: logger.getLogDir(),
    });

    // Expose internal references on app for diagnostic and testing queries
    const appGlobals = app as unknown as Record<string, unknown>;
    appGlobals.__nemu_supervisor__ = supervisor;
    appGlobals.__nemu_stage_window__ = stageWindowManager;
    appGlobals.__nemu_panel_window__ = panelWindowManager;
    appGlobals.__nemu_bus__ = appBus;
    appGlobals.__nemu_channels__ = channelManager;
    appGlobals.__nemu_display__ = displayManager;

    if (settings.supervisor.autoStart && process.env.NEMU_DISABLE_SUPERVISOR !== '1') {
      logger.info('Starting process supervisor...');
      await supervisor.startAll();
    }

    logger.info('Nemu desktop shell startup complete.');
  }

  app.on('second-instance', () => {
    if (panelWindowManager) {
      panelWindowManager.show();
    }
  });

  app
    .whenReady()
    .then(async () => {
      session.defaultSession.setPermissionRequestHandler((_contents, _permission, callback) =>
        callback(false),
      );
      await bootstrap();
    })
    .catch((error: unknown) => {
      if (process.env.NEMU_EVIDENCE_DIR) {
        process.stderr.write(`Desktop startup failed: ${String(error)}\n`);
      }
      app.exit(1);
    });

  let isShuttingDown = false;
  async function cleanShutdown(): Promise<void> {
    if (isShuttingDown) return;
    isShuttingDown = true;

    try {
      logger?.info('Cleaning up Nemu desktop shell and child processes...');
      shortcutManager?.unregisterAll();
      trayManager?.destroy();
      channelManager?.destroy();
      panelWindowManager?.prepareQuit();

      if (supervisor) {
        await supervisor.stopAll();
        supervisor = null;
      }
    } catch (err) {
      console.error('Error during shutdown cleanup:', err);
    }
  }

  app.on('before-quit', (event) => {
    if (!isShuttingDown && supervisor) {
      event.preventDefault();
      cleanShutdown().then(() => {
        app.quit();
      });
    }
  });

  app.on('window-all-closed', () => {
    app.quit();
  });
}
