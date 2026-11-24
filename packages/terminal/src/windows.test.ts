import { createHash, randomBytes } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { expect, test, vi } from 'vitest';
import { windowsIdentity } from './windows.js';

const launched = vi.hoisted(() => vi.fn());
vi.mock('node:child_process', async (importOriginal) => {
  const actual = await importOriginal<typeof import('node:child_process')>();
  const execFile = (...args: Parameters<typeof actual.execFile>) => {
    launched(args[1], args[2]);
    return actual.execFile(...args);
  };
  return { ...actual, execFile };
});

test.skipIf(process.platform !== 'win32')(
  'current-user DPAPI round trip keeps the token out of argv, environment overrides, and errors',
  async () => {
    const directory = mkdtempSync(join(tmpdir(), 'nemu-p6-dpapi-'));
    const tokenPath = join(directory, '终端.token');
    const token = randomBytes(32);
    const fingerprint = (bytes: Uint8Array) => createHash('sha256').update(bytes).digest('hex');
    const powershell = join(
      process.env.SystemRoot!,
      'System32',
      'WindowsPowerShell',
      'v1.0',
      'powershell.exe',
    );
    try {
      const encrypted = execFileSync(
        powershell,
        [
          '-NoProfile',
          '-NonInteractive',
          '-Command',
          `
      $ErrorActionPreference = 'Stop'
      Add-Type -AssemblyName System.Security
      $inputBytes = [IO.MemoryStream]::new()
      [Console]::OpenStandardInput().CopyTo($inputBytes)
      $encrypted = [Security.Cryptography.ProtectedData]::Protect(
        $inputBytes.ToArray(), $null, [Security.Cryptography.DataProtectionScope]::CurrentUser)
      [Console]::OpenStandardOutput().Write($encrypted, 0, $encrypted.Length)
    `,
        ],
        { input: token, windowsHide: true },
      );
      writeFileSync(tokenPath, encrypted);
      const identity = await windowsIdentity(tokenPath);
      try {
        expect(fingerprint(identity.token)).toBe(fingerprint(token));
        expect(identity.installationId).toBe('desktop');
        expect(identity.pipe).toMatch(/nemu-core-S-1-(?:\d+-)*\d+$/);
        expect(launched).toHaveBeenCalledTimes(1);
        const [args, options] = launched.mock.calls[0]!;
        expect(JSON.stringify(args)).not.toContain(token.toString('hex'));
        expect(JSON.stringify(args)).not.toContain(token.toString('base64'));
        expect(options.env).toBeUndefined();
        expect(options.windowsHide).toBe(true);
        await expect(windowsIdentity(join(directory, 'absent.token'))).rejects.toThrow(
          /^Cannot read or decrypt the Windows terminal token$/,
        );
      } finally {
        identity.token.fill(0);
      }
    } finally {
      token.fill(0);
      rmSync(directory, { recursive: true, force: true });
    }
  },
  15_000,
);
