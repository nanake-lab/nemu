import { readFile, mkdir, writeFile, rename, unlink } from 'node:fs/promises';
import { dirname, isAbsolute, join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { z } from 'zod';

export const stateSchema = z.strictObject({
  version: z.literal(1),
  enabled: z.boolean(),
  configDir: z.string().refine(isAbsolute),
  pipe: z
    .string()
    .refine((path) =>
      process.platform === 'win32'
        ? /^\\\\\.\\pipe\\nemu-codex-session-[a-f0-9-]{36}$/.test(path)
        : isAbsolute(path),
    ),
  command: z.string().min(1),
  commandWindows: z.string().min(1),
});
export type HookState = z.infer<typeof stateSchema>;
export const statePath = (directory: string): string => join(directory, 'state.json');

export async function readState(file: string): Promise<HookState | undefined> {
  try {
    return stateSchema.parse(JSON.parse(await readFile(file, 'utf8')) as unknown);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') return;
    throw new Error('Invalid Codex session state; refusing to use it.', { cause: error });
  }
}

export async function writeJson(file: string, value: unknown): Promise<void> {
  await mkdir(dirname(file), { recursive: true });
  const temporary = `${file}.${randomUUID()}.tmp`;
  try {
    await writeFile(temporary, `${JSON.stringify(value, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
    await rename(temporary, file);
  } finally {
    await unlink(temporary).catch(() => undefined);
  }
}
