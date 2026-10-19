import { MessageChannelMain, type BrowserWindow, ipcMain } from 'electron';
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

export interface ChannelManagerOptions {
  appBus: AppBus;
  stageWindowManager: StageWindowManager;
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
  private controlPortMain: Electron.MessagePortMain | null = null;
  private unsubBus?: (() => void) | undefined = undefined;

  constructor(options: ChannelManagerOptions) {
    this.appBus = options.appBus;
    this.stageWindowManager = options.stageWindowManager;
    this.setupIpcFallback();
  }

  public establishChannels(stageWindow: BrowserWindow, panelWindow?: BrowserWindow): {
    audioPortVoice: Electron.MessagePortMain;
    micPortVoice: Electron.MessagePortMain;
  } {
    // 1. Control channel (Main <-> Stage)
    const controlChannel = new MessageChannelMain();
    this.controlPortMain = controlChannel.port1;
    this.setupControlPort(this.controlPortMain);

    // 2. Audio channel (Voice <-> Stage)
    const audioChannel = new MessageChannelMain();

    // 3. Microphone channel (Stage -> Voice)
    const micChannel = new MessageChannelMain();

    // Transfer ports to Stage window
    stageWindow.webContents.postMessage('nemu:channel:control', null, [controlChannel.port2]);
    stageWindow.webContents.postMessage('nemu:channel:audio', null, [audioChannel.port2]);
    stageWindow.webContents.postMessage('nemu:channel:mic', null, [micChannel.port1]);

    // 4. Panel bus channel (Panel <-> Main AppBus)
    if (panelWindow) {
      const panelBusChannel = new MessageChannelMain();
      this.appBus.connectPort('panel', panelBusChannel.port1);
      panelWindow.webContents.postMessage('nemu:channel:bus', null, [panelBusChannel.port2]);
    }

    return {
      audioPortVoice: audioChannel.port1,
      micPortVoice: micChannel.port2,
    };
  }

  private setupControlPort(port: Electron.MessagePortMain): void {
    port.on('message', (event) => {
      const data = event.data as StageWireMessage;
      this.handleStageMessage(data);
    });
    port.start();

    // Subscribe to forwarded AppBus events according to CONTRACTS.md 2.3:
    // voice.vad, voice.bargeIn, output.state, core.runState, desktop.dnd, desktop.idle
    this.unsubBus = this.appBus.subscribe('*', (appEvent: ApplicationEvent) => {
      if ((forwardedAppBusTypes as readonly string[]).includes(appEvent.type)) {
        this.forwardEventToStage(appEvent as ForwardedAppBusEvent);
      }
    });
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
    } else if (wireMsg.type && (MIRRORED_STAGE_TYPES as readonly string[]).includes(wireMsg.type)) {
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

  private setupIpcFallback(): void {
    // Also listen on IPC for hit testing and stage events if renderer posts via IPC
    ipcMain.on('nemu:stage:hitTest', (_event, payload: { overCharacter: boolean }) => {
      if (typeof payload?.overCharacter === 'boolean') {
        this.stageWindowManager.handleHitTest(payload.overCharacter);
      }
    });

    ipcMain.on('nemu:stage:event', (_event, msg: { type: string; payload: unknown }) => {
      this.handleStageMessage(msg);
    });
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
  }
}
