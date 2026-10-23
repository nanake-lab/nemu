import { afterEach, describe, expect, it } from 'vitest';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { randomUUID } from 'node:crypto';
import { connect, createServer, type Server, type Socket } from 'node:net';
import { exec, spawn } from 'node:child_process';
import { install, uninstall } from '../../../../tools/codex-hooks/install.js';
import { readState, statePath, writeJson } from '../../../../tools/codex-hooks/state.js';
import { projectHookInput, sessionEvents } from '../../../../tools/codex-hooks/protocol.js';
import { sendHook } from '../../../../tools/codex-hooks/send.js';
import { AppBus } from '../bus/index.js';
import { CodexSessionReceiver } from './receiver.js';
import { CodexSessionService } from './service.js';

const directories: string[] = [];
const cleanup: Array<() => Promise<void>> = [];
afterEach(async () => {
  for (const close of cleanup.splice(0).reverse()) await close();
  for (const path of directories.splice(0)) await rm(path, { recursive: true, force: true });
});

async function fixture(): Promise<{ root: string; configDir: string; stateDir: string }> {
  const parent = process.env.NEMU_TEST_TEMP ?? tmpdir();
  await mkdir(parent, { recursive: true });
  const root = await mkdtemp(join(parent, 'nemu-codex-hooks-'));
  directories.push(root);
  const configDir = join(root, "fake Codex 空格 $x 'quote'");
  const stateDir = join(root, "fake Nemu 空格 $x 'quote'");
  await mkdir(configDir);
  return { root, configDir, stateDir };
}

async function listen(server: Server, pipe: string): Promise<void> {
  await new Promise<void>((done, fail) => {
    server.once('error', fail);
    server.listen(pipe, done);
  });
  cleanup.push(() => new Promise<void>((done) => server.close(() => done())));
}

function endpoint(root: string): string {
  return process.platform === 'win32'
    ? `\\\\.\\pipe\\nemu-codex-session-${randomUUID()}`
    : join(root, 'receiver.sock');
}

async function transmit(pipe: string, chunks: Array<string | Buffer>): Promise<void> {
  await new Promise<void>((done) => {
    const socket = connect(pipe);
    socket.on('error', () => socket.destroy());
    socket.on('close', () => done());
    socket.on('connect', () => {
      for (const chunk of chunks) socket.write(chunk);
      socket.end();
    });
  });
}

const rawEvent = (event: string): Record<string, unknown> => ({
  hook_event_name: event,
  session_id: 'thr_123-test',
  cwd: 'D:\\PRIVATE_CODE',
  transcript_path: 'D:\\PRIVATE_TRANSCRIPT',
  prompt: 'PRIVATE_PROMPT',
  tool_input: { command: 'PRIVATE_COMMAND', file: 'PRIVATE_FILE_CONTENT' },
  last_assistant_message: 'PRIVATE_RESPONSE',
  permission_mode: 'bypassPermissions',
});

