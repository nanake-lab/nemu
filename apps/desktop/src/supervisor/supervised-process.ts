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
  private currentGeneration = 0;
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
    if (this.isIntentionallyStopped) return;

    this.clearBackoffTimer();
    this.stopHealthCheck();

    // Guard against duplicate active child processes
    if (this.process) {
      const existing = this.process;
      this.process = null;
      if (existing.pid) {
        if (process.platform === 'win32') {
          exec(`taskkill /pid ${existing.pid} /T /F`);
        } else {
          try {
            existing.kill('SIGKILL');
          } catch {
            // ignore
          }
        }
      }
    }

    const generation = ++this.currentGeneration;
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
      this.handleProcessCrash(generation);
      return;
    }

    this.startedAt = Date.now();
    this.lastHeartbeat = Date.now();
    this.consecutiveFailures = 0;
    this.state = 'running';

    const child = this.process;
    if (child) {
      // Drain stdout and listen for heartbeat
      if (child.stdout) {
        child.stdout.on('data', (chunk: Buffer) => {
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
      if (child.stderr) {
        child.stderr.on('data', (chunk: Buffer) => {
          const text = chunk.toString('utf-8');
          try {
            appendFileSync(this.logFilePath, text, 'utf-8');
          } catch {
            // ignore
          }
        });
      }

      child.on('exit', (code) => {
        this.handleProcessExit(generation, code);
      });

      child.on('error', (err) => {
        console.warn(`[Supervisor] Process ${this.config.name} error:`, err);
      });

      this.startHealthCheck(generation);
    }
  }

  private startHealthCheck(generation: number): void {
    this.stopHealthCheck();
    if (!this.config.healthCheck) return;

    const probeType = this.config.healthCheck.type ?? 'stdout-heartbeat';
    if (probeType === 'none') return;

    const intervalMs = this.config.healthCheck.intervalMs ?? 3000;
    const timeoutMs = this.config.healthCheck.timeoutMs ?? 5000;
    const maxFailures = this.config.healthCheck.maxConsecutiveFailures ?? 2;

    this.healthCheckTimer = setInterval(async () => {
      if (generation !== this.currentGeneration || this.state !== 'running' || !this.process) {
        return;
      }

      let isHealthy = true;
      if (probeType === 'stdout-heartbeat') {
        const elapsed = Date.now() - this.lastHeartbeat;
        if (elapsed > timeoutMs) {
          isHealthy = false;
        }
      } else if (probeType === 'http-get') {
        const endpoint = this.config.healthCheck?.endpoint;
        if (endpoint) {
          try {
            const res = await fetch(endpoint, {
              signal: AbortSignal.timeout(timeoutMs),
            });
            isHealthy = res.ok;
          } catch {
            isHealthy = false;
          }
        }
      }

      if (generation !== this.currentGeneration || this.state !== 'running' || !this.process) {
        return;
      }

      if (isHealthy) {
        this.consecutiveFailures = 0;
      } else {
        this.consecutiveFailures++;
        if (this.consecutiveFailures >= maxFailures) {
          console.warn(`[Supervisor] Process ${this.config.name} failed health check (${probeType}). Killing process tree for restart.`);
          this.stopHealthCheck();
          const procToKill = this.process;
          const pid = procToKill?.pid;
          if (pid) {
            if (process.platform === 'win32') {
              exec(`taskkill /pid ${pid} /T /F`, (err) => {
                if (err && procToKill) {
                  try {
                    procToKill.kill('SIGKILL');
                  } catch {
                    // ignore
                  }
                }
              });
            } else {
              try {
                procToKill.kill('SIGKILL');
              } catch {
                // ignore
              }
            }
          }
          // Note: process termination naturally emits 'exit', which calls handleProcessExit(generation, code).
          // We intentionally do NOT call handleProcessCrash() here to avoid duplicate restarts or race conditions.
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

  private handleProcessExit(generation: number, code: number | null): void {
    // If exit event belongs to an older generation, ignore it to prevent race conditions
    if (generation !== this.currentGeneration) {
      return;
    }

    this.stopHealthCheck();
    this.lastExitCode = code;
    this.process = null;

    if (this.isIntentionallyStopped) {
      this.state = 'stopped';
      return;
    }

    this.handleProcessCrash(generation);
  }

  private handleProcessCrash(generation?: number): void {
    if (generation !== undefined && generation !== this.currentGeneration) {
      return;
    }

    this.stopHealthCheck();
    this.clearBackoffTimer();
    this.state = 'crashed';
    this.lastCrashTime = Date.now();

    // Check if process was stable before crashing
    if (this.startedAt && Date.now() - this.startedAt >= this.config.stableThresholdMs!) {
      this.backoffDelay = this.config.baseDelayMs!;
    }

    if (!this.config.autoRestart || this.isIntentionallyStopped) {
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
      if (!this.isIntentionallyStopped && (generation === undefined || generation === this.currentGeneration)) {
        this.spawnChild();
      }
    }, currentDelay);
  }

  public async stop(): Promise<void> {
    this.isIntentionallyStopped = true;
    this.currentGeneration++;
    this.clearBackoffTimer();
    this.stopHealthCheck();

    const proc = this.process;
    this.process = null;

    if (!proc || this.state === 'stopped') {
      this.state = 'stopped';
      return;
    }

    this.state = 'stopping';
    const pid = proc.pid;

    return new Promise<void>((resolve) => {
      let resolved = false;
      const finish = () => {
        if (!resolved) {
          resolved = true;
          this.state = 'stopped';
          resolve();
        }
      };

      if (!pid) {
        finish();
        return;
      }

      if (process.platform === 'win32') {
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
