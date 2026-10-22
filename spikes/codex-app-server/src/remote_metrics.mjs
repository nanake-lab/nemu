export function quantile(values, probability) {
  const sorted = values.filter(Number.isFinite).sort((a, b) => a - b);
  return sorted.length ? sorted[Math.ceil(sorted.length * probability) - 1] : null;
}

export function summary(values) {
  const finite = values.filter(Number.isFinite);
  return { count: finite.length, p50: quantile(finite, .5), p90: quantile(finite, .9), mean: finite.length ? finite.reduce((a, b) => a + b, 0) / finite.length : null };
}

export function remoteMetrics(rows) {
  const completed = rows.filter(row => row.turnStatus === "completed");
  const phases = completed.filter(row => Number.isFinite(row.firstDeltaMs) && Number.isFinite(row.completedMs));
  if (phases.some(row => row.firstDeltaMs < 0 || row.completedMs < row.firstDeltaMs)) throw new Error("Invalid per-response phase order");
  const usageRows = rows.filter(row => row.usage !== null && typeof row.usage === "object");
  const fields = ["totalTokens", "inputTokens", "cachedInputTokens", "outputTokens", "reasoningOutputTokens"];
  const usage = Object.fromEntries(fields.map(field => {
    const values = usageRows.map(row => row.usage[field]).filter(Number.isFinite);
    return [field, { reportedRows: values.length, totalReported: values.reduce((a, b) => a + b, 0), ...summary(values) }];
  }));
  return {
    attempts: rows.length, completed: completed.length, failed: rows.length - completed.length,
    completionMs: summary(completed.map(row => row.completedMs)),
    beforeFirstDeltaMs: summary(phases.map(row => row.firstDeltaMs)),
    afterFirstDeltaMs: summary(phases.map(row => row.completedMs - row.firstDeltaMs)),
    phaseRows: phases.length, usageRows: usageRows.length, usage,
    schemaValid: rows.some(row => "schemaValid" in row) ? rows.filter(row => row.schemaValid).length : null,
    sourceBindingValid: rows.some(row => "sourceBindingValid" in row) ? rows.filter(row => row.sourceBindingValid).length : null,
    rawToolCallCount: rows.flatMap(row => row.rawToolCallNames ?? []).length,
    rawCaptureRows: rows.filter(row => Array.isArray(row.rawToolCallNames)).length,
    rawCaptureUnknownRows: rows.filter(row => !Array.isArray(row.rawToolCallNames)).length,
    modelReroutes: rows.filter(row => row.reroute).length
  };
}
