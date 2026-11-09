// ADR 0035 §4 / P5-68: boundary after structural decoding; no provenance claim.
import { normalize, type Limits } from './runtime.js';

const names = new Set([
  'FastBasisIdentity',
  'LawLearnedCoordinate',
  'LawStateVector',
  'FastBasisSlot',
  'FastStateSnapshotV3',
  'FastEffectRecord',
  'FastStateMigrationRecord',
]);
type Obj = Record<string, unknown>;
// Only normalized schema fields reach these internal helpers.
const obj = (v: unknown): Obj => v as Obj;
const list = (v: unknown): Obj[] => v as Obj[];
function require(ok: boolean, why: string): void {
  if (!ok) throw new Error(why);
}
function coordinates(v: Obj): string[] {
  return list(v.learned).map((c) => c.tendency_id as string);
}
function aligned(a: Obj, b: Obj): boolean {
  const x = coordinates(a),
    y = coordinates(b);
  return x.length === y.length && x.every((id, i) => id === y[i]);
}
function slot(v: Obj): void {
  require(aligned(obj(v.current), obj(v.target)), 'slot coordinates');
}
export function checkLawStateMessage(name: string, value: unknown, limits: Limits): void {
  if (!names.has(name)) return;
  const v = normalize(name, value, limits);
  if (name === 'FastBasisSlot') slot(v);
  else if (name === 'FastStateSnapshotV3') {
    for (const s of list(v.slots)) {
      slot(s);
      require(aligned(obj(s.current), v), 'snapshot coordinates');
    }
    require(obj(v.law_version).object_type === 'echo.law-version', 'snapshot law reference');
  } else if (name === 'FastEffectRecord') {
    require(list(v.interpretation_refs).length > 0 &&
      list(v.governance_refs).length > 0, 'effect references');
    require(obj(v.theta_before).object_type === 'echo.theta-snapshot-v3', 'effect theta reference');
    require(obj(v.law_version).object_type === 'echo.law-version', 'effect law reference');
    require(v.role !== 4 || obj(v.basis).kind === 2, 'internal role');
    require(v.role !== 1 || obj(v.basis).kind === 1, 'new basis role');
    // ADR 0035 §4.2–4.3; §5.2 still permits internal target updates.
    require(obj(v.basis).kind !== 2 || v.role !== 3, 'internal revisit');
  } else if (name === 'FastStateMigrationRecord') {
    require(v.prior_decomposition_unknown === true, 'unknown prior decomposition');
    require(obj(v.previous_snapshot).object_type === 'echo.fast-state-v2', 'migration predecessor');
    require(obj(v.migrated_snapshot).object_type === 'echo.fast-state-v3', 'migration successor');
  }
}
