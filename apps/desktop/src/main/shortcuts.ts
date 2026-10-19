import { globalShortcut } from 'electron';
import type { DesktopSettings } from '../settings/index.js';
import type { PanelWindowManager } from './panel-window.js';
import type { AppBus } from '../bus/bus.js';

export interface ShortcutManagerOptions {
  settings: DesktopSettings;
  panelWindowManager: PanelWindowManager;
  appBus: AppBus;
}

export class ShortcutManager {
  private panelWindowManager: PanelWindowManager;
  private appBus: AppBus;
  private settings: DesktopSettings;
  private isTalking = false;

  constructor(options: ShortcutManagerOptions) {
    this.settings = options.settings;
    this.panelWindowManager = options.panelWindowManager;
    this.appBus = options.appBus;
  }

  public registerAll(): void {
    this.unregisterAll();

    const summonKey = this.settings.shortcuts.summonPanel || 'CommandOrControl+Shift+N';
    const talkKey = this.settings.shortcuts.pushToTalk || 'CommandOrControl+Shift+Space';

    try {
      globalShortcut.register(summonKey, () => {
        this.panelWindowManager.toggle();
      });
    } catch (err) {
      console.warn(`[Shortcuts] Failed to register summonPanel shortcut '${summonKey}':`, err);
    }

    try {
      globalShortcut.register(talkKey, () => {
        // Toggle push-to-talk speech state
        this.isTalking = !this.isTalking;
        const vadEvent = this.appBus.createEvent('voice', 'voice.vad', {
          state: this.isTalking ? 'speechStart' : 'speechEnd',
        });
        this.appBus.publish(vadEvent);
      });
    } catch (err) {
      console.warn(`[Shortcuts] Failed to register pushToTalk shortcut '${talkKey}':`, err);
    }
  }

  public updateSettings(newSettings: DesktopSettings): void {
    this.settings = newSettings;
    this.registerAll();
  }

  public unregisterAll(): void {
    globalShortcut.unregisterAll();
  }
}
