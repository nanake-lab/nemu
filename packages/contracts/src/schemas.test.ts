import { expect, test } from 'vitest';
import { z } from 'zod';
import * as contracts from './index.js';
import { mirroredStagePayloads } from './stage-events.js';

const rect = { x: -1920, y: 0, w: 800, h: 600 };
const platform = { id: 'p', hwnd: '0x1234', x1: -1920, x2: -1000, y: 20, z: 1 };
const cue = { kind: 'act', value: 'wave' };
const readout = {
  version: 'v1',
  expressionBaseline: { happy: 0.4 },
  idleTempo: 1,
  earPose: 0,
  tailAmplitude: 0,
  gazeSociability: 0.5,
  posture: 0,
};
const id = '01ARZ3NDEKTSV4RRFFQ69G5FAV';
const profile = {
  version: 2,
  model: 'nemu.vrm',
  heightM: 1.5,
  headTopOffsetM: 0.2,
  hitRegions: [{ id: 'head', bone: 'head', shape: 'sphere', radiusM: 0.1 }],
  gazeLimits: { yawDeg: 50, pitchUpDeg: 20, pitchDownDeg: 30 },
  framing: {
    fullBody: { fovDeg: 20, distanceM: 4, targetBone: 'hips' },
    upperBody: { fovDeg: 20, distanceM: 2, targetBone: 'chest', offsetM: [0, 0.1, 0] },
  },
  rig: {
    earBonesL: ['earL'],
    earBonesR: ['earR'],
    tailBones: ['tail'],
    earBackMaxDeg: 30,
    tailSwingDeg: [0, 30],
  },
  emotionMap: Object.fromEntries(
    contracts.emotionSchema.options.map((emotion) => [
      emotion,
      [{ expression: emotion, weight: 0.5 }],
    ]),
  ),
  visemeMap: { A: 'aa', I: 'ih', U: 'ou', E: 'ee', O: 'oh' },
  lipSyncProfiles: { zh: 'lipsync/zh.json', en: 'lipsync/en.json', ja: 'lipsync/ja.json' },
  idleSet: [{ motionId: 'idle_stand', weight: 1, cooldownS: 5 }],
};
const stageSamples: Record<keyof contracts.StagePayloads, unknown> = {
  'stage.loadAvatar': { vrmUrl: 'nemu.vrm', profileUrl: 'nemu.profile.json' },
  'stage.setPlacement': {
    displayId: 1,
    anchor: { kind: 'free', x: -20, y: 10 },
    transitionMs: 150,
  },
  'stage.setPhysicalState': { state: 'idle', reason: 'test' },
  'stage.perform': { cue },
  'stage.action': { actionId: 'a', action: { kind: 'motion', motionId: 'fidget_yawn' } },
  'stage.readout': { readout },
  'stage.readout.v2': { readout: { version: 'v2', facing: 0.5, tension: 0, energy: 0.5 } },
  'stage.event': {
    event: {
      v: 2,
      id,
      ts: 0,
      source: 'main',
      type: 'voice.vad',
      payload: { state: 'speechStart' },
    },
  },
  'stage.gaze': { target: 'cursor', holdMs: 500 },
  'stage.setQuality': { fpsActive: 60, fpsIdle: 30, fpsSleep: 12, msaa: 4 },
  'stage.mic.set': { enabled: true, mode: 'pushToTalk' },
  'stage.desktop.update': { platforms: [platform], caret: rect, dnd: 'none' },
  'stage.ready': { avatarName: 'Nemu', boundsPx: rect },
  'stage.hitTest': { overCharacter: true },
  'stage.hit': { region: 'head', kind: 'pat' },
  'stage.drag': { phase: 'move', screenX: -50, screenY: 60, vx: -10, vy: 4 },
  'stage.bounds': { screenRect: rect },
  'stage.frameStats': { fps: 60, p95FrameMs: 16.6, state: 'speaking' },
  'stage.actionResult': { actionId: 'a', status: 'succeeded', detail: 'done' },
  'stage.error': { code: 'test', message: 'sample' },
  'stage.speech.begin': {
    outputId: 'o',
    chunkIndex: 0,
    sentenceId: 's',
    text: 'sample',
    lang: 'en',
    cues: [cue],
    sampleRate: 24000,
  },
  'stage.speech.chunk': { sentenceId: 's', pcm: new Float32Array([0, 0.5]) },
  'stage.speech.end': { sentenceId: 's' },
  'stage.speech.stop': { fadeMs: 100 },
  'stage.speech.started': { sentenceId: 's', at: 100 },
  'stage.speech.progress': { sentenceId: 's', playedMs: 100 },
  'stage.speech.done': { sentenceId: 's', interrupted: false, playedChars: 6 },
};
const queryItem = {
  ref: 'r',
  at: '2026-10-14T00:00:00Z',
  kind: 'user_text',
  summary: 'sample',
  source: 'user',
  retracted: false,
};
const appSamples: Record<keyof contracts.AppBusPayloads, unknown> = {
  'voice.vad': { state: 'speechStart' },
  'voice.bargeIn': {},
  'voice.recognized': {
    utteranceKey: 'u',
    text: 'sample',
    lang: 'en',
    durationMs: 100,
    confidence: 0.9,
  },
  'output.state': { outputId: 'o', state: 'streaming' },
  'output.chunk': { outputId: 'o', chunkIndex: 0, text: 'sample', cues: [cue] },
  'core.runState': {
    states: ['NO-GROWTH'],
    speechMode: 'A_LOCAL_STRICT',
    strict: true,
    lineage: 'REHEARSAL',
    autonomy: 'paused',
    reasons: ['MODULATION_NOT_ENABLED'],
  },
  'approval.request': {
    approvalId: 'a',
    actionId: 'x',
    tier: 'T3',
    title: 'sample',
    detail: 'sample',
    expiresAt: 100,
  },
  'approval.decision': { approvalId: 'a', decision: 'deny' },
  'core.notice': { noticeId: 'n', kind: 'review_hold', at: 'now', summary: 'review', lRef: 'l' },
  'growth.control': { action: 'pause' },
  'approval.request.v2': {
    approvalId: 'a',
    actionId: 'x',
    tier: 'T3',
    title: '',
    detail: '',
    expiresAt: 100,
    actionPayload: new Uint8Array([1]),
    payloadDigest: new Uint8Array(32),
  },
  'approval.decision.v2': { approvalId: 'a', decision: 'deny', payloadDigest: new Uint8Array(32) },
  'chat.send': { text: 'sample' },
  'memory.retract': { targetRefs: ['r'], note: 'sample' },
  'core.query': {
    queryId: 'q',
    kind: 'memory',
    filter: { text: 'sample', includeRetracted: false },
    limit: 10,
  },
  'core.queryResult.v2': {
    queryId: 'q',
    kind: 'growth',
    status: 'ok',
    items: [],
    growthEntries: [],
  },
  'core.queryResult': { queryId: 'q', status: 'ok', items: [queryItem] },
  'desktop.dnd': { state: 'meeting' },
  'desktop.idle': { idleMs: 100 },
  'desktop.observation': {
    kind: 'foreground_changed',
    processName: 'sample.exe',
    title: 'sample',
    dnd: 'none',
    idleMs: 0,
  },
  'codex.session': { event: 'SessionStart', sessionId: 's', cwdLabel: 'sample', summary: 'sample' },
  'metrics.latency': {
    utteranceKey: 'u',
    marks: {
      vadEnd: 0,
      asrDone: 1,
      received: 2,
      committed: 3,
      firstChunk: 4,
      ttsFirstChunk: 5,
      audioStart: 6,
    },
  },
  'stage.ready': stageSamples['stage.ready'],
  'stage.hit': stageSamples['stage.hit'],
  'stage.drag': stageSamples['stage.drag'],
  'stage.frameStats': stageSamples['stage.frameStats'],
  'stage.error': stageSamples['stage.error'],
};
const stage = (type: keyof contracts.StagePayloads) => ({
  v: 2,
  id: 's',
  type,
  payload: stageSamples[type],
});
const bus = (type: keyof contracts.AppBusPayloads) => ({
  v: 2,
  id,
  ts: 0,
  source: 'main',
  type,
  payload: appSamples[type],
});

