import { execFile, execFileSync } from 'node:child_process';
import { promisify } from 'node:util';
import { z } from 'zod';
const execAsync = promisify(execFile);
const args = [
  '-NoProfile',
  '-Command',
  "$ErrorActionPreference='Stop'; ConvertTo-Json -InputObject @(Get-CimInstance Win32_Process | Select-Object Name,ProcessId,ParentProcessId) -Compress",
];
const processSchema = z.object({
  Name: z.string(),
  ProcessId: z.number(),
  ParentProcessId: z.number(),
});
function validate(out: string, allowedElectronRoot?: number): void {
  const processes = z.array(processSchema).parse(JSON.parse(out));
  const allowed = new Set<number>(allowedElectronRoot === undefined ? [] : [allowedElectronRoot]);
  for (let i = 0; i < processes.length; i++)
    for (const p of processes) if (allowed.has(p.ParentProcessId)) allowed.add(p.ProcessId);
  const busy = processes.filter(
    (p) =>
      /^(blender|Runner\.Worker|electron|playwright)\.exe$/i.test(p.Name) &&
      !(p.Name.toLowerCase() === 'electron.exe' && allowed.has(p.ProcessId)),
  );
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
