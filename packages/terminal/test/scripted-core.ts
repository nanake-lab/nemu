import { createHash, randomUUID } from 'node:crypto';
import { createServer, type Socket } from 'node:net';
import { join } from 'node:path';

import { once } from 'node:events';
import { EchoCore } from '@nemu/contracts';
import { decodeCanonical } from '../../contracts/src/generated/echo/index.js';
import { checkRequest, checkUploaded } from '../../contracts/src/generated/echo/ingress.js';
import { decodeFrame, encodeFrame, FrameReader, frameLimits } from '../src/framing.js';

const token = () => Buffer.alloc(32, 0xa5);
const installation = 'scripted-local-terminal';
const enums = EchoCore.schema.enums;
const FixtureTime = 1_000_000_000n;

export async function scriptedCore(
  directory: string,
  options: {
    dropFirstReceipt?: boolean;
    rejectFirstEvent?: NonNullable<EchoCore.TerminalResponse['reason']>;
  } = {},
) {
  const path =
    process.platform === 'win32'
      ? '\\\\.\\pipe\\nemu-chat-test-' + randomUUID()
      : join(directory, 'core.sock');
  const sockets = new Set<Socket>();
  const input: EchoCore.TerminalEvent[] = [];
  const messages = new Map<string, string>();
  const controls: EchoCore.TerminalRequest[] = [];
  const failures: unknown[] = [];
  const timestamps: { authenticated: number[]; disconnected: number[]; durable: number[] } = {
    authenticated: [],
    disconnected: [],
    durable: [],
  };
  const budget: EchoCore.TerminalBudget = {
    schema_version: 1,
    version: 'scripted',
    max_frame_bytes: 8192,
    max_chunk_bytes: 40,
    max_object_bytes: 4096,
    max_uploads: 2,
    max_upload_bytes: 8192,
  };
  const server = createServer((socket) => {
    sockets.add(socket);
    socket.on('error', () => socket.destroy());
    socket.once('close', () => sockets.delete(socket));
    let session: EchoCore.TerminalSession | undefined;
    const reader = new FrameReader();
    const uploads = new Map<string, Buffer>();
    const objects = new Map<string, Buffer>();
    const key = (ref: EchoCore.ObjectRef) => Buffer.from(ref.digest).toString('hex');
    socket.on('data', (data: Buffer) => {
      try {
        for (const body of reader.push(data)) {
          if (!session) {
            const auth = decodeFrame('TerminalAuthenticate', body);
            if (auth.installation_id !== installation || !token().equals(auth.token)) {
              throw new Error('Authentication mismatch');
            }
            session = {
              schema_version: 1,
              protocol_version: 1,
              installation_id: installation,
              session_id: randomUUID(),
              capabilities: [1, 2, 4, 5],
              budget,
            };
            timestamps.authenticated.push(performance.now());
            socket.write(
              encodeFrame('TerminalAuthResult', {
                schema_version: 1,
                request_id: auth.request_id,
                status: 1,
                session,
              }),
            );
            continue;
          }
          const request = decodeFrame('TerminalRequest', body);
          checkRequest(request, session, frameLimits);
          const reply = {
            schema_version: 1,
            request_id: request.request_id,
            session_id: session.session_id,
            protocol_version: 1,
            reply_to: request.kind,
          } satisfies Partial<EchoCore.TerminalResponse>;
          if (request.upload) {
            const upload = request.upload;
            const previous = uploads.get(upload.upload_id) ?? Buffer.alloc(0);
            if (BigInt(previous.length) !== upload.offset) throw new Error('Invalid upload offset');
            const bytes = Buffer.concat([previous, upload.data]);
            if (upload.final_chunk) {
              checkUploaded(upload.object, bytes, budget, frameLimits);
              uploads.delete(upload.upload_id);
              objects.set(key(upload.object), bytes);
            } else {
              uploads.set(upload.upload_id, bytes);
            }
            socket.write(
              encodeFrame('TerminalResponse', {
                ...reply,
                kind: 2,
                upload: {
                  schema_version: 1,
                  upload_id: upload.upload_id,
                  object: upload.object,
                  next_offset: BigInt(bytes.length),
                  status: upload.final_chunk ? 2 : 1,
                },
              }),
            );
          } else if (request.event) {
            const e = request.event;
            const payload = decodeCanonical(
              'UserText',
              objects.get(key(e.payload_ref))!,
              frameLimits,
            );
            const provenance = decodeCanonical(
              'SourceMetadata',
              objects.get(key(e.provenance))!,
              frameLimits,
            );
            if (key(provenance.source_span) !== key(e.payload_ref))
              throw new Error('Incorrect provenance');
            const existing = messages.get(e.stable_source_key);
            if (existing !== undefined && existing !== payload.text) throw new Error('ID conflict');
            input.push(e);
            if (options.rejectFirstEvent !== undefined && input.length === 1) {
              socket.write(
                encodeFrame('TerminalResponse', {
                  ...reply,
                  kind: enums.IngressResponseKind!
                    .INGRESS_REJECTED as EchoCore.TerminalResponse['kind'],
                  reason: options.rejectFirstEvent,
                }),
              );
              objects.delete(key(e.payload_ref));
              objects.delete(key(e.provenance));
              continue;
            }
            messages.set(e.stable_source_key, payload.text);
            if (options.dropFirstReceipt && input.length === 1) {
              timestamps.disconnected.push(performance.now());
              socket.destroy();
              return;
            }
            const receipt = {
              schema_version: 1,
              stable_source_key: e.stable_source_key,
              event_id: e.id,
            };
            timestamps.durable.push(performance.now());
            // Coalesce durable and settlement frames to exercise the stream reader.
            socket.write(
              Buffer.concat([
                encodeFrame('TerminalResponse', {
                  ...reply,
                  kind: 1,
                  receipt: {
                    ...receipt,
                    stage: 1,
                    raw_log_position: BigInt(input.length),
                    ingest_time_ns: FixtureTime,
                  },
                }),
                encodeFrame('TerminalResponse', {
                  ...reply,
                  kind: 1,
                  receipt: { ...receipt, stage: existing === undefined ? 2 : 3 },
                }),
              ]),
            );
            objects.delete(key(e.payload_ref));
            objects.delete(key(e.provenance));
          } else {
            controls.push(request);
            socket.write(encodeFrame('TerminalResponse', { ...reply, kind: 4 }));
          }
        }
      } catch (error) {
        failures.push(error);
        socket.destroy();
      }
    });
  }).listen(path);
  await once(server, 'listening');

  return {
    input,
    messages,
    controls,
    failures,
    timestamps,
    identity: async () => ({ pipe: path, installationId: installation, token: token() }),
    disconnect: () => {
      for (const s of sockets) s.destroy();
    },
    close: async () => {
      for (const s of sockets) s.destroy();
      await new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
    },
  };
}

export function scriptedChunk(text: string, chunk_index: bigint): EchoCore.OutputChunk {
  return {
    schema_version: 1,
    output_id: 'scripted-output',
    action_id: 'scripted-action',
    chunk_index,
    text,
    lang: enums.Lang!.EN as EchoCore.Lang,
    snapshot_id: 'scripted-snapshot',
    speech_mode: enums.SpeechMode!.B_REMOTE_PROJECTED as EchoCore.SpeechMode,
    strict: false,
  };
}

export const scriptApproval: EchoCore.ApprovalRequestV2 = {
  schema_version: 1,
  approval_id: 'scripted-approval',
  action_id: 'scripted-action',
  tier: 2,
  title: 'Scripted approval',
  detail: 'Synthetic payload',
  expires_at_ns: BigInt(Date.now() + 120_000) * 1_000_000n,
  action_payload: Buffer.from('fixture'),
  payload_digest: createHash('sha256').update('fixture').digest(),
};