describe('P6-11 install and privacy', () => {
  it('installs and uninstalls idempotently while preserving unrelated hooks and config.toml', async () => {
    const options = await fixture();
    const original = JSON.parse(
      '{"description":"mine","__proto__":{"keep":true},"hooks":{"SessionStart":[{"matcher":"resume","hooks":[{"type":"command","command":"mine"}]}],"PostToolUse":[{"hooks":[{"type":"command","command":"keep"}]}]}}',
    ) as unknown;
    const file = join(options.configDir, 'hooks.json');
    await writeJson(file, original);
    await writeFile(join(options.configDir, 'config.toml'), 'model = "unchanged"\n');
    await install(options);
    const first = await readFile(file, 'utf8');
    const firstState = await readFile(statePath(options.stateDir), 'utf8');
    await install(options);
    expect(await readFile(file, 'utf8')).toBe(first);
    expect(await readFile(statePath(options.stateDir), 'utf8')).toBe(firstState);
    expect((await readState(statePath(options.stateDir)))?.enabled).toBe(true);
    await uninstall(options);
    const disabled = await readFile(file, 'utf8');
    await uninstall(options);
    expect(await readFile(file, 'utf8')).toBe(disabled);
    expect(JSON.parse(disabled)).toEqual(original);
    expect(await readFile(join(options.configDir, 'config.toml'), 'utf8')).toBe(
      'model = "unchanged"\n',
    );
    expect((await readState(statePath(options.stateDir)))?.enabled).toBe(false);
  });

  it('refuses invalid hooks without altering their bytes or enabling state', async () => {
    const options = await fixture();
    const file = join(options.configDir, 'hooks.json');
    await writeFile(file, '{broken');
    await expect(install(options)).rejects.toThrow('left untouched');
    expect(await readFile(file, 'utf8')).toBe('{broken');
    expect(await readState(statePath(options.stateDir))).toBeUndefined();
    await expect(install({ ...options, configDir: 'relative' })).rejects.toThrow('absolute');
  });

  it('requires explicit CLI targets even with a fake CODEX_HOME', async () => {
    const options = await fixture();
    const sentinel = join(options.configDir, 'hooks.json');
    await writeFile(sentinel, 'DO_NOT_CHANGE');
    const result = await new Promise<{ code: number | null; stdout: string; stderr: string }>(
      (done) => {
        const child = spawn(
          process.execPath,
          ['--import', import.meta.resolve('tsx'), resolve('tools/codex-hooks/cli.ts'), 'install'],
          {
            env: { ...process.env, CODEX_HOME: options.configDir },
            stdio: 'pipe',
            windowsHide: true,
          },
        );
        let stdout = '';
        let stderr = '';
        child.stdout.on('data', (data: Buffer) => {
          stdout += data.toString();
        });
        child.stderr.on('data', (data: Buffer) => {
          stderr += data.toString();
        });
        child.on('close', (code) => done({ code, stdout, stderr }));
      },
    );
    expect(result.code).toBe(1);
    expect(result.stdout).toBe('');
    expect(result.stderr).toContain('explicit paths');
    expect(await readFile(sentinel, 'utf8')).toBe('DO_NOT_CHANGE');
  });

  it('revokes consent even when uninstall cannot edit a corrupt hooks file', async () => {
    const options = await fixture();
    await install(options);
    const file = join(options.configDir, 'hooks.json');
    await writeFile(file, '{corrupt after install');
    await expect(uninstall(options)).rejects.toThrow('left untouched');
    expect((await readState(statePath(options.stateDir)))?.enabled).toBe(false);
    expect(await readFile(file, 'utf8')).toBe('{corrupt after install');
  });

  it('projects only the three event names and a bounded identifier', () => {
    for (const event of sessionEvents) {
      expect(projectHookInput(rawEvent(event))).toEqual({ event, sessionId: 'thr_123-test' });
    }
    for (const event of ['PostToolUse', 'SessionEnd', 'UserPromptSubmit']) {
      expect(projectHookInput(rawEvent(event))).toBeUndefined();
    }
    expect(
      projectHookInput({ ...rawEvent('Stop'), session_id: 'PRIVATE text code()' }),
    ).toBeUndefined();
    expect(projectHookInput({ ...rawEvent('Stop'), session_id: 'a'.repeat(129) })).toBeUndefined();
  });

  it('runs the installed hook command with quoting and emits only minimal bytes', async () => {
    const options = await fixture();
    await install(options);
    const state = (await readState(statePath(options.stateDir)))!;
    let wire = '';
    await listen(
      createServer((socket) => {
        socket.on('data', (chunk: Buffer) => {
          wire += chunk.toString('utf8');
        });
        socket.on('end', () => socket.end());
      }),
      state.pipe,
    );
    const output = await new Promise<{ stdout: string; stderr: string }>((done, fail) => {
      const child = exec(
        process.platform === 'win32' ? state.commandWindows : state.command,
        { windowsHide: true, timeout: 5000 },
        (error, stdout, stderr) => {
          if (error) fail(error);
          else done({ stdout, stderr });
        },
      );
      child.stdin?.end(JSON.stringify(rawEvent('PermissionRequest')));
    });
    expect(output).toEqual({ stdout: '', stderr: '' });
    expect(wire).toBe('{"event":"PermissionRequest","sessionId":"thr_123-test"}\n');
    expect(wire).not.toContain('PRIVATE');
  }, 10000);
});

