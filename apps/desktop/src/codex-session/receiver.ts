import { createServer, type Server, type Socket } from 'node:net';
import { TextDecoder } from 'node:util';
import { appBusPayloads } from '@nemu/contracts';
import type { AppBus } from '../bus/index.js';

const metadataSchema = appBusPayloads['codex.session']
  .pick({ event: true, sessionId: true })
  .strict();

export class CodexSessionReceiver {
  private server: Server | undefined;
  private sockets = new Set<Socket>();
  private enabled = false;
  private generation = 0;

  constructor(
    private readonly bus: AppBus,
    private readonly consent: () => Promise<boolean> = async () => true,
  ) {}

  async start(pipe: string): Promise<void> {
    if (this.server) throw new Error('Codex session receiver is already listening.');
    const server = createServer((socket) => this.accept(socket));
    this.server = server;
    try {
      await new Promise<void>((resolve, reject) => {
        server.once('error', reject);
        server.listen({ path: pipe, readableAll: false, writableAll: false }, () => {
          server.removeListener('error', reject);
          resolve();
        });
      });
      // An error never emits received bytes or private hook input.
      server.on('error', () => {
        this.enabled = false;
      });
      this.enabled = true;
      this.generation++;
    } catch (error) {
      this.server = undefined;
      throw error;
    }
  }

  async stop(): Promise<void> {
    this.enabled = false;
    this.generation++;
    for (const socket of this.sockets) socket.destroy();
    this.sockets.clear();
    const server = this.server;
    this.server = undefined;
    if (server) await new Promise<void>((resolve) => server.close(() => resolve()));
  }

  private accept(socket: Socket): void {
    if (!this.enabled || this.sockets.size >= 16) {
      socket.destroy();
      return;
    }
    this.sockets.add(socket);
    socket.setTimeout(500, () => socket.destroy());
    socket.on('error', () => socket.destroy());
    socket.on('close', () => this.sockets.delete(socket));
    let buffer = Buffer.alloc(0);
    socket.on('data', (chunk: Buffer) => {
      if (!this.enabled || buffer.length + chunk.length > 1024) {
        socket.destroy();
        return;
      }
      buffer = Buffer.concat([buffer, chunk]);
      const lf = buffer.indexOf(10);
      if (lf < 0) return;
      // Exactly one LF-terminated metadata object per connection.
      if (lf !== buffer.length - 1) {
        socket.destroy();
        return;
      }
      try {
        const raw: unknown = JSON.parse(
          new TextDecoder('utf-8', { fatal: true }).decode(buffer.subarray(0, lf)),
        );
        const result = metadataSchema.safeParse(raw);
        if (
          result.success &&
          /^[a-zA-Z0-9_-]{1,128}$/.test(result.data.sessionId) &&
          this.enabled
        ) {
          const generation = this.generation;
          void this.consent()
            .then((allowed) => {
              if (allowed && this.enabled && generation === this.generation) {
                this.bus.publish(this.bus.createEvent('main', 'codex.session', result.data));
              }
            })
            .catch(() => undefined);
        }
      } catch {
        /* Malformed input is discarded without logging its content. */
      }
      socket.destroy();
    });
  }
}
