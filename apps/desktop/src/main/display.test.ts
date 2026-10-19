import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { BrowserWindow } from 'electron';
import { DisplayManager } from './display.js';

vi.mock('electron', () => {
  const displays = [
    {
      id: 1,
      scaleFactor: 1,
      bounds: { x: 0, y: 0, width: 1920, height: 1080 },
      workArea: { x: 0, y: 0, width: 1920, height: 1040 },
    },
    {
      id: 2,
      scaleFactor: 1.5,
      bounds: { x: 1920, y: 0, width: 2560, height: 1440 },
      workArea: { x: 1920, y: 0, width: 2560, height: 1400 },
    },
  ];

  return {
    screen: {
      getAllDisplays: vi.fn(() => displays),
      getPrimaryDisplay: vi.fn(() => displays[0]),
      getDisplayNearestPoint: vi.fn((point: { x: number; y: number }) => {
        return point.x >= 1920 ? displays[1] : displays[0];
      }),
      on: vi.fn(),
    },
  };
});

describe('DisplayManager multi-display character migration', () => {
  let mockWindow: BrowserWindow;
  let currentBounds: { x: number; y: number; width: number; height: number };

  beforeEach(() => {
    currentBounds = { x: 0, y: 0, width: 1920, height: 1040 };
    mockWindow = {
      isDestroyed: () => false,
      setBounds: (b: { x: number; y: number; width: number; height: number }) => {
        currentBounds = b;
      },
    } as unknown as BrowserWindow;
  });

  it('keeps stage window on primary display when character is on primary display', () => {
    const manager = new DisplayManager();
    manager.bindStageWindow(mockWindow);

    expect(manager.getCurrentDisplay().id).toBe(1);
    expect(currentBounds).toEqual({ x: 0, y: 0, width: 1920, height: 1040 });

    // Character bounds reported at (500, 500) physical px (Display 1)
    manager.handleCharacterBounds({ x: 400, y: 400, w: 200, h: 200 });

    expect(manager.getCurrentDisplay().id).toBe(1);
    expect(currentBounds.x).toBe(0);
  });

  it('migrates stage overlay to secondary display when character crosses boundary', () => {
    let migratedDisplayId: number | null = null;
    const manager = new DisplayManager({
      onDisplayMigrated: (id) => {
        migratedDisplayId = id;
      },
    });
    manager.bindStageWindow(mockWindow);

    expect(manager.getCurrentDisplay().id).toBe(1);

    // Character is dragged onto Display 2:
    // Display 2 starts at x = 1920 with scale 1.5, physical x starts at 1920 * 1.5 = 2880
    manager.handleCharacterBounds({ x: 3000, y: 500, w: 200, h: 200 });

    expect(manager.getCurrentDisplay().id).toBe(2);
    expect(migratedDisplayId).toBe(2);
    expect(currentBounds).toEqual({ x: 1920, y: 0, width: 2560, height: 1400 });
  });

  it('migrates overlay back to primary display when character is dragged back', () => {
    const manager = new DisplayManager();
    manager.bindStageWindow(mockWindow);

    // Drag to Display 2
    manager.handleCharacterDrag({
      phase: 'move',
      screenX: 3200,
      screenY: 600,
      vx: 10,
      vy: 0,
    });
    expect(manager.getCurrentDisplay().id).toBe(2);

    // Drag back to Display 1
    manager.handleCharacterDrag({
      phase: 'end',
      screenX: 800,
      screenY: 500,
      vx: -10,
      vy: 0,
    });
    expect(manager.getCurrentDisplay().id).toBe(1);
    expect(currentBounds.x).toBe(0);
  });
});
