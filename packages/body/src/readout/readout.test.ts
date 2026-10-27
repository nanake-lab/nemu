import { describe, it, expect, vi } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve, join } from 'node:path';
import ts from 'typescript';
import type { BodyReadoutV2 } from '@nemu/contracts';
import { ReadoutApplicator, sanitizeBodyReadout, areReadoutsEqual } from './applicator.js';
import { NEUTRAL_BODY_READOUT } from './constants.js';
import { StageReadoutInput } from './stage-input.js';

const target: BodyReadoutV2 = { version: 'readout-v2-test', facing: 1, tension: 1, energy: 1 };

describe('P3-06 v2 readout application', () => {
  it.each([0, 1])('accepts facing/energy endpoint %s and both tension endpoints', (value) => {
    for (const tension of [-1, 1]) {
      const input = { ...target, facing: value, tension, energy: value };
      expect(sanitizeBodyReadout(input)).toEqual(input);
    }
  });

  it('uses the neutral triple for absent, partial, v1, extra or invalid input', () => {
    for (const input of [
      null,
      undefined,
      {},
      { ...target, expressionBaseline: {} },
      { version: 'old', idleTempo: 1 },
      { ...target, version: '' },
    ]) {
      expect(sanitizeBodyReadout(input)).toEqual(NEUTRAL_BODY_READOUT);
    }
    for (const key of ['facing', 'tension', 'energy']) {
      for (const value of [NaN, Infinity, -Infinity, null, '1', 1.01, -1.01]) {
        expect(sanitizeBodyReadout({ ...target, [key]: value })).toEqual(NEUTRAL_BODY_READOUT);
      }
    }
  });

  it.each(['DISCONNECTED', 'CORE_UNMODULATED', 'MODULATION_NOT_ENABLED'] as const)(
    'clears modulation for %s',
    (status) => {
      const app = new ReadoutApplicator({ smoothingDurationSec: 0 });
      app.applyReadout(target);
      app.applyReadout(target, status);
      expect(app.getCurrentReadout()).toEqual(NEUTRAL_BODY_READOUT);
    },
  );

  it('does not restart interpolation or accumulate on duplicate targets', () => {
    const app = new ReadoutApplicator();
    expect(app.applyReadout(target)).toBe(true);
    expect(app.update(1)).toEqual({ ...target, facing: 0.75, tension: 0.5, energy: 0.75 });
    expect(app.applyReadout({ ...target })).toBe(false);
    expect(app.update(1)).toEqual(target);
    for (let i = 0; i < 100; i++) {
      expect(app.applyReadout(target)).toBe(false);
      expect(app.update(0.1)).toEqual(target);
    }
  });

  it('retargets from the currently rendered triple and supports immediate completion', () => {
    const app = new ReadoutApplicator();
    app.applyReadout(target);
    app.update(1);
    app.resetToNeutral();
    expect(app.update(1).tension).toBeCloseTo(0.25);
    app.resetToNeutral(true);
    expect(app.getCurrentReadout()).toEqual(NEUTRAL_BODY_READOUT);
    expect(app.isInterpolating()).toBe(false);
  });

  it('defensively copies input, callback and returned triples', () => {
    const onReadoutUpdated = vi.fn((value: BodyReadoutV2) => {
      value.facing = 0;
    });
    const app = new ReadoutApplicator({ smoothingDurationSec: 0, onReadoutUpdated });
    const input = { ...target };
    app.applyReadout(input);
    input.facing = 0;
    app.getCurrentReadout().tension = 0;
    app.getTargetReadout().energy = 0;
    app.update(1).energy = 0;
    expect(app.getCurrentReadout()).toEqual(target);
    expect(onReadoutUpdated).toHaveBeenCalledTimes(1);
  });

  it('does not discard small valid updates and ignores invalid frame durations', () => {
    expect(areReadoutsEqual(target, { ...target, tension: 1 - 1e-10 })).toBe(false);
    const app = new ReadoutApplicator();
    app.applyReadout(target);
    for (const delta of [NaN, Infinity, -1, 0]) app.update(delta);
    expect(app.getCurrentReadout()).toEqual(NEUTRAL_BODY_READOUT);
    expect(() => new ReadoutApplicator({ smoothingDurationSec: NaN })).toThrow();
  });

  it('validates stage successor input and honors run-state gates without stale replay', () => {
    const app = new ReadoutApplicator({ smoothingDurationSec: 0 });
    const input = new StageReadoutInput(app);
    const command = { v: 2, id: 'test', type: 'stage.readout.v2', payload: { readout: target } };
    expect(input.handle(command)).toBe(true);
    expect(app.getCurrentReadout()).toEqual(target);
    input.setConnectionStatus('CORE_UNMODULATED');
    input.handle(command);
    expect(app.getCurrentReadout()).toEqual(NEUTRAL_BODY_READOUT);
    input.setConnectionStatus('CONNECTED');
    expect(app.getCurrentReadout()).toEqual(NEUTRAL_BODY_READOUT);
    input.handle(command);
    expect(app.getCurrentReadout()).toEqual(target);
    expect(input.handle({ ...command, payload: { readout: { ...target, energy: NaN } } })).toBe(
      false,
    );
    expect(input.handle({ ...command, v: 1 })).toBe(false);
    expect(app.getCurrentReadout()).toEqual(target);
    const runState = {
      v: 2,
      id: 'run',
      type: 'stage.event',
      payload: {
        event: {
          v: 2,
          id: '01ARZ3NDEKTSV4RRFFQ69G5FAV',
          ts: 0,
          source: 'main',
          type: 'core.runState',
          payload: {
            states: ['MODULATION_NOT_ENABLED'],
            speechMode: 'A_LOCAL_STRICT',
            strict: true,
            lineage: 'REHEARSAL',
            autonomy: 'paused',
            reasons: [],
          },
        },
      },
    };
    expect(input.handle(runState)).toBe(true);
    expect(app.getCurrentReadout()).toEqual(NEUTRAL_BODY_READOUT);
  });
});

