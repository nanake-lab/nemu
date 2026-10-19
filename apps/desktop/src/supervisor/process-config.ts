import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { ProcessConfig } from './types.js';
import type { DesktopSettings } from '../settings/migrations.js';

export interface ProcessTargetSpec {
  name: string;
  isWorker: boolean;
  command: string;
  args: string[];
  cwd?: string;
  entryCheckFile?: string;
  minFileSize?: number;
}

const PLACEHOLDER_SCRIPT = "setInterval(() => { console.log('HEARTBEAT'); }, 2000); console.log('HEARTBEAT');";

function resolvePythonCommand(projectDir: string): string {
  const isWin = process.platform === 'win32';
  const venvPython = isWin
    ? resolve(projectDir, '.venv/Scripts/python.exe')
    : resolve(projectDir, '.venv/bin/python');
  if (existsSync(venvPython)) {
    return venvPython;
  }
  return 'python';
}

export const REQUIRED_PROCESSES: ProcessTargetSpec[] = [
  {
    name: 'nemu-core.exe',
    isWorker: false,
    command: resolve('core/target/release/nemu-core.exe'),
    args: [],
    entryCheckFile: resolve('core/target/release/nemu-core.exe'),
  },
  {
    name: 'irl-worker',
    isWorker: true,
    command: resolvePythonCommand(resolve('workers/irl')),
    args: ['src/main.py'],
    cwd: resolve('workers/irl'),
    entryCheckFile: resolve('workers/irl/src/main.py'),
  },
  {
    name: 'drive-worker',
    isWorker: true,
    command: resolvePythonCommand(resolve('workers/drive')),
    args: ['src/main.py'],
    cwd: resolve('workers/drive'),
    entryCheckFile: resolve('workers/drive/src/main.py'),
  },
  {
    name: 'model-adapter',
    isWorker: true,
    command: resolvePythonCommand(resolve('workers/model-adapter')),
    args: ['src/main.py'],
    cwd: resolve('workers/model-adapter'),
    entryCheckFile: resolve('workers/model-adapter/src/main.py'),
  },
  {
    name: 'voice-sidecar',
    isWorker: false,
    command: resolvePythonCommand(resolve('services/voice-sidecar')),
    args: ['src/main.py'],
    cwd: resolve('services/voice-sidecar'),
    entryCheckFile: resolve('services/voice-sidecar/src/main.py'),
  },
  {
    name: 'codex-bridge',
    isWorker: false,
    command: process.execPath,
    args: [resolve('executors/codex-bridge/dist/index.js')],
    cwd: resolve('executors/codex-bridge'),
    entryCheckFile: resolve('executors/codex-bridge/dist/index.js'),
    minFileSize: 200, // Empty scaffold file is < 100 bytes
  },
  {
    name: 'office-connector',
    isWorker: false,
    command: resolve('executors/office-connector/bin/Release/net10.0/OfficeConnector.exe'),
    args: [],
    entryCheckFile: resolve('executors/office-connector/bin/Release/net10.0/OfficeConnector.exe'),
  },
];

export function isProcessTargetReady(spec: ProcessTargetSpec): boolean {
  if (!spec.entryCheckFile) return true;
  const fullPath = spec.entryCheckFile;
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

export function createProcessConfigs(
  specs: ProcessTargetSpec[] = REQUIRED_PROCESSES,
  settings?: DesktopSettings,
): ProcessConfig[] {
  const configs: ProcessConfig[] = [];

  for (const spec of specs) {
    const processSetting = settings?.supervisor?.processes?.[spec.name];
    if (processSetting && processSetting.enabled === false) {
      // Explicitly disabled in settings
      continue;
    }

    const customCommand = processSetting?.customPath;
    const isReady = customCommand ? existsSync(customCommand) : isProcessTargetReady(spec);

    const command = customCommand ?? (isReady ? spec.command : process.execPath);
    const args = isReady ? spec.args : ['-e', PLACEHOLDER_SCRIPT];

    configs.push({
      name: spec.name,
      isWorker: spec.isWorker,
      command,
      args,
      cwd: spec.cwd,
      placeholderCommand: process.execPath,
      placeholderArgs: ['-e', PLACEHOLDER_SCRIPT],
      autoRestart: true,
      env: {
        ELECTRON_RUN_AS_NODE: '1',
      },
      healthCheck: {
        intervalMs: 3000,
        timeoutMs: 5000,
        maxConsecutiveFailures: 2,
      },
    });
  }

  return configs;
}
