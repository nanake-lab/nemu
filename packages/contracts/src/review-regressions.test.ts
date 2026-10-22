import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { expect, test } from 'vitest';
import { validateApproval } from './approval.js';
import { appBusPayloads } from './appbus.js';
import { queryResultToPanel } from './queries.js';
import { proactivityBoundariesSchema, proactivityToWire } from './proactivity.js';
import { wireInputSchema, resourceInputSchema } from './generated/echo/browser.js';
import { encodeCanonical, decodeCanonical } from './generated/echo/index.js';
import type {
  ApprovalRequestV2,
  ApprovalDecisionV2,
  LedgerEntry,
  QueryV2,
  QueryResultV2,
} from './generated/echo/schema.js';

function approval() {
  const payload = new Uint8Array([65]);
  const digest = new Uint8Array(createHash('sha256').update(payload).digest());
  const request: ApprovalRequestV2 = {
    schema_version: 1,
    approval_id: 'a',
    action_id: 'x',
    tier: 4,
    title: '',
    detail: '',
    expires_at_ns: 10n,
    action_payload: payload.slice(),
    payload_digest: digest,
  };
  const decision: ApprovalDecisionV2 = {
    schema_version: 1,
    approval_id: 'a',
    decision: 1,
    payload_digest: digest.slice(),
  };
  return { payload, request, decision };
}

test.each([
  'execution',
  'requestPayload',
  'requestDigest',
  'decisionDigest',
  'decisionChoice',
  'tier',
  'expiry',
  'actionId',
  'approvalId',
  'title',
])('B1: reject %s mutation during await', async (changed) => {
  const { payload, request, decision } = approval();
  const pending = validateApproval(request, decision, payload, 0n);
  if (changed === 'execution') payload[0] = 66;
  if (changed === 'requestPayload') request.action_payload[0] = 66;
  if (changed === 'requestDigest') request.payload_digest.fill(0);
  if (changed === 'decisionDigest') decision.payload_digest.fill(0);
  if (changed === 'decisionChoice') decision.decision = 3;
  if (changed === 'tier') request.tier = 3;
  if (changed === 'expiry') request.expires_at_ns = 100n;
  if (changed === 'actionId') request.action_id = 'changed';
  if (changed === 'approvalId') decision.approval_id = 'changed';
  if (changed === 'title') request.title = 'changed';
  expect(await pending).toBeUndefined();
});

test('B1: consume the immutable validated snapshot, once, with fresh time', async () => {
  const { payload, request, decision } = approval();
  const validated = await validateApproval(request, decision, payload, 0n);
  expect(validated).toBeDefined();
  payload.fill(66);
  request.action_payload.fill(66);
  decision.payload_digest.fill(0);
  const executed = validated!.consume(9n, (payloadHex, binding) => {
    expect(payloadHex).toBe('41');
    expect(binding.actionId).toBe('x');
    expect(Object.isFrozen(binding)).toBe(true);
    expect(binding.payloadDigest).toBe(
      createHash('sha256').update(Buffer.from(payloadHex, 'hex')).digest('hex'),
    );
    return Buffer.from(payloadHex, 'hex');
  });
  expect([...executed]).toEqual([65]);
  expect(() => validated!.consume(9n, () => true)).toThrow();
  const another = approval();
  const expired = await validateApproval(another.request, another.decision, another.payload, 0n);
  expect(() => expired!.consume(10n, () => true)).toThrow();
  expect(() => expired!.consume(9n, () => true)).toThrow();
  expect(() =>
    Reflect.construct(validated!.constructor, [Symbol('fake'), request, decision, 0n]),
  ).toThrow('Unvalidated approval');
});

test('B1: rejection/throw consumes the validation snapshot, no blind retries', async () => {
  const { payload, request, decision } = approval();
  const validated = await validateApproval(request, decision, payload, 0n);
  expect(() =>
    validated!.consume(1n, () => {
      throw new Error('executor failed');
    }),
  ).toThrow('executor failed');
  expect(() => validated!.consume(2n, () => true)).toThrow('Approval unavailable');
});

const entry: LedgerEntry = {
  schema_version: 1,
  entry_id: 'l-1',
  event_id: 'h-1',
  kind: 1,
  tendency_change: {
    schema_version: 1,
    tendency_id: 't-1',
    pi_before: 0,
    pi_after: 0.8,
    evidence_groups: ['g-1'],
  },
};
const query: QueryV2 = { schema_version: 1, query_id: 'q', kind: 4, limit: 10n };
const result: QueryResultV2 = {
  schema_version: 1,
  query_id: 'q',
  status: 1,
  items: [],
  growth_entries: [entry],
};

