// Explicit test/debug JSON bridge; integers never pass through Number.
import { schema } from './schema.js';
import { requireValue } from './runtime.js';

export function fromJson(name: string, value: Record<string, unknown>): Record<string, unknown> {
  const m = schema.messages[name];
  requireValue(m, 'message');
  const atom = (t: string, v: unknown): unknown => {
    if (schema.messages[t] && v !== null && typeof v === 'object' && !Array.isArray(v))
      return fromJson(t, v as Record<string, unknown>);
    if ((t === 'uint64' || t === 'sint64') && typeof v === 'string') {
      requireValue(/^-?(0|[1-9][0-9]*)$/.test(v), 'decimal');
      const n = BigInt(v);
      requireValue(String(n) === v, 'decimal spelling');
      return n;
    }
    if (t === 'bytes' && typeof v === 'string') {
      requireValue(/^(?:[a-fA-F0-9]{2})*$/.test(v), 'hex');
      return Buffer.from(v, 'hex');
    }
    return v;
  };
  return Object.fromEntries(
    Object.entries(value).map(([k, v]) => {
      const f = m.fields.find((f) => f.name === k);
      return [
        k,
        !f || v === null
          ? v
          : f.repeated && Array.isArray(v)
            ? v.map((x) => atom(f.type, x))
            : atom(f.type, v),
      ];
    }),
  );
}
export function toJson(name: string, value: Record<string, unknown>): Record<string, unknown> {
  const m = schema.messages[name];
  requireValue(m, 'message');
  const atom = (t: string, v: unknown): unknown => {
    if (schema.messages[t]) return toJson(t, v as Record<string, unknown>);
    if (t === 'uint64' || t === 'sint64') return String(v);
    if (t === 'bytes') return Buffer.from(v as Uint8Array).toString('hex');
    return v;
  };
  return Object.fromEntries(
    Object.entries(value).map(([k, v]) => {
      const f = m.fields.find((f) => f.name === k);
      requireValue(f, 'field');
      return [k, f.repeated ? (v as unknown[]).map((x) => atom(f.type, x)) : atom(f.type, v)];
    }),
  );
}
