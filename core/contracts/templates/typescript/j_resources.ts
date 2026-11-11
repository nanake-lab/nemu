// ADR 0036: field consistency is not committed-state or calibration authority.
import { normalize, digest, encodeCanonicalData, type Limits } from './runtime.js';
import { schema } from './schema.js';
type Obj = Record<string, unknown>;
const obj = (v: unknown): Obj => v as Obj;
const list = (v: Obj, k: string): Obj[] => v[k] as Obj[];
const num = (v: Obj, k: string): number => v[k] as number;
const names = new Set([
  'JHookBinding',
  'JDirectionVector',
  'JDirection',
  'JDirectionSet',
  'JValidatedDirection',
  'JDirectionValidation',
  'BodyReadoutParameters',
  'JDirectionUse',
  'JEffectReceipt',
  'GenesisManifestV3',
]);
function require(ok: boolean, message: string): void {
  if (!ok) throw new Error(message);
}
function equalRef(a: unknown, b: unknown, l: Limits): boolean {
  return encodeCanonicalData('ObjectRef', a, l).equals(encodeCanonicalData('ObjectRef', b, l));
}
export function checkJEnvelope(envelope: Obj, receipt: Obj, l: Limits): void {
  const e = normalize('ModulationEnvelopeV2', envelope, l),
    r = normalize('JEffectReceipt', receipt, l);
  checkJMessage('JEffectReceipt', r, l);
  require(Buffer.from(e['effect_receipt_digest'] as Uint8Array).toString('hex') ===
    digest('JEffectReceipt', r, l), 'receipt digest');
  for (const k of ['snapshot_id', 'projected_at_ns', 'total_bound'])
    require(e[k] === r[k], 'envelope binding');
  for (const k of ['x_digest', 'r_digest', 'theta_digest'])
    require(Buffer.from(e[k] as Uint8Array).equals(
      Buffer.from(r[k] as Uint8Array),
    ), 'envelope digest');
  for (const k of ['carrier', 'model', 'tokenizer', 'direction_set'])
    require(equalRef(e[k], r[k], l), 'envelope resource');
  const ds = list(r, 'directions');
  ['connection', 'empathy', 'curiosity', 'achievement'].forEach((k, i) =>
    require(obj(e['nature_coefficients'])[k] === ds[i]!['coefficient'], 'nature coefficient'),
  );
  const learned = ds.slice(4).filter((d) => d['status'] === 1),
    actual = list(e, 'learned_coefficients');
  require(learned.length === actual.length, 'learned coefficients');
  learned.forEach((d, i) =>
    require(d['tendency_id'] === actual[i]!['tendency_id'] &&
      d['coefficient'] === actual[i]!['coefficient'], 'learned coefficient'),
  );
}
export function addJBound(a: number, b: number): number {
  const hi = Math.max(a, b),
    lo = Math.min(a, b);
  let sum = hi + lo;
  require(Number.isFinite(sum), 'beta overflow');
  if (lo - (sum - hi) > 0) {
    const bits = new DataView(new ArrayBuffer(8));
    bits.setFloat64(0, sum);
    bits.setBigUint64(0, bits.getBigUint64(0) + 1n);
    sum = bits.getFloat64(0);
  }
  require(Number.isFinite(sum), 'beta overflow');
  return sum;
}
function ordered(items: Obj[]): void {
  require(items.length >= 4, 'four nature directions required');
  for (let i = 0; i < 4; i++) require(items[i]!['kind'] === i + 1, 'nature order');
  let previous: Buffer | undefined;
  for (const d of items.slice(4)) {
    require(d['kind'] === 5, 'learned kind');
    const text = Buffer.from(d['tendency_id'] as string, 'utf8'),
      length = Buffer.alloc(4);
    length.writeUInt32LE(text.length);
    const key = Buffer.concat([length, text]);
    require(previous === undefined || Buffer.compare(previous, key) < 0, 'learned order/duplicate');
    previous = key;
  }
}
export function checkJMessage(name: string, value: Obj, l: Limits): void {
  if (!names.has(name)) return;
  const v = normalize(name, value, l);
  const tag = (key: string, expected: string): void => {
    if (key in v) require(obj(v[key])['object_type'] === expected, key + ' type');
  };
  if (name === 'JDirectionVector') require((v['values'] as number[]).length > 0, 'empty vector');
  else if (name === 'JDirection' || name === 'JDirectionUse') {
    const learned = v['kind'] === 5;
    require('tendency_id' in v === learned, 'tendency identity');
    if (name === 'JDirection') {
      require('concept' in v === learned, 'concept identity');
      tag('vector', 'echo.j-direction-vector');
    } else {
      const included = v['status'] === 1;
      require(learned || included, 'nature cannot be excluded');
      require(['direction', 'validation', 'beta', 'coefficient'].every(
        (k) => k in v === included,
      ), 'use presence');
      tag('direction', 'echo.j-direction-vector');
      tag('validation', 'echo.j-direction-validation');
      if (included)
        require(Math.abs(num(v, 'coefficient')) <= num(v, 'beta'), 'coefficient budget');
    }
  } else if (name === 'JDirectionSet' || name === 'JDirectionValidation') {
    const ds = list(v, 'directions');
    if (name === 'JDirectionSet') ordered(ds);
    else {
      tag('direction_set', 'echo.j-direction-set');
      require(ds.length >= 4, 'validation list');
    }
    for (const d of ds) {
      checkJMessage(name === 'JDirectionSet' ? 'JDirection' : 'JValidatedDirection', d, l);
    }
  } else if (name === 'JValidatedDirection') {
    tag('vector', 'echo.j-direction-vector');
    require(Math.abs(num(v, 'measured_norm') - 1) <= num(v, 'norm_tolerance'), 'unit norm');
  } else if (name === 'JEffectReceipt') {
    require((v['projected_at_ns'] as bigint) >=
      (v['anchor_time_ns'] as bigint), 'time before anchor');
    for (const [key, type] of [
      ['direction_set', 'echo.j-direction-set'],
      ['validation', 'echo.j-direction-validation'],
      ['body_parameters', 'echo.body-readout-parameters'],
      ['genesis', 'echo.genesis-v3'],
    ] as const)
      tag(key, type);
    ordered(list(v, 'directions'));
    let total = 0;
    for (const d of list(v, 'directions')) {
      checkJMessage('JDirectionUse', d, l);
      if (d['status'] === 1) {
        total = addJBound(total, num(d, 'beta'));
        if (v['mode'] === 2) require(d['coefficient'] === 0, 'rehearsal coefficient');
      }
    }
    require(total === v['total_bound'], 'total bound');
    if (v['mode'] === 2) {
      const b = obj(v['body']);
      require(b['facing'] === 0.5 && b['tension'] === 0 && b['energy'] === 0.5, 'rehearsal body');
    }
  } else if (name === 'GenesisManifestV3') {
    tag('direction_set', 'echo.j-direction-set');
    tag('direction_validation', 'echo.j-direction-validation');
    tag('body_readout', 'echo.body-readout-parameters');
  }
}
export function checkJResources(
  directionSet: Obj,
  validation: Obj,
  body: Obj,
  receipt: Obj,
  genesis: Obj,
  vectors: Obj[],
  formal: boolean,
  l: Limits,
): void {
  const pairs: [string, Obj][] = [
    ['JDirectionSet', directionSet],
    ['JDirectionValidation', validation],
    ['BodyReadoutParameters', body],
    ['JEffectReceipt', receipt],
    ['GenesisManifestV3', genesis],
  ];
  for (const [name, value] of pairs) checkJMessage(name, value, l);
  const s = normalize('JDirectionSet', directionSet, l),
    v = normalize('JDirectionValidation', validation, l),
    b = normalize('BodyReadoutParameters', body, l),
    r = normalize('JEffectReceipt', receipt, l),
    g = normalize('GenesisManifestV3', genesis, l);
  const bound = (reference: unknown, name: string, value: Obj): void => {
    const ref = obj(reference);
    require(ref['object_type'] === schema.messages[name]!.tag, 'resource type');
    require(Buffer.from(ref['digest'] as Uint8Array).toString('hex') ===
      digest(name, value, l), 'resource digest');
    require(ref['byte_length'] ===
      BigInt(encodeCanonicalData(name, value, l).length), 'resource length');
    if ('version' in value)
      require(ref['resource_version'] === value['version'], 'resource version');
    else require(ref['resource_version'] === '1', 'resource schema version');
  };
  for (const ref of [v['direction_set'], r['direction_set']]) bound(ref, 'JDirectionSet', s);
  for (const ref of [r['validation']]) bound(ref, 'JDirectionValidation', v);
  for (const ref of [r['body_parameters'], g['body_readout']])
    bound(ref, 'BodyReadoutParameters', b);
  bound(r['genesis'], 'GenesisManifestV3', g);
  for (const key of ['carrier', 'model', 'tokenizer'])
    require(equalRef(r[key], obj(s['hook'])[key], l), 'carrier binding');
  require(obj(r['body'])['version'] === b['version'], 'body version');
  if (formal) {
    require(v['fixture'] === false && r['mode'] === 1, 'fixture/rehearsal not formal');
    require(obj(s['hook'])['prefill_mask'] !== 3 &&
      obj(s['hook'])['decode_mask'] === 1, 'formal prediction positions');
  }
  const ds = list(s, 'directions'),
    checks = list(v, 'directions');
  require(ds.length === checks.length && ds.length === vectors.length, 'resource lists');
  for (let i = 0; i < ds.length; i++) {
    const d = ds[i]!,
      checked = checks[i]!;
    checkJMessage('JDirectionVector', vectors[i]!, l);
    const vector = normalize('JDirectionVector', vectors[i]!, l);
    require(equalRef(d['vector'], checked['vector'], l), 'validation direction order');
    bound(d['vector'], 'JDirectionVector', vector);
    const values = vector['values'] as number[];
    require(values.length === obj(s['hook'])['width'], 'vector width');
    let total = 0;
    for (const x of values) {
      total += x * x;
      require(Number.isFinite(total), 'norm overflow');
    }
    require(Math.sqrt(total) === checked['measured_norm'], 'measured norm');
    require(num(d, 'beta') <= num(checked, 'max_beta'), 'single beta budget');
  }
  require(num(r, 'total_bound') <= num(v, 'combination_max_beta'), 'combination budget');
  for (const use of list(r, 'directions')) {
    if (use['status'] !== 1) continue;
    const found = ds.filter(
      (d) => d['kind'] === use['kind'] && d['tendency_id'] === use['tendency_id'],
    );
    require(found.length === 1, 'included identity');
    const d = found[0]!;
    require(equalRef(use['direction'], d['vector'], l) &&
      use['beta'] === d['beta'], 'included binding');
    require(equalRef(use['validation'], r['validation'], l), 'included validation');
  }
}
