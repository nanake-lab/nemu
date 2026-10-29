// Generated runtime template. No clocks, stores or authority writes.
import { createHash } from 'node:crypto';
import { z } from 'zod';
import { schema, type MessageMap } from './schema.js';

export interface Field {
  name: string;
  type: string;
  number: number;
  repeated: boolean;
  required: boolean;
  nonempty: boolean;
  min?: number;
  max?: number;
  size?: number;
  set?: string;
  oneof?: string;
  json_name?: string;
  json_map?: string;
}
export interface Schema {
  json_enums?: Record<string, Record<string, string>>;
  messages: Record<
    string,
    {
      tag: string;
      source: string;
      resource?: boolean;
      transient?: boolean;
      file: string;
      fields: Field[];
      rules: {
        positive?: string[];
        at_least_abs?: Record<string, string>;
        oneofs?: Record<string, { fields: string[]; required: boolean }>;
        discriminator?: string;
        common?: string[];
        lengths?: Record<string, string>;
        cases?: Record<
          string,
          { required?: string[]; optional?: string[]; values?: Record<string, number[]> }
        >;
      };
      /** Lifecycle metadata only (P5-21); never changes encoding. */
      superseded?: Supersession;
      supersedes?: string;
      retains?: string;
      reserved?: number[];
    }
  >;
  enums: Record<string, Record<string, number>>;
  superseded_enums?: Record<string, Supersession>;
  superseded_values?: Record<string, Record<string, Supersession>>;
}
export interface Supersession {
  by: string;
  reason: string;
}
export interface Limits {
  max_bytes: number;
  max_items: number;
  max_depth: number;
}
export function requireValue(ok: unknown, why: string): asserts ok {
  if (!ok) throw new Error(why);
}
function message(name: string) {
  const m = schema.messages[name];
  requireValue(m, 'unknown message');
  return m;
}
function object(v: unknown): Record<string, unknown> {
  requireValue(
    v !== null && typeof v === 'object' && !Array.isArray(v) && !(v instanceof Uint8Array),
    'object',
  );
  return v as Record<string, unknown>;
}
function utf8(s: string): Buffer {
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c >= 0xd800 && c <= 0xdbff) {
      const next = s.charCodeAt(++i);
      requireValue(next >= 0xdc00 && next <= 0xdfff, 'lone surrogate');
    } else requireValue(c < 0xdc00 || c > 0xdfff, 'lone surrogate');
  }
  return Buffer.from(s, 'utf8');
}
const text = (b: Uint8Array): string =>
  new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(b);
const u32 = (n: number): Buffer => {
  requireValue(Number.isInteger(n) && n >= 0 && n <= 0xffffffff, 'u32 overflow');
  const b = Buffer.alloc(4);
  b.writeUInt32LE(n);
  return b;
};
const blob = (b: Uint8Array): Buffer => Buffer.concat([u32(b.length), b]);

function scalar(f: Field, v: unknown, l: Limits, depth: number): unknown {
  const t = f.type;
  if (schema.messages[t]) return normalize(t, v, l, depth + 1);
  if (schema.enums[t])
    requireValue(
      typeof v === 'number' && v !== 0 && Object.values(schema.enums[t]).includes(v),
      'unknown enum',
    );
  else if (t === 'bool') requireValue(typeof v === 'boolean', 'boolean');
  else if (t === 'uint32')
    requireValue(
      typeof v === 'number' && Number.isInteger(v) && v >= 0 && v <= 0xffffffff,
      'u32 overflow',
    );
  else if (t === 'uint64' || t === 'sint64') {
    requireValue(typeof v === 'bigint', '64-bit integers require bigint');
    requireValue(
      t === 'uint64' ? v >= 0n && v <= 0xffffffffffffffffn : v >= -(1n << 63n) && v < 1n << 63n,
      'integer overflow',
    );
  } else if (t === 'double') {
    requireValue(typeof v === 'number' && Number.isFinite(v), 'nonfinite/invalid double');
    v = v === 0 ? 0 : v;
  } else if (t === 'string') {
    requireValue(typeof v === 'string', 'string');
    requireValue(utf8(v).length <= l.max_bytes, 'string limit');
    if (f.nonempty) requireValue(v.length > 0, 'empty');
  } else if (t === 'bytes') {
    requireValue(v instanceof Uint8Array && v.length <= l.max_bytes, 'bytes');
    if (f.size !== undefined) requireValue(v.length === f.size, 'byte size');
  } else throw new Error('unknown type');
  if (f.min !== undefined) requireValue(typeof v === 'number' && v >= f.min, 'minimum');
  if (f.max !== undefined) requireValue(typeof v === 'number' && v <= f.max, 'maximum');
  return v;
}

