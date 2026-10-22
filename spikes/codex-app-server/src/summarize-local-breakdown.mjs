import { readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";
import { summary } from "./remote_metrics.mjs";
import { validateSchema } from "./candidate-validation.mjs";
import { validateSemanticCandidate } from "./probe.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const base = resolve(root, process.argv[2] ?? "reports/P0-08/review-r1/local-breakdown");
const readJson = async path => JSON.parse((await readFile(path, "utf8")).replace(/^\uFEFF/, ""));
const readRows = async path => (await readFile(path, "utf8")).trim().split(/\r?\n/).filter(Boolean).map(JSON.parse);
const sha = bytes => createHash("sha256").update(bytes).digest("hex");
const binding = await readJson(resolve(base, "binding.json"));
const capture = await readJson(resolve(root, "reports/P0-08/review-r1/local-source-archives.json"));
for (const entry of capture.sources) {
  if (sha(gunzipSync(await readFile(resolve(root, "reports/P0-08/review-r1", entry.archive)))) !== binding[entry.bindingField]) throw Error(`Source binding mismatch: ${entry.archive}`);
}
const fixtures = (await readJson(resolve(root, "spikes/codex-app-server/fixtures/irl-cases.json"))).cases.slice(0, 50);
const rows = (await readRows(resolve(base, "samples.jsonl"))).filter(row => !row.warmup);
const completion = await readJson(resolve(base, "completed.json"));
if (rows.length !== 100 || completion.formalPairs !== 50 || binding.model !== "Qwen/Qwen3-4B-Instruct-2507" || binding.revision !== "cdbee75f17c01a7cc42f958dc650907174af0554" || binding.m !== 0 || binding.modulationHookInstalled !== false) throw Error("Incomplete matched local timing batch");
if (sha(await readFile(resolve(root, "spikes/codex-app-server/fixtures/irl-cases.json"))) !== binding.fixturesSha256 || sha(await readFile(resolve(root, "spikes/codex-app-server/semantic-candidate.wire.schema.json"))) !== binding.wireSchemaSha256) throw Error("Local input/schema mismatch");
for (const [index, fixture] of fixtures.entries()) {
  const expected = index % 2 === 0 ? ["unconstrained", "constrained"] : ["constrained", "unconstrained"];
  const pair = rows.slice(index * 2, index * 2 + 2);
  if (pair.some((row, i) => row.caseId !== fixture.id || row.arm !== expected[i] || row.index !== index + 1) || pair[0].inputTokens !== pair[1].inputTokens) throw Error("Matched pair order/input mismatch");
}
for (const row of rows) {
  if (!Array.isArray(row.tokenTimesMs) || row.tokenTimesMs.length !== row.outputTokens || row.tokenTimesMs[0] !== row.firstTokenMs || row.tokenTimesMs.at(-1) !== row.lastTokenMs) throw Error("Generated token trace mismatch");
  if ([row.completedMs, row.firstTokenMs, row.lastTokenMs, row.prefillWallMs, row.prefillCudaElapsedMs].some(value => !Number.isFinite(value) || value <= 0) || row.lastTokenMs < row.firstTokenMs || row.completedMs < row.lastTokenMs) throw Error("Local timing boundary invalid");
  const { outputText, candidate } = await readJson(resolve(base, `outputs/${row.caseId}-${row.arm}.json`));
  let parsed = null;
  try { parsed = JSON.parse(outputText); } catch { /* Preserve malformed outputs as failures. */ }
  if (JSON.stringify(parsed) !== JSON.stringify(candidate)) throw Error("Candidate does not match raw generated text");
  if (sha(Buffer.from(outputText)) !== row.outputSha256 || validateSchema(candidate).valid !== row.schemaValid) throw Error("Local output hash/schema mismatch");
  const fixture = fixtures.find(item => item.id === row.caseId);
  if ((row.schemaValid && validateSemanticCandidate(candidate, fixture)) !== row.sourceBindingValid) throw Error("Local source validation mismatch");
}
const keys = ["completedMs", "firstTokenMs", "prefillWallMs", "prefillCudaElapsedMs", "parserBuildMs", "constraintCallbackWallMs", "outputTokens", "totalTokensPerSecond", "decodeTokensPerSecond"];
const arms = Object.fromEntries(["unconstrained", "constrained"].map(arm => {
  const subset = rows.filter(row => row.arm === arm);
  return [arm, { count: subset.length, schemaValid: subset.filter(row => row.schemaValid).length, sourceBindingValid: subset.filter(row => row.sourceBindingValid).length, tokenLimitCount: subset.filter(row => row.tokenLimitReached).length, ...Object.fromEntries(keys.map(key => [key, summary(subset.map(row => row[key]))])) }];
}));
const pairs = fixtures.map(fixture => {
  const constrained = rows.find(row => row.caseId === fixture.id && row.arm === "constrained");
  const unconstrained = rows.find(row => row.caseId === fixture.id && row.arm === "unconstrained");
  const matchedTokens = Math.min(constrained.outputTokens, unconstrained.outputTokens);
  const constrainedSpan = constrained.tokenTimesMs[matchedTokens - 1] - constrained.firstTokenMs;
  const unconstrainedSpan = unconstrained.tokenTimesMs[matchedTokens - 1] - unconstrained.firstTokenMs;
  return { caseId: fixture.id, completionRatio: constrained.completedMs / unconstrained.completedMs, decodeRateRatio: unconstrained.decodeTokensPerSecond !== null && constrained.decodeTokensPerSecond !== null ? unconstrained.decodeTokensPerSecond / constrained.decodeTokensPerSecond : null, matchedTokens, matchedPrefixDecodeRateRatio: matchedTokens > 1 && unconstrainedSpan > 0 ? constrainedSpan / unconstrainedSpan : null, constraintCallbackShare: constrained.constraintCallbackWallMs / constrained.completedMs };
});
const original = (await readRows(resolve(root, "reports/P0-08/part2/local-irl-20261022-v2/samples.jsonl"))).filter(row => !row.warmup).slice(0, 50);
const result = {
  arms, paired: { completionRatio: summary(pairs.map(pair => pair.completionRatio)), unconstrainedToConstrainedDecodeRateRatio: summary(pairs.map(pair => pair.decodeRateRatio)), matchedPrefixDecodeRateRatio: summary(pairs.map(pair => pair.matchedPrefixDecodeRateRatio)), constraintCallbackShare: summary(pairs.map(pair => pair.constraintCallbackShare)) }, pairs,
  originalFirst50: { outputTokens: summary(original.map(row => row.outputTokens)), totalTokensPerSecond: summary(original.map(row => row.outputTokens * 1000 / row.completedMs)), constrainedOutputHashMatches: original.filter(row => rows.find(item => item.caseId === row.caseId && item.arm === "constrained").outputSha256 === row.outputSha256).length },
  method: "50 fixed fixtures; 17 zh/17 en/16 ja. Alternating arm order, one warmup per arm excluded. Both arms use the same streamer/first-forward hooks. Prefill CUDA elapsed is a stream span, not a sum of kernel-only compute. First token is a generated token ID, not audible speech. Callback wall time includes its own Python/CPU transfers. Outputs can differ in length, content and cap incidence; rates and paired ratios are descriptive, not a causal decomposition of every millisecond. Original 200-case backend rule population remains unchanged."
};
await writeFile(resolve(base, "summary.json"), JSON.stringify(result, null, 2) + "\n");
process.stdout.write(JSON.stringify(result, null, 2) + "\n");
