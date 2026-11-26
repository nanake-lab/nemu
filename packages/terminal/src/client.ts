import { EchoCore } from '@nemu/contracts';
import { CoreConnection, type Control, type Downlink } from './connection.js';
import { textObservation } from './observation.js';
import { ChatOutbox, type ChatEvent } from './outbox.js';
import { windowsIdentity, type TerminalIdentity } from './windows.js';

const { IngressKind, IngressReason, IngressResponseKind, ReceiptStage } = EchoCore.schema.enums;

class CoreRejection extends Error {
  constructor(
    readonly reason: string,
    readonly retry: boolean,
  ) {
    super(reason);
  }
}

function rejection(code: number | undefined): CoreRejection {
  const reason = Object.keys(IngressReason!).find((key) => IngressReason![key] === code);
  return new CoreRejection(
    'Core rejected the request (' + (reason ?? 'unknown reason') + ')',
    code === IngressReason!.PENDING_RECORDS_FULL ||
      code === IngressReason!.PENDING_BYTES_FULL ||
      code === IngressReason!.UPLOAD_BUDGET_FULL ||
      code === IngressReason!.AUDIT_FULL ||
      code === IngressReason!.SESSION_EXPIRED,
  );
}

export class TerminalClient {
  private readonly abort = new AbortController();
  private readonly failedRemovals = new Set<string>();
  private connection: CoreConnection | undefined;
  private timer: NodeJS.Timeout | undefined;
  // A short first retry makes startup responsive. Cap retries to avoid a tight loop.
  private backoff = 250;
  private busy = false;
  private ready = false;
  private downlink: Downlink | undefined;
  private started = false;

  constructor(
    private readonly outbox: ChatOutbox,
    private readonly status: (
      reason: string | null,
      disconnected?: boolean,
      rejectedEventId?: string,
    ) => void,
    private readonly identity: (signal: AbortSignal) => Promise<TerminalIdentity> = (signal) =>
      windowsIdentity(undefined, signal),
  ) {}

  start(downlink?: Downlink): void {
    if (this.started || this.abort.signal.aborted) return;
    this.started = true;
    this.downlink = downlink;
    this.status('Connecting to Echo Core…');
    void this.connect();
  }

  stop(): void {
    this.abort.abort();
    clearTimeout(this.timer);
    this.connection?.close();
  }

  enqueue(event: ChatEvent): void {
    try {
      textObservation(event); // Validate IDL data before accepting it into the outbox.
      this.outbox.add(event);
    } catch {
      this.status('Could not save the message to the terminal outbox');
      return;
    }
    void this.flush();
  }

  async sendControl(control: Control): Promise<boolean> {
    if (!this.ready || !this.connection) {
      this.status('Core disconnected before the control could be sent');
      return false;
    }
    try {
      const reply = await this.connection.send(control);
      if (reply.kind !== IngressResponseKind!.CONTROL_ACK) throw rejection(reply.reason);
      return true;
    } catch (error) {
      this.status(
        error instanceof CoreRejection ? error.message : 'Core control was not acknowledged',
      );
      return false;
    }
  }

  private async connect(): Promise<void> {
    let connection: CoreConnection | undefined;
    try {
      const identity = await this.identity(this.abort.signal);
      if (this.abort.signal.aborted) {
        identity.token.fill(0);
        return;
      }
      connection = new CoreConnection(
        identity,
        (reason) => {
          this.connection = undefined;
          this.ready = false;
          this.retry(reason);
        },
        this.downlink,
        (receipt) => {
          if (receipt.stage === ReceiptStage!.REJECTED && receipt.event_id !== undefined) {
            this.status(rejection(receipt.reason_code).message, false, receipt.event_id);
          }
        },
      );
      this.connection = connection;
      await connection.authenticate(identity);
      if (this.abort.signal.aborted || this.connection !== connection) return;
      this.ready = true;
      this.status(null);
      await this.flush();
    } catch (error) {
      const reason = connection
        ? 'Core authentication failed or the pipe is unavailable'
        : 'Cannot open the Windows terminal identity';
      connection?.close(reason);
      if (!connection) this.retry(error instanceof Error ? error.message : reason);
    }
  }

  private retry(reason: string): void {
    if (this.abort.signal.aborted) return;
    this.status(reason, true);
    this.timer = setTimeout(() => {
      void this.connect();
    }, this.backoff);
    this.backoff = Math.min(this.backoff * 2, 5000);
  }

  private async flush(): Promise<void> {
    const connection = this.connection;
    if (this.busy || !connection || !this.ready) return;
    this.busy = true;
    try {
      for (const input of this.outbox.pending()) {
        if (this.failedRemovals.has(input.id)) continue;
        try {
          const { event, payload, source } = textObservation(input);
          const budget = connection.budget;
          if (
            !connection.capabilities.includes(
              IngressKind!.INGRESS_UPLOAD as EchoCore.IngressKind,
            ) ||
            !connection.capabilities.includes(IngressKind!.INGRESS_EVENT as EchoCore.IngressKind)
          ) {
            throw new CoreRejection('Core session does not allow chat uploads', false);
          }
          if (
            !budget ||
            payload.bytes.length > budget.max_object_bytes ||
            source.bytes.length > budget.max_object_bytes ||
            payload.bytes.length + source.bytes.length > budget.max_upload_bytes ||
            budget.max_uploads < 2
          )
            throw new CoreRejection('Message exceeds the core upload budget', false);

          for (const [objectIndex, { object, bytes }] of [payload, source].entries()) {
            const uploadId = input.id + ':' + objectIndex;
            for (let offset = 0; offset < bytes.length; offset += budget.max_chunk_bytes) {
              const end = Math.min(offset + budget.max_chunk_bytes, bytes.length);
              const reply = await connection.send({
                kind: IngressKind!.INGRESS_UPLOAD as EchoCore.IngressKind,
                upload: {
                  schema_version: 1,
                  upload_id: uploadId,
                  object,
                  offset: BigInt(offset),
                  data: bytes.subarray(offset, end),
                  final_chunk: end === bytes.length,
                },
              });
              if (!reply.upload) throw rejection(reply.reason);
            }
          }

          const reply = await connection.send(
            {
              kind: IngressKind!.INGRESS_EVENT as EchoCore.IngressKind,
              event,
            },
            input.id,
          );
          if (reply.receipt?.stage !== ReceiptStage!.RECEIVED_DURABLE) {
            throw rejection(reply.reason ?? reply.receipt?.reason_code);
          }
          this.outbox.remove(input.id);
          this.backoff = 250;
          this.status(null);
        } catch (error) {
          if (error instanceof CoreRejection && !error.retry) {
            // ADR 0024 §7 forbids replaying a permanently rejected request.
            let reason = error.reason;
            try {
              this.outbox.remove(input.id);
            } catch {
              // A filesystem failure must not cause another send in this session.
              this.failedRemovals.add(input.id);
              reason += '. Could not remove the message from the terminal outbox.';
            }
            this.status(reason, false, input.id);
          } else {
            connection.close(
              error instanceof CoreRejection ? error.message : 'Core receipt unavailable',
            );
            break;
          }
        }
      }
    } finally {
      this.busy = false;
    }
  }
}
