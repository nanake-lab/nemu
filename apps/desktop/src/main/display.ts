import { screen, type BrowserWindow, type Display, type Rectangle } from 'electron';

export class DisplayManager {
  private stageWindow: BrowserWindow | null = null;
  private currentDisplayId: number | null = null;
  private onDisplayMigrated?: ((displayId: number, bounds: Rectangle) => void) | undefined = undefined;

  constructor(options?: {
    onDisplayMigrated?: ((displayId: number, bounds: Rectangle) => void) | undefined;
  }) {
    this.onDisplayMigrated = options?.onDisplayMigrated;
  }

  public bindStageWindow(stageWindow: BrowserWindow): void {
    this.stageWindow = stageWindow;
    this.setupListeners();
    this.updateStageBounds();
  }

  public getCurrentDisplay(): Display {
    const all = screen.getAllDisplays();
    if (this.currentDisplayId !== null) {
      const found = all.find((d) => d.id === this.currentDisplayId);
      if (found) return found;
    }
    const primary = screen.getPrimaryDisplay();
    this.currentDisplayId = primary.id;
    return primary;
  }

  public setStageDisplay(displayId: number): void {
    const all = screen.getAllDisplays();
    const found = all.find((d) => d.id === displayId);
    if (!found) return;

    this.currentDisplayId = displayId;
    this.updateStageBounds();
  }

  public updateStageBounds(): void {
    if (!this.stageWindow || this.stageWindow.isDestroyed()) return;

    const display = this.getCurrentDisplay();
    const workArea = display.workArea;

    this.stageWindow.setBounds({
      x: workArea.x,
      y: workArea.y,
      width: workArea.width,
      height: workArea.height,
    });

    if (this.onDisplayMigrated) {
      this.onDisplayMigrated(display.id, workArea);
    }
  }

  private setupListeners(): void {
    screen.on('display-added', () => {
      // Re-evaluate display arrangement
      this.updateStageBounds();
    });

    screen.on('display-removed', (_event, oldDisplay) => {
      if (this.currentDisplayId === oldDisplay.id) {
        // Fallback to primary display if current display was detached
        const primary = screen.getPrimaryDisplay();
        this.currentDisplayId = primary.id;
        this.updateStageBounds();
      }
    });

    screen.on('display-metrics-changed', (_event, display) => {
      if (this.currentDisplayId === display.id) {
        this.updateStageBounds();
      }
    });
  }
}
