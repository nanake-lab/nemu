import { readFileSync } from 'node:fs';
import { expect, test } from 'vitest';
import { proactivityBoundariesSchema, proactivityToWire } from './proactivity.js';
import { encodeCanonical, digest, type ProactivityBoundaries } from './generated/echo/index.js';

const config: unknown = JSON.parse(
  readFileSync(new URL('../../../config/opportunities.json', import.meta.url), 'utf8'),
);
const base = proactivityBoundariesSchema.parse(config);
const limits = { max_bytes: 1048576, max_items: 4096, max_depth: 32 };

test('A1: JSON own keys survive validation and project to the exact frozen authority payload', () => {
  const vectors = JSON.parse(
    readFileSync(new URL('../../../core/contracts/golden/legal.json', import.meta.url), 'utf8'),
  ) as Array<{
    request: { name: string; value: ProactivityBoundaries };
    expected: { canonical: string; digest: string };
  }>;
  const special = vectors.filter(
    (v) =>
      v.request.name === 'ProactivityBoundaries' &&
      v.request.value.cooldowns.some((c) => c.kind === '__proto__'),
  );
  expect(special).toHaveLength(2);
  for (const vector of special) {
    const expected = vector.request.value;
    // JSON.parse produces an own data property, unlike object-literal __proto__ syntax.
    const cooldownS: Record<string, number> = JSON.parse(
      JSON.stringify(Object.fromEntries(expected.cooldowns.map((c) => [c.kind, c.seconds]))),
    );
    const input = { ...base, cooldownS };
    const parsed = proactivityBoundariesSchema.parse(input);
    expect(Object.keys(parsed.cooldownS)).toEqual(Object.keys(cooldownS));
    for (const k of Object.keys(cooldownS)) {
      expect(Object.hasOwn(parsed.cooldownS, k)).toBe(true);
      expect(parsed.cooldownS[k]).toBe(cooldownS[k]);
    }
    expect(Object.getPrototypeOf(parsed.cooldownS)).toBe(Object.prototype);
    expect(Object.getPrototypeOf(cooldownS)).toBe(Object.prototype);
    expect(proactivityToWire(input)).toEqual(expected);
    expect(proactivityToWire(parsed)).toEqual(expected);
    const wire = proactivityToWire(parsed);
    expect(
      Buffer.from(encodeCanonical('ProactivityBoundaries', wire, limits)).toString('hex'),
    ).toBe(vector.expected.canonical);
    expect(digest('ProactivityBoundaries', wire, limits)).toBe(vector.expected.digest);
    if (expected.cooldowns.length === 1)
      expect(vector.expected.digest).toBe(
        'b46fb3719dc49a58bde3f243c19f116ab294bf5a495f9d76fe5e446c5cec431d',
      );
  }
});

test.each([-1, null, 'invalid', Infinity, NaN])(
  'A1: __proto__ invalid value %s is explicitly rejected at its path',
  (value) => {
    const cooldownS: Record<string, unknown> = JSON.parse('{"__proto__":17}');
    cooldownS['__proto__'] = value;
    const input = { ...base, cooldownS };
    const parsed = proactivityBoundariesSchema.safeParse(input);
    expect(parsed.success).toBe(false);
    if (!parsed.success)
      expect(parsed.error.issues.some((i) => i.path.join('.') === 'cooldownS.__proto__')).toBe(
        true,
      );
    expect(() => proactivityToWire(input as typeof base)).toThrow();
  },
);

test('A1: null-prototype dictionaries preserve own entries; malformed dictionaries/keys reject', () => {
  const cooldownS = Object.assign(
    Object.create(null) as Record<string, number>,
    JSON.parse('{"__proto__":0,"hasOwnProperty":1,"0":2}'),
  );
  const parsed = proactivityBoundariesSchema.parse({ ...base, cooldownS });
  expect(Object.keys(parsed.cooldownS)).toEqual(Object.keys(cooldownS));
  expect(proactivityToWire(parsed).cooldowns).toEqual(
    Object.entries(cooldownS).map(([kind, seconds]) => ({ schema_version: 1, kind, seconds })),
  );
  for (const invalid of [null, [], 'invalid', { '': 1 }, { '\ud800': 1 }])
    expect(proactivityBoundariesSchema.safeParse({ ...base, cooldownS: invalid }).success).toBe(
      false,
    );
});
