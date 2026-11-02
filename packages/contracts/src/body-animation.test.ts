import { expect, test } from 'vitest';
import { readFileSync } from 'node:fs';
import {
  bodyReadoutMappingSchema,
  CANDIDATE_BODY_READOUT_MAPPING,
  controlCommandSchema,
  nemuProfileSchema,
} from './index.js';

test('new stage entry accepts only the v2 triple, legacy entry still decodes identical fields', () => {
  const legacy = {
    v: 2,
    id: 'old',
    type: 'stage.readout',
    payload: {
      readout: {
        version: 'v1',
        expressionBaseline: { happy: 0.4 },
        posture: 0.1,
        idleTempo: 1.2,
        earPose: -0.2,
        tailAmplitude: 0.7,
        gazeSociability: 0.6,
      },
    },
  };
  expect(controlCommandSchema.parse(legacy)).toEqual(legacy);
  const successor = {
    ...legacy,
    type: 'stage.readout.v2',
    payload: {
      readout: { version: 'core-readout-2', facing: 0.5, tension: 0, energy: 0.5 },
    },
  };
  expect(controlCommandSchema.parse(successor)).toEqual(successor);
  expect(controlCommandSchema.safeParse({ ...legacy, type: 'stage.readout.v2' }).success).toBe(
    false,
  );
  expect(controlCommandSchema.safeParse({ ...successor, type: 'stage.readout' }).success).toBe(
    false,
  );
  expect(
    controlCommandSchema.safeParse({
      ...successor,
      payload: {
        readout: { ...successor.payload.readout, posture: 0 },
      },
    }).success,
  ).toBe(false);
});

test('old profile remains identical, new optional mapping is explicit and validated', () => {
  const old: unknown = JSON.parse(readFileSync('apps/stage/public/sample.profile.json', 'utf8'));
  const parsed = nemuProfileSchema.parse(old);
  expect(parsed).toEqual(old);
  expect(
    nemuProfileSchema.parse({ ...parsed, bodyReadoutMapping: CANDIDATE_BODY_READOUT_MAPPING })
      .bodyReadoutMapping,
  ).toEqual(CANDIDATE_BODY_READOUT_MAPPING);
  for (const field of [
    'facingBodyYawDeg',
    'postureScale',
    'earPoseScale',
    'tailTensionScale',
    'energyTempoScale',
    'energyAmplitudeScale',
  ]) {
    for (const invalid of [NaN, Infinity, -Infinity, null, 100, -100]) {
      expect(
        bodyReadoutMappingSchema.safeParse({ ...CANDIDATE_BODY_READOUT_MAPPING, [field]: invalid })
          .success,
      ).toBe(false);
    }
  }
  expect(
    bodyReadoutMappingSchema.safeParse({ ...CANDIDATE_BODY_READOUT_MAPPING, version: 'unknown' })
      .success,
  ).toBe(false);
  expect(
    bodyReadoutMappingSchema.safeParse({
      ...CANDIDATE_BODY_READOUT_MAPPING,
      calibration: 'approved',
    }).success,
  ).toBe(false);
});
