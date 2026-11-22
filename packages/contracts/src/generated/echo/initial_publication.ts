// ADR 0049. Resource resolution and the single-writer capability belong to the trusted caller.
import { normalize, type Limits } from './runtime.js';
import type { BirthNameRecord, InitialPublication, InitialRuntimeRoot, ObjectRef } from './schema.js';

function check(ok: boolean, message: string): void {
  if (!ok) throw new Error(message);
}

function sameRef(a: ObjectRef, b: ObjectRef | undefined): boolean {
  return b !== undefined && a.object_type === b.object_type && a.resource_version === b.resource_version
    && a.byte_length === b.byte_length && Buffer.compare(a.digest, b.digest) === 0;
}

function runtime(q: InitialRuntimeRoot): void {
  const env = q.environment;
  for (const [stage, resource] of [
    [q.irl, env.irl],
    [q.appraiser, env.nature_appraiser],
    [q.matcher, env.situation_matcher],
  ]) {
    check(!stage || sameRef(stage, resource), 'initial stage resource mismatch');
  }
  for (const stage of [q.fast, q.growth, q.dynamics, q.compiler]) {
    check(!stage || env.implementations.some(ref => sameRef(ref, stage)), 'unbound initial implementation');
  }
  check(!(q.fast || q.growth || q.dynamics) || !!env.law_version, 'initial law missing');
  check(!q.compiler || !!env.direction_set, 'initial directions missing');
}

function birth(b: BirthNameRecord): void {
  check(['echo.genesis-v2', 'echo.genesis-v3'].includes(b.manifest.object_type)
    && b.manifest.resource_version === '1', 'birth manifest type or schema version');
  check(b.signature_evidence.object_type === 'nemu.storage.raw-bytes-v1'
    && b.signature_evidence.resource_version === '1' && b.signature_evidence.byte_length > 0n, 'birth signature evidence');
}

export function checkInitialMessage(name: string, value: Record<string, unknown>, limits: Limits): void {
  if (!['EmptyPrefixRoot', 'InitialRuntimeRoot', 'BirthNameRecord', 'InitialPublication'].includes(name)) return;
  const v = normalize(name, value, limits);
  if (name === 'EmptyPrefixRoot') return;
  if (name === 'InitialRuntimeRoot') return runtime(v as unknown as InitialRuntimeRoot);
  if (name === 'BirthNameRecord') return birth(v as unknown as BirthNameRecord);
  const p = v as unknown as InitialPublication;
  const q = p.q;
  runtime(q);
  check(['echo.fast-state-v2', 'echo.fast-state-v3'].includes(p.x.object_type)
    && p.r.object_type === 'echo.r-snapshot-v3'
    && ['echo.theta-snapshot-v2', 'echo.theta-snapshot-v3'].includes(p.theta.object_type), 'initial snapshot type');
  if (p.header.lineage === 1) {
    check(!q.growth && !q.dynamics, 'rehearsal growth');
  } else {
    check([q.irl, q.appraiser, q.matcher, q.fast, q.growth, q.dynamics, q.compiler].every(Boolean), 'incomplete initial stages');
  }
  if (p.header.lineage === 2) {
    if (!p.birth) throw new Error('missing birth record');
    birth(p.birth);
    check(!!p.header.genesis_manifest_digest
      && Buffer.compare(p.header.genesis_manifest_digest, p.birth.manifest.digest) === 0, 'birth manifest digest');
  } else {
    check(p.empty_history?.kind === 1, 'initial H requires empty history');
  }
}
