import { readFile, writeFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";
import { validateSchema } from "./candidate-validation.mjs";
import { validateSemanticCandidate } from "./probe.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const evidence = resolve(root, "reports/P0-08/part2");
const readJson = async (path) => JSON.parse((await readFile(path, "utf8")).replace(/^\uFEFF/, ""));
const readJsonl = async (path) => (await readFile(path, "utf8")).trim().split(/\r?\n/).filter(Boolean).map(JSON.parse);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const fixturesSha256 = hash(await readFile(resolve(root, "spikes/codex-app-server/fixtures/irl-cases.json")));
const fixtures = (await readJson(resolve(root, "spikes/codex-app-server/fixtures/irl-cases.json"))).cases;
const wireSchemaSha256 = hash(await readFile(resolve(root, "spikes/codex-app-server/semantic-candidate.wire.schema.json")));
const sourceArchives = await readJson(resolve(evidence, "protocol/source-archives.json"));
for (const source of sourceArchives.sources) {
  if (hash(gunzipSync(await readFile(resolve(evidence, "protocol", source.originalArchive)))) !== source.sha256Original) throw new Error(`Archived source hash mismatch: ${source.originalArchive}`);
}

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
    firstDeltaObservedCount: rows.some((row) => Object.hasOwn(row, "firstDeltaMs")) ? completed.filter((row) => Number.isFinite(row.firstDeltaMs)).length : null,
    emptyReplyCount: completed.filter((row) => row.outputChars === 0).length,
    firstDeltaP50Ms: percentile(completed.map((row) => row.firstDeltaMs), .5),
    firstDeltaP90Ms: percentile(completed.map((row) => row.firstDeltaMs), .9),
    schemaValidCount: schemaApplicable ? completed.filter((row) => row.schemaValid === true).length : null,
    schemaValidCompletionP50Ms: percentile(completed.filter((row) => row.schemaValid).map((row) => row.completedMs), .5),
    schemaValidCompletionP90Ms: percentile(completed.filter((row) => row.schemaValid).map((row) => row.completedMs), .9),
    schemaValidity: schemaApplicable && rows.length ? completed.filter((row) => row.schemaValid === true).length / rows.length : null,
    sourceBindingValidCount: rows.some((row) => Object.hasOwn(row, "sourceBindingValid")) ? rows.filter((row) => row.sourceBindingValid === true).length : null,
    emptyGraphCount: rows.some((row) => Object.hasOwn(row, "nodeCount")) ? completed.filter((row) => row.nodeCount === 0).length : null,
    tokenLimitCount: rows.some((row) => Object.hasOwn(row, "tokenLimitReached")) ? rows.filter((row) => row.tokenLimitReached === true).length : null,
    reroutes: rows.some((row) => Object.hasOwn(row, "reroute")) ? rows.filter((row) => row.reroute).length : null,
    toolItemCount: rows.some((row) => Object.hasOwn(row, "observedItemTypes")) ? rows.flatMap((row) => row.observedItemTypes ?? []).filter((type) => /Execution|Tool|fileChange|webSearch|imageGeneration/.test(type)).length : null
  };
}

