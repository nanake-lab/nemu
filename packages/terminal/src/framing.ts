import type { EchoCore } from '@nemu/contracts';
import {
  decodeProtobuf,
  encodeProtobuf,
  type Limits,
} from '../../contracts/src/generated/echo/index.js';

// ADR 0024: bound decoding before the gateway advertises its session budget.
export const frameLimits: Limits = { max_bytes: 1_048_576, max_items: 4096, max_depth: 32 };

export function encodeFrame<K extends keyof EchoCore.MessageMap>(
  type: K,
  message: EchoCore.MessageMap[K],
): Buffer {
  const body = encodeProtobuf(type, message, frameLimits);
  const frame = Buffer.allocUnsafe(4 + body.length);
  frame.writeUInt32LE(body.length);
  frame.set(body, 4);
  // Do not retain a second encoded copy of TerminalAuthenticate.
  body.fill(0);
  return frame;
}

export function decodeFrame<K extends keyof EchoCore.MessageMap>(type: K, body: Uint8Array) {
  return decodeProtobuf(type, body, frameLimits);
}

export class FrameReader {
  private header = Buffer.alloc(4);
  private headerUsed = 0;
  private body: Buffer | undefined;
  private bodyUsed = 0;

  get partial(): boolean {
    return this.headerUsed !== 0;
  }

  *push(data: Buffer): Generator<Buffer> {
    let offset = 0;
    while (offset < data.length) {
      if (!this.body) {
        const count = data.copy(this.header, this.headerUsed, offset, offset + 4 - this.headerUsed);
        this.headerUsed += count;
        offset += count;
        if (this.headerUsed < 4) return;
        const length = this.header.readUInt32LE();
        if (!length || length > frameLimits.max_bytes) throw new Error('Invalid core frame length');
        this.body = Buffer.allocUnsafe(length);
      }
      const count = data.copy(
        this.body,
        this.bodyUsed,
        offset,
        offset + this.body.length - this.bodyUsed,
      );
      this.bodyUsed += count;
      offset += count;
      if (this.bodyUsed === this.body.length) {
        const result = this.body;
        this.headerUsed = 0;
        this.body = undefined;
        this.bodyUsed = 0;
        yield result;
      }
    }
  }
}
