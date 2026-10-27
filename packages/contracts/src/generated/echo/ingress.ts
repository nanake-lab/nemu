// ADR 0024 pure contract checks. Session must come from the authenticated connection.
import type {
  TerminalSession,
  TerminalRequest,
  TerminalResponse,
  TerminalAuthenticate,
  TerminalAuthResult,
  ObjectUploadChunk,
  ObjectRef,
  TerminalBudget,
} from './schema.js';
import {
  normalize,
  encodeProtobuf,
  decodeCanonicalData,
  digest,
  requireValue as check,
  type Limits,
} from './runtime.js';

import { schema } from './schema.js';
const uploadTypes: Readonly<Record<string, string>> = Object.fromEntries(
  Object.entries(schema.messages)
    .filter(([, m]) => !m.transient && !m.superseded && m.file !== 'terminal_ingress.proto')
    .map(([name, m]) => [m.tag, name]),
);
export function checkUploadWindow(
  c: ObjectUploadChunk,
  b: TerminalBudget,
  nextOffset: bigint,
  active: number,
  reserved: bigint,
  declaredId: string,
  declaredObject: ObjectRef,
  limits: Limits,
): void {
  checkChunk(c, b, limits);
  check(
    c.upload_id === declaredId &&
      c.object.schema_version === declaredObject.schema_version &&
      c.object.object_type === declaredObject.object_type &&
      c.object.resource_version === declaredObject.resource_version &&
      c.object.byte_length === declaredObject.byte_length &&
      Buffer.from(c.object.digest).equals(Buffer.from(declaredObject.digest)),
    'upload declaration changed',
  );
  check(c.offset === nextOffset, 'nonsequential upload');
  check(
    Number.isInteger(active) &&
      active > 0 &&
      active <= b.max_uploads &&
      reserved >= c.object.byte_length &&
      reserved <= BigInt(b.max_upload_bytes),
    'session upload budget',
  );
}
export function checkSession(s: TerminalSession, limits: Limits): void {
  normalize('TerminalSession', s, limits);
  const b = s.budget;
  check(
    b.max_chunk_bytes <= b.max_object_bytes &&
      b.max_object_bytes <= b.max_upload_bytes &&
      b.max_chunk_bytes < b.max_frame_bytes,
    'terminal budget',
  );
}
export function checkAuthPair(
  q: TerminalAuthenticate,
  a: TerminalAuthResult,
  limits: Limits,
): void {
  normalize('TerminalAuthenticate', q, limits);
  normalize('TerminalAuthResult', a, limits);
  check(q.request_id === a.request_id, 'auth pairing');
  if (a.session) {
    checkSession(a.session, limits);
    check(
      q.installation_id === a.session.installation_id &&
        q.protocol_versions.includes(a.session.protocol_version),
      'auth negotiation',
    );
  }
}
export function checkChunk(c: ObjectUploadChunk, b: TerminalBudget, limits: Limits): void {
  normalize('ObjectUploadChunk', c, limits);
  normalize('TerminalBudget', b, limits);
  const end = c.offset + BigInt(c.data.length);
  check(
    c.data.length > 0 &&
      c.data.length <= b.max_chunk_bytes &&
      c.object.byte_length <= BigInt(b.max_object_bytes),
    'chunk budget',
  );
  check(
    end < 2n ** 64n &&
      end <= c.object.byte_length &&
      c.final_chunk === (end === c.object.byte_length),
    'chunk extent',
  );
  check(Object.hasOwn(uploadTypes, c.object.object_type), 'upload type');
}
export function checkUploaded(
  ref: ObjectRef,
  data: Uint8Array,
  b: TerminalBudget,
  limits: Limits,
): void {
  normalize('ObjectRef', ref, limits);
  normalize('TerminalBudget', b, limits);
  check(
    BigInt(data.length) === ref.byte_length && data.length <= b.max_object_bytes,
    'object size',
  );
  check(Object.hasOwn(uploadTypes, ref.object_type), 'upload type');
  const name = uploadTypes[ref.object_type];
  const value = decodeCanonicalData(name, Buffer.from(data), limits);
  check(digest(name, value, limits) === Buffer.from(ref.digest).toString('hex'), 'object digest');
}
export function checkRequest(
  q: TerminalRequest,
  s: TerminalSession | undefined,
  limits: Limits,
): void {
  const bytes = encodeProtobuf('TerminalRequest', q, limits);
  check(s, 'unauthenticated dispatch');
  checkSession(s, limits);
  check(
    q.session_id === s.session_id && q.protocol_version === s.protocol_version,
    'session mismatch',
  );
  check(
    s.capabilities.includes(q.kind) && bytes.length <= s.budget.max_frame_bytes,
    'request budget or capability',
  );
  if (q.event)
    check(
      Buffer.from(q.event.payload_digest).equals(Buffer.from(q.event.payload_ref.digest)),
      'event digest',
    );
  if (q.upload) checkChunk(q.upload, s.budget, limits);
}
export function checkResponse(
  q: TerminalRequest,
  a: TerminalResponse,
  s: TerminalSession,
  limits: Limits,
): void {
  checkRequest(q, s, limits);
  check(
    encodeProtobuf('TerminalResponse', a, limits).length <= s.budget.max_frame_bytes,
    'response budget',
  );
  check(
    q.request_id === a.request_id &&
      q.session_id === a.session_id &&
      q.protocol_version === a.protocol_version &&
      q.kind === a.reply_to,
    'response pairing',
  );
  if (a.kind === 5) return;
  check(a.kind === (q.kind <= 3 ? q.kind : 4), 'reply kind');
  if (q.kind === 1)
    check(
      q.event &&
        a.receipt &&
        q.event.id === a.receipt.event_id &&
        q.event.stable_source_key === a.receipt.stable_source_key,
      'event pairing',
    );
  if (q.kind === 2) {
    const c = q.upload,
      v = a.upload;
    check(c && v, 'upload pairing');
    check(
      c.upload_id === v.upload_id &&
        c.offset + BigInt(c.data.length) === v.next_offset &&
        c.final_chunk === (v.status === 2),
      'upload pairing',
    );
    check(
      c.object.object_type === v.object.object_type &&
        c.object.resource_version === v.object.resource_version &&
        c.object.byte_length === v.object.byte_length &&
        Buffer.from(c.object.digest).equals(Buffer.from(v.object.digest)),
      'upload object pairing',
    );
  }
  if (q.kind === 3)
    check(q.query && a.query && q.query.query_id === a.query.query_id, 'query pairing');
}
