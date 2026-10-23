// Diagnostic frequency bands only: no acceptance threshold is defined here.
import { readFile, writeFile } from 'node:fs/promises';
import { gunzipSync } from 'node:zlib';
const directory = 'reports/P2-05/selected-B';
const raw = JSON.parse(gunzipSync(await readFile(`${directory}/jitter-B.json.gz`)).toString('utf8'));
const result = { reportingBandHz: 15, bandIsAcceptanceThreshold: false, regions: {} };
for (const region of ['hair', 'tail', 'ear', 'skirt']) {
  const peak = raw.regions[region].peak;
  const index = raw.jointNames.indexOf(peak.bone), n = 240;
  const start = Math.max(120, Math.min(peak.frame - n / 2, 3600 - n));
  const power = Array(n / 2 + 1).fill(0);
  for (let axis = 0; axis < 3; axis++) {
    const values = raw.angularVelocityVectors.slice(start, start + n).map(frame => frame[index][axis]);
    const mean = values.reduce((a, b) => a + b, 0) / n;
    const series = values.map((v, j) => (v - mean) * (.5 - .5 * Math.cos(2 * Math.PI * j / (n - 1))));
    for (let k = 1; k <= n / 2; k++) {
      let real = 0, imaginary = 0;
      for (let j = 0; j < n; j++) {
        real += series[j] * Math.cos(2 * Math.PI * k * j / n);
        imaginary += series[j] * Math.sin(2 * Math.PI * k * j / n);
      }
      power[k] += real ** 2 + imaginary ** 2;
    }
  }
  const total = power.reduce((a, b) => a + b, 0);
  let dominant = 1;
  for (let k = 2; k < power.length; k++) if (power[k] > power[dominant]) dominant = k;
  result.regions[region] = {
    bone: peak.bone, peakRadPerSec: peak.speed, peakTimeSec: peak.frame / raw.hz,
    windowSec: [start / raw.hz, (start + n) / raw.hz],
    dominantSignedAngularVelocityHz: dominant * raw.hz / n,
    powerAbove15HzFraction: total ? power.reduce((sum, p, k) => sum + (k * raw.hz / n > 15 ? p : 0), 0) / total : 0,
    vectorsAroundPeak: Array.from({ length: 5 }, (_, j) => peak.frame - 2 + j).map(frame => ({
      time: frame / raw.hz, velocity: raw.angularVelocityVectors[frame][index],
    })),
  };
}
await writeFile(`${directory}/frequency-summary.json`, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result.regions, null, 2));
