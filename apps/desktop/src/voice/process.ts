import { asrModelDirectory } from '@nemu/voice';
import { utilityProcess, type UtilityProcess, type MessagePortMain } from 'electron';
import { appBusEventSchema, type ControlCommand } from '@nemu/contracts';
import { join } from 'node:path';
import { existsSync } from 'node:fs';
import type { AppBus } from '../bus/bus.js';
import type { DesktopLogger } from '../main/logger.js';
export class VoiceProcess {
  private child: UtilityProcess | undefined;
  private ready = false;
  private stopped = false;
  private attempts = 0;
  private restart: ReturnType<typeof setTimeout> | undefined;
  private onExit: (() => void) | undefined;
  onUnexpectedExit(callback: () => void): void {
    this.onExit = callback;
  }
  private pending: [MessagePortMain, MessagePortMain] | undefined;
  private command: ControlCommand | undefined;
  constructor(
    private bus: AppBus,
    private logger: DesktopLogger,
  ) {}
  isReady(): boolean {
    return this.ready;
  }
  start(): void {
    if (this.stopped || this.child) return;
    const models = asrModelDirectory();
    if (!existsSync(join(models, 'model.int8.onnx'))) {
      this.logger.warn('ASR models absent; run pnpm asr:models to enable local voice');
      return;
    }
    this.child = utilityProcess.fork(join(import.meta.dirname, 'voice.js'), [], {
      serviceName: 'Nemu Voice',
      env: {
        ...process.env,
        NEMU_ASR_MODELS: models,
        ...(process.env.NEMU_ASR_LOCK ? { NEMU_ASR_LOCK: process.env.NEMU_ASR_LOCK } : {}),
      },
    });
    this.child.on('message', (raw: unknown) => {
      if (raw === 'ready') {
        this.ready = true;
        this.transfer();
        return;
      }
      const parsed = appBusEventSchema.safeParse(raw);
      if (
        parsed.success &&
        parsed.data.source === 'voice' &&
        [
          'voice.vad',
          'voice.bargeIn',
          'voice.recognized',
          'metrics.latency',
          'stage.error',
        ].includes(parsed.data.type)
      )
        this.bus.publish(parsed.data);
    });
    this.child.on('exit', (code) => {
      this.ready = false;
      this.child = undefined;
      this.logger.warn('Voice utility exited', { code });
      if (!this.stopped) {
        this.command = undefined;
        this.onExit?.();
        if (this.attempts < 3)
          this.restart = setTimeout(() => this.start(), 1000 * 2 ** this.attempts++);
      }
    });
  }
  attach(mic: MessagePortMain, audio: MessagePortMain): void {
    this.pending?.forEach((p) => p.close());
    this.pending = [mic, audio];
    this.transfer();
  }
  private transfer(): void {
    if (!this.ready || !this.child || !this.pending) return;
    this.child.postMessage(null, this.pending);
    this.pending = undefined;
    if (this.command) this.child.postMessage(this.command);
  }
  send(command: ControlCommand): void {
    this.command = command;
    if (this.ready) this.child?.postMessage(command);
  }
  stop(): void {
    this.stopped = true;
    if (this.restart) clearTimeout(this.restart);
    this.pending?.forEach((p) => p.close());
    this.pending = undefined;
    this.child?.kill();
    this.child = undefined;
    this.ready = false;
  }
}