const batch = await readJson(resolve(evidence, "strict-speech/remote-speech-all-summary.json"));
if (batch.calls !== 150 || batch.summaries?.length !== 3) throw new Error("Strict speech batch incomplete");
const speech = {
  ...batch, protocol: "strict-tools-v3", remoteDirectory: "strict-speech", model: "gpt-6-luna", effort: "max",
  segments: batch.summaries.map((summary, index) => {
    const lang = ["zh", "en", "ja"][index];
    if (summary.mode !== `remote-speech-${lang}` || summary.count !== 50) throw new Error("Strict speech language coverage mismatch");
    return { lang, kind: "primary", runId: summary.runId, path: `strict-speech/raw/remote-speech-${lang}-${summary.runId}.jsonl`, sourceSnapshot: "protocol/remote-strict-v3-source.mjs.gz" };
  })
};
const remoteEvidence = resolve(evidence, speech.remoteDirectory);
const requireNoTools = (rows) => {
  if (rows.some((row) => !Array.isArray(row.rawToolCallNames) || row.rawToolCallNames.length !== 0)) throw new Error("Strict remote raw tool-call evidence missing or nonempty");
};
const speechByLanguage = {};
for (const lang of ["zh", "en", "ja"]) {
  const segments = speech.segments.filter((segment) => segment.lang === lang);
  const rows = (await Promise.all(segments.map((segment) => readJsonl(resolve(evidence, segment.path))))).flat();
  requireNoTools(rows);
  const languageFixtures = fixtures.filter((fixture) => fixture.lang === lang);
  if (rows.some((row) => row.lang !== lang || row.caseId !== languageFixtures[row.index - 1]?.id)) throw new Error("Speech fixture binding mismatch");
  if (rows.some((row) => row.model !== "gpt-6-luna" || (row.effort !== "max" && row.turnStatus === "completed"))) throw new Error("Speech protocol mismatch");
  const completed = rows.filter((row) => row.turnStatus === "completed");
  if (completed.length !== 50 || new Set(completed.map((row) => row.index)).size !== 50) throw new Error("Speech completed fixture coverage mismatch");
  const primary = rows.filter((row) => segments.find((segment) => segment.runId === row.runId).kind !== "supplemental");
  speechByLanguage[lang] = { ...describe(rows), primaryAttempts: primary.length, primaryCompleted: primary.filter((row) => row.turnStatus === "completed").length, primaryFailures: primary.filter((row) => row.turnStatus !== "completed").length, supplementalAttempts: rows.length - primary.length };
}
const remoteSummary = await readJson(resolve(remoteEvidence, "remote-irl-summary.json"));
const remoteRows = await readJsonl(resolve(remoteEvidence, `raw/remote-irl-${remoteSummary.runId}.jsonl`));
requireNoTools(remoteRows);
for (const row of remoteRows.filter((row) => row.turnStatus === "completed")) {
  const { candidate } = await readJson(resolve(remoteEvidence, `candidates/${remoteSummary.runId}/${row.caseId}.json`));
  if (validateSchema(candidate).valid !== row.schemaValid) throw new Error(`Schema evidence mismatch: ${row.caseId}`);
  const fixture = fixtures.find((sample) => sample.id === row.caseId);
  if ((row.schemaValid && validateSemanticCandidate(candidate, fixture)) !== row.sourceBindingValid) throw new Error(`Source evidence mismatch: ${row.caseId}`);
}
if (new Set(remoteRows.map((row) => row.caseId)).size !== 200 || remoteRows.some((row, index) => row.caseId !== fixtures[index].id)) throw new Error("Remote fixture coverage mismatch");
if (remoteRows.length !== 200 || remoteRows.some((row) => row.model !== "gpt-6-luna" || row.effort !== "max")) throw new Error("IRL batch protocol mismatch");
let local = null;
if (process.argv[2]) {
  const rows = (await readJsonl(resolve(process.argv[2], "samples.jsonl"))).filter((row) => !row.warmup);
  if (rows.length !== 200 || new Set(rows.map((row) => row.caseId)).size !== 200 || rows.some((row, index) => row.caseId !== fixtures[index].id)) throw new Error("Local IRL fixture coverage incomplete");
  const binding = await readJson(resolve(process.argv[2], "binding.json"));
  if (binding.fixturesSha256 !== fixturesSha256 || binding.wireSchemaSha256 !== wireSchemaSha256) throw new Error("Local fixture/schema binding mismatch");
  if (binding.m !== 0 || binding.hookInstalled !== false || binding.model !== "Qwen/Qwen3-4B-Instruct-2507" || binding.revision !== "cdbee75f17c01a7cc42f958dc650907174af0554") throw new Error("Local carrier binding mismatch");
  if (hash(gunzipSync(await readFile(resolve(evidence, "protocol/local-irl-v2-source.py.gz")))) !== binding.sourceSha256 || hash(gunzipSync(await readFile(resolve(evidence, "protocol/decoder-adapter-v2-source.py.gz")))) !== binding.decoderAdapterSourceSha256) throw new Error("Local source archive mismatch");
  for (const row of rows) {
    const { candidate } = await readJson(resolve(process.argv[2], `candidates/${row.caseId}.json`));
    if (validateSchema(candidate).valid !== row.schemaValid) throw new Error(`Local schema evidence mismatch: ${row.caseId}`);
    const fixture = fixtures.find((sample) => sample.id === row.caseId);
    if ((row.schemaValid && validateSemanticCandidate(candidate, fixture)) !== row.sourceBindingValid) throw new Error(`Local source evidence mismatch: ${row.caseId}`);
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
  model: "gpt-6-luna", effort: "max", percentileMethod: "nearest rank; completed generation responses, including schema-invalid responses; transport failures are censored; all attempts remain in the IRL schema-rate denominator. Schema-valid-only latency is separately reported and does not replace the preregistered population.",
  provenance: { fixturesSha256, wireSchemaSha256, remoteSourceSha256: sourceArchives.sources.find((source) => source.preview === "remote-strict-v3-source.mjs").sha256Original, catalog: "protocol/no-tools-catalog-strict-speech.json", localDirectory: process.argv[2] ?? null },
  speech: speechByLanguage, remoteIrl: remote, localIrl: local,
  protocol: { profile: speech.protocol, remoteDirectory: speech.remoteDirectory, remoteReroutes: remote.reroutes, remoteToolItems: remote.toolItemCount, rawToolCallCount: 0, note: "Supported catalog restriction, startup apps/MCP disabled, empty environments and strict feature flags; adverse probes retained. Every final speech/IRL row has raw tool-call capture with zero observed calls. This is evidence for the measured profile, not a universal sandbox proof." },
  selection: selected, irlDeadlineP90Ms: selected ? (selected === "local" ? local : remote).p90Ms : null,
  modes: { A: { p50Ms: null, p90Ms: null }, B: { p50Ms: null, p90Ms: null }, C: { p50Ms: null, p90Ms: null } },
  modeEstimateMissing: ["ASR and VAD end-to-end samples", "A/M and core transaction timings", "TTS and audio buffer timings", "mode C draft-to-rewrite timing"],
  quota: { speechBefore: speech.rateLimitsBefore, speechAfter100: speech.rateLimitsAfter100, speechAfterFinalSupplement: speech.rateLimitsAfter, speechAfter150PrimarySlots: speech.rateLimitsAfter150PrimarySlots, speechQuotaNote: speech.quotaNote, irlBefore: remoteSummary.rateLimitsBefore, irlAfter100: remoteSummary.rateLimitsAfter100, irlAfter200: remoteSummary.rateLimitsAfter },
  accountQuotaAttribution: "Shared account; snapshots report observed changes and cannot isolate this spike from other account activity."
};
await writeFile(resolve(evidence, "remote-speech-all-summary.json"), `${JSON.stringify(speech, null, 2)}\n`, "utf8");
await writeFile(resolve(evidence, "analysis.json"), `${JSON.stringify(output, null, 2)}\n`, "utf8");
process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
