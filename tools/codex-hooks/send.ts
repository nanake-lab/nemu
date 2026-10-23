import { connect } from 'node:net';
import { readState } from './state.js';
import { projectHookInput } from './protocol.js';

export async function sendHook(raw: unknown, file: string): Promise<void> {
  const payload = projectHookInput(raw);
  if (!payload) return;
  const state = await readState(file);
  if (!state?.enabled) return;
  await new Promise<void>((resolve) => {
    const socket = connect(state.pipe);
    const finish = (): void => {
      socket.destroy();
      resolve();
    };
    const timer = setTimeout(finish, 300);
    socket.on('error', finish);
    socket.on('close', () => {
      clearTimeout(timer);
      resolve();
    });
    socket.on('connect', async () => {
      // Recheck consent after connecting, including a disable/install race.
      const current = await readState(file).catch(() => undefined);
      if (!current?.enabled || current.pipe !== state.pipe) return finish();
      socket.end(`${JSON.stringify(payload)}\n`);
    });
  });
}

async function main(): Promise<void> {
  const file = process.argv[2];
  if (!file) return;
  const chunks: Buffer[] = [];
  let bytes = 0;
  // Hook input can include private content. Never log, persist, or open any referenced file.
  for await (const chunk of process.stdin) {
    const buffer = Buffer.from(chunk as Uint8Array);
    bytes += buffer.length;
    if (bytes > 1024 * 1024) return;
    chunks.push(buffer);
  }
  await sendHook(JSON.parse(Buffer.concat(chunks).toString('utf8')) as unknown, file);
}

// This file is also imported by transport tests.
if (process.argv[1]?.replaceAll('\\', '/').endsWith('/send.ts')) {
  await main().catch(() => undefined);
}
