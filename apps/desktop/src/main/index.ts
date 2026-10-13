import { app, BrowserWindow, session } from 'electron';
import { join, resolve, relative, isAbsolute, basename } from 'node:path';
import { tmpdir } from 'node:os';

// The scaffold never opens Nemu's rehearsal or life storage.
const scratch = process.env.NEMU_SCAFFOLD_USER_DATA;
if (
  !scratch ||
  !isAbsolute(scratch) ||
  !basename(scratch).startsWith('nemu-scaffold-') ||
  relative(resolve(tmpdir()), resolve(scratch)).startsWith('..')
) {
  throw new Error('Start through pnpm dev or the smoke test with an isolated temporary directory.');
}
app.setPath('userData', scratch);

async function openWindow(kind: 'stage' | 'panel'): Promise<void> {
  const stage = kind === 'stage';
  const window = new BrowserWindow({
    width: stage ? 420 : 680,
    height: stage ? 600 : 520,
    title: `Nemu ${kind}`,
    transparent: stage,
    frame: !stage,
    backgroundColor: stage ? '#00000000' : '#f6f7f9',
    show: false,
    webPreferences: {
      preload: join(import.meta.dirname, '../preload/index.cjs'),
      sandbox: true,
      contextIsolation: true,
      nodeIntegration: false,
      webSecurity: true,
    },
  });
  window.webContents.setWindowOpenHandler(() => ({ action: 'deny' }));
  window.webContents.on('will-navigate', (event) => event.preventDefault());
  const devUrl = process.env.ELECTRON_RENDERER_URL;
  if (devUrl) await window.loadURL(`${devUrl}/${kind}/index.html`);
  else await window.loadFile(join(import.meta.dirname, `../renderer/${kind}/index.html`));
  window.show();
}
app
  .whenReady()
  .then(async () => {
    session.defaultSession.setPermissionRequestHandler((_contents, _permission, callback) =>
      callback(false),
    );
    await Promise.all([openWindow('stage'), openWindow('panel')]);
  })
  .catch(() => {
    app.exit(1);
  });
app.on('window-all-closed', () => app.quit());
