export type ProcessState =
  | 'stopped'
  | 'starting'
  | 'running'
  | 'crashed'
  | 'backing-off'
  | 'stopping';

export interface HealthCheckConfig {
  intervalMs: number;
  timeoutMs: number;
  maxConsecutiveFailures: number;
}

export interface ProcessConfig {
  name: string;
  isWorker: boolean;
  command: string;
  args: string[];
  cwd?: string | undefined;
  env?: Record<string, string> | undefined;
  placeholderCommand?: string | undefined;
  placeholderArgs?: string[] | undefined;
  autoRestart?: boolean | undefined;
  baseDelayMs?: number | undefined;
  maxDelayMs?: number | undefined;
  backoffFactor?: number | undefined;
  stableThresholdMs?: number | undefined;
  healthCheck?: HealthCheckConfig | undefined;
}

export interface ProcessStatus {
  name: string;
  state: ProcessState;
  pid?: number | undefined;
  restartCount: number;
  lastExitCode?: number | null | undefined;
  lastCrashTime?: number | undefined;
  isPlaceholder: boolean;
}
