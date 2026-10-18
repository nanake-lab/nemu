import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { lintVrm } from './validator.js';
import { createFixture } from './test-fixture.js';

async function withFixture(
  name: string,
  mutate: Parameters<typeof createFixture>[1] = {},
  options: { profile?: boolean; manifest?: boolean } = {},
) {
  const directory = await mkdtemp(join(tmpdir(), `nemu-asset-lint-${name}-`));
  const paths = await createFixture(directory, mutate);
  if (options.manifest === false) await rm(paths.manifest, { force: true });
  const result = await lintVrm({
    file: paths.file,
    ...(options.profile === false ? {} : { profile: paths.profile }),
    ...(options.manifest === false ? {} : { manifest: paths.manifest }),
  });
  return { directory, paths, result };
}

test('valid VRM 1.0, profile, spring rig, and manifest have no errors', async (t) => {
  const fixture = await withFixture('valid');
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.equal(fixture.result.errors, 0, JSON.stringify(fixture.result.diagnostics, null, 2));
});

test('rejects a non-VRM 1.0 model', async (t) => {
  const fixture = await withFixture('version', { mutate: (json) => {
    const extensions = json.extensions as Record<string, Record<string, unknown>>;
    extensions.VRMC_vrm!.specVersion = '0.0';
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.1-vrm-1.0' && item.severity === 'error'));
});

test('rejects soles that do not land at y=0', async (t) => {
  const fixture = await withFixture('floor', { mutate: (json) => {
    const accessors = json.accessors as Array<Record<string, unknown>>;
    accessors[0]!.min = [0, 0.05, 0];
    accessors[0]!.max = [0.1, 1.75, 0];
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.1-floor-y-zero'));
});

test('rejects an avatar root that faces away from +Z', async (t) => {
  const fixture = await withFixture('forward', { mutate: (json) => {
    const nodes = json.nodes as Array<Record<string, unknown>>;
    nodes[0]!.rotation = [0, 1, 0, 0];
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.1-forward-positive-z'));
});

test('rejects a profile height outside the 2 cm tolerance', async (t) => {
  const fixture = await withFixture('height', { mutateProfile: (profile) => { profile.heightM = 1.8; } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.1-profile-height'));
});

test('rejects geometry above the triangle hard limit', async (t) => {
  const fixture = await withFixture('triangles', { triangleCount: 100_001 });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.1-triangle-budget'));
});

test('warns when geometry exceeds the 70,000 triangle target', async (t) => {
  const fixture = await withFixture('triangle-warning', { triangleCount: 70_001 });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.1-triangle-target' && item.severity === 'warning'));
});

test('rejects a texture edge over 2048 pixels', async (t) => {
  const fixture = await withFixture('texture-edge', { imageDimensions: [{ width: 4096, height: 1 }] });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.1-texture-edge'));
});

test('rejects RGBA8 plus mipmaps over 256 MiB', async (t) => {
  const fixture = await withFixture('texture-memory', { imageDimensions: Array.from({ length: 13 }, () => ({ width: 2048, height: 2048 })) });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.1-texture-vram'));
});

test('rejects missing required humanoid bones', async (t) => {
  const fixture = await withFixture('bone', { mutate: (json) => {
    const ext = json.extensions as Record<string, Record<string, unknown>>;
    const humanoid = ext.VRMC_vrm!.humanoid as Record<string, unknown>;
    const bones = humanoid.humanBones as Record<string, unknown>;
    delete bones.leftShoulder;
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.2-human-bones'));
});

test('rejects missing required preset expressions', async (t) => {
  const fixture = await withFixture('preset', { mutate: (json) => {
    const ext = json.extensions as Record<string, Record<string, unknown>>;
    const vrm = ext.VRMC_vrm!;
    const expressions = vrm.expressions as Record<string, Record<string, unknown>>;
    const preset = expressions.preset!;
    delete preset.neutral;
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.3-preset-expressions'));
});

test('rejects an expression with a wrong mouth override', async (t) => {
  const fixture = await withFixture('override', { mutate: (json) => {
    const ext = json.extensions as Record<string, Record<string, unknown>>;
    const vrm = ext.VRMC_vrm!;
    const expressions = vrm.expressions as Record<string, Record<string, unknown>>;
    const preset = expressions.preset!;
    preset.happy = { overrideMouth: 'block', overrideBlink: 'blend' };
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.3-happy-override'));
});

test('rejects binary viseme expressions', async (t) => {
  const fixture = await withFixture('viseme', { mutate: (json) => {
    const ext = json.extensions as Record<string, Record<string, unknown>>;
    const expressions = ext.VRMC_vrm!.expressions as Record<string, Record<string, unknown>>;
    const preset = expressions.preset!;
    preset.aa = { isBinary: true };
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.3-viseme-nonbinary'));
});

test('rejects missing Nemu custom expressions', async (t) => {
  const fixture = await withFixture('custom', { mutate: (json) => {
    const ext = json.extensions as Record<string, Record<string, unknown>>;
    const expressions = ext.VRMC_vrm!.expressions as Record<string, Record<string, unknown>>;
    const custom = expressions.custom!;
    delete custom.nemuPout;
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.3-custom-expressions'));
});

test('rejects materials without MToon or unlit', async (t) => {
  const fixture = await withFixture('material', { mutate: (json) => {
    const materials = json.materials as Array<Record<string, unknown>>;
    delete materials[0]!.extensions;
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.1-mtoon-material'));
});

test('rejects ear spring chains with fewer than two joints', async (t) => {
  const fixture = await withFixture('ear-spring', { mutateProfile: (profile) => {
    const rig = profile.rig as Record<string, unknown>;
    rig.earBonesL = ['EarL1'];
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.4-cat-ear-spring-bones'));
});

test('rejects a tail rig shorter than eight joints', async (t) => {
  const fixture = await withFixture('tail-spring', { mutateProfile: (profile) => {
    const rig = profile.rig as Record<string, unknown>;
    rig.tailBones = (rig.tailBones as string[]).slice(0, 7);
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.4-tail-spring-bones'));
});

test('rejects missing required collider groups', async (t) => {
  const fixture = await withFixture('collider', { mutate: (json) => {
    const ext = json.extensions as Record<string, Record<string, unknown>>;
    const springs = ext.VRMC_springBone!.springs as Array<Record<string, unknown>>;
    springs[0]!.colliderGroups = [];
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.4-spring-colliders'));
});

test('rejects a profile that references an absent bone', async (t) => {
  const fixture = await withFixture('profile-bone', { mutateProfile: (profile) => {
    const rig = profile.rig as Record<string, unknown>;
    rig.tailBones = ['NoSuchBone'];
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.6-profile-bones'));
});

test('accepts VRM humanoid semantic names when profile nodes use exporter-specific names', async (t) => {
  const fixture = await withFixture('semantic-profile-bone', { mutate: (json) => {
    const nodes = json.nodes as Array<Record<string, unknown>>;
    const head = nodes.find((node) => node.name === 'head');
    assert.ok(head);
    head.name = 'J_Bip_C_Head';
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(!fixture.result.diagnostics.some((item) => item.rule === 'A1.6-profile-bones'));
});

test('does not require body collider groups for unrelated spring chains', async (t) => {
  const fixture = await withFixture('unrelated-spring', { mutate: (json) => {
    const ext = json.extensions as Record<string, Record<string, unknown>>;
    const springs = ext.VRMC_springBone!.springs as Array<Record<string, unknown>>;
    springs.push({ name: 'Bust', joints: [{ node: 0 }, { node: 1 }], colliderGroups: [] });
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(!fixture.result.diagnostics.some((item) => item.rule === 'A1.4-spring-colliders'));
});

test('checks hair spring count when a profile is supplied', async (t) => {
  const fixture = await withFixture('hair-count-profile', { mutate: (json) => {
    const ext = json.extensions as Record<string, Record<string, unknown>>;
    const springs = ext.VRMC_springBone!.springs as Array<Record<string, unknown>>;
    const index = springs.findIndex((spring) => spring.name === 'hair-5');
    assert.ok(index >= 0);
    springs.splice(index, 1);
  } });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.4-hair-spring-bones' && item.message.includes('5 identifiable')));
});

test('rejects manifest entries without required license data', async (t) => {
  const fixture = await withFixture('manifest', { manifest: [{ path: 'fixture.vrm', kind: 'model', source: '', license: '', redistributable: true }] });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.5-manifest-entry'));
});

test('returns machine-readable results without profile or manifest', async (t) => {
  const fixture = await withFixture('optional', {}, { profile: false, manifest: false });
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  assert.ok(fixture.result.diagnostics.some((item) => item.rule === 'A1.5-manifest-missing' && item.severity === 'warning'));
  assert.equal(fixture.result.errors, 0);
});

test('CLI --json emits parseable JSON for a valid model', async (t) => {
  const fixture = await withFixture('cli-json');
  t.after(() => rm(fixture.directory, { recursive: true, force: true }));
  const cliPath = fileURLToPath(new URL('./cli.js', import.meta.url));
  const child = spawnSync(process.execPath, [cliPath, fixture.paths.file, '--profile', fixture.paths.profile, '--manifest', fixture.paths.manifest, '--json'], { encoding: 'utf8' });
  assert.equal(child.status, 0, child.stderr);
  const result: unknown = JSON.parse(child.stdout);
  assert.equal((result as { errors: number }).errors, 0);
});
