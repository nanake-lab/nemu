// The profile shape and vocabulary are owned exclusively by @nemu/contracts.
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = resolve(import.meta.dirname, '../..');
const build = spawnSync(process.execPath, [resolve(root, 'node_modules/typescript/bin/tsc'), '-p', resolve(root, 'packages/contracts/tsconfig.json')], { encoding: 'utf8' });
if (build.status !== 0) {
  process.stderr.write(build.stdout + build.stderr);
  process.exit(build.status ?? 1);
}
const { nemuProfileSchema } = await import(pathToFileURL(resolve(root, 'packages/contracts/dist/index.js')).href);
const profile = JSON.parse(readFileSync(0, 'utf8'));
const parsed = nemuProfileSchema.safeParse({ ...profile, version: 2, model: 'nemu.vrm', heightM: 0, headTopOffsetM: 0, rig: { ...profile.rig, tailBones: [] } });
if (!parsed.success) {
  console.error(parsed.error.issues.map((e) => `profile.${e.path.join('.')}: ${e.message}`).join('\n'));
  process.exitCode = 1;
}
