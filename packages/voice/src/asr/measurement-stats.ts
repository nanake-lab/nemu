/** Linear quantiles; negative latency values are retained and missing sets stay null. */
export function sampleStats(values: readonly number[]) {
  if (values.some((value) => !Number.isFinite(value))) throw new Error('Nonfinite measurement');
  const ordered = [...values].sort((a, b) => a - b);
  const q = (p: number) => {
    if (!ordered.length) return null;
    const index = (ordered.length - 1) * p,
      low = Math.floor(index),
      high = Math.ceil(index);
    return ordered[low]! + (ordered[high]! - ordered[low]!) * (index - low);
  };
  return {
    count: ordered.length,
    min: q(0),
    p05: q(0.05),
    p25: q(0.25),
    median: q(0.5),
    p75: q(0.75),
    p95: q(0.95),
    max: q(1),
  };
}