const samples: Record<string, unknown> = {
  langSchema: 'zh',
  emotionSchema: 'happy',
  dndStateSchema: 'none',
  toolTierSchema: 'T0',
  speechModeSchema: 'A_LOCAL_STRICT',
  lineageSchema: 'LIFE',
  hitRegionIdSchema: 'head',
  motionIdSchema: 'idle_stand',
  performableMotionIdSchema: 'wave',
  requestableMotionIdSchema: 'fidget_yawn',
  nemuProfileSchema: profile,
  assetManifestEntrySchema: {
    path: 'nemu.vrm',
    kind: 'model',
    source: 'original',
    license: 'custom:private',
    redistributable: false,
    author: 'test',
    notes: 'fixture',
  },
  bodyReadoutSchema: readout,
  bodyReadoutV2Schema: { version: 'v2', facing: 0.5, tension: 0, energy: 0.5 },
  bodyReadoutMappingSchema: contracts.CANDIDATE_BODY_READOUT_MAPPING,
  proactiveKindSchema: 'speak',
  socialActionKindSchema: 'social.message',
  proactivityBoundariesSchema: {
    version: '2',
    allowedKinds: ['speak'],
    cooldownS: {},
    gates: ['budget'],
    budgets: {
      minIntervalS: 180,
      speakPerHour: 2,
      quietDayFactor: 0.5,
      autonomousHelperPerDay: 30,
    },
  },
  anchorSchema: { kind: 'free', x: -50, y: 10 },
  gazeTargetSchema: 'user',
  platformSchema: platform,
  rectSchema: rect,
  physicalStateSchema: 'idle',
  bodyActionSchema: { kind: 'sleep' },
  performanceCueSchema: cue,
  performanceSentenceSchema: {
    id: 's',
    outputId: 'o',
    index: 0,
    lang: 'en',
    text: 'sample',
    cues: [cue],
    ttsStyle: 'neutral',
  },
  desktopObservationKindSchema: 'session_locked',
  queryKindSchema: 'memory',
  queryStatusSchema: 'no_match',
  queryFilterSchema: {
    text: 'sample',
    from: '2026-01-01',
    to: '2026-12-31',
    kinds: ['user_text'],
    includeRetracted: false,
  },
  queryItemSchema: queryItem,
  latencyMarksSchema: { vadEnd: 10 },
  appBusEventSchema: bus('chat.send'),
  forwardedAppBusEventSchema: bus('voice.vad'),
  controlCommandSchema: stage('stage.loadAvatar'),
  controlEventSchema: stage('stage.ready'),
  audioCommandSchema: stage('stage.speech.begin'),
  audioEventSchema: stage('stage.speech.done'),
  controlMessageSchema: stage('stage.ready'),
  audioMessageSchema: stage('stage.speech.done'),
  stageMessageSchema: stage('stage.loadAvatar'),
  microphoneMessageSchema: { v: 2, seq: 0, pcm: new Int16Array(320) },
};
const exportedEntries: Array<[string, unknown]> = Object.entries(contracts);
const schemas = exportedEntries.filter(
  (entry): entry is [string, z.ZodType] => entry[1] instanceof z.ZodType,
);
test('every exported schema has a positive and negative fixture', () => {
  expect(schemas.map(([name]) => name).sort()).toEqual(Object.keys(samples).sort());
});
test.each(schemas)('%s accepts a valid example and rejects invalid data', (name, schema) => {
  expect(schema.safeParse(samples[name]).success).toBe(true);
  expect(schema.safeParse(null).success).toBe(false);
  expect(schema.safeParse({ unexpected: true }).success).toBe(false);
});
test.each(Object.entries(contracts.stagePayloads))(
  'stage payload %s validates fields and enforces channel',
  (type, schema) => {
    const key = type as keyof contracts.StagePayloads;
    const message = stage(key);
    expect(schema.safeParse(stageSamples[key]).success).toBe(true);
    expect(schema.safeParse(null).success).toBe(false);
    expect(schema.safeParse({ ...(stageSamples[key] as object), unexpected: 1 }).success).toBe(
      false,
    );
    expect(contracts.stageMessageSchema.safeParse(message).success).toBe(true);
    expect(contracts.controlMessageSchema.safeParse(message).success).toBe(
      !type.startsWith('stage.speech.'),
    );
    expect(contracts.audioMessageSchema.safeParse(message).success).toBe(
      type.startsWith('stage.speech.'),
    );
    for (const field of Object.keys(stageSamples[key] as object)) {
      expect(schema.safeParse({ ...(stageSamples[key] as object), [field]: null }).success).toBe(
        false,
      );
    }
  },
);
test.each(Object.entries(contracts.appBusPayloads))(
  'app bus %s validates payload and stage forwarding',
  (type, schema) => {
    const key = type as keyof contracts.AppBusPayloads;
    const message = bus(key);
    expect(schema.safeParse(appSamples[key]).success).toBe(true);
    expect(schema.safeParse(null).success).toBe(false);
    expect(schema.safeParse({ ...(appSamples[key] as object), extra: true }).success).toBe(false);
    expect(contracts.appBusEventSchema.safeParse(message).success).toBe(true);
    expect(contracts.forwardedAppBusEventSchema.safeParse(message).success).toBe(
      contracts.forwardedAppBusTypes.some((allowed) => allowed === type),
    );
    for (const field of Object.keys(appSamples[key] as object)) {
      expect(schema.safeParse({ ...(appSamples[key] as object), [field]: null }).success).toBe(
        false,
      );
    }
  },
);
test.each(Object.entries(mirroredStagePayloads))(
  'mirrored stage schema %s is identical on the bus',
  (type, schema) => {
    expect(contracts.appBusPayloads[type as keyof typeof mirroredStagePayloads]).toBe(schema);
  },
);
test.each([
  { kind: 'windowEdge', hwnd: '0x1234', offsetX: -100 },
  { kind: 'screenEdge', side: 'left', along: 0.5 },
  { kind: 'taskbar', along: 0.5 },
])('anchor variant %j', (anchor) => {
  expect(contracts.anchorSchema.safeParse(anchor).success).toBe(true);
  expect(contracts.anchorSchema.safeParse({ ...anchor, kind: 'invalid' }).success).toBe(false);
});
test.each([
  { kind: 'goTo', anchor: { kind: 'free', x: 0, y: 0 } },
  { kind: 'sit', platformId: 'p' },
  { kind: 'sleep' },
  { kind: 'wake' },
  { kind: 'peek', side: 'right' },
  { kind: 'gaze', target: { x: -50, y: 10 }, holdMs: 100 },
])('body action variant %j', (action) => {
  expect(contracts.bodyActionSchema.safeParse(action).success).toBe(true);
  expect(contracts.bodyActionSchema.safeParse({ ...action, kind: 'invalid' }).success).toBe(false);
});
test('motion catalogue has exactly 30 distinct slots with correct capabilities', () => {
  expect(contracts.motionSlots).toHaveLength(30);
  expect(new Set(contracts.motionSlots.map((s) => s.id)).size).toBe(30);
  expect(contracts.motionSlots.filter((s) => s.performable)).toHaveLength(10);
  expect(contracts.motionSlots.filter((s) => s.requestable)).toHaveLength(16);
  for (const slot of contracts.motionSlots) {
    expect(contracts.performanceCueSchema.safeParse({ kind: 'act', value: slot.id }).success).toBe(
      slot.performable,
    );
    expect(
      contracts.bodyActionSchema.safeParse({ kind: 'motion', motionId: slot.id }).success,
    ).toBe(slot.requestable);
  }
});
test.each([
  ['idleTempo', 0.5, 1.5],
  ['earPose', -1, 1],
  ['tailAmplitude', 0, 1],
  ['gazeSociability', 0, 1],
  ['posture', -1, 1],
] as const)('readout %s enforces inclusive bounds', (field, min, max) => {
  for (const value of [min, max])
    expect(contracts.bodyReadoutSchema.safeParse({ ...readout, [field]: value }).success).toBe(
      true,
    );
  for (const value of [min - 0.001, max + 0.001, NaN, Infinity, -Infinity, '0'])
    expect(contracts.bodyReadoutSchema.safeParse({ ...readout, [field]: value }).success).toBe(
      false,
    );
});
test('expression readout accepts partial map but rejects unknown emotions and out-of-range weights', () => {
  for (const expressionBaseline of [{}, { happy: 0 }, { happy: 0.4 }])
    expect(contracts.bodyReadoutSchema.safeParse({ ...readout, expressionBaseline }).success).toBe(
      true,
    );
  for (const expressionBaseline of [
    { happy: 0.401 },
    { happy: -0.01 },
    { love: 0.1 },
    { happy: Infinity },
  ])
    expect(contracts.bodyReadoutSchema.safeParse({ ...readout, expressionBaseline }).success).toBe(
      false,
    );
});
test('profile enforces v2, complete maps, tuple size and capsule length', () => {
  const invalid = [
    { ...profile, version: 1 },
    { ...profile, emotionImpulse: {} },
    { ...profile, emotionMap: { happy: [] } },
    { ...profile, lipSyncProfiles: { zh: 'zh.json' } },
    { ...profile, visemeMap: { ...profile.visemeMap, A: 'oh' } },
    { ...profile, hitRegions: [{ id: 'head', bone: 'head', shape: 'capsule', radiusM: 0.1 }] },
    { ...profile, rig: { ...profile.rig, tailSwingDeg: [1] } },
    { ...profile, heightM: Infinity },
  ];
  for (const sample of invalid)
    expect(contracts.nemuProfileSchema.safeParse(sample).success).toBe(false);
  expect(
    contracts.nemuProfileSchema.safeParse({
      ...profile,
      hitRegions: [
        {
          id: 'tail',
          bone: 'tail',
          shape: 'capsule',
          radiusM: 0.1,
          lengthM: 1,
          offsetM: [0, 0, 0],
        },
      ],
    }).success,
  ).toBe(true);
});
test('wire versions, ids, and binary microphone format are validated', () => {
  expect(
    contracts.appBusEventSchema.safeParse({ ...bus('chat.send'), id: 'not-ulid' }).success,
  ).toBe(false);
  expect(contracts.appBusEventSchema.safeParse({ ...bus('chat.send'), v: 1 }).success).toBe(false);
  expect(contracts.stageMessageSchema.safeParse({ ...stage('stage.ready'), v: 1 }).success).toBe(
    false,
  );
  for (const pcm of [new Int16Array(319), new Float32Array(320), Array(320).fill(0)])
    expect(contracts.microphoneMessageSchema.safeParse({ v: 2, seq: 0, pcm }).success).toBe(false);
  expect(
    contracts.audioCommandSchema.safeParse({
      ...stage('stage.speech.chunk'),
      payload: { sentenceId: 's', pcm: [0] },
    }).success,
  ).toBe(false);
});
test.each([
  '/absolute/model.vrm',
  'C:\\model.vrm',
  '\\\\server\\model.vrm',
  'https://example.test/model.vrm',
  '',
])('profile rejects non-relative asset location %s', (model) => {
  expect(contracts.nemuProfileSchema.safeParse({ ...profile, model }).success).toBe(false);
  expect(
    contracts.nemuProfileSchema.safeParse({
      ...profile,
      lipSyncProfiles: { ...profile.lipSyncProfiles, zh: model },
    }).success,
  ).toBe(false);
});