export function normalize(
  name: string,
  value: unknown,
  l: Limits,
  depth = 0,
): Record<string, unknown> {
  requireValue(depth <= l.max_depth, 'depth limit');
  const m = message(name);
  requireValue(m, 'message type');
  const input = object(value);
  requireValue(
    Object.keys(input).every((k) => m.fields.some((f) => f.name === k)),
    'unknown field',
  );
  const out: Record<string, unknown> = {};
  for (const f of m.fields) {
    let v = input[f.name];
    if (v === null || v === undefined) {
      requireValue(!f.required, 'missing required');
      continue;
    }
    if (f.repeated) {
      requireValue(
        Array.isArray(v) && v.length <= Math.min(l.max_items, 0xffffffff),
        'collection limit',
      );
      let a = v.map((item) => scalar(f, item, l, depth));
      if (f.set) {
        const pairs = a.map((item) => {
          let key: Buffer;
          if (f.set === 'self') key = canonicalScalar(f.type, item, l);
          else {
            const field = message(f.type).fields.find((k) => k.name === f.set);
            requireValue(field, 'set key');
            key = canonicalScalar(field.type, object(item)[field.name], l);
          }
          return { key, item };
        });
        requireValue(
          new Set(pairs.map((p) => p.key.toString('hex'))).size === pairs.length,
          'duplicate set key',
        );
        pairs.sort((a, b) => Buffer.compare(a.key, b.key));
        a = pairs.map((p) => p.item);
      }
      v = a;
    } else v = scalar(f, v, l, depth);
    out[f.name] = v;
  }
  requireValue(out.schema_version === 1, 'unknown schema');
  for (const [key, count] of Object.entries(m.rules.lengths ?? {}))
    requireValue((out[key] as unknown[]).length === out[count], 'dimension mismatch');
  if (m.rules.cases) {
    const discriminator = m.rules.discriminator ?? 'kind';
    const c = m.rules.cases[String(out[discriminator])];
    requireValue(c, 'variant');
    const required = c.required ?? [];
    const allowed = [
      ...required,
      ...(c.optional ?? []),
      ...(m.rules.common ?? []),
      'schema_version',
      discriminator,
    ];
    requireValue(
      required.every((k) => k in out) && Object.keys(out).every((k) => allowed.includes(k)),
      'variant fields',
    );
    for (const [k, values] of Object.entries(c.values ?? {}))
      requireValue(values.includes(out[k] as number), 'variant enum');
  }
  for (const group of Object.values(m.rules.oneofs ?? {})) {
    requireValue(group.fields.filter(k => out[k] !== undefined).length === 1, 'required oneof');
  }
  for (const key of m.rules.positive ?? [])
    requireValue((out[key] as number) > 0, 'positive field required');
  for (const [key, signed] of Object.entries(m.rules.at_least_abs ?? {}))
    requireValue(
      (out[key] as number) >= Math.abs(out[signed] as number),
      'absolute evidence below signed magnitude',
    );
  return out;
}

export function inputSchema(name: string, limits: Limits) {
  return z.unknown().transform((value, ctx) => {
    try {
      return normalize(name, value, limits);
    } catch (error) {
      ctx.addIssue({ code: 'custom', message: String(error) });
      return z.NEVER;
    }
  });
}

function canonicalScalar(t: string, v: unknown, l: Limits): Buffer {
  if (schema.messages[t]) return blob(encodeCanonicalData(t, v, l));
  if (schema.enums[t] || t === 'uint32') return u32(v as number);
  const b = Buffer.alloc(8);
  if (t === 'uint64') {
    b.writeBigUInt64LE(v as bigint);
    return b;
  }
  if (t === 'sint64') {
    b.writeBigInt64LE(v as bigint);
    return b;
  }
  if (t === 'double') {
    b.writeDoubleLE(v as number);
    return b;
  }
  if (t === 'bool') return Buffer.from([v ? 1 : 0]);
  return blob(t === 'string' ? utf8(v as string) : (v as Uint8Array));
}

