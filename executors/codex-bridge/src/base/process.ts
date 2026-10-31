import { spawn, execFile } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { promisify } from 'node:util';
import { once } from 'node:events';
import { z } from 'zod';
import { isolatedEnvironment, strictArguments } from './isolation.js';
import type { Transport } from './rpc.js';
import { claimHome } from './home.js';

export interface LaunchOptions {
  executable: string; lockFile: string; codeHome: string; ownedRoot: string;
  cwd: string; catalogPath: string; maxFrameBytes: number; timeoutMs: number;
}

export async function startProcess(options: LaunchOptions): Promise<Transport> {
  if (!Number.isSafeInteger(options.maxFrameBytes) || options.maxFrameBytes < 1 ||
    !Number.isSafeInteger(options.timeoutMs) || options.timeoutMs < 1 || options.timeoutMs > 2_147_483_647) throw new Error('launch_budget');
  const lock = z.object({ codex: z.object({ version: z.string().regex(/^\d+\.\d+\.\d+$/) }) }).parse(
    JSON.parse(await readFile(options.lockFile, 'utf8')));
  const env = isolatedEnvironment(process.env, options.codeHome);
  const version = await promisify(execFile)(options.executable, ['--version'], { env, timeout: options.timeoutMs, windowsHide: true });
  if (version.stdout.trim() !== `codex-cli ${lock.codex.version}`) throw new Error('codex_version_mismatch');
  const claim = await claimHome(options.codeHome, options.ownedRoot);
  try {
    const child = spawn(options.executable, [...strictArguments, '-c', `model_catalog_json=${JSON.stringify(options.catalogPath)}`, 'app-server', '--stdio'], {
      cwd: options.cwd, env: { ...env, CODEX_HOME: claim.home }, stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true,
    });
    child.stderr.resume(); // Never copy potentially sensitive app-server diagnostics into ordinary logs.
    let ended = false;
    let closing: Promise<void> | null = null;
    let failure = false;
    let buffer = Buffer.alloc(0);
    const readers = new Set<(line: string) => void>();
    const closers = new Set<() => void>();
    const fail = (): void => { failure = true; for (const close of closers) close(); };
    child.on('error', fail);
    child.stdin.on('error', fail);
    child.on('exit', () => { ended = true; fail(); });
    child.stdout.on('data', (chunk: Buffer) => {
      if (failure) return;
      buffer = Buffer.concat([buffer, chunk]);
      let end: number;
      while ((end = buffer.indexOf(10)) >= 0) {
        if (end > options.maxFrameBytes) { child.kill(); fail(); return; }
        let line: string;
        try { line = new TextDecoder('utf-8', { fatal: true }).decode(buffer.subarray(0, end)); }
        catch { child.kill(); fail(); return; }
        buffer = buffer.subarray(end + 1);
        for (const reader of readers) reader(line);
      }
      if (buffer.length > options.maxFrameBytes) { child.kill(); fail(); }
    });
    return {
      send: (line) => {
        if (failure || !child.stdin.writable) throw new Error('app_server_disconnected');
        if (Buffer.byteLength(line) > options.maxFrameBytes) throw new Error('frame_limit');
        child.stdin.write(line);
      },
      receive: (reader, close) => {
        readers.add(reader); closers.add(close);
        if (failure) queueMicrotask(close);
        return () => { readers.delete(reader); closers.delete(close); };
      },
      close: async () => {
        closing ??= (async () => {
          if (!ended && child.pid !== undefined) {
            const exited = once(child, 'exit');
            if (!child.kill()) throw new Error('app_server_stop_failed');
            let timer: ReturnType<typeof setTimeout> | undefined;
            try {
              await Promise.race([exited, new Promise<never>((_, reject) => {
                timer = setTimeout(() => reject(new Error('app_server_stop_timeout')), options.timeoutMs);
              })]);
            } finally { clearTimeout(timer); }
          }
          await claim.release();
        })();
        await closing;
      },
    };
  } catch (error) {
    await claim.release(); throw error;
  }
}
