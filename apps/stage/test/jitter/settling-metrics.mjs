// Offline signal measurements, not acceptance thresholds or a solver modification.
export function rms(values) { return Math.sqrt(values.reduce((s, v) => s + v * v, 0) / values.length); }
export function slope(values) {
  const meanX = (values.length - 1) / 2, meanY = values.reduce((s, v) => s + v, 0) / values.length;
  let numerator = 0, denominator = 0;
  for (const [i, v] of values.entries()) { numerator += (i - meanX) * (v - meanY); denominator += (i - meanX) ** 2; }
  return denominator ? numerator / denominator : 0;
}
export function motionRange(points, directions) {
  let distanceSquared = 0, angle = 0;
  for (let i = 0; i < points.length; i++) for (let j = i + 1; j < points.length; j++) {
    const a = points[i], b = points[j];
    distanceSquared = Math.max(distanceSquared, (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2 + (a[2] - b[2]) ** 2);
    const u = directions[i], v = directions[j];
    angle = Math.max(angle, Math.atan2(Math.hypot(u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]),
      u[0] * v[0] + u[1] * v[1] + u[2] * v[2]));
  }
  return { endpointPeakToPeakM: Math.sqrt(distanceSquared), directionPeakToPeakDeg: angle * 180 / Math.PI };
}
export function spectrum(vectors, hz) {
  const n = vectors.length;
  const mean = [0, 1, 2].map(axis => vectors.reduce((s, v) => s + v[axis], 0) / n);
  // Subtract a stable anchor first: an exactly constant input must have zero
  // spectrum, regardless of floating-point summation of its absolute coordinate.
  const anchor = vectors[0];
  const relative = vectors.map(v => v.map((x, axis) => x - anchor[axis]));
  const relativeMean = [0, 1, 2].map(axis => relative.reduce((s, v) => s + v[axis], 0) / n);
  const centered = relative.map(v => v.map((x, axis) => x - relativeMean[axis]));
  const centeredRms = Math.sqrt(centered.reduce((s, v) => s + v.reduce((ss, x) => ss + x * x, 0), 0) / n);
  if (centeredRms === 0) return { dominantHz: null, centeredRms: 0, windowedRms: 0, above15HzWindowedRms: 0, above15HzPowerFraction: 0, mean };
  const windowed = centered.map((v, i) => v.map(x => x * (.5 - .5 * Math.cos(2 * Math.PI * i / (n - 1)))));
  let total = 0, high = 0, peakPower = -1, peakBin = 0;
  for (let k = 1; k <= Math.floor(n / 2); k++) {
    const real = [0, 0, 0], imag = [0, 0, 0];
    for (let i = 0; i < n; i++) {
      const phase = 2 * Math.PI * k * i / n, c = Math.cos(phase), s = Math.sin(phase);
      for (let axis = 0; axis < 3; axis++) { real[axis] += windowed[i][axis] * c; imag[axis] -= windowed[i][axis] * s; }
    }
    const power = real.reduce((s, v, axis) => s + v * v + imag[axis] ** 2, 0) * (k === n / 2 ? 1 : 2);
    total += power; if (k * hz / n > 15) high += power;
    if (power > peakPower) { peakPower = power; peakBin = k; }
  }
  const windowPower = Array.from({ length: n }, (_, i) => (.5 - .5 * Math.cos(2 * Math.PI * i / (n - 1))) ** 2).reduce((s, v) => s + v, 0);
  return { dominantHz: peakBin * hz / n, centeredRms,
    windowedRms: Math.sqrt(total / (n * windowPower)), above15HzWindowedRms: Math.sqrt(high / (n * windowPower)),
    above15HzPowerFraction: high / total, mean };
}