export function encodeCanonicalData(name: string, value: unknown, l: Limits): Buffer {
  const v = inputSchema(name, l).parse(value);
  const m = message(name);
  const parts: Buffer[] = [Buffer.from('NCV1'), u32(1), blob(utf8(m.tag))];
  let size = parts.reduce((sum, b) => sum + b.length, 0);
  const push = (b: Buffer) => {
    size += b.length;
    requireValue(size <= l.max_bytes, 'message size');
    parts.push(b);
  };
  for (const f of m.fields.slice(1)) {
    const item = v[f.name];
    if (f.oneof) {
      const group = m.rules.oneofs![f.oneof]!.fields;
      if (f.name !== group[0]) continue;
      const chosen = m.fields.find(x => group.includes(x.name) && v[x.name] !== undefined)!;
      push(u32(chosen.number));
      push(canonicalScalar(chosen.type, v[chosen.name], l));
    } else if (f.repeated) {
      const a = item as unknown[];
      push(u32(a.length));
      for (const x of a) push(canonicalScalar(f.type, x, l));
    } else {
      if (!f.required) push(Buffer.from([item === undefined ? 0 : 1]));
      if (item !== undefined) push(canonicalScalar(f.type, item, l));
    }
  }
  requireValue(size <= l.max_bytes, 'message size');
  return Buffer.concat(parts);
}

class Reader {
  pos = 0;
  data: Buffer;
  constructor(data: Uint8Array, l: Limits) {
    requireValue(data.length <= l.max_bytes, 'message size');
    this.data = Buffer.from(data);
  }
  take(n: number): Buffer {
    requireValue(
      Number.isSafeInteger(n) && n >= 0 && n <= this.data.length - this.pos,
      'truncated',
    );
    const b = this.data.subarray(this.pos, this.pos + n);
    this.pos += n;
    return b;
  }
  uint(): number {
    return this.take(4).readUInt32LE();
  }
  blob(): Buffer {
    return this.take(this.uint());
  }
  done(): boolean {
    return this.pos === this.data.length;
  }
}

export function decodeCanonicalData(
  name: string,
  data: Uint8Array,
  l: Limits,
  depth = 0,
): Record<string, unknown> {
  requireValue(depth <= l.max_depth && schema.messages[name], 'depth/type');
  const r = new Reader(data, l);
  const m = message(name);
  requireValue(r.take(4).equals(Buffer.from('NCV1')) && r.uint() === 1, 'magic/schema');
  requireValue(text(r.blob()) === m.tag, 'type tag');
  const atom = (t: string): unknown => {
    if (schema.messages[t]) return decodeCanonicalData(t, r.blob(), l, depth + 1);
    if (schema.enums[t] || t === 'uint32') return r.uint();
    if (t === 'uint64') return r.take(8).readBigUInt64LE();
    if (t === 'sint64') return r.take(8).readBigInt64LE();
    if (t === 'double') return r.take(8).readDoubleLE();
    if (t === 'bool') {
      const b = r.take(1).readUInt8(0);
      requireValue(b < 2, 'boolean');
      return b === 1;
    }
    const b = r.blob();
    return t === 'string' ? text(b) : b;
  };
  const value: Record<string, unknown> = { schema_version: 1 };
  for (const f of m.fields.slice(1)) {
    if (f.oneof) {
      const group = m.rules.oneofs![f.oneof]!.fields;
      if (f.name !== group[0]) continue;
      const number = r.uint();
      const chosen = m.fields.find(x => x.number === number && group.includes(x.name));
      requireValue(chosen, 'unknown oneof field');
      value[chosen.name] = atom(chosen.type);
    } else if (f.repeated) {
      const count = r.uint();
      requireValue(count <= l.max_items, 'count limit');
      value[f.name] = Array.from({ length: count }, () => atom(f.type));
    } else if (f.required) value[f.name] = atom(f.type);
    else {
      const p = r.take(1).readUInt8(0);
      requireValue(p < 2, 'presence');
      if (p) value[f.name] = atom(f.type);
    }
  }
  requireValue(r.done(), 'trailing bytes');
  const v = normalize(name, value, l, depth);
  requireValue(encodeCanonicalData(name, v, l).equals(Buffer.from(data)), 'noncanonical encoding');
  return v;
}

export function digest(name: string, value: unknown, l: Limits): string {
  return createHash('sha256')
    .update(
      Buffer.concat([
        Buffer.from('NEMU-CANONICALV1\0'),
        blob(utf8(message(name).tag)),
        encodeCanonicalData(name, value, l),
      ]),
    )
    .digest('hex');
}

function varint(n: bigint): Buffer {
  requireValue(n >= 0n && n <= 0xffffffffffffffffn, 'varint overflow');
  const a: number[] = [];
  while (n >= 128n) {
    a.push(Number(n & 127n) | 128);
    n >>= 7n;
  }
  a.push(Number(n));
  return Buffer.from(a);
}
function readVarint(r: Reader): bigint {
  let n = 0n;
  for (let i = 0; i < 10; i++) {
    const b = r.take(1).readUInt8(0);
    requireValue(i < 9 || b <= 1, 'varint overflow');
    n |= BigInt(b & 127) << BigInt(7 * i);
    if (b < 128) return n;
  }
  throw new Error('varint overflow');
}
function length(r: Reader): Buffer {
  const n = readVarint(r);
  requireValue(n <= BigInt(r.data.length - r.pos), 'length overflow');
  return r.take(Number(n));
}
const wireType = (t: string): number =>
  schema.messages[t] || t === 'string' || t === 'bytes' ? 2 : t === 'double' ? 1 : 0;

