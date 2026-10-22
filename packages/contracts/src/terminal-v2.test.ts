import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { expect, test } from 'vitest';
import { approvalDecisionMatches } from './approval.js';
import {
  appBusPayloads,
  lineageSchema,
  bodyReadoutV2Schema,
  proactivityBoundariesSchema,
  socialPermission,
} from './index.js';
import {
  inputSchema,
  encodeCanonical,
  decodeCanonical,
  encodeProtobuf,
  decodeProtobuf,
} from './generated/echo/index.js';
import type { ApprovalRequestV2, ApprovalDecisionV2, ReadoutV2 } from './generated/echo/schema.js';

const limits = { max_bytes: 1048576, max_items: 4096, max_depth: 32 };
const readout: ReadoutV2 = {
  schema_version: 1,
  snapshot_id: 'A',
  projected_at_ns: 0n,
  v2: { schema_version: 1, version: 'V', facing: 0.5, tension: 0, energy: 0.5 },
};

test('ADR 0007: independent oneof byte oracle and both transports', () => {
  const u32 = (n: number) => {
    const b = Buffer.alloc(4);
    b.writeUInt32LE(n);
    return b;
  };
  const blob = (b: Buffer) => Buffer.concat([u32(b.length), b]);
  const header = (tag: string) =>
    Buffer.concat([Buffer.from('NCV1'), u32(1), blob(Buffer.from(tag))]);
  const doubles = Buffer.alloc(24);
  doubles.writeDoubleLE(0.5);
  doubles.writeDoubleLE(0, 8);
  doubles.writeDoubleLE(0.5, 16);
  const body = Buffer.concat([header('echo.body-readout-v2'), blob(Buffer.from('V')), doubles]);
  const prefix = Buffer.concat([
    header('echo.readout-v2'),
    blob(Buffer.from('A')),
    Buffer.alloc(8),
  ]);
  const expected = Buffer.concat([prefix, u32(6), blob(body)]);
  expect(encodeCanonical('ReadoutV2', readout, limits)).toEqual(expected);
  expect(decodeCanonical('ReadoutV2', expected, limits)).toEqual(readout);
  expect(decodeProtobuf('ReadoutV2', encodeProtobuf('ReadoutV2', readout, limits), limits)).toEqual(
    readout,
  );
  for (const n of [0, 1, 2, 3, 4, 7, 99, 0xffffffff]) {
    expect(() =>
      decodeCanonical('ReadoutV2', Buffer.concat([prefix, u32(n), blob(body)]), limits),
    ).toThrow();
  }
  expect(() =>
    decodeCanonical('ReadoutV2', Buffer.concat([expected, Buffer.from([0])]), limits),
  ).toThrow();
  expect(() => decodeCanonical('ReadoutV2', prefix, limits)).toThrow();
  expect(() => inputSchema('ReadoutV2', limits).parse({ ...readout, v2: undefined })).toThrow();
});

test('readout bounds and no expression baseline in v2', () => {
  const neutral = { version: 'v2', facing: 0.5, tension: 0, energy: 0.5 };
  expect(bodyReadoutV2Schema.parse(neutral)).toEqual(neutral);
  for (const field of ['facing', 'tension', 'energy']) {
    for (const value of [-2, 2, NaN, Infinity, -Infinity]) {
      expect(bodyReadoutV2Schema.safeParse({ ...neutral, [field]: value }).success).toBe(false);
    }
  }
  expect(bodyReadoutV2Schema.safeParse({ ...neutral, expressionBaseline: {} }).success).toBe(false);
});

test('notice and growth controls do not expose a vote; experiment never reaches terminal', () => {
  const notice = {
    noticeId: 'n',
    kind: 'review_hold',
    at: 'now',
    summary: 'Review law',
    lRef: 'l',
  };
  for (const key of ['response', 'verdict', 'accept', 'vote']) {
    expect(appBusPayloads['core.notice'].safeParse({ ...notice, [key]: 'yes' }).success).toBe(
      false,
    );
    expect(
      appBusPayloads['growth.control'].safeParse({ action: 'pause', [key]: true }).success,
    ).toBe(false);
  }
  expect(appBusPayloads['growth.control'].safeParse({ action: 'review' }).success).toBe(false);
  expect(lineageSchema.safeParse('EXPERIMENT').success).toBe(false);
  expect(() =>
    inputSchema('RunStateV2', limits).parse({
      schema_version: 1,
      states: [],
      speech_mode: 1,
      strict: true,
      autonomy: 1,
      reasons: [],
      lineage: 3,
    }),
  ).toThrow();
});

test('boundary config has no behavior triggers; defaults and social tiers', () => {
  const value: unknown = JSON.parse(
    readFileSync(new URL('../../../config/opportunities.json', import.meta.url), 'utf8'),
  );
  const config = proactivityBoundariesSchema.parse(value);
  expect(config.budgets).toEqual({
    minIntervalS: 180,
    speakPerHour: 2,
    quietDayFactor: 0.5,
    autonomousHelperPerDay: 30,
  });
  for (const key of ['trigger', 'candidate', 'rules']) {
    expect(proactivityBoundariesSchema.safeParse({ ...config, [key]: [] }).success).toBe(false);
  }
  expect(
    proactivityBoundariesSchema.safeParse({ ...config, cooldownS: { speak: -1 } }).success,
  ).toBe(false);
  expect(socialPermission('social.message')).toEqual({ tier: 'T2', trustAllowed: true });
  expect(socialPermission('social.commit')).toEqual({ tier: 'T3', trustAllowed: false });
});

test('I26 approval binds the viewed bytes and executed bytes; mismatches, expiry and T3 trust fail', async () => {
  const payload = new TextEncoder().encode('原样行动🙂');
  const digest = createHash('sha256').update(payload).digest();
  const request: ApprovalRequestV2 = {
    schema_version: 1,
    approval_id: 'a',
    action_id: 'x',
    tier: 4,
    title: 'gate title',
    detail: 'gate detail',
    expires_at_ns: 10n,
    action_payload: payload,
    payload_digest: digest,
  };
  const decision: ApprovalDecisionV2 = {
    schema_version: 1,
    approval_id: 'a',
    decision: 1,
    payload_digest: digest,
  };
  expect(await approvalDecisionMatches(request, decision, payload, 9n)).toBe(true);
  expect(
    await approvalDecisionMatches(
      request,
      { ...decision, payload_digest: new Uint8Array(32) },
      payload,
      9n,
    ),
  ).toBe(false);
  expect(await approvalDecisionMatches(request, decision, new Uint8Array([1]), 9n)).toBe(false);
  expect(
    await approvalDecisionMatches(
      { ...request, action_payload: new Uint8Array([1]) },
      decision,
      new Uint8Array([1]),
      9n,
    ),
  ).toBe(false);
  expect(
    await approvalDecisionMatches(request, { ...decision, approval_id: 'b' }, payload, 9n),
  ).toBe(false);
  expect(await approvalDecisionMatches(request, decision, payload, 10n)).toBe(false);
  expect(await approvalDecisionMatches(request, { ...decision, decision: 2 }, payload, 9n)).toBe(
    false,
  );
  expect(await approvalDecisionMatches(request, { ...decision, decision: 3 }, payload, 9n)).toBe(
    false,
  );
  expect(
    await approvalDecisionMatches(
      { ...request, tier: 3 },
      { ...decision, decision: 2 },
      payload,
      9n,
    ),
  ).toBe(true);
});
