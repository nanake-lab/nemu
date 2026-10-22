import { readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";
import { remoteMetrics } from "./remote_metrics.mjs";
import { validateSchema } from "./candidate-validation.mjs";
import { validateSemanticCandidate } from "./probe.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const evidence = resolve(root, "reports/P0-08");
const readJson = async path => JSON.parse((await readFile(path, "utf8")).replace(/^\uFEFF/, ""));
const readRows = async path => (await readFile(path, "utf8")).trim().split(/\r?\n/).filter(Boolean).map(JSON.parse);
const hash = bytes => createHash("sha256").update(bytes).digest("hex");
const fixtures = (await readJson(resolve(root, "spikes/codex-app-server/fixtures/irl-cases.json"))).cases;
const plan = await readJson(resolve(evidence, "review-r1/plan.json"));
if (hash(gunzipSync(await readFile(resolve(evidence, "review-r1/remote-low-probe.mjs.gz")))) !== plan.sourceSha256) throw Error("Low source archive mismatch");
if (hash(await readFile(resolve(root, "spikes/codex-app-server/fixtures/irl-cases.json"))) !== plan.fixtureSha256 || hash(await readFile(resolve(root, "spikes/codex-app-server/semantic-candidate.wire.schema.json"))) !== plan.wireSchemaSha256) throw Error("Fixture/schema changed");

async function analyzeGroup(directory, effort) {
  const base = resolve(evidence, "part2", directory);
  const all = await readJson(resolve(base, "remote-speech-all-summary.json"));
  if (all.calls !== 150 || all.summaries.length !== 3) throw Error("Speech batch incomplete");
  const check = rows => {
    if (rows.some(row => row.model !== "gpt-6-luna" || row.effort !== effort || row.reroute || !Array.isArray(row.rawToolCallNames) || row.rawToolCallNames.length)) throw Error("Remote protocol mismatch");
    if (rows.some(row => row.turnStatus !== "completed" || !Number.isFinite(row.firstDeltaMs) || !Number.isFinite(row.completedMs))) throw Error("Remote completed sample missing");
  };
  const speech = {};
  const speechRows = [];
  for (const [index, lang] of ["zh", "en", "ja"].entries()) {
    const source = all.summaries[index];
    const rows = await readRows(resolve(base, `raw/remote-speech-${lang}-${source.runId}.jsonl`));
    check(rows);
    const expected = fixtures.filter(item => item.lang === lang).slice(0, 50);
    if (rows.length !== 50 || rows.some((row, i) => row.caseId !== expected[i].id || row.index !== i + 1)) throw Error("Speech fixture coverage mismatch");
    speech[lang] = { ...remoteMetrics(rows), runId: source.runId };
    speechRows.push(...rows);
  }
  const source = await readJson(resolve(base, "remote-irl-summary.json"));
  const rows = await readRows(resolve(base, `raw/remote-irl-${source.runId}.jsonl`));
  check(rows);
  if (rows.length !== 200 || rows.some((row, i) => row.caseId !== fixtures[i].id || row.index !== i + 1)) throw Error("IRL fixture coverage mismatch");
  for (const [i, row] of rows.entries()) {
    const { candidate } = await readJson(resolve(base, `candidates/${source.runId}/${row.caseId}.json`));
    const schemaValid = validateSchema(candidate).valid;
    if (schemaValid !== row.schemaValid || (schemaValid && validateSemanticCandidate(candidate, fixtures[i])) !== row.sourceBindingValid) throw Error("IRL candidate validation mismatch");
  }
  return { model: "gpt-6-luna", effort, directory, speech, speechAll: remoteMetrics(speechRows), irl: { ...remoteMetrics(rows), runId: source.runId }, quota: { speechBefore: all.rateLimitsBefore, speechAfter100: all.rateLimitsAfter100, speechAfter150: all.rateLimitsAfter, irlBefore: source.rateLimitsBefore, irlAfter100: source.rateLimitsAfter100, irlAfter200: source.rateLimitsAfter } };
}

const max = await analyzeGroup("strict-speech", "max");
const low = await analyzeGroup("low-effort", "low");
const local = (await readJson(resolve(evidence, "part2/analysis.json"))).localIrl;
function ruleOutput(remote) {
  const qualifies = (p50, rate) => p50 <= 400 && rate >= .95;
  const remoteRate = remote.irl.schemaValid / remote.irl.attempts;
  const backend = qualifies(local.p50Ms, local.schemaValidity) ? "local" : qualifies(remote.irl.completionMs.p50, remoteRate) ? "remote" : local.p50Ms <= remote.irl.completionMs.p50 ? "local" : "remote";
  return { backend, deadlineP90Ms: backend === "local" ? local.p90Ms : remote.irl.completionMs.p90, deployable: false, localQualifies: qualifies(local.p50Ms, local.schemaValidity), remoteQualifies: qualifies(remote.irl.completionMs.p50, remoteRate) };
}
const result = {
  groups: { userSpecifiedMaxControl: max, cardLowEffort: low },
  rules: { max: ruleOutput(max), low: ruleOutput(low) },
  deployment: { backend: null, deadlineP90Ms: null, reason: "Rule outputs are not deployment values. P0-09 must decide architecture and latency targets." },
  method: "Nearest rank; phase differences calculated per sample. Before first delta combines client/network/server queue/prefill/reasoning; after first delta is client-observed stream tail, not isolated GPU decoding. Token fields are reported API counters, not billing or exclusive quota attribution. Max was explicitly requested by user and remains a control."
};
await writeFile(resolve(evidence, "review-r1/remote-comparison.json"), JSON.stringify(result, null, 2) + "\n");
process.stdout.write(JSON.stringify(result, null, 2) + "\n");
