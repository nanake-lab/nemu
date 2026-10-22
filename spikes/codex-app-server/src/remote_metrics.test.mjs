import test from "node:test";
import assert from "node:assert/strict";
import { remoteMetrics } from "./remote_metrics.mjs";

test("phase quantiles use per-response differences, not marginal subtraction", () => {
  const values = [[1, 101], [100, 102], [101, 103]];
  const result = remoteMetrics(values.map(([firstDeltaMs, completedMs]) => ({ turnStatus: "completed", firstDeltaMs, completedMs })));
  assert.equal(result.afterFirstDeltaMs.p50, 2);
  assert.equal(result.afterFirstDeltaMs.p90, 100);
  assert.equal(result.completionMs.p90 - result.beforeFirstDeltaMs.p90, 2);
});

test("missing usage and transport failures remain visible", () => {
  const result = remoteMetrics([{ turnStatus: "completed", firstDeltaMs: 1, completedMs: 2, usage: { outputTokens: 5 } }, { turnStatus: "failed" }]);
  assert.equal(result.attempts, 2);
  assert.equal(result.failed, 1);
  assert.equal(result.usage.outputTokens.reportedRows, 1);
  assert.equal(result.usage.outputTokens.totalReported, 5);
  assert.equal(result.usage.reasoningOutputTokens.count, 0);
  assert.equal(result.usage.reasoningOutputTokens.p50, null);
});
