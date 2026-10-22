import type { ApprovalDecisionV2, ApprovalRequestV2 } from './generated/echo/schema.js';

function same(a: Uint8Array, b: Uint8Array): boolean {
  return a.length === b.length && a.every((v, i) => v === b[i]);
}

/** Pure validation boundary. P5-05 owns authentication, single-use and execution.
 * nowNs and the actual execution payload must come from that gate, never the panel.
 */
export async function approvalDecisionMatches(
  request: ApprovalRequestV2,
  decision: ApprovalDecisionV2,
  executionPayload: Uint8Array,
  nowNs: bigint,
): Promise<boolean> {
  if (
    request.schema_version !== 1 ||
    decision.schema_version !== 1 ||
    !request.approval_id ||
    request.approval_id !== decision.approval_id ||
    request.payload_digest.length !== 32 ||
    decision.payload_digest.length !== 32 ||
    nowNs >= request.expires_at_ns ||
    ![1, 2, 3, 4].includes(request.tier) ||
    ![1, 2].includes(decision.decision) ||
    (request.tier === 4 && decision.decision === 2) ||
    !same(request.action_payload, executionPayload) ||
    !same(request.payload_digest, decision.payload_digest)
  )
    return false;
  const digest = new Uint8Array(
    await crypto.subtle.digest('SHA-256', Uint8Array.from(executionPayload)),
  );
  return same(digest, request.payload_digest);
}
