import { readFile, mkdir, open, unlink, lstat } from 'node:fs/promises';
import { isAbsolute, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { randomUUID } from 'node:crypto';
import { z } from 'zod';
import { sessionEvents } from './protocol.js';
import { readState, statePath, writeJson, type HookState } from './state.js';

const object = z.record(z.string(), z.unknown());
const groupSchema = object.and(z.object({ hooks: z.array(object) }));
const configSchema = object.and(
  z.object({ hooks: z.record(z.string(), z.array(groupSchema)).optional() }),
);
type HookConfig = z.infer<typeof configSchema>;
export interface InstallOptions {
  configDir: string;
  stateDir: string;
}

function explicitDirectory(path: string): string {
  if (!isAbsolute(path)) throw new Error('An explicit absolute directory is required.');
  return resolve(path);
}

const quotePosix = (text: string): string => `'${text.replaceAll("'", "'\\''")}'`;
const quotePowerShell = (text: string): string => `'${text.replaceAll("'", "''")}'`;

function commands(stateFile: string): Pick<HookState, 'command' | 'commandWindows'> {
  const loader = import.meta.resolve('tsx');
  const sender = fileURLToPath(new URL('./send.ts', import.meta.url));
  const args = [process.execPath, '--import', loader, sender, stateFile];
  const script = `& ${args.map(quotePowerShell).join(' ')}; exit $LASTEXITCODE`;
  return {
    command: args.map(quotePosix).join(' '),
    commandWindows: `powershell.exe -NoProfile -NonInteractive -WindowStyle Hidden -EncodedCommand ${Buffer.from(script, 'utf16le').toString('base64')}`,
  };
}

async function readConfig(file: string): Promise<HookConfig> {
  try {
    if ((await lstat(file)).isSymbolicLink())
      throw new Error('Symbolic hooks files are not supported.');
    const raw: unknown = JSON.parse(await readFile(file, 'utf8'));
    configSchema.parse(raw);
    return raw as HookConfig;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') return {};
    throw new Error('Invalid or linked hooks.json; left untouched.', { cause: error });
  }
}

function removeOwned(config: HookConfig, state: HookState): void {
  if (!config.hooks) return;
  for (const event of sessionEvents) {
    const groups = config.hooks[event];
    if (!groups) continue;
    config.hooks[event] = groups.flatMap((group) => {
      const hooks = group.hooks.filter(
        (hook) =>
          !(
            hook.type === 'command' &&
            hook.command === state.command &&
            hook.commandWindows === state.commandWindows
          ),
      );
      // Keep preexisting empty groups and all unrelated fields and handlers.
      return hooks.length === 0 && group.hooks.length !== 0 ? [] : [{ ...group, hooks }];
    });
    if (config.hooks[event]?.length === 0) delete config.hooks[event];
  }
}

async function change(options: InstallOptions, enable: boolean): Promise<void> {
  const configDir = explicitDirectory(options.configDir);
  const stateDir = explicitDirectory(options.stateDir);
  // Both paths are explicit: never infer either from HOME, CODEX_HOME or an account.
  await mkdir(stateDir, { recursive: true });
  const lockFile = join(stateDir, 'install.lock');
  const lock = await open(lockFile, 'wx');
  try {
    const file = statePath(stateDir);
    let previous = await readState(file);
    if (previous && previous.configDir !== configDir && (previous.enabled || !enable)) {
      throw new Error('Uninstall the previous target before choosing another configuration.');
    }
    if (previous?.configDir !== configDir && enable) previous = undefined;
    if (!enable && !previous) return;
    // Disabling must revoke consent even if hooks.json has become invalid or read-only.
    if (!enable && previous) await writeJson(file, { ...previous, enabled: false });
    const configFile = join(configDir, 'hooks.json');
    const config = await readConfig(configFile);
    const state: HookState = previous ?? {
      version: 1,
      enabled: false,
      configDir,
      pipe:
        process.platform === 'win32'
          ? `\\\\.\\pipe\\nemu-codex-session-${randomUUID()}`
          : join(stateDir, 'events.sock'),
      ...commands(file),
    };
    // Revoke first; failures cannot leave an enabled sender after an attempted disable.
    await writeJson(file, { ...state, enabled: false });
    removeOwned(config, state);
    if (enable) {
      const currentCommands = commands(file);
      Object.assign(state, currentCommands);
      config.hooks ??= {};
      for (const event of sessionEvents) {
        config.hooks[event] ??= [];
        config.hooks[event].push({
          hooks: [
            {
              type: 'command',
              ...currentCommands,
              timeout: 3,
            },
          ],
        });
      }
    }
    await writeJson(configFile, config);
    await writeJson(file, { ...state, enabled: enable });
  } finally {
    await lock.close();
    await unlink(lockFile);
  }
}

export const install = (options: InstallOptions): Promise<void> => change(options, true);
export const uninstall = (options: InstallOptions): Promise<void> => change(options, false);
