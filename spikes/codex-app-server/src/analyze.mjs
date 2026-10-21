import { readFile, writeFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { validateSchema } from "./candidate-validation.mjs";
import { validateSemanticCandidate } from "./probe.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const evidence = resolve(root, "reports/P0-08/part2");
const readJson = async (path) => JSON.parse((await readFile(path, "utf8")).replace(/^\uFEFF/, ""));
const readJsonl = async (path) => (await readFile(path, "utf8")).trim().split(/\r?\n/).filter(Boolean).map(JSON.parse);

function percentile(values, proportion) {
  const sorted = values.filter(Number.isFinite).sort((a, b) => a - b);
  return sorted.length ? sorted[Math.ceil(proportion * sorted.length) - 1] : null;
}

function describe(rows) {
  const completed = rows.filter((row) => row.turnStatus === "completed" || (row.warmup === false && row.completedMs !== undefined));
  const values = completed.map((row) => row.completedMs);
  const schemaApplicable = rows.some((row) => Object.hasOwn(row, "schemaValid"));
  return {
    attempts: rows.length,
    completed: completed.length,
    failed: rows.length - completed.length,
    p50Ms: percentile(values, .5),
    p90Ms: percentile(values, .9),
    firstDeltaObservedCount: completed.filter((row) => Number.isFinite(row.firstDeltaMs)).length,
    emptyReplyCount: completed.filter((row) => row.outputChars === 0).length,
    firstDeltaP50Ms: percentile(completed.map((row) => row.firstDeltaMs), .5),
    firstDeltaP90Ms: percentile(completed.map((row) => row.firstDeltaMs), .9),
    schemaValidCount: schemaApplicable ? completed.filter((row) => row.schemaValid === true).length : null,
    schemaValidity: schemaApplicable && rows.length ? completed.filter((row) => row.schemaValid === true).length / rows.length : null,
    sourceBindingValidCount: rows.filter((row) => row.sourceBindingValid === true).length,
    emptyGraphCount: completed.filter((row) => row.nodeCount === 0).length,
    tokenLimitCount: rows.filter((row) => row.tokenLimitReached === true).length,
    reroutes: rows.filter((row) => row.reroute).length,
    toolItemCount: rows.flatMap((row) => row.observedItemTypes ?? []).filter((type) => /Execution|Tool|fileChange|webSearch|imageGeneration/.test(type)).length
  };
}

const speech = await readJson(resolve(evidence, "remote-speech-all-summary.json"));
const speechByLanguage = {};
for (const lang of ["zh", "en", "ja"]) {
  const segments = speech.segments.filter((segment) => segment.lang === lang);
  const rows = (await Promise.all(segments.map((segment) => readJsonl(resolve(evidence, segment.path))))).flat();
  if (rows.some((row) => row.model !== "gpt-6-luna" || (row.effort !== "max" && row.turnStatus === "completed"))) throw new Error("Speech protocol mismatch");
  const completed = rows.filter((row) => row.turnStatus === "completed");
  if (completed.length !== 50 || new Set(completed.map((row) => row.index)).size !== 50) throw new Error("Speech completed fixture coverage mismatch");
  const primary = rows.filter((row) => segments.find((segment) => segment.runId === row.runId).kind !== "supplemental");
  speechByLanguage[lang] = { ...describe(rows), primaryAttempts: primary.length, primaryCompleted: primary.filter((row) => row.turnStatus === "completed").length, primaryFailures: primary.filter((row) => row.turnStatus !== "completed").length, supplementalAttempts: rows.length - primary.length };
}
const remoteSummary = await readJson(resolve(evidence, "remote-irl-summary.json"));
const remoteRows = await readJsonl(resolve(evidence, `raw/remote-irl-${remoteSummary.runId}.jsonl`));
const fixtures = (await readJson(resolve(root, "spikes/codex-app-server/fixtures/irl-cases.json"))).cases;
for (const row of remoteRows.filter((row) => row.turnStatus === "completed")) {
  const { candidate } = await readJson(resolve(evidence, `candidates/${remoteSummary.runId}/${row.caseId}.json`));
  if (validateSchema(candidate).valid !== row.schemaValid) throw new Error(`Schema evidence mismatch: ${row.caseId}`);
  const fixture = fixtures.find((sample) => sample.id === row.caseId);
  if ((row.schemaValid && validateSemanticCandidate(candidate, fixture)) !== row.sourceBindingValid) throw new Error(`Source evidence mismatch: ${row.caseId}`);
}
if (remoteRows.length !== 200 || remoteRows.some((row) => row.model !== "gpt-6-luna" || row.effort !== "max")) throw new Error("IRL batch protocol mismatch");
let local = null;
if (process.argv[2]) {
  const rows = (await readJsonl(resolve(process.argv[2], "samples.jsonl"))).filter((row) => !row.warmup);
  if (rows.length !== 200) throw new Error("Local IRL batch incomplete");
  for (const row of rows) {
    const { candidate } = await readJson(resolve(process.argv[2], `candidates/${row.caseId}.json`));
    if (validateSchema(candidate).valid !== row.schemaValid) throw new Error(`Local schema evidence mismatch: ${row.caseId}`);
  }
  local = describe(rows);
}
const remote = describe(remoteRows);
const qualifies = (backend) => backend !== null && backend.attempts === 200 && backend.p50Ms !== null && backend.p50Ms <= 400 && backend.schemaValidity >= .95;
let selected = null;
if (local && local.p50Ms !== null && remote.p50Ms !== null) {
  if (qualifies(local)) selected = "local";
  else if (qualifies(remote)) selected = "remote";
  else selected = local.p50Ms <= remote.p50Ms ? "local" : "remote";
}
const output = {
  model: "gpt-6-luna", effort: "max", percentileMethod: "nearest rank; completed samples; failed attempts remain in schema-rate denominator",
  speech: speechByLanguage, remoteIrl: remote, localIrl: local,
  protocol: { remoteReroutes: remote.reroutes, remoteToolItems: remote.toolItemCount, note: "Configured capability flags and observed item types are recorded; zero tool calls does not independently enumerate every model-visible built-in tool." },
  selection: selected, irlDeadlineP90Ms: selected ? (selected === "local" ? local : remote).p90Ms : null,
  modes: { A: { p50Ms: null, p90Ms: null }, B: { p50Ms: null, p90Ms: null }, C: { p50Ms: null, p90Ms: null } },
  modeEstimateMissing: ["ASR and VAD end-to-end samples", "A/M and core transaction timings", "TTS and audio buffer timings", "mode C draft-to-rewrite timing"],
  quota: { speechBefore: speech.rateLimitsBefore, speechAfter100: speech.rateLimitsAfter100, speechAfterFinalSupplement: speech.rateLimitsAfter, speechAfter150PrimarySlots: speech.rateLimitsAfter150PrimarySlots, speechQuotaNote: speech.quotaNote, irlBefore: remoteSummary.rateLimitsBefore, irlAfter100: remoteSummary.rateLimitsAfter100, irlAfter200: remoteSummary.rateLimitsAfter },
  accountQuotaAttribution: "Shared account; snapshots report observed changes and cannot isolate this spike from other account activity."
};
await writeFile(resolve(evidence, "analysis.json"), `${JSON.stringify(output, null, 2)}\n`, "utf8");
process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
