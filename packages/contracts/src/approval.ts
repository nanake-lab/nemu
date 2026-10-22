import type { ApprovalDecisionV2, ApprovalRequestV2 } from './generated/echo/schema.js';

function same(a: Uint8Array, b: Uint8Array): boolean {
  return a.length === b.length && a.every((v, i) => v === b[i]);
}
function hex(bytes: Uint8Array): string {
  return Array.from(bytes, (v) => v.toString(16).padStart(2, '0')).join('');
}
type ApprovalBinding = Readonly<{
  approvalId: string;
  actionId: string;
  tier: number;
  decision: number;
  payloadDigest: string;
}>;
const constructionKey = Symbol('validated approval');

/** This is payload validation, not identity, permission or an execution credential (P5-05).
 * Only an immutable hex string is exposed; no mutable caller buffer can reach execution.
 */
class ValidatedApproval {
  #used = false;
  readonly #payload: string;
  readonly #binding: ApprovalBinding;
  readonly #expiresAt: bigint;
  readonly #validatedAt: bigint;
  constructor(key: symbol, request: ApprovalRequestV2, decision: ApprovalDecisionV2, at: bigint) {
    if (key !== constructionKey) throw new Error('Unvalidated approval');
    this.#payload = hex(request.action_payload);
    this.#expiresAt = request.expires_at_ns;
    this.#validatedAt = at;
    this.#binding = Object.freeze({
      approvalId: request.approval_id,
      actionId: request.action_id,
      tier: request.tier,
      decision: decision.decision,
      payloadDigest: hex(request.payload_digest),
    });
    Object.freeze(this);
  }
  /** Gate supplies fresh time. Executor consumes this same immutable validated snapshot, once.
   * A throw or failed execution consumes it too; it cannot authorize a retry.
   */
  consume<T>(nowNs: bigint, execute: (payloadHex: string, binding: ApprovalBinding) => T): T {
    if (this.#used) throw new Error('Approval unavailable');
    this.#used = true;
    if (nowNs < this.#validatedAt || nowNs >= this.#expiresAt)
      throw new Error('Approval unavailable');
    return execute(this.#payload, this.#binding);
  }
}

/** Snapshot before awaiting, reject changes during hashing, and return the snapshot to consume.
 * No bare boolean can be used to authorize execution against a different buffer.
 */
export async function validateApproval(
  request: ApprovalRequestV2,
  decision: ApprovalDecisionV2,
  executionPayload: Uint8Array,
  nowNs: bigint,
): Promise<ValidatedApproval | undefined> {
  const r = {
    ...request,
    action_payload: Uint8Array.from(request.action_payload),
    payload_digest: Uint8Array.from(request.payload_digest),
  };
  const d = { ...decision, payload_digest: Uint8Array.from(decision.payload_digest) };
  const payload = Uint8Array.from(executionPayload);
  const scalarsMatch = () =>
    r.schema_version === request.schema_version &&
    r.approval_id === request.approval_id &&
    r.action_id === request.action_id &&
    r.tier === request.tier &&
    r.expires_at_ns === request.expires_at_ns &&
    r.title === request.title &&
    r.detail === request.detail &&
    d.schema_version === decision.schema_version &&
    d.approval_id === decision.approval_id &&
    d.decision === decision.decision;
  const bytesMatch = () =>
    same(r.action_payload, request.action_payload) &&
    same(r.payload_digest, request.payload_digest) &&
    same(d.payload_digest, decision.payload_digest) &&
    same(payload, executionPayload);
  if (
    r.schema_version !== 1 ||
    d.schema_version !== 1 ||
    !r.approval_id ||
    !r.action_id ||
    r.approval_id !== d.approval_id ||
    r.payload_digest.length !== 32 ||
    d.payload_digest.length !== 32 ||
    nowNs >= r.expires_at_ns ||
    ![1, 2, 3, 4].includes(r.tier) ||
    ![1, 2].includes(d.decision) ||
    (r.tier === 4 && d.decision === 2) ||
    !same(r.action_payload, payload) ||
    !same(r.payload_digest, d.payload_digest)
  )
    return undefined;
  const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', payload));
  if (!scalarsMatch() || !bytesMatch() || !same(digest, r.payload_digest)) return undefined;
  return new ValidatedApproval(constructionKey, r, d, nowNs);
}