describe('terminal structural boundaries', () => {
  const directories = [
    'packages/body/src',
    'packages/avatar-runtime/src',
    'apps/stage/src',
    'packages/terminal/src',
  ];
  function* files(dir: string): Generator<string> {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) yield* files(path);
      else if (/\.tsx?$/.test(path) && !/\.test\.tsx?$/.test(path)) yield path;
    }
  }
  it('has no mood/favorability persistence or named hidden state', () => {
    for (const dir of directories)
      for (const path of files(resolve(dir))) {
        const text = readFileSync(path, 'utf8');
        const source = ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true);
        function visit(node: ts.Node): void {
          if (
            (ts.isVariableDeclaration(node) || ts.isPropertyDeclaration(node)) &&
            ts.isIdentifier(node.name)
          ) {
            expect(node.name.text, path).not.toMatch(
              /^(mood|affection|favorability|intimacy|心情|情绪|好感)(state|score|level|value)?$/i,
            );
          }
          ts.forEachChild(node, visit);
        }
        visit(source);
        expect(text, path).not.toMatch(
          /(?:localStorage|sessionStorage)\s*\.\s*setItem\s*\(\s*['"`][^'"`]*(?:mood|affection|favorability|intimacy|心情|情绪|好感)/i,
        );
      }
  });
  it('L3 has no expression-baseline field, input or readout route', () => {
    const face = readFileSync(resolve('packages/avatar-runtime/src/animation/L3-face.ts'), 'utf8');
    expect(face).not.toMatch(/expressionBaseline|setExpressionBaseline|BodyReadout/);
    const compositor = readFileSync(
      resolve('packages/avatar-runtime/src/animation/compositor.ts'),
      'utf8',
    );
    const apply = compositor.slice(
      compositor.indexOf('  applyReadout('),
      compositor.indexOf('  setReadoutMapping('),
    );
    expect(apply).not.toMatch(/this\.l3/);
  });
});
