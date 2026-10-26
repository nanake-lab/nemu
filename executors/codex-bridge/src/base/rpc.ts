import { z } from 'zod';
import { message, type Message, type Parameters } from './protocol.js';

/** A process-local transport port; no Nemu wire contract is defined here. */
export interface Transport {
  send(line: string): void;
  receive(onLine: (line: string) => void, onClose: () => void): () => void;
  close(): Promise<void>;
}

export class RpcError extends Error {
  constructor(readonly code: number) { super('app_server_rpc_error'); }
}

export class RpcClient {
  private nextId = 0;
  private closed = false;
  private readonly listeners = new Set<(event: Message) => void>();
  private readonly pending = new Map<string, {
    resolve: (value: unknown) => void; reject: (error: Error) => void;
    timer: ReturnType<typeof setTimeout>;
  }>();
  private readonly unsubscribe: () => void;

  constructor(private readonly transport: Transport, private readonly maxFrameBytes: number) {
    if (!Number.isSafeInteger(maxFrameBytes) || maxFrameBytes < 1) throw new Error('frame_limit');
    this.unsubscribe = transport.receive((line) => this.accept(line), () => this.fail());
  }

  subscribe(listener: (event: Message) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private accept(line: string): void {
    if (this.closed) return;
    try {
      if (Buffer.byteLength(line) > this.maxFrameBytes) throw new Error('frame_limit');
      const event = message.parse(JSON.parse(line));
      if (event.method !== undefined) {
        if (event.id !== undefined) {
          // IRL is a talk-only client. No command, file or permission request is approved.
          this.transport.send(JSON.stringify({ id: event.id, error: { code: -32601, message: 'unsupported_request' } }) + '\n');
        }
        for (const listener of this.listeners) listener(event);
      } else if (event.id !== undefined) {
        const entry = this.pending.get(String(event.id));
        if (!entry) return; // A timed-out response is never reassigned to a new request.
        this.pending.delete(String(event.id));
        clearTimeout(entry.timer);
        if (event.error) entry.reject(new RpcError(event.error.code));
        else if (Object.hasOwn(event, 'result')) entry.resolve(event.result);
        else entry.reject(new Error('missing_rpc_result'));
      } else throw new Error('invalid_rpc_message');
    } catch { this.fail(); }
  }

  async request<M extends keyof Parameters, T>(method: M, params: Parameters[M], schema: z.ZodType<T>, timeoutMs: number): Promise<T> {
    if (this.closed) throw new Error('app_server_disconnected');
    if (!Number.isFinite(timeoutMs) || timeoutMs <= 0 || timeoutMs > 2_147_483_647) throw new Error('rpc_deadline');
    const id = String(++this.nextId);
    const value = await new Promise<unknown>((resolve, reject) => {
      const timer = setTimeout(() => { this.pending.delete(id); reject(new Error('rpc_timeout')); }, timeoutMs);
      this.pending.set(id, { resolve, reject, timer });
      try { this.transport.send(JSON.stringify({ id, method, params }) + '\n'); }
      catch { this.fail(); }
    });
    return schema.parse(value);
  }

  initialized(): void {
    if (this.closed) throw new Error('app_server_disconnected');
    this.transport.send('{"method":"initialized"}\n');
  }

  private fail(): void {
    if (this.closed) return;
    this.closed = true;
    for (const entry of this.pending.values()) {
      clearTimeout(entry.timer);
      entry.reject(new Error('app_server_disconnected'));
    }
    this.pending.clear();
    for (const listener of this.listeners) listener({ method: 'bridge/disconnected' });
  }

  async close(): Promise<void> {
    this.fail();
    this.unsubscribe();
    this.listeners.clear();
    await this.transport.close();
  }
}