describe('P6-11 receiver and consent', () => {
  it('replays the synthetic recording through the real pipe into the bus', async () => {
    const options = await fixture();
    await install(options);
    const state = (await readState(statePath(options.stateDir)))!;
    const bus = new AppBus();
    const receiver = new CodexSessionReceiver(bus);
    await receiver.start(state.pipe);
    cleanup.push(() => receiver.stop());
    const recording = await readFile(
      resolve('tools/codex-hooks/fixtures/session-flow.jsonl'),
      'utf8',
    );
    for (const line of recording.trim().split('\n')) {
      await sendHook(JSON.parse(line) as unknown, statePath(options.stateDir));
    }
    expect(bus.getHistory().map((item) => item.payload)).toEqual(
      sessionEvents.map((event) => ({
        event,
        sessionId: 'synthetic-recording-001',
      })),
    );
    expect(
      bus
        .getHistory()
        .filter(
          (item) => item.type === 'codex.session' && item.payload.event === 'PermissionRequest',
        ),
    ).toHaveLength(1);
  });

  it('is silent before installation and after uninstall, including an already connected client', async () => {
    const options = await fixture();
    const bus = new AppBus({ enableValidation: false });
    const service = new CodexSessionService(bus, options.stateDir);
    await service.start();
    cleanup.push(() => service.stop());
    await sendHook(rawEvent('Stop'), statePath(options.stateDir));
    expect(bus.getHistory()).toEqual([]);
    await install(options);
    const state = (await readState(statePath(options.stateDir)))!;
    await expect
      .poll(async () => {
        await sendHook(rawEvent('SessionStart'), statePath(options.stateDir));
        return bus.getHistory().length;
      })
      .toBeGreaterThan(0);
    const connected = await new Promise<Socket>((done, fail) => {
      const socket = connect(state.pipe);
      socket.once('connect', () => done(socket));
      socket.once('error', fail);
    });
    connected.on('error', () => undefined);
    cleanup.push(async () => {
      connected.destroy();
    });
    await uninstall(options);
    bus.clearHistory();
    connected.end('{"event":"PermissionRequest","sessionId":"late-event"}\n');
    for (const event of sessionEvents) await sendHook(rawEvent(event), statePath(options.stateDir));
    await new Promise((done) => setTimeout(done, 350));
    expect(bus.getHistory()).toEqual([]);
  });

  it('forwards valid, fragmented events and rejects extra/private fields and malformed bytes', async () => {
    const { root } = await fixture();
    const bus = new AppBus({ enableValidation: false });
    const receiver = new CodexSessionReceiver(bus);
    const pipe = endpoint(root);
    await receiver.start(pipe);
    cleanup.push(() => receiver.stop());
    for (const event of sessionEvents) {
      await transmit(pipe, ['{"event":', JSON.stringify(event), ',"sessionId":"synthetic"}\n']);
    }
    expect(bus.getHistory().map((event) => event.payload)).toEqual(
      sessionEvents.map((event) => ({ event, sessionId: 'synthetic' })),
    );
    bus.clearHistory();
    for (const frame of [
      '{"event":"Stop","sessionId":"s","summary":"PRIVATE"}\n',
      '{"event":"Stop","sessionId":"s","prompt":"PRIVATE"}\n',
      '{"event":"PostToolUse","sessionId":"s"}\n',
      '{"event":"Stop","sessionId":"PRIVATE content"}\n',
      '{"event":"Stop","sessionId":"s"}',
      'x'.repeat(1025),
      'invalid\n',
      Buffer.from([0xff, 10]),
      '{"event":"Stop","sessionId":"s"}\n{"event":"Stop","sessionId":"s"}\n',
    ])
      await transmit(pipe, [frame]);
    expect(bus.getHistory()).toEqual([]);
    await receiver.stop();
    await transmit(pipe, ['{"event":"Stop","sessionId":"s"}\n']);
    expect(bus.getHistory()).toEqual([]);
  });

  it('does not block Codex or return an approval decision when Nemu is unavailable', async () => {
    const options = await fixture();
    await install(options);
    await expect(
      sendHook(rawEvent('PermissionRequest'), statePath(options.stateDir)),
    ).resolves.toBeUndefined();
  });

  it('discards an old pending consent result after a receiver restart', async () => {
    const { root } = await fixture();
    const bus = new AppBus();
    let consentCalls = 0;
    let release: ((allowed: boolean) => void) | undefined;
    const pendingConsent = new Promise<boolean>((done) => {
      release = done;
    });
    const receiver = new CodexSessionReceiver(bus, () => {
      consentCalls++;
      return consentCalls === 1 ? pendingConsent : Promise.resolve(true);
    });
    const pipe = endpoint(root);
    await receiver.start(pipe);
    cleanup.push(() => receiver.stop());
    const frame = '{"event":"Stop","sessionId":"s"}\n';
    await transmit(pipe, [frame]);
    await expect.poll(() => consentCalls).toBe(1);
    await receiver.stop();
    await receiver.start(pipe);
    release?.(true);
    await transmit(pipe, [frame]);
    expect(bus.getHistory()).toHaveLength(1);
  });
});