export function encodeProtobufData(name: string, value: unknown, l: Limits): Buffer {
  const v = inputSchema(name, l).parse(value);
  const parts: Buffer[] = [];
  let size = 0;
  for (const f of message(name).fields) {
    const value = v[f.name];
    if (value === undefined) continue;
    for (const item of f.repeated ? (value as unknown[]) : [value]) {
      const wire = wireType(f.type);
      let b: Buffer;
      if (wire === 2) {
        const raw = schema.messages[f.type]
          ? encodeProtobufData(f.type, item, l)
          : f.type === 'string'
            ? utf8(item as string)
            : Buffer.from(item as Uint8Array);
        b = Buffer.concat([varint(BigInt(raw.length)), raw]);
      } else if (wire === 1) {
        b = Buffer.alloc(8);
        b.writeDoubleLE(item as number);
      } else {
        let n = BigInt(item as bigint | number | boolean);
        if (f.type === 'sint64') n = (n << 1n) ^ (n >> 63n);
        b = varint(n);
      }
      const key = varint((BigInt(f.number) << 3n) | BigInt(wire));
      size += key.length + b.length;
      requireValue(size <= l.max_bytes, 'message size');
      parts.push(key, b);
    }
  }
  return Buffer.concat(parts);
}

export function decodeProtobufData(
  name: string,
  data: Uint8Array,
  l: Limits,
  depth = 0,
): Record<string, unknown> {
  requireValue(depth <= l.max_depth && schema.messages[name], 'depth/type');
  const r = new Reader(data, l);
  const fields = message(name).fields;
  const value: Record<string, unknown> = {};
  for (const f of fields) if (f.repeated) value[f.name] = [];
  const atom = (t: string, wire: number, reader: Reader): unknown => {
    requireValue(wire === wireType(t), 'wire type');
    if (wire === 2) {
      const b = length(reader);
      return schema.messages[t]
        ? decodeProtobufData(t, b, l, depth + 1)
        : t === 'string'
          ? text(b)
          : b;
    }
    if (wire === 1) return reader.take(8).readDoubleLE();
    const n = readVarint(reader);
    if (t === 'uint64') return n;
    if (t === 'sint64') return (n >> 1n) ^ -(n & 1n);
    if (t === 'bool') {
      requireValue(n < 2n, 'boolean');
      return n === 1n;
    }
    requireValue(n <= 0xffffffffn, 'u32 overflow');
    return Number(n);
  };
  while (!r.done()) {
    const key = readVarint(r);
    const wire = Number(key & 7n);
    const f = fields.find((f) => BigInt(f.number) === key >> 3n);
    requireValue(f, 'unknown field');
    if (!f.repeated) {
      requireValue(!(f.name in value), 'duplicate singular');
      value[f.name] = atom(f.type, wire, r);
    } else {
      const a = value[f.name] as unknown[];
      if (wire === 2 && wireType(f.type) !== 2) {
        const sub = new Reader(length(r), l);
        while (!sub.done()) {
          requireValue(a.length < l.max_items, 'count limit');
          a.push(atom(f.type, wireType(f.type), sub));
        }
      } else {
        requireValue(a.length < l.max_items, 'count limit');
        a.push(atom(f.type, wire, r));
      }
    }
  }
  return normalize(name, value, l, depth);
}

export function encodeCanonical<K extends keyof MessageMap>(
  name: K,
  value: MessageMap[K],
  limits: Limits,
): Uint8Array {
  return encodeCanonicalData(name, value, limits);
}
export function encodeProtobuf<K extends keyof MessageMap>(
  name: K,
  value: MessageMap[K],
  limits: Limits,
): Uint8Array {
  return encodeProtobufData(name, value, limits);
}
export function decodeCanonical<K extends keyof MessageMap>(
  name: K,
  bytes: Uint8Array,
  limits: Limits,
): MessageMap[K] {
  return decodeCanonicalData(name, bytes, limits) as unknown as MessageMap[K];
}
export function decodeProtobuf<K extends keyof MessageMap>(
  name: K,
  bytes: Uint8Array,
  limits: Limits,
): MessageMap[K] {
  return decodeProtobufData(name, bytes, limits) as unknown as MessageMap[K];
}
