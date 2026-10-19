import { BrowserWindow, screen } from 'electron';

export interface StageWindowOptions {
  preloadPath: string;
}

export class StageWindowManager {
  private window: BrowserWindow | null = null;
  private isIgnored = true;

  constructor(private readonly options: StageWindowOptions) {}

  public getWindow(): BrowserWindow | null {
    return this.window;
  }

  public create(): BrowserWindow {
    const primary = screen.getPrimaryDisplay();
    const workArea = primary.workArea;
    const isWin = process.platform === 'win32';

    this.window = new BrowserWindow({
      x: workArea.x,
      y: workArea.y,
      width: workArea.width,
      height: workArea.height,
      title: 'Nemu stage',
      transparent: true,
      frame: false,
      skipTaskbar: true,
      focusable: false,
      hasShadow: false,
      resizable: false,
      backgroundColor: '#00000000',
      type: isWin ? 'toolbar' : 'normal',
      show: false,
      webPreferences: {
        preload: this.options.preloadPath,
        sandbox: true,
        contextIsolation: true,
        nodeIntegration: false,
        webSecurity: true,
      },
    });

    // Enforce top-most overlay layer
    this.window.setAlwaysOnTop(true, 'screen-saver');

    // Default to click-through forward
    this.setClickThrough(true);

    this.window.webContents.setWindowOpenHandler(() => ({ action: 'deny' }));
    this.window.webContents.on('will-navigate', (event) => event.preventDefault());

    this.window.on('closed', () => {
      this.window = null;
    });

    return this.window;
  }

  public setClickThrough(ignore: boolean): void {
    if (!this.window || this.window.isDestroyed()) return;
    if (this.isIgnored === ignore) return;

    this.isIgnored = ignore;
    if (ignore) {
      this.window.setIgnoreMouseEvents(true, { forward: true });
    } else {
      this.window.setIgnoreMouseEvents(false);
    }
  }

  public handleHitTest(overCharacter: boolean): void {
    // If cursor is over the character mesh/pixels, stop ignoring mouse events so clicks register on stage.
    // If cursor is over transparent background, ignore and forward clicks to underlying windows.
    this.setClickThrough(!overCharacter);
  }

  public isClickThrough(): boolean {
    return this.isIgnored;
  }
}
