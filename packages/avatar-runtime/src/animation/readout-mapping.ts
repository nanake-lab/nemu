import {
  bodyReadoutV2Schema,
  bodyReadoutMappingSchema,
  CANDIDATE_BODY_READOUT_MAPPING,
  type BodyReadoutV2,
  type BodyReadoutMapping,
} from '@nemu/contracts';

export interface BodyAnimationParameters {
  gazeUserWeight: number;
  bodyTurnMaxDeg: number;
  posture: number;
  earPose: number;
  tailAmplitude: number;
  idleTempo: number;
  motionAmplitude: number;
}

/** Fixed animation mapping, distinct from the core's nature-to-readout function. */
export function mapBodyReadout(
  readout: BodyReadoutV2,
  mapping: Readonly<BodyReadoutMapping> = CANDIDATE_BODY_READOUT_MAPPING,
): BodyAnimationParameters {
  const { facing, tension, energy } = bodyReadoutV2Schema.parse(readout);
  const params = bodyReadoutMappingSchema.parse(mapping);
  return {
    gazeUserWeight: facing,
    bodyTurnMaxDeg: params.facingBodyYawDeg,
    posture: tension * params.postureScale,
    earPose: tension * params.earPoseScale,
    tailAmplitude: 0.5 + tension * params.tailTensionScale,
    idleTempo: 1 + (2 * energy - 1) * params.energyTempoScale,
    motionAmplitude: 1 + (2 * energy - 1) * params.energyAmplitudeScale,
  };
}
