import * as c from '../../packages/contracts/src/generated/echo/ingress.js';
import type { MessageMap } from '../../packages/contracts/src/generated/echo/schema.js';
import { fromJson } from '../../packages/contracts/src/generated/echo/bridge.js';
import { normalize, type Limits } from '../../packages/contracts/src/generated/echo/runtime.js';
export function ingressProbe(q: Record<string, unknown>, limits: Limits): void {
  function field<K extends keyof MessageMap>(key: string, name: K): MessageMap[K] {
    return normalize(
      name,
      fromJson(name, q[key] as Record<string, unknown>),
      limits,
    ) as unknown as MessageMap[K];
  }
  switch (q.check) {
    case 'auth':
      c.checkAuthPair(
        field('request', 'TerminalAuthenticate'),
        field('response', 'TerminalAuthResult'),
        limits,
      );
      break;
    case 'request':
      c.checkRequest(
        field('request', 'TerminalRequest'),
        q.session ? field('session', 'TerminalSession') : undefined,
        limits,
      );
      break;
    case 'response':
      c.checkResponse(
        field('request', 'TerminalRequest'),
        field('response', 'TerminalResponse'),
        field('session', 'TerminalSession'),
        limits,
      );
      break;
    case 'window':
      c.checkUploadWindow(
        field('chunk', 'ObjectUploadChunk'),
        field('budget', 'TerminalBudget'),
        BigInt(q.next_offset as number),
        q.active as number,
        BigInt(q.reserved as number),
        (q.declared_id as string | undefined) ?? field('chunk', 'ObjectUploadChunk').upload_id,
        q.declared_object
          ? field('declared_object', 'ObjectRef')
          : field('chunk', 'ObjectUploadChunk').object,
        limits,
      );
      break;
    case 'chunk':
      c.checkChunk(field('chunk', 'ObjectUploadChunk'), field('budget', 'TerminalBudget'), limits);
      break;
    case 'object':
      c.checkUploaded(
        field('reference', 'ObjectRef'),
        Buffer.from(q.hex as string, 'hex'),
        field('budget', 'TerminalBudget'),
        limits,
      );
      break;
    default:
      throw new Error('unknown ingress check');
  }
}
