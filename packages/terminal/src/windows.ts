import { execFile } from 'node:child_process';
import { join } from 'node:path';

// The path crosses stdin and the token crosses private binary stdout.
// Keep stdout binary so the clear token is never turned into a JS string.
const readIdentity = String.raw`
$ErrorActionPreference = 'Stop'
[Console]::InputEncoding = [Text.Encoding]::UTF8
Add-Type -AssemblyName System.Security
$path = [Console]::ReadLine()
$sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$ciphertext = [IO.File]::ReadAllBytes($path)
$token = [Security.Cryptography.ProtectedData]::Unprotect(
  $ciphertext, $null, [Security.Cryptography.DataProtectionScope]::CurrentUser)
try {
  $stdout = [Console]::OpenStandardOutput()
  $prefix = [Text.Encoding]::ASCII.GetBytes($sid + [char]10)
  $stdout.Write($prefix, 0, $prefix.Length)
  $stdout.Write($token, 0, $token.Length)
  $stdout.Flush()
} finally {
  [Array]::Clear($token, 0, $token.Length)
}
`;

export interface TerminalIdentity {
  pipe: string;
  installationId: string;
  token: Buffer;
}

export async function windowsIdentity(
  tokenPath?: string,
  signal?: AbortSignal,
): Promise<TerminalIdentity> {
  const { SystemRoot, LOCALAPPDATA } = process.env;
  if (process.platform !== 'win32' || !SystemRoot || (!tokenPath && !LOCALAPPDATA)) {
    throw new Error('Terminal authentication requires Windows and a local user profile');
  }
  const path = tokenPath ?? join(LOCALAPPDATA!, 'Nemu', 'terminal.token');
  const executable = join(SystemRoot, 'System32', 'WindowsPowerShell', 'v1.0', 'powershell.exe');
  const result = await new Promise<Buffer>((resolve, reject) => {
    const child = execFile(
      executable,
      ['-NoProfile', '-NonInteractive', '-Command', readIdentity],
      { encoding: 'buffer', windowsHide: true, timeout: 5000, maxBuffer: 4096, signal },
      (error, stdout) => {
        if (!error) resolve(stdout);
        else {
          stdout.fill(0);
          // execFile errors can include process output; keep only fixed text.
          reject(new Error('Cannot read or decrypt the Windows terminal token'));
        }
      },
    );
    child.stdin?.on('error', () => {});
    child.stdin?.end(path + '\n', 'utf8');
  });
  const line = result.indexOf(10);
  const sid = line >= 0 ? result.toString('ascii', 0, line) : '';
  if (!/^S-1-(?:\d+-)*\d+$/.test(sid) || result.length - line - 1 !== 32) {
    result.fill(0);
    throw new Error('Invalid Windows terminal identity');
  }
  return {
    pipe: `\\\\.\\pipe\\nemu-core-${sid}`,
    // HOST-01 names the installation "desktop"; the user SID scopes the pipe.
    installationId: 'desktop',
    token: result.subarray(line + 1),
  };
}
