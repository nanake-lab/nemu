import { globalShortcut } from 'electron';
import type { DesktopSettings } from '../settings/index.js';
import type { PanelWindowManager } from './panel-window.js';
import type { ChannelManager } from './channels.js';

export interface ShortcutManagerOptions {
  settings: DesktopSettings;
  panelWindowManager: PanelWindowManager;
  channelManager: ChannelManager;
}

export class ShortcutManager {
  private panelWindowManager: PanelWindowManager;
  private channelManager: ChannelManager;
  private settings: DesktopSettings;
  private isTalking = false;

  constructor(options: ShortcutManagerOptions) {
    this.settings = options.settings;
    this.panelWindowManager = options.panelWindowManager;
    this.channelManager = options.channelManager;
  }

  public registerAll(): void {
    this.unregisterAll();

    const summonKey = this.settings.shortcuts.summonPanel || 'CommandOrControl+Shift+N';
    const talkKey = this.settings.shortcuts.pushToTalk || 'CommandOrControl+Shift+Space';

    const summonRegistered = globalShortcut.register(summonKey, () => {
      this.panelWindowManager.toggle();
    });
    if (!summonRegistered) {
      console.warn(`[Shortcuts] Failed to register summonPanel shortcut '${summonKey}' (accelerator may be in use).`);
    }

    const talkRegistered = globalShortcut.register(talkKey, () => {
      // Toggle push-to-talk speech state via stage.mic.set command without forging voice.vad
      this.isTalking = !this.isTalking;
      this.channelManager.sendStageCommand('stage.mic.set', {
        enabled: this.isTalking,
        mode: 'pushToTalk',
      });
    });
    if (!talkRegistered) {
      console.warn(`[Shortcuts] Failed to register pushToTalk shortcut '${talkKey}' (accelerator may be in use).`);
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
