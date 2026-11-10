// ADR 0030: call after structural decoding, before consuming. This is not
// reference resolution, current-Q authorization, carrier proof or atomic publish.
import { normalize, encodeCanonicalData, type Limits } from './runtime.js';
type Obj = Record<string, unknown>;
const names = new Set([
  'CognitionBinding',
  'CognitiveMaterial',
  'CognitiveReturnedResult',
  'CognitiveInput',
  'CognitiveResultAndReceipt',
  'InternalActivityRecord',
  'ActivityState',
  'CognitiveInflightAction',
  'CognitionState',
  'InternalActivityCommit',
  'CoreTransactionV3',
  'SystemMaintenanceRecord',
]);
function require(ok: boolean, message: string): void {
  if (!ok) throw new Error(message);
}
const obj = (v: unknown): Obj => v as Obj; // structural codec already validated shape
function equalBytes(a: unknown, b: unknown): boolean {
  const x = a as Uint8Array,
    y = b as Uint8Array;
  return x.length === y.length && x.every((v, i) => v === y[i]);
}
export function checkCognitionMessage(name: string, value: Obj, limits: Limits): void {
  if (!names.has(name)) return;
  const v = normalize(name, value, limits);
  const tag = (key: string, expected: string): void => {
    if (key in v) require(obj(v[key])['object_type'] === expected, `${name}.${key} type`);
  };
  const child = (key: string, type: string): void => {
    if (key in v) checkCognitionMessage(type, obj(v[key]), limits);
  };
  if (name === 'CognitiveReturnedResult') {
    require(v['status'] !== 2 || 'result' in v, 'successful result missing');
    require(v['status'] !== 1 || !('result' in v), 'waiting has result');
  } else if (name === 'CognitiveInput') {
    for (const item of v['results'] as Obj[])
      checkCognitionMessage('CognitiveReturnedResult', item, limits);
  } else if (name === 'CognitiveResultAndReceipt') {
    tag('input', 'echo.cognitive-input');
    tag('modulation', 'echo.modulation-v2');
    const b = obj(v['binding']),
      env = obj(b['environment']);
    require(equalBytes(v['model_hash'], obj(env['model'])['digest']), 'model binding');
    require(equalBytes(v['tokenizer_hash'], obj(env['tokenizer'])['digest']), 'tokenizer binding');
    require(![1, 2].includes(v['completion'] as number) ||
      'carrier_receipt' in v, 'carrier receipt missing');
    if (v['completion'] !== 1) {
      require('reason' in v, 'incomplete reason missing');
      require(!('next_focus' in v) &&
        !('action_plan' in v) &&
        (v['queries'] as unknown[]).length === 0, 'incomplete successor');
    }
    if ('action_plan' in v) {
      const p = obj(v['action_plan']);
      require(p['snapshot_id'] === b['snapshot_id'], 'action snapshot');
      require(p['task_version'] === b['q_revision'], 'action Q revision');
      require(p['protocol_version'] === obj(b['protocol'])['resource_version'], 'action protocol');
    }
  } else if (name === 'InternalActivityRecord') {
    tag('previous', 'echo.internal-activity-record');
    require((v['finished_at_ns'] as bigint) >=
      (v['started_at_ns'] as bigint), 'request execution interval');
    child('result', 'CognitiveResultAndReceipt');
  } else if (name === 'ActivityState') {
    tag('latest_step', 'echo.internal-activity-record');
    tag('successor', 'echo.cognitive-input');
  } else if (name === 'CognitiveInflightAction') {
    tag('origin_record', 'echo.internal-activity-record');
  } else if (name === 'CognitionState') {
    tag('active_request', 'echo.cognitive-input');
    tag('runnable_successor', 'echo.cognitive-input');
    tag('transition', 'echo.system-maintenance-record');
    const count = Number('active_request' in v) + Number('runnable_successor' in v);
    require(v['mode'] === 1 ? count === 1 : count === 0 && 'reason' in v, 'global continuation');
    for (const item of v['activities'] as Obj[])
      checkCognitionMessage('ActivityState', item, limits);
    for (const item of v['inflight'] as Obj[])
      checkCognitionMessage('CognitiveInflightAction', item, limits);
  } else if (name === 'InternalActivityCommit') {
    child('record', 'InternalActivityRecord');
    require((v['interpretations'] as unknown[]).length >
      0, 'accepted interpretation outcomes missing');
    require(equalBytes(
      encodeCanonicalData('CognitionBinding', obj(v['binding']), limits),
      encodeCanonicalData(
        'CognitionBinding',
        obj(obj(v['record'])['result'])['binding'] as Obj,
        limits,
      ),
    ), 'internal binding');
  } else if (name === 'CoreTransactionV3') child('internal_commit', 'InternalActivityCommit');
}
