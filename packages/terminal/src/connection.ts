import { randomUUID } from 'node:crypto';
import { createConnection, type Socket } from 'node:net';
import { EchoCore } from '@nemu/contracts';
import {
  checkAuthPair,
  checkRequest,
  checkResponse,
} from '../../contracts/src/generated/echo/ingress.js';
import { decodeFrame, encodeFrame, FrameReader, frameLimits } from './framing.js';
import type { TerminalIdentity } from './windows.js';

const { AuthStatus, IngressKind, IngressResponseKind, ReceiptStage } = EchoCore.schema.enums;
const eventRequestPrefix = 'event:';
// Bound a stalled request, while leaving a healthy idle pipe open for push/replies.
const requestDeadlineMs = 2500;

export type Control = Pick<EchoCore.TerminalRequest, 'kind' | 'approval' | 'cancel'>;
type Dispatch =
  | Control
  | Pick<EchoCore.TerminalRequest, 'kind' | 'event'>
  | Pick<EchoCore.TerminalRequest, 'kind' | 'upload'>;

interface Pending {
  accept(bytes: Uint8Array): void;
  reject(error: Error): void;
}

export class CoreConnection {
  private readonly socket: Socket;
  private readonly pending = new Map<string, Pending>();
  private auth: Pending | undefined;
  private session: EchoCore.TerminalSession | undefined;
  private failure = 'Core pipe disconnected';

  constructor(identity: TerminalIdentity, onClose: (reason: string) => void) {
    this.socket = createConnection(identity.pipe);
    this.socket.on('error', () => this.close('Cannot reach the core pipe'));
    this.socket.once('close', () => {
      clearTimeout(partialTimer);
      const error = new Error(this.failure);
      this.auth?.reject(error);
      for (const waiter of this.pending.values()) waiter.reject(error);
      onClose(this.failure);
    });

    const reader = new FrameReader();
    let partialTimer: NodeJS.Timeout | undefined;
    this.socket.on('data', (data: Buffer) => {
      try {
        for (const body of reader.push(data)) {
          clearTimeout(partialTimer);
          partialTimer = undefined;
          this.receive(body);
        }
        if (reader.partial && !partialTimer) {
          partialTimer = setTimeout(() => this.close('Incomplete core frame'), requestDeadlineMs);
        }
      } catch {
        this.close('Core sent an invalid or unpaired frame');
      }
    });
  }

  get capabilities() {
    return this.session?.capabilities ?? [];
  }

  get budget() {
    return this.session?.budget;
  }

  async authenticate(identity: TerminalIdentity): Promise<void> {
    const request: EchoCore.TerminalAuthenticate = {
      schema_version: 1,
      request_id: randomUUID(),
      protocol_versions: [1],
      installation_id: identity.installationId,
      token: identity.token,
    };
    let frame: Buffer;
    try {
      frame = encodeFrame('TerminalAuthenticate', request);
    } finally {
      identity.token.fill(0);
    }
    try {
      await this.exchange<void>(
        frame,
        (waiter) => {
          this.auth = waiter;
        },
        (bytes) => {
          const reply = decodeFrame('TerminalAuthResult', bytes);
          checkAuthPair(request, reply, frameLimits);
          if (reply.status !== AuthStatus!.AUTHENTICATED || !reply.session) {
            throw new Error('Core rejected the terminal credentials');
          }
          this.session = reply.session;
        },
      );
    } finally {
      this.auth = undefined;
    }
  }

  async send(message: Dispatch, eventId?: string): Promise<EchoCore.TerminalResponse> {
    const session = this.session;
    if (!session || this.socket.destroyed) throw new Error(this.failure);
    const request: EchoCore.TerminalRequest = {
      schema_version: 1,
      request_id: eventId ? eventRequestPrefix + eventId : randomUUID(),
      session_id: session.session_id,
      protocol_version: session.protocol_version,
      ...message,
    };
    checkRequest(request, session, frameLimits);
    try {
      return await this.exchange(
        encodeFrame('TerminalRequest', request),
        (waiter) => {
          this.pending.set(request.request_id, waiter);
        },
        (bytes) => {
          const response = decodeFrame('TerminalResponse', bytes);
          checkResponse(request, response, session, frameLimits);
          return response;
        },
      );
    } finally {
      this.pending.delete(request.request_id);
    }
  }

  close(reason = 'Core pipe disconnected'): void {
    if (this.socket.destroyed) return;
    this.failure = reason;
    this.socket.destroy();
  }

  private receive(body: Buffer): void {
    if (!this.session) {
      if (!this.auth) throw new Error('Unexpected auth reply');
      this.auth.accept(body);
      return;
    }
    const response = decodeFrame('TerminalResponse', body);
    const waiter = this.pending.get(response.request_id);
    const receipt = response.receipt;
    // The second gateway receipt may arrive with the first in the same read.
    // It is settlement, never acknowledgement of a pending outbox record.
    if (
      !waiter &&
      receipt &&
      response.kind === IngressResponseKind!.EVENT_RECEIPT &&
      response.reply_to === IngressKind!.INGRESS_EVENT &&
      receipt?.stage !== ReceiptStage!.RECEIVED_DURABLE &&
      receipt?.event_id === receipt?.stable_source_key &&
      response.request_id === eventRequestPrefix + receipt?.event_id &&
      response.session_id === this.session.session_id &&
      response.protocol_version === this.session.protocol_version &&
      body.length <= this.session.budget.max_frame_bytes
    )
      return;

    if (!waiter) throw new Error('Unexpected response');
    this.pending.delete(response.request_id);
    waiter.accept(body);
  }

  private async exchange<T>(
    frame: Buffer,
    register: (waiter: Pending) => void,
    parse: (bytes: Uint8Array) => T,
  ): Promise<T> {
    const timer = setTimeout(() => this.close('Core request timed out'), requestDeadlineMs);
    try {
      return await new Promise<T>((resolve, reject) => {
        register({
          reject,
          accept: (body) => {
            try {
              resolve(parse(body));
            } catch (error) {
              reject(error);
              this.close('Core refused authentication or sent an invalid reply');
            }
          },
        });
        this.socket.write(frame, (error) => {
          frame.fill(0);
          if (error) this.close('Cannot write to the core pipe');
        });
      });
    } finally {
      clearTimeout(timer);
    }
  }
}
