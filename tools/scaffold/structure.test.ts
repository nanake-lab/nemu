import { expect, test } from 'vitest';
import { readFileSync, existsSync } from 'node:fs';
import { Ajv2020 } from 'ajv/dist/2020.js';
const read = (path: string): string => readFileSync(path, 'utf8');
test('kernel has no storage/runtime dependency; runtime points downwards', () => {
  expect(read('core/crates/core-kernel/Cargo.toml')).not.toMatch(/core-(store|runtime)/);
  const runtime = read('core/crates/core-runtime/Cargo.toml');
  for (const name of ['store', 'kernel', 'policy', 'types'])
    expect(runtime).toContain(`core-${name}.workspace = true`);
});
test('model lock validates and rejects a malformed Codex version', () => {
  const validate = new Ajv2020().compile(JSON.parse(read('config/models.lock.schema.json')));
  expect(validate(JSON.parse(read('config/models.lock.json')))).toBe(true);
  expect(validate({ codex: { version: 'floating' } })).toBe(false);
});
test('content template keeps empty manifests and all required directories', () => {
  expect(JSON.parse(read('content-template/nemu/manifest.json'))).toEqual([]);
  expect(JSON.parse(read('content-template/nemu/motions/manifest.json'))).toEqual([]);
  for (const path of [
    'bible',
    'source',
    'motions',
    'voice/ref/zh',
    'voice/ref/en',
    'voice/ref/ja',
    'lipsync',
    'core/genesis',
    'core/protocol',
    'core/exemplars',
  ])
    expect(existsSync(`content-template/nemu/${path}`)).toBe(true);
});
test('unconfigured profiles accept only the empty placeholder', () => {
  for (const name of ['structural_fixture', 'deployment']) {
    const validate = new Ajv2020().compile(
      JSON.parse(read(`profiles/${name}/profile.schema.json`)),
    );
    expect(validate(JSON.parse(read(`profiles/${name}/profile.json`)))).toBe(true);
    expect(validate({ invented: true })).toBe(false);
  }
});
