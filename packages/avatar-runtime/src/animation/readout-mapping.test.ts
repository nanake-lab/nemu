import { expect, test } from 'vitest';
import { CANDIDATE_BODY_READOUT_MAPPING } from '@nemu/contracts';
import { mapBodyReadout } from './readout-mapping.js';

test('neutral triple gives neutral offsets, original breathing amplitude and tempo', () => {
  expect(mapBodyReadout({ version: 'core-test', facing: 0.5, tension: 0, energy: 0.5 })).toEqual({
    gazeUserWeight: 0.5,
    bodyTurnMaxDeg: 10,
    posture: -0,
    earPose: -0,
    tailAmplitude: 0.5,
    idleTempo: 1,
    motionAmplitude: 1,
  });
});

test('all corner triples have bounded independent axis mappings', () => {
  for (const facing of [0, 1])
    for (const tension of [-1, 1])
      for (const energy of [0, 1]) {
        const output = mapBodyReadout({ version: 'test', facing, tension, energy });
        expect(output.gazeUserWeight).toBe(facing);
        expect(output.posture).toBe(tension === -1 ? 0.5 : -0.5);
        expect(output.earPose).toBe(tension === -1 ? 0.5 : -0.5);
        expect(output.tailAmplitude).toBe(tension === -1 ? 0.75 : 0.25);
        expect(output.idleTempo).toBe(energy === 0 ? 0.5 : 1.5);
        expect(output.motionAmplitude).toBe(energy === 0 ? 0.5 : 1.5);
      }
});

test('candidate profile tuning is explicit, deterministic and leaves input unchanged', () => {
  const readout = Object.freeze({ version: 'core', facing: 0.9, tension: 1, energy: 1 });
  const mapping = Object.freeze({
    ...CANDIDATE_BODY_READOUT_MAPPING,
    postureScale: -0.3,
    energyTempoScale: 0.2,
  });
  const first = mapBodyReadout(readout, mapping);
  for (let i = 0; i < 100; i++) expect(mapBodyReadout(readout, mapping)).toEqual(first);
  expect(first.posture).toBe(-0.3);
  expect(first.idleTempo).toBe(1.2);
  expect(CANDIDATE_BODY_READOUT_MAPPING.postureScale).toBe(-0.5);
});
