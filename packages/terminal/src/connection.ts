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

export interface Downlink {
  chunk(message: EchoCore.OutputChunk): void;
  end(message: EchoCore.OutputEnd): void;
  runState(message: EchoCore.RunStateV2): void;
  approval(message: EchoCore.ApprovalRequestV2): void;
}

const { AuthStatus, ReceiptStage } = EchoCore.schema.enums;
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
  private readonly awaitingSettlement = new Map<string, EchoCore.TerminalRequest>();
  private auth: Pending | undefined;
  private session: EchoCore.TerminalSession | undefined;
  private failure = 'Core pipe disconnected';

  constructor(
    identity: TerminalIdentity,
    onClose: (reason: string) => void,
    private readonly downlink?: Downlink,
    private readonly settled?: (receipt: EchoCore.ReceiptV2) => void,
  ) {
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
          if (request.event && response.receipt?.stage === ReceiptStage!.RECEIVED_DURABLE)
            this.awaitingSettlement.set(request.request_id, request);
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
    if (body.length > this.session.budget.max_frame_bytes)
      throw new Error('Session frame budget exceeded');
    // The five allowed protobuf descriptors have distinct required wire shapes.
    // Use the generated strict decoders. Unknown or ambiguous bodies fail closed.
    const receivers: (() => void)[] = [];
    const match = <K extends keyof EchoCore.MessageMap>(
      type: K,
      receive: (message: EchoCore.MessageMap[K]) => void,
    ) => {
      let message: EchoCore.MessageMap[K];
      try {
        message = decodeFrame(type, body);
      } catch {
        return;
      }
      receivers.push(() => receive(message));
    };
    match('TerminalResponse', (reply) => this.receiveReply(body, reply));
    const sink = this.downlink;
    match('OutputChunk', (chunk) => sink?.chunk(chunk));
    match('OutputEnd', (end) => sink?.end(end));
    match('ApprovalRequestV2', (approval) => sink?.approval(approval));
    match('RunStateV2', (state) => sink?.runState(state));
    if (receivers.length !== 1) throw new Error('Unrecognized core frame');
    receivers[0]!();
  }

  private receiveReply(body: Buffer, response: EchoCore.TerminalResponse): void {
    const waiter = this.pending.get(response.request_id);
    const request = this.awaitingSettlement.get(response.request_id);
    if (
      !waiter &&
      request &&
      response.receipt &&
      response.receipt.stage !== ReceiptStage!.RECEIVED_DURABLE
    ) {
      checkResponse(request, response, this.session!, frameLimits);
      this.awaitingSettlement.delete(response.request_id);
      this.settled?.(response.receipt);
      return;
    }

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
