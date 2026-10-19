import { SupervisedProcess } from './supervised-process.js';
import { createProcessConfigs } from './process-config.js';
import type { ProcessConfig, ProcessStatus } from './types.js';
import type { DesktopSettings } from '../settings/migrations.js';

export class ProcessSupervisor {
  private processes: Map<string, SupervisedProcess> = new Map();
  private logDir?: string | undefined;

  constructor(options?: { configs?: ProcessConfig[]; settings?: DesktopSettings; logDir?: string } | ProcessConfig[]) {
    let configs: ProcessConfig[];
    let logDir: string | undefined;

    if (Array.isArray(options)) {
      configs = options;
    } else {
      logDir = options?.logDir;
      configs = options?.configs ?? createProcessConfigs(undefined, options?.settings);
    }

    this.logDir = logDir;
    for (const config of configs) {
      this.processes.set(config.name, new SupervisedProcess(config, logDir));
    }
  }

  public addProcess(config: ProcessConfig): SupervisedProcess {
    const proc = new SupervisedProcess(config, this.logDir);
    this.processes.set(config.name, proc);
    return proc;
  }

  public getProcess(name: string): SupervisedProcess | undefined {
    return this.processes.get(name);
  }

  public getProcesses(): SupervisedProcess[] {
    return Array.from(this.processes.values());
  }

  public getRunningPids(): number[] {
    const pids: number[] = [];
    for (const proc of this.processes.values()) {
      const pid = proc.getStatus().pid;
      if (pid) pids.push(pid);
    }
    return pids;
  }

  public getStatus(): Record<string, ProcessStatus> {
    const result: Record<string, ProcessStatus> = {};
    for (const [name, proc] of this.processes.entries()) {
      result[name] = proc.getStatus();
    }
    return result;
  }

  public async startAll(): Promise<void> {
    const startPromises: Array<Promise<void>> = [];
    for (const proc of this.processes.values()) {
      startPromises.push(proc.start());
    }
    await Promise.all(startPromises);
  }

  public async stopAll(): Promise<void> {
    const stopPromises: Array<Promise<void>> = [];
    for (const proc of this.processes.values()) {
      stopPromises.push(proc.stop());
    }
    await Promise.all(stopPromises);
  }
}
