import { controlCommandPayloads } from '@nemu/contracts';

export type StageCommandType = keyof typeof controlCommandPayloads;

export interface ValidationResult {
  valid: boolean;
  errors?: string[] | undefined;
  parsed?: unknown | undefined;
}

export const STAGE_COMMAND_PRESETS: Record<StageCommandType, Record<string, unknown>> = {
  'stage.readout.v2': { readout: { version: 'readout-v2-preview', facing: 0.5, tension: 0, energy: 0.5 } },
  'stage.perform': {
    cue: {
      kind: 'emo',
      value: 'happy',
      intensity: 0.8,
    },
  },
  'stage.setPhysicalState': {
    state: 'idle',
    reason: 'manual debug trigger',
  },
  'stage.setPlacement': {
    displayId: 0,
    anchor: {
      kind: 'free',
      x: 100,
      y: 100,
    },
    transitionMs: 300,
  },
  'stage.gaze': {
    target: 'cursor',
    holdMs: 2000,
  },
  'stage.setQuality': {
    fpsActive: 60,
    fpsIdle: 30,
    fpsSleep: 12,
    msaa: 4,
  },
  'stage.mic.set': {
    enabled: true,
    mode: 'pushToTalk',
  },
  'stage.action': {
    actionId: 'debug-act-001',
    action: {
      kind: 'motion',
      motionId: 'wave',
    },
  },
  'stage.readout': {
    readout: {
      version: 'genesis-v1',
      expressionBaseline: {
        happy: 0.2,
      },
      idleTempo: 1.0,
      earPose: 0,
      tailAmplitude: 0.5,
      gazeSociability: 0.8,
      posture: 0,
    },
  },
  'stage.loadAvatar': {
    vrmUrl: 'content/nemu/nemu.vrm',
    profileUrl: 'content/nemu/nemu.profile.json',
  },
  'stage.event': {
    event: {
      v: 2,
      id: '01AN4Z07BY79KA1307SR9X4MV3',
      ts: Date.now(),
      source: 'main',
      type: 'voice.vad',
      payload: {
        state: 'speechStart',
      },
    },
  },
  'stage.desktop.update': {
    platforms: [],
    dnd: 'none',
  },
};

export const STAGE_COMMAND_LIST = Object.keys(controlCommandPayloads) as StageCommandType[];

/**
 * Validates a payload against the zod schema for the selected stage command type.
 */
export function validateStageCommand(type: StageCommandType, payloadRaw: unknown): ValidationResult {
  const schema = controlCommandPayloads[type];
  if (!schema) {
    return {
      valid: false,
      errors: [`未知的舞台命令类型: ${String(type)}`],
    };
  }

  const result = schema.safeParse(payloadRaw);
  if (result.success) {
    return {
      valid: true,
      parsed: result.data,
    };
  }

  const errors = result.error.issues.map((issue) => {
    const path = issue.path.length > 0 ? issue.path.join('.') : 'root';
    return `${path}: ${issue.message}`;
  });

  return {
    valid: false,
    errors,
  };
}

/**
 * Helper to parse and validate JSON text.
 */
export function validateJsonAndSchema(type: StageCommandType, jsonText: string): ValidationResult {
  let parsedJson: unknown;
  try {
    parsedJson = JSON.parse(jsonText);
  } catch (err) {
    return {
      valid: false,
      errors: [`JSON 语法错误: ${err instanceof Error ? err.message : String(err)}`],
    };
  }

  return validateStageCommand(type, parsedJson);
}
