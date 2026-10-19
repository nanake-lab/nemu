import { SupervisedProcess } from './supervised-process.js';
import { createProcessConfigs } from './process-config.js';
import type { ProcessConfig, ProcessStatus } from './types.js';

export class ProcessSupervisor {
  private processes: Map<string, SupervisedProcess> = new Map();

  constructor(customConfigs?: ProcessConfig[]) {
    const configs = customConfigs ?? createProcessConfigs();
    for (const config of configs) {
      this.processes.set(config.name, new SupervisedProcess(config));
    }
  }

  public getProcess(name: string): SupervisedProcess | undefined {
    return this.processes.get(name);
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
