import { execFile, execFileSync } from 'node:child_process';
import { promisify } from 'node:util';
import { z } from 'zod';
const execAsync = promisify(execFile);
const args = [
  '-NoProfile',
  '-Command',
  "$ErrorActionPreference='Stop'; ConvertTo-Json -InputObject @(Get-CimInstance Win32_Process | Select-Object Name,ProcessId,ParentProcessId,CommandLine) -Compress",
];
const processSchema = z.object({
  Name: z.string(),
  ProcessId: z.number(),
  ParentProcessId: z.number(),
  CommandLine: z.string().nullish(),
});
type HostProcess = z.infer<typeof processSchema>;
export function busyProcesses(
  processes: HostProcess[],
  allowedElectronRoot?: number,
  ownPid = process.pid,
): Array<{ Name: string; ProcessId: number; reason: string }> {
  const allowed = new Set<number>(allowedElectronRoot === undefined ? [] : [allowedElectronRoot]);
  for (let i = 0; i < processes.length; i++)
    for (const p of processes) if (allowed.has(p.ParentProcessId)) allowed.add(p.ProcessId);
  const ancestors = new Set<number>();
  let current: number | undefined = ownPid;
  while (current !== undefined && !ancestors.has(current)) {
    ancestors.add(current);
    current = processes.find((p) => p.ProcessId === current)?.ParentProcessId;
  }
  const namedHeavy = new Set([
    'blender.exe',
    'runner.worker.exe',
    'electron.exe',
    'playwright.exe',
    'cargo.exe',
    'rustc.exe',
    'nvcc.exe',
    'llama-bench.exe',
    'gpu-bench.exe',
    'gpu-probe.exe',
  ]);
  const worker = new Set(['python.exe', 'pythonw.exe', 'node.exe', 'uv.exe', 'mfa.exe']);
  const heavyCommand = (value: string): boolean => {
    const cmd = value.toLowerCase().replaceAll('\\', '/');
    return (
      [
        '/probe/',
        '/bench/',
        '/benchmark/',
        '/tests/',
        'run_all.py',
        'measure.ts',
        'measure.py',
        'benchmark',
        'gpu_probe',
        'gpu-probe',
        'pytest',
        'align_user.py',
        'context_references.py',
        'voice_sidecar.server',
        'mfa align',
      ].some((part) => cmd.includes(part)) ||
      (cmd.includes('playwright') &&
        (cmd.includes(' test') || cmd.includes('/test/') || cmd.includes('cli.js')))
    );
  };
  return processes.flatMap((p) => {
    if (allowed.has(p.ProcessId)) return [];
    if (namedHeavy.has(p.Name.toLowerCase()))
      return [{ Name: p.Name, ProcessId: p.ProcessId, reason: 'heavy-process' }];
    if (!ancestors.has(p.ProcessId) && worker.has(p.Name.toLowerCase())) {
      if (!p.CommandLine)
        return [{ Name: p.Name, ProcessId: p.ProcessId, reason: 'cannot-inspect-worker' }];
      if (heavyCommand(p.CommandLine))
        return [{ Name: p.Name, ProcessId: p.ProcessId, reason: 'active-test-or-model-work' }];
    }
    return [];
  });
}
function validate(out: string, allowedElectronRoot?: number): void {
  const busy = busyProcesses(z.array(processSchema).parse(JSON.parse(out)), allowedElectronRoot);
  // Never print other processes' command lines: they may contain private arguments.
  if (busy.length) throw new Error(`Quiet window unavailable: ${JSON.stringify(busy)}`);
}
export function assertQuiet(allowedElectronRoot?: number): void {
  if (process.platform !== 'win32') throw new Error('P4-01 measurement requires the Windows host');
  validate(
    execFileSync('pwsh', args, { encoding: 'utf8', windowsHide: true }),
    allowedElectronRoot,
  );
}
export async function assertQuietAsync(): Promise<void> {
  if (process.platform !== 'win32') throw new Error('P4-01 measurement requires the Windows host');
  const { stdout } = await execAsync('pwsh', args, {
    encoding: 'utf8',
    windowsHide: true,
    timeout: 5000,
  });
  validate(stdout);
}
