import { describe, it, expect } from 'vitest';
import {
  STAGE_COMMAND_LIST,
  STAGE_COMMAND_PRESETS,
  validateStageCommand,
  validateJsonAndSchema,
} from './schema-validator.js';

describe('schema-validator', () => {
  it('validates all default preset payloads successfully', () => {
    for (const cmdType of STAGE_COMMAND_LIST) {
      const preset = STAGE_COMMAND_PRESETS[cmdType];
      expect(preset, `Preset for ${cmdType} should exist`).toBeDefined();
      const res = validateStageCommand(cmdType, preset);
      expect(res.valid, `Preset for ${cmdType} should be valid: ${res.errors?.join(', ')}`).toBe(true);
      expect(res.parsed).toBeDefined();
    }
  });

  it('validates stage.perform with valid performance cues', () => {
    const validPerform = {
      cue: {
        kind: 'emo',
        value: 'happy',
        intensity: 0.8,
      },
    };
    const res = validateStageCommand('stage.perform', validPerform);
    expect(res.valid).toBe(true);
  });

  it('rejects stage.perform with invalid cue kind', () => {
    const invalidPerform = {
      cue: {
        kind: 'nonexistent-kind',
        value: 'happy',
      },
    };
    const res = validateStageCommand('stage.perform', invalidPerform);
    expect(res.valid).toBe(false);
    expect(res.errors?.length).toBeGreaterThan(0);
    expect(res.errors?.[0]).toContain('cue.kind');
  });

  it('rejects invalid JSON string syntax', () => {
    const res = validateJsonAndSchema('stage.perform', '{ invalid json');
    expect(res.valid).toBe(false);
    expect(res.errors?.[0]).toContain('JSON 语法错误');
  });

  it('parses and validates valid JSON text for stage.perform', () => {
    const jsonStr = JSON.stringify({
      cue: {
        kind: 'act',
        value: 'wave',
      },
    });
    const res = validateJsonAndSchema('stage.perform', jsonStr);
    expect(res.valid).toBe(true);
    expect(res.parsed).toEqual({
      cue: {
        kind: 'act',
        value: 'wave',
      },
    });
  });

  it('validates stage.setPlacement with free anchor and windowEdge anchor', () => {
    const freeRes = validateStageCommand('stage.setPlacement', {
      displayId: 0,
      anchor: { kind: 'free', x: 200, y: 300 },
      transitionMs: 500,
    });
    expect(freeRes.valid).toBe(true);

    const windowRes = validateStageCommand('stage.setPlacement', {
      displayId: 1,
      anchor: { kind: 'windowEdge', hwnd: '0x1234', offsetX: 50 },
    });
    expect(windowRes.valid).toBe(true);
  });
});
