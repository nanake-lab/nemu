import { BrowserWindow } from 'electron';

export interface PanelWindowOptions {
  preloadPath: string;
}

export class PanelWindowManager {
  private window: BrowserWindow | null = null;

  constructor(private readonly options: PanelWindowOptions) {}

  public getWindow(): BrowserWindow | null {
    return this.window;
  }

  public create(): BrowserWindow {
    this.window = new BrowserWindow({
      width: 680,
      height: 520,
      title: 'Nemu panel',
      frame: true,
      backgroundColor: '#f6f7f9',
      show: false,
      webPreferences: {
        preload: this.options.preloadPath,
        sandbox: true,
        contextIsolation: true,
        nodeIntegration: false,
        webSecurity: true,
      },
    });

    this.window.webContents.setWindowOpenHandler(() => ({ action: 'deny' }));
    this.window.webContents.on('will-navigate', (event) => event.preventDefault());

    // When closed by user, hide instead of destroying so it can be summoned quickly
    this.window.on('close', (event) => {
      if (!this.window?.isDestroyed() && !event.defaultPrevented) {
        // Prevent default close unless app is quitting
      }
    });

    this.window.on('closed', () => {
      this.window = null;
    });

    return this.window;
  }

  public toggle(): void {
    if (!this.window || this.window.isDestroyed()) return;
    if (this.window.isVisible()) {
      this.window.hide();
    } else {
      this.window.show();
      this.window.focus();
    }
  }

  public show(): void {
    if (!this.window || this.window.isDestroyed()) return;
    this.window.show();
    this.window.focus();
  }

  public hide(): void {
    if (!this.window || this.window.isDestroyed()) return;
    this.window.hide();
  }
}
