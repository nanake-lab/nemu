// ADR 0047. The authenticated service separately checks issuer, holder and live resources.
import { createHash } from 'node:crypto';
import { isDeepStrictEqual as equal } from 'node:util';
import { checkSpeakThinkMessage } from './speak_think.js';
import { normalize, type Limits } from './runtime.js';
import { schema, type MessageMap } from './schema.js';

function require(ok: boolean, message: string): void {
  if (!ok) throw new Error(message);
}

function scope(s: MessageMap['ConnectorScope']): void {
  require(s.wallet_minor_units === undefined || s.wallet_minor_units > 0n, 'empty wallet transfer');
}

function budget(b: MessageMap['ExecutionBudget']): void {
  require(b.charge <= b.limit, 'charge exceeds grant');
}

function trust(t: MessageMap['TrustMemory']): void {
  require(t.tier === 2 || t.tier === 3, 'only T1/T2 trust');
  scope(t.scope);
}

function command(c: MessageMap['GrowthCommandV2']): void {
  if (c.kind === 3) {
    require(c.alarm_record!.object_type === 'echo.system-maintenance-record', 'wrong alarm record');
    require((c.replacement_law !== undefined) === (c.verdict === 2), 'replacement law');
    require(!equal(c.replacement_law, c.law_version), 'law unchanged');
  }
}

function management(m: MessageMap['GrowthManagementRequest']): void {
  command(m.command);
  require(m.entry === 2 || m.command.kind !== 3, 'review requires echoctl');
}

function grant(g: MessageMap['ExecutionCredential']): void {
  const s = g.scope;
  scope(s);
  budget(g.budget);
  require(g.issued_at_ns < g.expires_at_ns, 'empty validity interval');
  if (g.trust) {
    trust(g.trust);
    require(equal(g.trust.scope, s) &&
      g.trust.tier === g.tier &&
      g.trust.revocation_version === g.revocation_version, 'trust binding');
  }
  if (g.consent) {
    require(g.consent.action_id === g.action_id &&
      equal(g.consent.payload_digest, g.payload_digest) &&
      g.consent.confirmed_at_ns <= g.issued_at_ns, 'consent binding');
  }
  if (s.kind === 2) {
    if (!s.reversible) {
      const tier = s.desktop!.irreversible;
      require(tier === 3 || (tier === 2 && !!g.consent), 'desktop policy');
    }
  } else if (g.tier === 4 || (s.kind === 1 && (g.tier === 2 || g.tier === 3) && !g.trust)) {
    require(!!g.consent, 'missing consent');
  }
}

function result(m: MessageMap['PermissionResult']): void {
  require(!m.management_commit ||
    [1, 2, 3].includes(m.management_commit.kind), 'invalid maintenance result');
}

export function checkPermissionMessage(
  name: string,
  value: Record<string, unknown>,
  limits: Limits,
): void {
  if (schema.messages[name as keyof MessageMap]?.file !== 'permission.proto') return;
  const normalized = normalize(name, value, limits);
  const typed = <K extends keyof MessageMap>(): MessageMap[K] =>
    normalized as unknown as MessageMap[K];
  switch (name) {
    case 'PermissionResult':
      return result(typed<'PermissionResult'>());
    case 'TerminalResponseV2': {
      const q = typed<'TerminalResponseV2'>();
      const m = q.management;
      require((q.reply_to === 6) === !!m, 'missing or unexpected management reply');
      if (m) {
        result(m);
        require(q.request_id === m.request_id &&
          ((q.kind === 4 && !!m.management_commit) ||
            (q.kind === 5 && !!m.denied)), 'wrong management reply');
      }
      break;
    }
    case 'PermissionContext': {
      const c = typed<'PermissionContext'>();
      if (c.local_execution) return grant(c.local_execution);
      return checkSpeakThinkMessage(
        'RemoteCognitiveReceipt',
        c.remote_text as unknown as Record<string, unknown>,
        limits,
      );
    }
    case 'ConnectorScope':
      return scope(typed<'ConnectorScope'>());
    case 'ExecutionBudget':
      return budget(typed<'ExecutionBudget'>());
    case 'TrustMemory':
      return trust(typed<'TrustMemory'>());
    case 'ExecutionCredential':
      return grant(typed<'ExecutionCredential'>());
    case 'GrowthCommandV2':
      return command(typed<'GrowthCommandV2'>());
    case 'GrowthManagementRequest':
      return management(typed<'GrowthManagementRequest'>());
    case 'TerminalRequestV2': {
      const q = typed<'TerminalRequestV2'>();
      if (q.management) {
        management(q.management);
        require(q.session_id === q.management.user.session_id, 'wrong session');
      }
      break;
    }
    case 'ActionRequestV2': {
      const q = typed<'ActionRequestV2'>();
      grant(q.grant);
      require(q.action_id === q.grant.action_id &&
        q.expires_at_ns === q.grant.expires_at_ns &&
        createHash('sha256')
          .update(q.action_payload)
          .digest()
          .equals(q.grant.payload_digest), 'dispatch binding');
      break;
    }
  }
}
