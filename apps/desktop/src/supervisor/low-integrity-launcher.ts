import { spawn, type ChildProcess, type SpawnOptions } from 'node:child_process';

/**
 * Low-integrity launcher placeholder.
 * 
 * PLACEHOLDER: To be replaced by P5-05 low-integrity launcher.
 * In production Windows environment, P5-05 will launch worker processes with
 * Low Integrity Level (Low IL) token so OS restricts authority storage access.
 * Currently runs as standard process with low integrity environment flags.
 */
export function launchLowIntegrityProcess(
  command: string,
  args: string[] = [],
  options: SpawnOptions = {},
): ChildProcess {
  const env = {
    ...process.env,
    ...options.env,
    NEMU_INTEGRITY_LEVEL: 'low',
    NEMU_SANDBOX_WORKER: '1',
  };

  return spawn(command, args, {
    ...options,
    env,
    stdio: options.stdio ?? ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
  });
}
