// Analyze the retained one-run trace. Never resimulates or edits spring values.
import { readFile, writeFile } from 'node:fs/promises';
import { gunzipSync } from 'node:zlib';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import { motionRange, rms, slope, spectrum } from './settling-metrics.mjs';
const output = 'reports/P2-05/selected-C-final';
const raw = gunzipSync(await readFile(`${output}/jitter-raw.json.gz`));
const inventory = JSON.parse(await readFile(`${output}/measurement.json`, 'utf8'));
assert.equal(createHash('sha256').update(raw).digest('hex'), inventory.rawSha256);
const data = JSON.parse(raw), hz = data.hz;
function speedStats(series, joint) {
  const bySecond = Array.from({ length: 35 }, (_, second) => rms(series.slice(second * hz, (second + 1) * hz).map(f => f[joint])));
  const driven = bySecond.slice(0, 30);
  let longestIncreasingRun = 1, run = 1;
  for (let i = 1; i < 30; i++) { run = driven[i] > driven[i - 1] ? run + 1 : 1; longestIncreasingRun = Math.max(run, longestIncreasingRun); }
  const cycleRms = [0, 10, 20].map(start => rms(series.slice(start * hz, (start + 10) * hz).map(f => f[joint])));
  const secondThirdBinDifference = Math.max(...bySecond.slice(10, 20).map((v, i) => Math.abs(v - bySecond[20 + i])));
  return { rmsBySecond: bySecond, driven10sCycleRms: cycleRms, secondThirdCycleMaximumBinDifference: secondThirdBinDifference,
    drivenRmsSlopePerSecond: slope(driven), longestIncreasingRunSeconds: longestIncreasingRun,
    first5sRms: rms(series.slice(0, 5 * hz).map(f => f[joint])), last5sDrivenRms: rms(series.slice(25 * hz, 30 * hz).map(f => f[joint])),
    final1sRms: bySecond[34], drivenSecondPeakRms: Math.max(...driven),
    monotonicallyIncreasingThroughoutDrive: longestIncreasingRun === 30 };
}
const joints = [];
for (let index = 0; index < data.settings.length; index++) {
  const positions = data.settleTailPositions.map(f => f[index]);
  const directions = data.settleDirections.map(f => f[index]);
  const velocities = data.settleAngularVelocities.map(f => f[index]);
  joints.push({ ...data.settings[index],
    localAngularSpeed: speedStats(data.angularSpeed, index),
    endpointSpeed: speedStats(data.endpointSpeed, index), directionSpeed: speedStats(data.directionSpeed, index),
    settling: { full5s: motionRange(positions, directions), final1s: motionRange(positions.slice(-hz), directions.slice(-hz)),
      endpointSpectrumFull5s: spectrum(positions, hz), endpointSpectrumFinal1s: spectrum(positions.slice(-hz), hz),
      directionSpectrumFull5s: spectrum(directions, hz), localAngularVelocitySpectrumFull5s: spectrum(velocities, hz),
    },
    rotationPeak: data.localRotationPeaks[index],
  });
}
const regions = {};
for (const part of ['hair', 'ear', 'tail', 'skirt', 'other']) {
  const list = joints.filter(j => j.part === part); if (!list.length) continue;
  function maximum(field) { return list.reduce((best, j) => field(j) > field(best) ? j : best); }
  const full = maximum(j => j.settling.full5s.endpointPeakToPeakM);
  const last = maximum(j => j.settling.final1s.endpointPeakToPeakM);
  regions[part] = { count: list.length,
    full5sEndpointPeakToPeakM: full.settling.full5s.endpointPeakToPeakM, full5sPeakBone: full.bone,
    full5sPeakBoneDominantHz: full.settling.endpointSpectrumFull5s.dominantHz,
    final1sEndpointPeakToPeakM: last.settling.final1s.endpointPeakToPeakM, final1sPeakBone: last.bone,
    final1sPeakBoneDominantHz: last.settling.endpointSpectrumFinal1s.dominantHz,
    maxFinal1sDirectionPeakToPeakDeg: Math.max(...list.map(j => j.settling.final1s.directionPeakToPeakDeg)),
    maxFinal1sEndpointSpeedRms: Math.max(...list.map(j => j.endpointSpeed.final1sRms)),
    maxFinal1sLocalAngularSpeedRms: Math.max(...list.map(j => j.localAngularSpeed.final1sRms)),
    maxFinal1sEndpointAbove15HzWindowedRmsM: Math.max(...list.map(j => j.settling.endpointSpectrumFinal1s.above15HzWindowedRms)),
    maxSecondThirdCycleEndpointRmsDifference: Math.max(...list.map(j => j.endpointSpeed.secondThirdCycleMaximumBinDifference)),
    monotonicallyIncreasingEndpointJoints: list.filter(j => j.endpointSpeed.monotonicallyIncreasingThroughoutDrive).map(j => j.bone),
    maximumEndpointIncreasingRunSeconds: Math.max(...list.map(j => j.endpointSpeed.longestIncreasingRunSeconds)),
    maxRotationPeak: maximum(j => j.rotationPeak.maximumFrameDegrees).rotationPeak,
  };
}
await writeFile(`${output}/settling-analysis.json`, JSON.stringify({
  modelSha256: data.modelSha256, rawSha256: inventory.rawSha256, hz, regions, joints,
  method: { speeds: 'Raw local quaternion rad/s, simulated world endpoint m/s, world segment-direction rad/s; no peaks filtered.',
    divergence: 'All 35 one-second RMS bins, 30 s drive slope, first/last 5 s RMS, longest rising run, and final 1 s RMS reported per joint. No new numerical acceptance threshold.',
    residual: 'Exact pairwise endpoint diameter and segment-direction angular diameter for all 600 settle samples and last 120 samples.',
    spectrum: 'Stable anchor subtraction, mean removal, Hann window, non-DC DFT summed xyz power; power RMS normalized by N times sum(window squared). Full 5 s resolution 0.2 Hz, final 1 s resolution 1 Hz. Null only if exactly constant. A dominant bin of a decaying transient is not evidence of sustained vibration. >15 Hz is diagnostic reporting only, not a new acceptance threshold.',
    split: 'Single-frame local-rotation discontinuities retained separately for P3-01 L4 per direct user instruction and PR #67; not converted into a P2-05 failure criterion.',
  },
}, null, 2) + '\n');
const header = 'bone,part,local_drive_rms_slope,endpoint_drive_rms_slope,endpoint_longest_increase_s,endpoint_first5_rms_m_s,endpoint_last5_drive_rms_m_s,endpoint_final1_rms_m_s,settle5_p2p_m,settle5_direction_p2p_deg,settle5_dominant_hz,final1_p2p_m,final1_direction_p2p_deg,final1_dominant_hz,maximum_local_frame_deg';
await writeFile(`${output}/per-joint.csv`, header + '\n' + joints.map(j => [j.bone, j.part, j.localAngularSpeed.drivenRmsSlopePerSecond,
  j.endpointSpeed.drivenRmsSlopePerSecond, j.endpointSpeed.longestIncreasingRunSeconds, j.endpointSpeed.first5sRms,
  j.endpointSpeed.last5sDrivenRms, j.endpointSpeed.final1sRms, j.settling.full5s.endpointPeakToPeakM,
  j.settling.full5s.directionPeakToPeakDeg, j.settling.endpointSpectrumFull5s.dominantHz ?? '',
  j.settling.final1s.endpointPeakToPeakM, j.settling.final1s.directionPeakToPeakDeg,
  j.settling.endpointSpectrumFinal1s.dominantHz ?? '', j.rotationPeak.maximumFrameDegrees].join(',')).join('\n') + '\n');
console.log(JSON.stringify(regions, null, 2));
