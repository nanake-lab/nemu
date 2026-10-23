// User-requested fixed comparison: original calm, unchanged B, two new calm-based candidates.
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
import { assetManifestEntrySchema } from '../../../../packages/contracts/dist/index.js';
import { springCategory } from './spring-settings.mjs';
const root = resolve('D:/Dev/nemu/content/_candidates/P2-05');
const dataPath = 'reports/P2-05/preview-data.json';
const data = JSON.parse(await readFile(dataPath, 'utf8'));
const values = {
  calmHeavy1: { hair: [1.08, .80, .024], ear: [2.295, .82, .002], tail: [1.60, .82, .050], skirt: [1.30, .82, .055] },
  calmHeavy2: { hair: [1.05, .84, .028], ear: [2.25, .87, .003], tail: [1.50, .87, .065], skirt: [1.25, .87, .070] },
};
data.fourComparison = [
  { id: 'calm', label: '沉稳 · 原始参照', intent: '原始参数', file: 'springs/heavy-review-original-calm.yaml' },
  { id: 'heavyBalanced', label: 'B · 历史对照', intent: '保留数值，不作为最终参数', file: 'springs/heavy-review-balanced.yaml' },
  { id: 'calmHeavy1', label: '新 C · 适度加重', intent: '沿用沉稳刚度，略增重力与阻尼', file: 'springs/calm-heavy-1.yaml' },
  { id: 'calmHeavy2', label: '新 D · 加重快稳', intent: '刚度接近沉稳，重力与阻尼再增加', file: 'springs/calm-heavy-2.yaml' },
];
const manifestPath = resolve(root, 'manifest.json');
const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
for (const [id, parts] of Object.entries(values)) {
  data.presets[id] = structuredClone(data.presets.calm);
  for (const [part, [stiffness, dragForce, gravityPower]] of Object.entries(parts)) {
    Object.assign(data.presets[id][part], { stiffness, dragForce, gravityPower });
  }
  const candidate = data.fourComparison.find(c => c.id === id);
  const cfg = structuredClone(data.build);
  cfg.provenance.notes += ` New calm-based candidate: ${candidate.label}; unapproved.`;
  for (const chain of cfg.springs.chains) {
    const part = chain.bones[0] === '$tail' ? 'tail' : springCategory(chain.bones[0]);
    if (part) Object.assign(chain, data.presets[id][part]);
  }
  const path = resolve(root, candidate.file);
  await writeFile(path, JSON.stringify(cfg, null, 2) + '\n');
  execFileSync('D:/Dev/nemu/tools/asset-pipeline/.venv/Scripts/python.exe', ['-c',
    'import sys; from pathlib import Path; sys.path.insert(0,sys.argv[1]); from config import load_config; load_config(Path(sys.argv[2]))',
    resolve('tools/asset-pipeline'), path], { windowsHide: true, stdio: 'pipe' });
  if (!manifest.some(e => e.path === candidate.file)) manifest.push(assetManifestEntrySchema.parse({
    path: candidate.file, kind: 'other', source: `P2-05 user-directed calm-based spring candidate ${id}`,
    license: 'custom:Nemu private candidate; license pending P7', redistributable: false, author: 'nanake',
    notes: 'Unapproved. K/D/G only; source, topology, colliders and radii unchanged.',
  }));
}
data.selectedSpringPreset = null;
const selectionPath = resolve(root, 'user-selection.json');
const selection = JSON.parse(await readFile(selectionPath, 'utf8'));
if (selection.springPreset) selection.previousSpringPreset = selection.springPreset;
selection.springPreset = null;
selection.springPresetLabel = '待四组诊断与手感评审后选择';
selection.springSelectionScope = 'B withdrawn as final parameter by user on 2026-10-23; historical comparison only';
selection.latestSpringDecisionDate = '2026-10-23';
selection.latestSpringDecisionStatement = 'B 不作为最终参数；在不翻转的组里按手感挑';
await writeFile(selectionPath, JSON.stringify(selection, null, 2) + '\n');
await writeFile(dataPath, JSON.stringify(data, null, 2) + '\n');
await writeFile(resolve(root, 'springs/four-review-parameters.json'), JSON.stringify({
  status: 'candidate-only; no final spring selection',
  candidates: data.fourComparison.map(c => ({ ...c, parameters: data.presets[c.id] })),
}, null, 2) + '\n');
const summaryPath = 'springs/four-review-parameters.json';
if (!manifest.some(e => e.path === summaryPath)) manifest.push(assetManifestEntrySchema.parse({
  path: summaryPath, kind: 'other', source: 'P2-05 four-group comparison parameters; B is historical control',
  license: 'custom:Nemu private candidate; license pending P7', redistributable: false, author: 'nanake',
}));
await writeFile(manifestPath, JSON.stringify(manifest, null, 2) + '\n');
console.log('Two new full configs validated; calm and B preserved. No final spring selection.');
