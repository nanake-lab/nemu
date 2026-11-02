// P5-36: explicit semantic boundary after the structural codec.
import { normalize, digest, type Limits } from './runtime.js';
export const THETA_NUMERICS_ID = 'P5-25-v2-93-98-libm-0.2.16';
const exact = 1n << 53n;
const names = new Set([
  'ThetaEstimatorParameters',
  'ThetaRunningTime',
  'ThetaLastEvent',
  'ThetaEstimatorState',
  'ThetaSnapshotV2',
]);
type Obj = Record<string, unknown>;
function require(ok: boolean, why: string): void {
  if (!ok) throw new Error(why);
}
function obj(v: unknown): Obj {
  if (typeof v !== 'object' || v === null || Array.isArray(v)) throw new Error('object');
  return v as Obj;
}
function num(v: unknown): number {
  if (typeof v !== 'number') throw new Error('number');
  return v;
}
function int(v: unknown): bigint {
  if (typeof v !== 'bigint') throw new Error('integer');
  return v;
}
function arr(v: unknown): unknown[] {
  if (!Array.isArray(v)) throw new Error('array');
  return v;
}
function scale(v: unknown): number[] {
  const o = obj(v);
  return ['connection', 'empathy', 'curiosity', 'achievement'].map((k) => num(o[k]));
}
function parameters(p: Obj): void {
  require(p.numerics_id === THETA_NUMERICS_ID, 'theta numerics version');
  for (const k of [
    'baseline_weight',
    'variance_weight',
    'duration_weight',
    'change_prior_weight',
    'gain_min',
    'gain_max',
    'recovery_min',
    'recovery_max',
    'detector_shift_sigma',
    'detector_threshold',
  ])
    require(num(p[k]) > 0 && num(p[k]) < Number(exact), 'theta positive exact parameter');
  for (const k of ['change_prior', 'change_min']) {
    const h = num(p[k]);
    require(h > 0 && h < 1 && 1 - h !== 1, 'theta representable retention');
  }
  for (const k of ['gain_prior', 'sigma_prior', 'tau_prior_s'])
    require(scale(p[k]).every((x) => x > 0), 'theta positive prior');
  require(num(p.gain_min) <= num(p.gain_max) &&
    num(p.recovery_min) <= num(p.recovery_max), 'theta ordered bounds');
  require(scale(p.gain_prior).every(
    (x) => x >= num(p.gain_min) && x <= num(p.gain_max),
  ), 'theta gain prior bounds');
  require(scale(p.sigma_prior).every(
    (x) => Number.isFinite(x * x) && x * x > 0,
  ), 'theta prior variance');
  require(num(p.jump_notice) > 0 &&
    int(p.drift_run) > 0n &&
    int(p.drift_run) < exact, 'theta calibration');
}
function running(t: Obj): void {
  const elapsed = int(t.event_ns) - int(t.origin_ns),
    stopped = int(t.stopped_ns);
  require(elapsed >= 0n &&
    stopped <= elapsed &&
    int(t.running_ns) === elapsed - stopped, 'theta running time identity');
}
function state(s: Obj): void {
  for (const k of [
    'reaction_sum',
    'square_sum',
    'variance',
    'duration_log_sum',
    'duration_mass',
    'duration_log_mean',
    'zero_duration_count',
    'run_sign',
    'run_start_ns',
    'detector_mean',
    'detector_positive',
    'detector_negative',
  ])
    require(arr(s[k]).length === 4, 'theta four channels');
  for (const k of ['drift_sign', 'drift_count'])
    require(arr(s[k]).length === 12, 'theta twelve components');
  const n = int(s.event_count),
    h = num(s.h);
  require(n < exact, 'theta exact event count');
  require(h > 0 && h < 1 && 1 - h !== 1, 'theta representable h');
  require([...scale(s.sigma), ...scale(s.tau_s), ...arr(s.variance).map(num)].every(
    (x) => x > 0,
  ), 'theta positive estimates');
  for (const k of ['detector_segment_count', 'detector_changes'])
    require(int(s[k]) <= n, 'theta detector count');
  for (const k of ['zero_duration_count', 'drift_count'])
    require(arr(s[k]).every((x) => int(x) <= n), 'theta count exceeds events');
  const has = s.last_event !== undefined;
  require(has === n > 0n &&
    (s.last_running_time !== undefined) === has, 'theta last event presence');
  if (has) {
    running(obj(s.last_running_time));
    require(obj(s.last_event).time_ns ===
      obj(s.last_running_time).event_ns, 'theta last time mismatch');
    require(int(s.detector_segment_count) > 0n && num(s.weight) > 0, 'theta nonempty statistics');
  } else {
    require(num(s.weight) === 0, 'theta genesis weight');
    for (const k of [
      'reaction_sum',
      'square_sum',
      'duration_log_sum',
      'duration_mass',
      'detector_mean',
      'detector_positive',
      'detector_negative',
    ])
      require(arr(s[k]).every((x) => num(x) === 0), 'theta genesis statistics');
  }
  const now = has ? int(obj(s.last_running_time).running_ns) : 0n;
  for (let i = 0; i < 4; i++) {
    const sign = int(arr(s.run_sign)[i]),
      start = int(arr(s.run_start_ns)[i]);
    require(sign >= -1n &&
      sign <= 1n &&
      start <= now &&
      (sign !== 0n || start === 0n) &&
      (has || sign === 0n), 'theta open run');
    require(num(arr(s.duration_mass)[i]) !== 0 ||
      num(arr(s.duration_log_sum)[i]) === 0, 'theta empty duration sum');
  }
  for (let i = 0; i < 12; i++) {
    const sign = int(arr(s.drift_sign)[i]),
      count = int(arr(s.drift_count)[i]);
    require(sign >= -1n &&
      sign <= 1n &&
      (sign === 0n) === (count === 0n), 'theta drift sign/count');
  }
}
export function checkThetaMessage(name: string, value: unknown, limits: Limits): void {
  if (!names.has(name)) return;
  const v = normalize(name, value, limits);
  if (name === 'ThetaEstimatorParameters') parameters(v);
  else if (name === 'ThetaRunningTime') running(v);
  else if (name === 'ThetaEstimatorState') state(v);
  else if (name === 'ThetaSnapshotV2') {
    const p = obj(v.parameters),
      s = obj(v.estimator),
      law = obj(v.law_version);
    parameters(p);
    state(s);
    require(law.object_type === 'echo.law-version' &&
      law.resource_version === p.law_id, 'theta law binding');
    require(v.parameters_digest instanceof Uint8Array &&
      Buffer.from(v.parameters_digest).toString('hex') ===
        digest('ThetaEstimatorParameters', p, limits), 'theta parameter digest');
    require(num(s.h) >= num(p.change_min), 'theta h floor');
    if (int(s.event_count) === 0n)
      require(num(s.h) === Math.max(num(p.change_prior), num(p.change_min)), 'theta initial h');
    for (const [field, lo, hi] of [
      ['gain', 'gain_min', 'gain_max'],
      ['recovery_per_s', 'recovery_min', 'recovery_max'],
    ] as const)
      require(scale(obj(v.values)[field]).every(
        (x) => x >= num(p[lo]) && x <= num(p[hi]),
      ), 'theta value bounds');
  }
}
