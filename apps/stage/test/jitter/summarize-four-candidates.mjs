import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
const dataPath = 'reports/P2-05/preview-data.json';
const data = JSON.parse(await readFile(dataPath, 'utf8'));
const groups = [];
for (const candidate of data.fourComparison) {
  const directory = resolve('reports/P2-05/four-review', candidate.id);
  const s = JSON.parse(await readFile(resolve(directory, 'jitter-summary.json'), 'utf8'));
  const raw = await readFile(resolve(directory, 'jitter.json.gz'));
  candidate.maximumHairFrameDegrees = s.maximumHairFrameDegrees;
  groups.push({ ...candidate, parameters: data.presets[candidate.id],
    bone: s.regions.hair.peak.bone, frame: s.regions.hair.peak.frame,
    timeSeconds: s.regions.hair.peak.frame / s.hz,
    modelSha256: s.modelSha256, rawSha256: s.rawSha256,
    compressedRawPath: resolve(directory, 'jitter.json.gz'), compressedBytes: raw.length,
    compressedSha256: createHash('sha256').update(raw).digest('hex'),
    observations: candidate.id === 'calmHeavy1' ? 'No comparable near-180-degree peak; 122.95 degrees remains a large discontinuity, not a stability pass.' : 'Near-180-degree local rotation change observed; not a stability pass.',
  });
}
await writeFile('reports/P2-05/four-review/comparison.json', JSON.stringify({
  hz: 120, drivenSeconds: 30, settlingSeconds: 5, runsPerGroup: 1,
  input: { hipsXM: .24, hipsXHz: 1.3, hipsYM: .12, hipsYHz: 2.1, headRollRad: .18, headHz: 1.7, spineRollRad: .10, spineHz: 1.3 },
  limitations: 'Local quaternion shortest angle, q/-q equivalence handled; not screen displacement or a new acceptance threshold. Identical rebuilt B model with live K/D/G overrides; candidate configs validated but new groups not yet separately exported.',
  groups,
}, null, 2) + '\n');
await writeFile(dataPath, JSON.stringify(data, null, 2) + '\n');
console.log(groups.map(g => `${g.label}: ${g.maximumHairFrameDegrees.toFixed(5)} degrees, ${g.bone}, frame ${g.frame}`).join('\n'));
