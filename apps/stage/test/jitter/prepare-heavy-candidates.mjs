import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
import { assetManifestEntrySchema } from '../../../../packages/contracts/dist/index.js';
import { springCategory } from './spring-settings.mjs';

const candidateRoot = resolve('D:/Dev/nemu/content/_candidates/P2-05');
const dataPath = resolve('reports/P2-05/preview-data.json');
const data = JSON.parse(await readFile(dataPath, 'utf8'));
const values = {
  heavyBalanced: { hair: [.95, .84, .045], ear: [2.1, .88, .006], tail: [1.2, .83, .09], skirt: [.95, .86, .085] },
  heavyDamped: { hair: [.85, .89, .065], ear: [2.0, .92, .01], tail: [.95, .88, .14], skirt: [.8, .90, .12] },
};
data.heavyComparison = [
  { id: 'calm', label: 'A 原沉稳', intent: '原始参照', file: 'springs/heavy-review-original-calm.yaml' },
  { id: 'heavyBalanced', label: 'B 加重·留惯性', intent: '适度降低刚度，增加重力与阻尼', file: 'springs/heavy-review-balanced.yaml' },
  { id: 'heavyDamped', label: 'C 加重·快收稳', intent: '更高重力与阻尼，更低刚度', file: 'springs/heavy-review-damped.yaml' },
];
for (const [id, parts] of Object.entries(values)) {
  data.presets[id] = structuredClone(data.presets.calm);
  for (const [part, [stiffness, dragForce, gravityPower]] of Object.entries(parts)) {
    Object.assign(data.presets[id][part], { stiffness, dragForce, gravityPower });
  }
}
const manifestPath = resolve(candidateRoot, 'manifest.json');
const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
for (const candidate of data.heavyComparison) {
  const cfg = structuredClone(data.build);
  cfg.provenance.notes += ` Heavy spring review: ${candidate.label}; not approved.`;
  for (const chain of cfg.springs.chains) {
    const part = chain.bones[0] === '$tail' ? 'tail' : springCategory(chain.bones[0]);
    if (!part) continue;
    Object.assign(chain, data.presets[candidate.id][part]);
  }
  const path = resolve(candidateRoot, candidate.file);
  await mkdir(resolve(candidateRoot, 'springs'), { recursive: true });
  await writeFile(path, JSON.stringify(cfg, null, 2) + '\n');
  const code = 'import sys; from pathlib import Path; sys.path.insert(0,sys.argv[1]); from config import load_config; load_config(Path(sys.argv[2]))';
  execFileSync('D:/Dev/nemu/tools/asset-pipeline/.venv/Scripts/python.exe', ['-c', code, resolve('tools/asset-pipeline'), path], { windowsHide: true, stdio: 'pipe' });
  if (!manifest.some(entry => entry.path === candidate.file)) {
    manifest.push(assetManifestEntrySchema.parse({ path: candidate.file, kind: 'other',
      source: `P2-05 user-directed heavier spring candidate; ${candidate.label}; original short-thick model unchanged`,
      license: 'custom:Nemu private candidate; license pending P7', redistributable: false, author: 'nanake',
      notes: 'Unapproved candidate. Gravity/drag/stiffness only; no explicit mass or artificial delay.' }));
  }
}
await writeFile(manifestPath, JSON.stringify(manifest, null, 2) + '\n');
await writeFile(dataPath, JSON.stringify(data, null, 2) + '\n');
await writeFile(resolve(candidateRoot, 'springs/heavy-review-parameters.json'), JSON.stringify({
  status: 'candidate-only', tail: 'A short-thick selected', base: 'calm selected as starting point; heavier variants not selected',
  candidates: data.heavyComparison.map(item => ({ ...item, parameters: data.presets[item.id] })),
}, null, 2) + '\n');
const summaryPath = 'springs/heavy-review-parameters.json';
if (!manifest.some(entry => entry.path === summaryPath)) manifest.push(assetManifestEntrySchema.parse({
  path: summaryPath, kind: 'other', source: 'P2-05 user-directed heavier spring comparison parameters',
  license: 'custom:Nemu private candidate; license pending P7', redistributable: false, author: 'nanake',
}));
await writeFile(manifestPath, JSON.stringify(manifest, null, 2) + '\n');
console.log('Validated three full build configs in content/_candidates/P2-05/springs.');
