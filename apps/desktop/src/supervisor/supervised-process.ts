import { spawn, type ChildProcess, type StdioOptions, exec } from 'node:child_process';
import { existsSync, appendFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
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
  private healthCheckTimer: NodeJS.Timeout | null = null;
  private lastHeartbeat = 0;
  private consecutiveFailures = 0;
  private isPlaceholder = false;
  private isIntentionallyStopped = false;
  private logFilePath: string;

  constructor(config: ProcessConfig, logDir?: string) {
    this.config = {
      autoRestart: true,
      baseDelayMs: 500,
      maxDelayMs: 30_000,
      backoffFactor: 2,
      stableThresholdMs: 5_000,
      ...config,
    };
    this.backoffDelay = this.config.baseDelayMs!;

    const logs = logDir ?? (process.env.NEMU_SCAFFOLD_USER_DATA
      ? join(process.env.NEMU_SCAFFOLD_USER_DATA, 'logs')
      : join(process.cwd(), 'logs'));
    try {
      if (!existsSync(logs)) {
        mkdirSync(logs, { recursive: true });
      }
    } catch {
      // Ignore directory creation failure
    }
    this.logFilePath = join(logs, `${this.config.name}.log`);
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

  public getChildProcess(): ChildProcess | null {
    return this.process;
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

    let cmd = this.config.command;
    let args = [...this.config.args];

    const isRunningAsNode = cmd === process.execPath;
    this.isPlaceholder = Boolean(
      isRunningAsNode &&
      args.length >= 2 &&
      args[0] === '-e' &&
      args[1]?.includes('setInterval'),
    );

    const isExplicitPath = cmd.includes('/') || cmd.includes('\\') || cmd.endsWith('.exe');
    if (isExplicitPath && !existsSync(cmd)) {
      if (this.config.placeholderCommand) {
        cmd = this.config.placeholderCommand;
        args = [...(this.config.placeholderArgs ?? [])];
        this.isPlaceholder = true;
      } else {
        cmd = process.execPath;
        args = ['-e', "setInterval(() => { console.log('HEARTBEAT'); }, 2000); console.log('HEARTBEAT');"];
        this.isPlaceholder = true;
      }
    }

    const stdio: StdioOptions = ['ignore', 'pipe', 'pipe'];
    const spawnEnv: Record<string, string> = {
      ...process.env as Record<string, string>,
      ...(this.config.env ?? {}),
      NEMU_PROCESS_NAME: this.config.name,
    };

    // If launching Node code via Electron binary, ensure Electron acts as Node
    if (cmd === process.execPath) {
      spawnEnv.ELECTRON_RUN_AS_NODE = '1';
    }

    const spawnOptions = {
      cwd: this.config.cwd,
      env: spawnEnv,
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
    this.lastHeartbeat = Date.now();
    this.consecutiveFailures = 0;
    this.state = 'running';

    if (this.process) {
      // Drain stdout and listen for heartbeat
      if (this.process.stdout) {
        this.process.stdout.on('data', (chunk: Buffer) => {
          const text = chunk.toString('utf-8');
          if (text.includes('HEARTBEAT')) {
            this.lastHeartbeat = Date.now();
            this.consecutiveFailures = 0;
          }
          try {
            appendFileSync(this.logFilePath, text, 'utf-8');
          } catch {
            // ignore
          }
        });
      }

      // Drain stderr
      if (this.process.stderr) {
        this.process.stderr.on('data', (chunk: Buffer) => {
          const text = chunk.toString('utf-8');
          try {
            appendFileSync(this.logFilePath, text, 'utf-8');
          } catch {
            // ignore
          }
        });
      }

      this.process.on('exit', (code) => {
        this.handleProcessExit(code);
      });

      this.process.on('error', (err) => {
        console.warn(`[Supervisor] Process ${this.config.name} error:`, err);
      });

      this.startHealthCheck();
    }
  }

  private startHealthCheck(): void {
    this.stopHealthCheck();
    if (!this.config.healthCheck) return;

    const { intervalMs, timeoutMs, maxConsecutiveFailures } = this.config.healthCheck;
    this.healthCheckTimer = setInterval(() => {
      if (this.state !== 'running' || !this.process) return;

      const elapsed = Date.now() - this.lastHeartbeat;
      if (elapsed > timeoutMs) {
        this.consecutiveFailures++;
        if (this.consecutiveFailures >= maxConsecutiveFailures) {
          console.warn(`[Supervisor] Process ${this.config.name} failed health check (no heartbeat for ${elapsed}ms). Killing and restarting.`);
          this.stopHealthCheck();
          const pid = this.process.pid;
          if (pid) {
            if (process.platform === 'win32') {
              exec(`taskkill /pid ${pid} /T /F`, () => {
                this.handleProcessCrash();
              });
              return;
            } else {
              try {
                this.process.kill('SIGKILL');
              } catch {
                // ignore
              }
            }
          }
          this.handleProcessCrash();
        }
      }
    }, intervalMs);
  }

  private stopHealthCheck(): void {
    if (this.healthCheckTimer) {
      clearInterval(this.healthCheckTimer);
      this.healthCheckTimer = null;
    }
  }

  private handleProcessExit(code: number | null): void {
    this.stopHealthCheck();
    this.lastExitCode = code;
    this.process = null;

    if (this.isIntentionallyStopped) {
      this.state = 'stopped';
      return;
    }

    this.handleProcessCrash();
  }

  private handleProcessCrash(): void {
    this.stopHealthCheck();
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
    this.stopHealthCheck();

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

      if (!pid) {
        finish();
        return;
      }

      if (process.platform === 'win32') {
        // Force kill full process tree immediately on Windows
        exec(`taskkill /pid ${pid} /T /F`, () => {
          finish();
        });
        setTimeout(finish, 1500);
      } else {
        const forceKillTimer = setTimeout(() => {
          try {
            proc.kill('SIGKILL');
          } catch {
            // ignore
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
