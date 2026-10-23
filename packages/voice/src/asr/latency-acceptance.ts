import { sampleStats } from './measurement-stats.js';

/** ADR 0017: medians over all 60 rows; early barge-ins are reported, not excluded. */
export function assessLatency(
  rows: readonly { id: string; vadEndMs: number | null; bargeInMs: number | null }[],
) {
  const ends = rows.flatMap((row) => (row.vadEndMs === null ? [] : [row.vadEndMs]));
  const barges = rows.flatMap((row) => (row.bargeInMs === null ? [] : [row.bargeInMs]));
  const medianVadEndMs = sampleStats(ends).median;
  const medianBargeInMs = sampleStats(barges).median;
  const earlyBargeInIds = rows
    .filter((row) => row.bargeInMs !== null && row.bargeInMs < 0)
    .map((row) => row.id);
  const complete =
    rows.length === 60 &&
    new Set(rows.map((row) => row.id)).size === 60 &&
    ends.length === 60 &&
    barges.length === 60;
  return {
    acceptanceBasis: 'ADR 0017; task P4-01 appended 2026-10-22',
    targets: { medianVadEndMs: 350, medianBargeInMs: 300, earlyBargeIn: 'report only' },
    complete,
    medianVadEndMs,
    medianBargeInMs,
    earlyBargeInCount: earlyBargeInIds.length,
    earlyBargeInRatio: rows.length ? earlyBargeInIds.length / rows.length : null,
    earlyBargeInIds,
    passed:
      complete &&
      ends.every((value) => value >= 0) &&
      medianVadEndMs !== null &&
      medianVadEndMs <= 350 &&
      medianBargeInMs !== null &&
      medianBargeInMs <= 300,
  };
}