test('B2: nonempty L survives IDL -> main -> panel, preserving evidence references', () => {
  const payload = queryResultToPanel(query, result);
  const envelope = appBusPayloads['core.queryResult.v2'].parse(payload);
  expect(envelope.kind).toBe('growth');
  expect(envelope.items).toEqual([]);
  expect(envelope.growthEntries).toEqual([entry]);
  expect(envelope.growthEntries[0]!.tendency_change!.evidence_groups).toEqual(['g-1']);
  expect(
    appBusPayloads['core.queryResult'].safeParse({ queryId: 'legacy', status: 'ok', items: [] })
      .success,
  ).toBe(true);
});

test('B2: both layers reject mismatched query identity, mixed results and malformed L', () => {
  expect(() => queryResultToPanel(query, { ...result, query_id: 'other' })).toThrow();
  expect(() => queryResultToPanel({ ...query, kind: 1 }, result)).toThrow();
  const item = {
    schema_version: 1,
    ref: 'r',
    at: 'now',
    kind: 'fact',
    summary: 's',
    source: 'h',
    retracted: false,
  };
  expect(() => queryResultToPanel(query, { ...result, items: [item] })).toThrow();
  expect(
    appBusPayloads['core.queryResult.v2'].safeParse({
      queryId: 'q',
      kind: 'memory',
      status: 'ok',
      items: [],
      growthEntries: [entry],
    }).success,
  ).toBe(false);
  expect(
    appBusPayloads['core.queryResult.v2'].safeParse({
      queryId: 'q',
      kind: 'growth',
      status: 'ok',
      items: [],
      growthEntries: [{ ...entry, vote: true }],
    }).success,
  ).toBe(false);
  expect(
    wireInputSchema('LedgerEntry').safeParse({
      ...entry,
      tendency_change: { ...entry.tendency_change, pi_after: Infinity },
    }).success,
  ).toBe(false);
  expect(wireInputSchema('LedgerEntry').safeParse({ ...entry, kind: 5 }).success).toBe(false);
  for (const status of [1, 2, 3, 4, 5, 6] as const) {
    expect(queryResultToPanel(query, { ...result, status, growth_entries: [] }).status).toBe(
      ['ok', 'no_match', 'index_lagging', 'denied', 'object_missing', 'budget_exhausted'][
        status - 1
      ],
    );
  }
});

test('D1: JSON boundary validation and wire projection derive from authority IDL', () => {
  const raw: unknown = JSON.parse(
    readFileSync(new URL('../../../config/opportunities.json', import.meta.url), 'utf8'),
  );
  const config = proactivityBoundariesSchema.parse(raw);
  expect(resourceInputSchema('ProactivityBoundaries').parse(raw)).toEqual(config);
  const wire = proactivityToWire({ ...config, cooldownS: { speak: 20, motion: 30 } });
  const limits = { max_bytes: 1048576, max_items: 4096, max_depth: 32 };
  expect(
    decodeCanonical(
      'ProactivityBoundaries',
      encodeCanonical('ProactivityBoundaries', wire, limits),
      limits,
    ),
  ).toEqual(wire);
  expect(wire.cooldowns).toEqual([
    { schema_version: 1, kind: 'speak', seconds: 20 },
    { schema_version: 1, kind: 'motion', seconds: 30 },
  ]);
  expect(wire.budgets.min_interval_s).toBe(180);
  for (const changed of [
    { ...config, allowedKinds: ['arbitrary'] },
    { ...config, gates: ['arbitrary'] },
    { ...config, allowedKinds: ['speak', 'speak'] },
    { ...config, gates: ['mode', 'mode'] },
    { ...config, cooldownS: { '': 1 } },
    { ...config, cooldownS: { speak: NaN } },
    { ...config, budgets: { ...config.budgets, speakPerHour: 4294967296 } },
    { ...config, budgets: { ...config.budgets, quietDayFactor: 1.1 } },
  ])
    expect(proactivityBoundariesSchema.safeParse(changed).success).toBe(false);
});

test('B2: nonempty ordinary queries keep every QueryItem field', () => {
  const item = {
    schema_version: 1,
    ref: 'fact',
    at: 'now',
    kind: 'fact',
    summary: 'content',
    source: 'h',
    retracted: true,
  };
  for (const kind of [1, 2, 3] as const) {
    const panel = queryResultToPanel(
      { ...query, kind },
      { ...result, items: [item], growth_entries: [] },
    );
    expect(panel.items).toEqual([
      { ref: 'fact', at: 'now', kind: 'fact', summary: 'content', source: 'h', retracted: true },
    ]);
    expect(panel.growthEntries).toEqual([]);
  }
});
