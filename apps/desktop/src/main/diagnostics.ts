import { app, powerMonitor } from 'electron';
import { appendFileSync } from 'node:fs';
import { join } from 'node:path';

// Opt-in test evidence only; no authority state or renderer contents are recorded.
export function installDiagnostics(): void {
  const directory = process.env.NEMU_EVIDENCE_DIR;
  if (!directory) return;
  const record = (event: string, detail: Record<string, unknown> = {}) => {
    try {
      appendFileSync(
        join(directory, 'electron-events.jsonl'),
        JSON.stringify({
          at: new Date().toISOString(),
          monotonicMs: performance.now(),
          pid: process.pid,
          event,
          ...detail,
        }) + '\n',
      );
    } catch (error) {
      process.stderr.write(`Diagnostic write failed: ${String(error)}\n`);
    }
  };
  record('main-start');
  app.on('browser-window-created', (_event, window) => {
    const id = window.id;
    record('window-created', { id });
    window.on('show', () => record('show', { id }));
    window.on('ready-to-show', () => record('ready-to-show', { id }));
    window.on('closed', () => record('closed', { id }));
    window.webContents.on('did-start-loading', () => record('did-start-loading', { id }));
    window.webContents.on('dom-ready', () => record('dom-ready', { id }));
    window.webContents.on('did-finish-load', () => record('did-finish-load', { id }));
    window.webContents.on('did-stop-loading', () => record('did-stop-loading', { id }));
    window.webContents.on('did-fail-load', (_event, code, description, _url, mainFrame) =>
      record('did-fail-load', { id, code, description, mainFrame }),
    );
    window.webContents.on('render-process-gone', (_event, details) =>
      record('render-process-gone', { id, ...details }),
    );
  });
  app.on('ready', () => {
    record('app-ready');
    record('desktop-idle-state', { state: powerMonitor.getSystemIdleState(1) });
    powerMonitor.on('lock-screen', () => record('lock-screen'));
    powerMonitor.on('unlock-screen', () => record('unlock-screen'));
    powerMonitor.on('suspend', () => record('suspend'));
    powerMonitor.on('resume', () => record('resume'));
  });
  app.on('child-process-gone', (_event, details) => record('child-process-gone', { ...details }));
  app.on('quit', (_event, exitCode) => record('quit', { exitCode }));
  process.on('uncaughtExceptionMonitor', (error) =>
    record('uncaught-exception', { message: error.message }),
  );
}
