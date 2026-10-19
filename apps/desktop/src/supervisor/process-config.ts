import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { ProcessConfig } from './types.js';

export interface ProcessTargetSpec {
  name: string;
  isWorker: boolean;
  command: string;
  args: string[];
  entryCheckFile?: string;
  minFileSize?: number;
}

export const REQUIRED_PROCESSES: ProcessTargetSpec[] = [
  {
    name: 'nemu-core.exe',
    isWorker: false,
    command: 'core/target/release/nemu-core.exe',
    args: [],
    entryCheckFile: 'core/target/release/nemu-core.exe',
  },
  {
    name: 'irl-worker',
    isWorker: true,
    command: 'python',
    args: ['workers/irl/main.py'],
    entryCheckFile: 'workers/irl/main.py',
  },
  {
    name: 'drive-worker',
    isWorker: true,
    command: 'python',
    args: ['workers/drive/main.py'],
    entryCheckFile: 'workers/drive/main.py',
  },
  {
    name: 'model-adapter',
    isWorker: true,
    command: 'python',
    args: ['workers/model-adapter/main.py'],
    entryCheckFile: 'workers/model-adapter/main.py',
  },
  {
    name: 'voice-sidecar',
    isWorker: false,
    command: 'python',
    args: ['services/voice-sidecar/main.py'],
    entryCheckFile: 'services/voice-sidecar/main.py',
  },
  {
    name: 'codex-bridge',
    isWorker: false,
    command: process.execPath,
    args: ['executors/codex-bridge/dist/index.js'],
    entryCheckFile: 'executors/codex-bridge/dist/index.js',
    minFileSize: 200, // Empty scaffold file is < 100 bytes
  },
  {
    name: 'office-connector',
    isWorker: false,
    command: 'executors/office-connector/bin/Release/net10.0/OfficeConnector.exe',
    args: [],
    entryCheckFile: 'executors/office-connector/bin/Release/net10.0/OfficeConnector.exe',
  },
];

export function isProcessTargetReady(spec: ProcessTargetSpec): boolean {
  if (!spec.entryCheckFile) return true;
  const fullPath = resolve(process.cwd(), spec.entryCheckFile);
  if (!existsSync(fullPath)) return false;

  if (spec.minFileSize) {
    try {
      const content = readFileSync(fullPath);
      if (content.length < spec.minFileSize) return false;
    } catch {
      return false;
    }
  }

  return true;
}

export function createProcessConfigs(specs: ProcessTargetSpec[] = REQUIRED_PROCESSES): ProcessConfig[] {
  return specs.map((spec) => {
    const isReady = isProcessTargetReady(spec);

    return {
      name: spec.name,
      isWorker: spec.isWorker,
      command: isReady ? spec.command : process.execPath,
      args: isReady ? spec.args : ['-e', 'setInterval(() => {}, 1000)'],
      placeholderCommand: process.execPath,
      placeholderArgs: ['-e', 'setInterval(() => {}, 1000)'],
      autoRestart: true,
    };
  });
}
