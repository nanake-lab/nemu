// Browser-safe structural validation and JSON resource projection from authority descriptors.
// No encoding, hashing, storage or Node imports. Wire size budgets are enforced by the codecs.
import { z } from 'zod';
import { schema, type MessageMap } from './schema.js';
import type { Field } from './runtime.js';
import type { ResourceMessageMap } from './resources.js';

const cache = new Map<string, z.ZodType>();
function object(v: unknown): Record<string, unknown> {
  return v as Record<string, unknown>;
}
function key(v: unknown): string {
  return JSON.stringify(v, (_, x: unknown) =>
    typeof x === 'bigint' ? x.toString() : x instanceof Uint8Array ? [...x] : x,
  );
}
// z.record filters __proto__; validate every JSON own entry before rebuilding safely.
function dictionary(keySchema: z.ZodType, valueSchema: z.ZodType): z.ZodType {
  return z
    .custom<Record<string, unknown>>(
      (v) =>
        v !== null &&
        typeof v === 'object' &&
        (Object.getPrototypeOf(v) === Object.prototype || Object.getPrototypeOf(v) === null),
    )
    .transform((input, ctx) => {
      const entries: Array<[string, unknown]> = [];
      for (const [name, item] of Object.entries(input)) {
        const k = keySchema.safeParse(name);
        const v = valueSchema.safeParse(item);
        for (const result of [k, v])
          if (!result.success)
            for (const issue of result.error.issues)
              ctx.addIssue({ ...issue, path: [name, ...issue.path] });
        if (k.success && v.success) entries.push([name, v.data]);
      }
      // fromEntries defines data properties, including __proto__, without invoking setters.
      return Object.fromEntries(entries);
    });
}
function atom(f: Field, resource: boolean): z.ZodType {
  if (schema.messages[f.type]) return build(f.type, resource);
  if (schema.enums[f.type]) {
    const values = resource
      ? Object.values(schema.json_enums?.[f.type] ?? {})
      : Object.values(schema.enums[f.type]!).filter((v) => v !== 0);
    return z.custom((v) => values.some((x) => x === v));
  }
  switch (f.type) {
    case 'uint32':
      return z
        .number()
        .int()
        .min(f.min ?? 0)
        .max(f.max ?? 4294967295);
    case 'uint64':
      return z.bigint().min(0n).max(18446744073709551615n);
    case 'sint64':
      return z.bigint().min(-9223372036854775808n).max(9223372036854775807n);
    case 'double':
      return z
        .number()
        .finite()
        .min(f.min ?? -Number.MAX_VALUE)
        .max(f.max ?? Number.MAX_VALUE);
    case 'bool':
      return z.boolean();
    case 'string':
      return z
        .string()
        .min(f.nonempty ? 1 : 0)
        .refine((v) => {
          for (const c of v) {
            const n = c.charCodeAt(0);
            if (c.length === 1 && n >= 0xd800 && n <= 0xdfff) return false;
          }
          return true;
        }, 'UTF-8');
    case 'bytes':
      return z.instanceof(Uint8Array).refine((v) => f.size === undefined || v.length === f.size);
    default:
      throw new Error('Unknown IDL type');
  }
}
function build(name: string, resource = false): z.ZodType {
  const id = `${resource}:${name}`;
  const cached = cache.get(id);
  if (cached) return cached;
  const m = schema.messages[name];
  if (!m || (resource && !m.resource)) throw new Error('Unknown IDL resource/message');
  const lazy = z.lazy(() => {
    const fields: Record<string, z.ZodType> = {};
    for (const f of m.fields) {
      if (resource && f.name === 'schema_version') continue;
      let value: z.ZodType;
      if (resource && f.json_map) {
        const [k, v] = f.json_map.split(':');
        const child = schema.messages[f.type]!;
        const keyField = child.fields.find((x) => x.name === k)!;
        const valueField = child.fields.find((x) => x.name === v)!;
        value = dictionary(atom(keyField, true), atom(valueField, true));
      } else {
        value = f.name === 'schema_version' ? z.literal(1) : atom(f, resource);
        if (f.repeated) {
          value = z
            .array(value)
            .refine(
              (items) =>
                !f.set ||
                new Set(items.map((v) => key(f.set === 'self' ? v : object(v)[f.set!]))).size ===
                  items.length,
              'Duplicate set key',
            );
        } else if (!f.required) value = value.optional();
      }
      fields[resource ? (f.json_name ?? f.name) : f.name] = value;
    }
    return z.strictObject(fields).superRefine((data, ctx) => {
      if (resource) return; // Resource constraints above come from the same field descriptors.
      const fail = (message: string) => ctx.addIssue({ code: 'custom', message });
      for (const key of m.rules.positive ?? [])
        if (!((data[key] as number) > 0)) fail('Positive field required');
      for (const [key, signed] of Object.entries(m.rules.at_least_abs ?? {}))
        if (!((data[key] as number) >= Math.abs(data[signed] as number)))
          fail('Absolute evidence below signed magnitude');
      for (const group of Object.values(m.rules.oneofs ?? {})) {
        if (group.fields.filter((k) => data[k] !== undefined).length !== 1) fail('Required oneof');
      }
      for (const [field, count] of Object.entries(m.rules.lengths ?? {})) {
        if (!Array.isArray(data[field]) || data[field].length !== Number(data[count]))
          fail('Dimension mismatch');
      }
      if (m.rules.cases) {
        const d = m.rules.discriminator ?? 'kind';
        const c = m.rules.cases[String(data[d])];
        if (!c) {
          fail('Variant');
          return;
        }
        const allowed = new Set([
          'schema_version',
          d,
          ...(m.rules.common ?? []),
          ...(c.required ?? []),
          ...(c.optional ?? []),
        ]);
        if (
          (c.required ?? []).some((k) => data[k] === undefined) ||
          Object.keys(data).some((k) => !allowed.has(k))
        )
          fail('Variant fields');
        for (const [k, values] of Object.entries(c.values ?? {}))
          if (!values.includes(data[k] as number)) fail('Variant enum');
      }
    });
  });
  cache.set(id, lazy);
  return lazy;
}
export function wireInputSchema<K extends keyof MessageMap>(name: K): z.ZodType<MessageMap[K]> {
  return build(name) as z.ZodType<MessageMap[K]>;
}
export function resourceInputSchema<K extends keyof ResourceMessageMap>(
  name: K,
): z.ZodType<ResourceMessageMap[K]> {
  return build(name, true) as z.ZodType<ResourceMessageMap[K]>;
}
export function resourceEnumSchema(name: string): z.ZodType<string> {
  return atom({ type: name } as Field, true) as z.ZodType<string>;
}
/** Projection is checked again against the authority schema, preserving every field. */
export function resourceToWire<K extends keyof ResourceMessageMap & keyof MessageMap>(
  name: K,
  input: ResourceMessageMap[K],
): MessageMap[K] {
  const value = resourceInputSchema(name).parse(input);
  function project(type: string, v: unknown): unknown {
    if (schema.enums[type]) {
      const entry = Object.entries(schema.json_enums?.[type] ?? {}).find(
        ([, alias]) => alias === v,
      );
      if (!entry) throw new Error('Unknown resource enum');
      return schema.enums[type]![entry[0]];
    }
    if (!schema.messages[type]) return v;
    const out: Record<string, unknown> = { schema_version: 1 };
    for (const f of schema.messages[type]!.fields) {
      if (f.name === 'schema_version') continue;
      const item = object(v)[f.json_name ?? f.name];
      if (f.json_map) {
        const [k, val] = f.json_map.split(':');
        out[f.name] = Object.entries(object(item)).map(([a, b]) => ({
          schema_version: 1,
          [k!]: a,
          [val!]: b,
        }));
      } else
        out[f.name] = f.repeated
          ? (item as unknown[]).map((x) => project(f.type, x))
          : project(f.type, item);
    }
    return out;
  }
  return wireInputSchema(name).parse(project(name, value));
}
