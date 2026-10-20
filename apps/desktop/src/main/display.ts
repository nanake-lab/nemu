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

    if (this.currentDisplayId === displayId) return;

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

  /**
   * Handle character bounding box reported by stage (in physical pixels).
   * Determines which display the character is located on and migrates the stage overlay window if changed.
   */
  public handleCharacterBounds(screenRectPx: { x: number; y: number; w: number; h: number }): void {
    const centerPhysicalX = screenRectPx.x + screenRectPx.w / 2;
    const centerPhysicalY = screenRectPx.y + screenRectPx.h / 2;

    const allDisplays = screen.getAllDisplays();
    let targetDisplay = allDisplays.find((d) => {
      const scale = d.scaleFactor || 1;
      const physX = d.bounds.x * scale;
      const physY = d.bounds.y * scale;
      const physW = d.bounds.width * scale;
      const physH = d.bounds.height * scale;
      return (
        centerPhysicalX >= physX &&
        centerPhysicalX < physX + physW &&
        centerPhysicalY >= physY &&
        centerPhysicalY < physY + physH
      );
    });

    if (!targetDisplay) {
      const dipPoint = typeof (screen as unknown as { screenToDipPoint?: (p: { x: number; y: number }) => { x: number; y: number } }).screenToDipPoint === 'function'
        ? (screen as unknown as { screenToDipPoint: (p: { x: number; y: number }) => { x: number; y: number } }).screenToDipPoint({
            x: Math.round(centerPhysicalX),
            y: Math.round(centerPhysicalY),
          })
        : {
            x: Math.round(centerPhysicalX / (screen.getPrimaryDisplay().scaleFactor || 1)),
            y: Math.round(centerPhysicalY / (screen.getPrimaryDisplay().scaleFactor || 1)),
          };
      targetDisplay = screen.getDisplayNearestPoint(dipPoint);
    }

    if (targetDisplay && targetDisplay.id !== this.currentDisplayId) {
      this.setStageDisplay(targetDisplay.id);
    }
  }

  /**
   * Handle character drag position reported by stage (in physical pixels).
   */
  public handleCharacterDrag(drag: {
    phase: 'start' | 'move' | 'end';
    screenX: number;
    screenY: number;
    vx: number;
    vy: number;
  }): void {
    if (drag.phase === 'move' || drag.phase === 'end') {
      this.handleCharacterBounds({
        x: drag.screenX - 50,
        y: drag.screenY - 50,
        w: 100,
        h: 100,
      });
    }
  }

  private setupListeners(): void {
    screen.on('display-added', () => {
      this.updateStageBounds();
    });

    screen.on('display-removed', (_event, oldDisplay) => {
      if (this.currentDisplayId === oldDisplay.id) {
        this.currentDisplayId = screen.getPrimaryDisplay().id;
      }
      this.updateStageBounds();
    });

    screen.on('display-metrics-changed', (_event, display, changedMetrics) => {
      if (this.currentDisplayId === display.id) {
        if (changedMetrics.includes('bounds') || changedMetrics.includes('workArea')) {
          this.updateStageBounds();
        }
      }
    });
  }
}
