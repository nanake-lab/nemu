import { MessageChannelMain, type BrowserWindow } from 'electron';
import {
  type ApplicationEvent,
  type AppBusPayloads,
  type ForwardedAppBusEvent,
  forwardedAppBusTypes,
  type StageWireMessage,
} from '@nemu/contracts';
import type { AppBus } from '../bus/bus.js';
import { generateUlid } from '../bus/ulid.js';
import type { StageWindowManager } from './stage-window.js';
import type { DisplayManager } from './display.js';

export interface ChannelManagerOptions {
  appBus: AppBus;
  stageWindowManager: StageWindowManager;
  displayManager?: DisplayManager | undefined;
}

const MIRRORED_STAGE_TYPES = [
  'stage.ready',
  'stage.hit',
  'stage.drag',
  'stage.frameStats',
  'stage.error',
] as const;

type MirroredType = (typeof MIRRORED_STAGE_TYPES)[number];

export class ChannelManager {
  private appBus: AppBus;
  private stageWindowManager: StageWindowManager;
  private displayManager?: DisplayManager | undefined;
  private controlPortMain: Electron.MessagePortMain | null = null;
  private unsubBus?: (() => void) | undefined = undefined;

  // Voice-sidecar ports retained for P4 integration per CONTRACTS.md 2.0
  private audioPortVoice: Electron.MessagePortMain | null = null;
  private micPortVoice: Electron.MessagePortMain | null = null;

  constructor(options: ChannelManagerOptions) {
    this.appBus = options.appBus;
    this.stageWindowManager = options.stageWindowManager;
    this.displayManager = options.displayManager;
  }

  public getVoiceAudioPort(): Electron.MessagePortMain | null {
    return this.audioPortVoice;
  }

  public getVoiceMicPort(): Electron.MessagePortMain | null {
    return this.micPortVoice;
  }

  public establishChannels(stageWindow: BrowserWindow, panelWindow?: BrowserWindow): void {
    // Listen for stage did-finish-load to establish/re-establish channels on reloads or initial load
    stageWindow.webContents.on('did-finish-load', () => {
      this.setupStageChannels(stageWindow);
    });

    // If the stage is already loaded, establish immediately
    if (!stageWindow.webContents.isLoading()) {
      this.setupStageChannels(stageWindow);
    }

    // Panel bus channel (Panel <-> Main AppBus)
    if (panelWindow) {
      panelWindow.webContents.on('did-finish-load', () => {
        this.setupPanelChannels(panelWindow);
      });
      if (!panelWindow.webContents.isLoading()) {
        this.setupPanelChannels(panelWindow);
      }
    }
  }

  public setupStageChannels(stageWindow: BrowserWindow): void {
    if (stageWindow.isDestroyed()) return;

    if (this.controlPortMain) {
      this.controlPortMain.close();
      this.controlPortMain = null;
    }

    // 1. Control channel (Main <-> Stage)
    const controlChannel = new MessageChannelMain();
    this.controlPortMain = controlChannel.port1;
    this.setupControlPort(this.controlPortMain);

    // 2. Audio channel (Voice <-> Stage)
    const audioChannel = new MessageChannelMain();
    this.audioPortVoice = audioChannel.port1;

    // 3. Microphone channel (Stage -> Voice)
    const micChannel = new MessageChannelMain();
    this.micPortVoice = micChannel.port2;

    // Transfer ports to Stage window
    try {
      stageWindow.webContents.postMessage('nemu:channel:control', null, [controlChannel.port2]);
      stageWindow.webContents.postMessage('nemu:channel:audio', null, [audioChannel.port2]);
      stageWindow.webContents.postMessage('nemu:channel:mic', null, [micChannel.port1]);
    } catch (err) {
      console.error('[Channels] Failed to transfer ports to stage:', err);
    }
  }

  public setupPanelChannels(panelWindow: BrowserWindow): void {
    if (panelWindow.isDestroyed()) return;
    const panelBusChannel = new MessageChannelMain();
    this.appBus.connectPort('panel', panelBusChannel.port1);
    try {
      panelWindow.webContents.postMessage('nemu:channel:bus', null, [panelBusChannel.port2]);
    } catch (err) {
      console.error('[Channels] Failed to transfer bus port to panel:', err);
    }
  }

