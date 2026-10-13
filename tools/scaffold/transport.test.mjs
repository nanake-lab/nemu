import test from 'node:test';
import assert from 'node:assert/strict';
import net from 'node:net';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { once } from 'node:events';
import { randomUUID } from 'node:crypto';

// Test-only transport substitute; no Echo Core messages or storage.
test('local byte transport: Windows named pipe / Unix socket', { timeout: 10000 }, async () => {
  const directory = await mkdtemp(join(tmpdir(), 'nemu-transport-'));
  const address =
    process.platform === 'win32'
      ? `\\\\.\\pipe\\nemu-scaffold-${randomUUID()}`
      : join(directory, 'test.sock');
  const server = net.createServer((socket) => socket.pipe(socket));
  let client;
  try {
    server.listen(address);
    await once(server, 'listening');
    client = net.connect(address);
    await once(client, 'connect');
    const response = once(client, 'data');
    client.write('scaffold');
    assert.equal((await response)[0].toString(), 'scaffold');
  } finally {
    client?.destroy();
    await new Promise((resolve) => server.close(resolve));
    await rm(directory, { recursive: true, force: true });
  }
});
