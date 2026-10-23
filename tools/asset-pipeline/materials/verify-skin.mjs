import { readFileSync, writeFileSync } from 'node:fs';
import { PNG } from 'pngjs';
const dir = 'reports/P2-06/skin-audit/visual/';
const data = JSON.parse(readFileSync(dir + 'screenshots.json', 'utf8'));
const patches = JSON.parse(
  readFileSync('content/_candidates/P2-06/skin/runtime-patches.json', 'utf8'),
);
const differences = [];
for (const r of data.results.filter((r) => r.config.variant !== 'control')) {
  const c = data.results.find(
    (c) =>
      c.config.variant === 'control' &&
      c.config.light === r.config.light &&
      c.config.background === r.config.background &&
      c.config.framing === r.config.framing,
  );
  const a = PNG.sync.read(readFileSync(dir + r.file)),
    b = PNG.sync.read(readFileSync(dir + c.file));
  let count = 0,
    max = 0;
  for (let i = 45 * a.width * 4; i < a.data.length; i += 4) {
    let changed = false;
    for (let k = 0; k < 3; k++) {
      const d = Math.abs(a.data[i + k] - b.data[i + k]);
      max = Math.max(max, d);
      changed ||= d > 0;
    }
    if (changed) count++;
  }
  if (!count) throw Error('Identical rendered candidate ' + r.file);
  for (const m of r.runtime.materials.filter((m) => m.name.includes('_SKIN'))) {
    if (JSON.stringify(m.shadeColor) !== JSON.stringify(patches[r.config.variant].shadeColorFactor))
      throw Error('Runtime factors mismatch ' + m.name);
  }
  differences.push({ file: r.file, differingPixelsBelowHeader: count, maxChannelDifference: max });
}
writeFileSync(
  'reports/P2-06/skin-audit/visual-verification.json',
  JSON.stringify({ cases: differences.length, allDistinct: true, differences }, null, 2),
);
console.log(differences.slice(0, 3));
