import { spawn, type ChildProcess, type StdioOptions, exec } from 'node:child_process';
import { existsSync } from 'node:fs';
import { launchLowIntegrityProcess } from './low-integrity-launcher.js';
import type { ProcessConfig, ProcessState, ProcessStatus } from './types.js';

export class SupervisedProcess {
  public readonly config: ProcessConfig;
  private process: ChildProcess | null = null;
  private state: ProcessState = 'stopped';
  private restartCount = 0;
  private lastExitCode: number | null | undefined = undefined;
  private lastCrashTime?: number | undefined = undefined;
  private startedAt?: number | undefined = undefined;
  private backoffDelay: number;
  private backoffTimer: NodeJS.Timeout | null = null;
  private isPlaceholder = false;
  private isIntentionallyStopped = false;

  constructor(config: ProcessConfig) {
    this.config = {
      autoRestart: true,
      baseDelayMs: 500,
      maxDelayMs: 30_000,
      backoffFactor: 2,
      stableThresholdMs: 5_000,
      ...config,
    };
    this.backoffDelay = this.config.baseDelayMs!;
  }

  public getStatus(): ProcessStatus {
    return {
      name: this.config.name,
      state: this.state,
      pid: this.process?.pid,
      restartCount: this.restartCount,
      lastExitCode: this.lastExitCode,
      lastCrashTime: this.lastCrashTime,
      isPlaceholder: this.isPlaceholder,
    };
  }

  public getBackoffDelay(): number {
    return this.backoffDelay;
  }

  public async start(): Promise<void> {
    if (this.state === 'running' || this.state === 'starting') {
      return;
    }
    this.isIntentionallyStopped = false;
    this.clearBackoffTimer();
    this.spawnChild();
  }

  private spawnChild(): void {
    this.state = 'starting';

    // Resolve command: if command is a file path and does not exist, use placeholder
    let cmd = this.config.command;
    let args = [...this.config.args];
    this.isPlaceholder = Boolean(
      cmd === process.execPath &&
      args.length >= 2 &&
      args[0] === '-e' &&
      args[1]?.includes('setInterval'),
    );

    // Check if the primary command exists or is a path
    const isExplicitPath = cmd.includes('/') || cmd.includes('\\') || cmd.endsWith('.exe');
    if (isExplicitPath && !existsSync(cmd)) {
      if (this.config.placeholderCommand) {
        cmd = this.config.placeholderCommand;
        args = [...(this.config.placeholderArgs ?? [])];
        this.isPlaceholder = true;
      } else {
        // Fallback default placeholder: node process running idle
        cmd = process.execPath;
        args = ['-e', 'setInterval(() => {}, 1000)'];
        this.isPlaceholder = true;
      }
    }

    const stdio: StdioOptions = ['ignore', 'pipe', 'pipe'];
    const spawnOptions = {
      cwd: this.config.cwd,
      env: {
        ...process.env,
        ...(this.config.env ?? {}),
        NEMU_PROCESS_NAME: this.config.name,
      },
      stdio,
      windowsHide: true,
    };

    try {
      if (this.config.isWorker) {
        this.process = launchLowIntegrityProcess(cmd, args, spawnOptions);
      } else {
        this.process = spawn(cmd, args, spawnOptions);
      }
    } catch (error) {
      console.error(`[Supervisor] Failed to spawn ${this.config.name}:`, error);
      this.handleProcessCrash();
      return;
    }

    this.startedAt = Date.now();
    this.state = 'running';

    if (this.process) {
      this.process.on('exit', (code) => {
        this.handleProcessExit(code);
      });

      this.process.on('error', (err) => {
        console.warn(`[Supervisor] Process ${this.config.name} error:`, err);
      });
    }
  }

  private handleProcessExit(code: number | null): void {
    this.lastExitCode = code;
    this.process = null;

    if (this.isIntentionallyStopped) {
      this.state = 'stopped';
      return;
    }

    this.handleProcessCrash();
  }

  private handleProcessCrash(): void {
    this.state = 'crashed';
    this.lastCrashTime = Date.now();

    // Check if process was stable before crashing
    if (this.startedAt && Date.now() - this.startedAt >= this.config.stableThresholdMs!) {
      this.backoffDelay = this.config.baseDelayMs!;
    }

    if (!this.config.autoRestart) {
      this.state = 'stopped';
      return;
    }

    this.restartCount++;
    this.state = 'backing-off';

    const currentDelay = this.backoffDelay;
    // Calculate next backoff delay
    this.backoffDelay = Math.min(
      this.backoffDelay * this.config.backoffFactor!,
      this.config.maxDelayMs!,
    );

    this.backoffTimer = setTimeout(() => {
      this.backoffTimer = null;
      if (!this.isIntentionallyStopped) {
        this.spawnChild();
      }
    }, currentDelay);
  }

  public async stop(): Promise<void> {
    this.isIntentionallyStopped = true;
    this.clearBackoffTimer();

    if (!this.process || this.state === 'stopped') {
      this.state = 'stopped';
      return;
    }

    this.state = 'stopping';
    const proc = this.process;
    const pid = proc.pid;

    return new Promise<void>((resolve) => {
      let resolved = false;
      const finish = () => {
        if (!resolved) {
          resolved = true;
          this.process = null;
          this.state = 'stopped';
          resolve();
        }
      };

      const forceKillTimer = setTimeout(() => {
        if (pid) {
          try {
            if (process.platform === 'win32') {
              exec(`taskkill /pid ${pid} /T /F`, () => finish());
              return;
            } else {
              proc.kill('SIGKILL');
            }
          } catch {
            // ignore
          }
        }
        finish();
      }, 1500);

      proc.once('exit', () => {
        clearTimeout(forceKillTimer);
        finish();
      });

      try {
        proc.kill('SIGTERM');
      } catch {
        clearTimeout(forceKillTimer);
        finish();
      }
    });
  }

  private clearBackoffTimer(): void {
    if (this.backoffTimer) {
      clearTimeout(this.backoffTimer);
      this.backoffTimer = null;
    }
  }
}
