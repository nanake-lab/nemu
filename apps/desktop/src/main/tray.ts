import { Tray, Menu, nativeImage, app } from 'electron';
import type { StageWindowManager } from './stage-window.js';
import type { PanelWindowManager } from './panel-window.js';
import type { AppBus } from '../bus/bus.js';
import type { SettingsStore } from '../settings/settings.js';
import type { DndState } from '@nemu/contracts';

export interface TrayManagerOptions {
  stageWindowManager: StageWindowManager;
  panelWindowManager: PanelWindowManager;
  appBus: AppBus;
  settingsStore: SettingsStore;
}

export class TrayManager {
  private tray: Tray | null = null;
  private stageWindowManager: StageWindowManager;
  private panelWindowManager: PanelWindowManager;
  private appBus: AppBus;
  private settingsStore: SettingsStore;

  constructor(options: TrayManagerOptions) {
    this.stageWindowManager = options.stageWindowManager;
    this.panelWindowManager = options.panelWindowManager;
    this.appBus = options.appBus;
    this.settingsStore = options.settingsStore;
  }

  public create(): Tray {
    // Generate a clean 16x16 icon in memory (transparent with a cyan dot)
    const icon = nativeImage.createFromBuffer(
      Buffer.from([
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x10,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0xf3, 0xff, 0x61, 0x00, 0x00, 0x00,
        0x19, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x60, 0x18, 0x05, 0xa3,
        0x60, 0x14, 0x8c, 0x82, 0x51, 0x30, 0x0a, 0x46, 0xc1, 0x28, 0x18, 0x00,
        0x00, 0x11, 0xb0, 0x02, 0x21, 0x93, 0xd0, 0xd4, 0x7a, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
      ]),
    );

    this.tray = new Tray(icon);
    this.tray.setToolTip('Nemu 3D 桌面伙伴');

    this.updateMenu();
    return this.tray;
  }

  public updateMenu(): void {
    if (!this.tray) return;

    const currentSettings = this.settingsStore.get();
    const isDnd = currentSettings.general.dnd !== 'none';

    const contextMenu = Menu.buildFromTemplate([
      {
        label: '显示/隐藏 舞台角色',
        click: () => {
          this.stageWindowManager.toggle();
        },
      },
      {
        label: '召唤/隐藏 面板',
        click: () => {
          this.panelWindowManager.toggle();
        },
      },
      {
        label: '勿扰模式',
        type: 'checkbox',
        checked: isDnd,
        click: (menuItem) => {
          const nextDnd: DndState = menuItem.checked ? 'busy' : 'none';
          this.settingsStore.update((curr) => ({
            general: { ...curr.general, dnd: nextDnd },
          }));
          const event = this.appBus.createEvent('main', 'desktop.dnd', { state: nextDnd });
          this.appBus.publish(event);
        },
      },
      {
        label: '设置',
        click: () => {
          this.panelWindowManager.show();
        },
      },
      { type: 'separator' },
      {
        label: '退出 Nemu',
        click: () => {
          app.quit();
        },
      },
    ]);

    this.tray.setContextMenu(contextMenu);
  }

  public destroy(): void {
    if (this.tray && !this.tray.isDestroyed()) {
      this.tray.destroy();
      this.tray = null;
    }
  }
}
