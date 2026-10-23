import { watchFile, unwatchFile } from 'node:fs';
import { join } from 'node:path';
import { readState, statePath } from '../../../../tools/codex-hooks/state.js';
import type { AppBus } from '../bus/index.js';
import { CodexSessionReceiver } from './receiver.js';

/** Runtime entry for a settings adapter. It never installs or changes Codex configuration. */
export class CodexSessionService {
  private readonly receiver: CodexSessionReceiver;
  private readonly file: string;
  private activePipe: string | undefined;
  private pending = Promise.resolve();
  private closed = false;

  constructor(bus: AppBus, stateDirectory: string) {
    this.file = statePath(stateDirectory);
    this.receiver = new CodexSessionReceiver(bus, async () => {
      const state = await readState(this.file).catch(() => undefined);
      return !this.closed && state?.enabled === true && state.pipe === this.activePipe;
    });
  }

  async start(): Promise<void> {
    watchFile(this.file, { persistent: false, interval: 250 }, this.onChange);
    await this.refresh();
  }

  private readonly onChange = (): void => {
    void this.refresh();
  };

  private refresh(): Promise<void> {
    this.pending = this.pending
      .then(async () => {
        const state = await readState(this.file).catch(() => undefined);
        const pipe = !this.closed && state?.enabled ? state.pipe : undefined;
        if (pipe === this.activePipe) return;
        await this.receiver.stop();
        this.activePipe = undefined;
        if (pipe) {
          await this.receiver.start(pipe);
          this.activePipe = pipe;
        }
      })
      .catch(() => {
        /* A conflicting endpoint stays disabled; no input is logged. */
      });
    return this.pending;
  }

  async stop(): Promise<void> {
    this.closed = true;
    unwatchFile(this.file, this.onChange);
    await this.pending;
    await this.receiver.stop();
    this.activePipe = undefined;
  }
}

interface Lifecycle {
  getPath(name: 'userData'): string;
  once(event: 'before-quit', callback: () => void): unknown;
}

// One module registration in the desktop bootstrap; missing settings mean disabled.
export function registerCodexSession(bus: AppBus, app: Lifecycle): void {
  const service = new CodexSessionService(bus, join(app.getPath('userData'), 'codex-session'));
  app.once('before-quit', () => {
    void service.stop();
  });
  void service.start();
}