  public sendStageCommand<T extends string, P>(type: T, payload: P): void {
    if (!this.controlPortMain) return;
    try {
      this.controlPortMain.postMessage({
        v: 2 as const,
        id: generateUlid(),
        type,
        payload,
      });
    } catch (err) {
      console.error(`[Channels] Failed to send ${type} command to stage:`, err);
    }
  }

  private setupControlPort(port: Electron.MessagePortMain): void {
    port.on('message', (event) => {
      const data = event.data as StageWireMessage;
      this.handleStageMessage(data);
    });
    port.start();

    // Subscribe to forwarded AppBus events according to CONTRACTS.md 2.3
    if (!this.unsubBus) {
      this.unsubBus = this.appBus.subscribe('*', (appEvent: ApplicationEvent) => {
        if ((forwardedAppBusTypes as readonly string[]).includes(appEvent.type)) {
          this.forwardEventToStage(appEvent as ForwardedAppBusEvent);
        }
      });
    }
  }

  public forwardEventToStage(appEvent: ForwardedAppBusEvent): void {
    if (!this.controlPortMain) return;

    const stageEventMsg = {
      v: 2 as const,
      id: generateUlid(),
      type: 'stage.event' as const,
      payload: {
        event: appEvent,
      },
    };

    try {
      this.controlPortMain.postMessage(stageEventMsg);
    } catch (err) {
      console.error('[Channels] Failed to send stage.event to control port:', err);
    }
  }

  private handleStageMessage(msg: unknown): void {
    if (!msg || typeof msg !== 'object') return;
    const wireMsg = msg as { type?: string; payload?: unknown };

    if (wireMsg.type === 'stage.hitTest') {
      const payload = wireMsg.payload as { overCharacter?: boolean };
      if (typeof payload?.overCharacter === 'boolean') {
        this.stageWindowManager.handleHitTest(payload.overCharacter);
      }
    } else if (wireMsg.type === 'stage.bounds') {
      const payload = wireMsg.payload as { screenRect?: { x: number; y: number; w: number; h: number } };
      if (payload?.screenRect) {
        this.displayManager?.handleCharacterBounds(payload.screenRect);
      }
    } else if (wireMsg.type && (MIRRORED_STAGE_TYPES as readonly string[]).includes(wireMsg.type)) {
      if (wireMsg.type === 'stage.drag' && this.displayManager) {
        const dragPayload = wireMsg.payload as {
          phase: 'start' | 'move' | 'end';
          screenX: number;
          screenY: number;
          vx: number;
          vy: number;
        };
        if (dragPayload?.phase) {
          this.displayManager.handleCharacterDrag(dragPayload);
        }
      }

      switch (wireMsg.type as MirroredType) {
        case 'stage.ready':
          this.appBus.publish(
            this.appBus.createEvent('stage', 'stage.ready', wireMsg.payload as AppBusPayloads['stage.ready']),
          );
          break;
        case 'stage.hit':
          this.appBus.publish(
            this.appBus.createEvent('stage', 'stage.hit', wireMsg.payload as AppBusPayloads['stage.hit']),
          );
          break;
        case 'stage.drag':
          this.appBus.publish(
            this.appBus.createEvent('stage', 'stage.drag', wireMsg.payload as AppBusPayloads['stage.drag']),
          );
          break;
        case 'stage.frameStats':
          this.appBus.publish(
            this.appBus.createEvent('stage', 'stage.frameStats', wireMsg.payload as AppBusPayloads['stage.frameStats']),
          );
          break;
        case 'stage.error':
          this.appBus.publish(
            this.appBus.createEvent('stage', 'stage.error', wireMsg.payload as AppBusPayloads['stage.error']),
          );
          break;
      }
    }
  }

  public destroy(): void {
    if (this.unsubBus) {
      this.unsubBus();
      this.unsubBus = undefined;
    }
    if (this.controlPortMain) {
      this.controlPortMain.close();
      this.controlPortMain = null;
    }
    if (this.audioPortVoice) {
      this.audioPortVoice.close();
      this.audioPortVoice = null;
    }
    if (this.micPortVoice) {
      this.micPortVoice.close();
      this.micPortVoice = null;
    }
  }
}
