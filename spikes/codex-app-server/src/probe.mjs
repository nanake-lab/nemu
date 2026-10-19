#!/usr/bin/env node
import { spawn, execFileSync } from "node:child_process";
import { createInterface } from "node:readline";
import { appendFile, mkdir, readFile, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { randomUUID } from "node:crypto";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const reportRoot = resolve(repoRoot, "reports/P0-08");
const schemaPath = resolve(repoRoot, "spikes/codex-app-server/semantic-candidate.schema.json");
const fixturesPath = resolve(repoRoot, "spikes/codex-app-server/fixtures/irl-cases.json");
const sampleProtocol = [
  "You are a small measurement assistant. Reply only to the supplied test input.",
  "Use the same language as the input. Do not call tools, browse, access files, or use external context.",
  "This is synthetic test data. Do not infer facts about the account owner."
].join(" ");

function parseArgs(argv) {
  const [mode, ...rest] = argv;
  const options = {};
  for (let i = 0; i < rest.length; i += 1) {
    const token = rest[i];
    if (!token.startsWith("--")) throw new Error(`unexpected argument: ${token}`);
    const key = token.slice(2);
    const next = rest[i + 1];
    if (next === undefined || next.startsWith("--")) options[key] = true;
    else { options[key] = next; i += 1; }
  }
  return { mode, options };
}

function required(options, name) {
  const value = options[name];
  if (typeof value !== "string" || value.length === 0) throw new Error(`missing --${name}`);
  return value;
}

function safeInteger(value, fallback, min, max) {
  const parsed = value === undefined ? fallback : Number(value);
  if (!Number.isInteger(parsed) || parsed < min || parsed > max) throw new Error(`expected integer in ${min}..${max}`);
  return parsed;
}

function percentile(values, p) {
  const sorted = values.filter(Number.isFinite).sort((a, b) => a - b);
  if (sorted.length === 0) return null;
  return sorted[Math.max(0, Math.ceil(p * sorted.length) - 1)];
}

function tokenSummary(value) {
  if (!value || typeof value !== "object") return null;
  const total = value.last ?? value.total;
  if (!total || typeof total !== "object") return null;
  return {
    totalTokens: total.totalTokens ?? null,
    inputTokens: total.inputTokens ?? null,
    cachedInputTokens: total.cachedInputTokens ?? null,
    outputTokens: total.outputTokens ?? null,
    reasoningOutputTokens: total.reasoningOutputTokens ?? null
  };
}

function rateLimitSummary(snapshot) {
  if (!snapshot || typeof snapshot !== "object") return null;
  const windowSummary = (window) => window ? {
    usedPercent: window.usedPercent ?? null,
    windowDurationMins: window.windowDurationMins ?? null,
    resetsAt: window.resetsAt ?? null
  } : null;
  return {
    limitName: snapshot.limitName ?? null,
    primary: windowSummary(snapshot.primary),
    secondary: windowSummary(snapshot.secondary),
    credits: snapshot.credits ? {
      hasCredits: snapshot.credits.hasCredits ?? null,
      unlimited: snapshot.credits.unlimited ?? null
    } : null
  };
}

class AppServer {
  constructor(codeHome) {
    if (!codeHome) throw new Error("provide an isolated CODEX_HOME with --code-home");
    this.pending = new Map();
    this.nextId = 0;
    this.turns = new Map();
    this.messages = new Map();
    this.firstDeltas = new Map();
    this.tokenUsage = new Map();
    this.approvals = [];
    this.rateLimitUpdates = [];
    this.tempWorkdirs = new Set();
    this.nonJsonStdoutLines = 0;
    this.child = spawn("codex", ["app-server", "--stdio"], {
      cwd: repoRoot,
      env: { ...process.env, CODEX_HOME: codeHome },
      stdio: ["pipe", "pipe", "pipe"]
    });
    const stdout = createInterface({ input: this.child.stdout });
    stdout.on("line", (line) => this.#handleLine(line));
    this.child.stderr.on("data", () => {}); // Never persist raw app-server stderr.
    this.child.on("error", () => this.#failPending("spawn_error"));
    this.child.on("exit", () => this.#failPending("server_exited"));
  }

  #send(message) {
    if (!this.child.stdin.writable) throw new Error("server_stdin_closed");
    this.child.stdin.write(`${JSON.stringify(message)}\n`);
  }

  #failPending(reason) {
    for (const entry of this.pending.values()) {
      clearTimeout(entry.timer);
      entry.reject(new Error(reason));
    }
    this.pending.clear();
  }

  #handleLine(line) {
    let message;
    try { message = JSON.parse(line); }
    catch { this.nonJsonStdoutLines += 1; return; }

    if (typeof message.method === "string" && Object.hasOwn(message, "id")) {
      this.#handleServerRequest(message);
      return;
    }
    if (Object.hasOwn(message, "id")) {
      const entry = this.pending.get(String(message.id));
      if (!entry) return;
      clearTimeout(entry.timer);
      this.pending.delete(String(message.id));
      if (message.error) {
        const error = new Error("rpc_error");
        error.rpcCode = Number.isInteger(message.error.code) ? message.error.code : null;
        entry.reject(error);
      } else entry.resolve(message.result);
      return;
    }
    if (typeof message.method === "string") this.#handleNotification(message);
  }

  #handleServerRequest(message) {
    const { method, id } = message;
    if (["item/commandExecution/requestApproval", "item/fileChange/requestApproval"].includes(method)) {
      this.approvals.push({ method, decision: "decline", at: new Date().toISOString() });
      const decision = method === "item/commandExecution/requestApproval" ? "decline" : "decline";
      this.#send({ jsonrpc: "2.0", id, result: { decision } });
      return;
    }
    if (["applyPatchApproval", "execCommandApproval"].includes(method)) {
      this.approvals.push({ method, decision: "denied", at: new Date().toISOString() });
      this.#send({ jsonrpc: "2.0", id, result: { decision: { denied: { rejection: "P0-08 probe denies all requested actions." } } } });
      return;
    }
    if (method === "item/permissions/requestApproval") {
      this.approvals.push({ method, decision: "no_additional_permissions", at: new Date().toISOString() });
      this.#send({ jsonrpc: "2.0", id, result: { permissions: {}, scope: "turn", strictAutoReview: true } });
      return;
    }
    this.#send({ jsonrpc: "2.0", id, error: { code: -32601, message: "P0-08 probe does not handle this request." } });
  }

  #handleNotification(message) {
    const params = message.params ?? {};
    if (message.method === "item/agentMessage/delta") {
      const turnId = params.turnId;
      if (typeof turnId !== "string") return;
      if (!this.firstDeltas.has(turnId)) this.firstDeltas.set(turnId, performance.now());
      this.messages.set(turnId, (this.messages.get(turnId) ?? "") + (typeof params.delta === "string" ? params.delta : ""));
      return;
    }
    if (message.method === "turn/completed" && typeof params.turn?.id === "string") {
      this.turns.set(params.turn.id, params);
      return;
    }
    if (message.method === "thread/tokenUsage/updated" && typeof params.turnId === "string") {
      this.tokenUsage.set(params.turnId, params.tokenUsage);
      return;
    }
    if (message.method === "account/rateLimits/updated") {
      this.rateLimitUpdates.push({ at: new Date().toISOString(), summary: rateLimitSummary(params.rateLimits) });
    }
  }

  async request(method, params = {}, timeoutMs = 30_000) {
    const id = `p0-08-${++this.nextId}`;
    return new Promise((resolvePromise, rejectPromise) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        const error = new Error("rpc_timeout");
        error.rpcCode = null;
        rejectPromise(error);
      }, timeoutMs);
      this.pending.set(id, { resolve: resolvePromise, reject: rejectPromise, timer });
      this.#send({ jsonrpc: "2.0", id, method, params });
    });
  }

  notify(method, params = {}) {
    this.#send({ jsonrpc: "2.0", method, params });
  }

  async initialize() {
    const response = await this.request("initialize", {
      clientInfo: { name: "nemu-p0-08-probe", title: "Nemu P0-08 measurement", version: "0.1.0" },
      capabilities: { experimentalApi: true, requestAttestation: false, optOutNotificationMethods: [] }
    });
    this.notify("initialized");
    return response;
  }

  async readRateLimits() {
    try {
      const result = await this.request("account/rateLimits/read", { excludeResetCreditDetails: true });
      const buckets = result.rateLimitsByLimitId && Object.keys(result.rateLimitsByLimitId).length > 0
        ? Object.entries(result.rateLimitsByLimitId).map(([id, value]) => ({ limitId: id, ...rateLimitSummary(value) }))
        : [{ limitId: null, ...rateLimitSummary(result.rateLimits) }];
      return { ordinaryUsageAllowed: result.ordinaryUsageAllowed ?? null, buckets };
    } catch (error) {
      return { unavailable: true, errorCode: error.rpcCode ?? null };
    }
  }

  async listModels() {
    const models = [];
    let cursor = null;
    do {
      const result = await this.request("model/list", { limit: 100, includeHidden: false, ...(cursor ? { cursor } : {}) });
      for (const model of result.data ?? []) {
        models.push({
          id: model.id,
          model: model.model,
          displayName: model.displayName,
          hidden: model.hidden,
          isDefault: model.isDefault,
          supportedReasoningEfforts: (model.supportedReasoningEfforts ?? []).map((item) => item.reasoningEffort ?? item)
        });
      }
      cursor = result.nextCursor ?? null;
    } while (cursor);
    return models;
  }

  async startThread({ model, mode = "speech", cwd }) {
    const readOnly = mode !== "approval";
    const workdir = cwd ?? join(tmpdir(), `nemu-p0-08-thread-${randomUUID()}`);
    if (!cwd) {
      await mkdir(workdir, { recursive: true });
      this.tempWorkdirs.add(workdir);
    }
    const params = {
      model,
      cwd: workdir,
      approvalPolicy: readOnly ? "never" : "on-request",
      sandbox: "read-only",
      baseInstructions: sampleProtocol,
      personality: "none",
      ephemeral: true,
      dynamicTools: [],
      config: {
        web_search: "disabled",
        features: {
          apps: false,
          hooks: false,
          multi_agent: false,
          remote_plugin: false,
          shell_tool: !readOnly
        }
      }
    };
    const result = await this.request("thread/start", params);
    return result.thread.id;
  }

  async startTurn({ threadId, model, text, outputSchema, effort = "low", approval = false }) {
    const params = {
      threadId,
      model,
      effort,
      approvalPolicy: approval ? "on-request" : "never",
      sandboxPolicy: { type: "readOnly", networkAccess: false },
      input: [{ type: "text", text, text_elements: [] }],
      ...(outputSchema ? { outputSchema } : {})
    };
    const startedAt = performance.now();
    const response = await this.request("turn/start", params);
    return { turnId: response.turn.id, startedAt };
  }

  async waitForTurn(turnId, timeoutMs = 120_000) {
    if (this.turns.has(turnId)) return this.turns.get(turnId);
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      if (this.turns.has(turnId)) return this.turns.get(turnId);
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 50));
    }
    throw new Error("turn_timeout");
  }

  async waitForFirstDelta(turnId, timeoutMs = 20_000) {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      if (this.firstDeltas.has(turnId)) return this.firstDeltas.get(turnId);
      if (this.turns.has(turnId)) return null;
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 25));
    }
    return null;
  }

  async runOne({ model, text, outputSchema, mode = "speech", cwd = repoRoot, approval = false }) {
    const threadId = await this.startThread({ model, mode, cwd });
    const { turnId, startedAt } = await this.startTurn({ threadId, model, text, outputSchema, approval });
    const completed = await this.waitForTurn(turnId);
    const firstDeltaAt = this.firstDeltas.get(turnId);
    const fullText = this.messages.get(turnId) ?? "";
    const usage = tokenSummary(this.tokenUsage.get(turnId));
    return {
      threadId,
      turnId,
      turnStatus: completed.turn.status,
      firstDeltaMs: firstDeltaAt === undefined ? null : Math.round((firstDeltaAt - startedAt) * 100) / 100,
      completedMs: completed.turn.durationMs ?? null,
      outputChars: fullText.length,
      text: fullText,
      usage
    };
  }

  async close() {
    this.child.kill();
    await new Promise((resolvePromise) => {
      if (this.child.exitCode !== null) resolvePromise();
      else this.child.once("exit", resolvePromise);
    });
    await Promise.all([...this.tempWorkdirs].map((workdir) => rm(workdir, { recursive: true, force: true })));
    this.tempWorkdirs.clear();
  }
}

function validateSemanticCandidate(value, testCase) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  if (Object.keys(value).some((key) => !["schemaVersion", "sourceRef", "nodes"].includes(key))) return false;
  if (value.schemaVersion !== "p0-08-candidate-v1" || value.sourceRef !== testCase.sourceRef || !Array.isArray(value.nodes)) return false;
  const keys = new Set();
  const maxSpan = Array.from(testCase.text).length;
  for (const node of value.nodes) {
    if (!node || typeof node !== "object" || Array.isArray(node)) return false;
    const common = ["id", "sourceRefs", "sourceSpan", "derivation", "epistemicType", "resolutionState", "representationVersion"];
    if (!common.every((key) => Object.hasOwn(node, key))) return false;
    const kindFields = {
      Ref: ["kind", "mention"],
      Claim: ["kind", "predicateLabel", "roles", "value", "polarity", "modality", "time"],
      Act: ["kind", "actType", "content"]
    }[node.kind];
    if (!kindFields || Object.keys(node).some((key) => !common.includes(key) && !kindFields.includes(key))) return false;
    if (typeof node.id !== "string" || node.id.length === 0 || keys.has(node.id)) return false;
    keys.add(node.id);
    if (!Array.isArray(node.sourceRefs) || node.sourceRefs.length === 0 || !node.sourceRefs.every((ref) => typeof ref === "string") || new Set(node.sourceRefs).size !== node.sourceRefs.length || !node.sourceRefs.includes(testCase.sourceRef)) return false;
    if (!Array.isArray(node.sourceSpan) || node.sourceSpan.length !== 2 || !node.sourceSpan.every(Number.isInteger)) return false;
    if (node.sourceSpan[0] < 0 || node.sourceSpan[1] < node.sourceSpan[0] || node.sourceSpan[1] > maxSpan) return false;
    if (!["verbatim", "normalized", "resolved", "inferred"].includes(node.derivation)) return false;
    if (!["user_assertion", "user_self_report", "quotation", "tool_output", "system_record", "model_inference", "unknown"].includes(node.epistemicType)) return false;
    if (!["resolved", "unresolved", "provisional"].includes(node.resolutionState)) return false;
    if (node.representationVersion !== "p0-08-candidate-v1") return false;
    if (node.kind === "Ref" && typeof node.mention === "string" && node.mention.length > 0) continue;
    const claimValueValid = node.value === null || ["string", "number", "boolean"].includes(typeof node.value);
    if (node.kind === "Claim" && typeof node.predicateLabel === "string" && node.predicateLabel.length > 0 && Array.isArray(node.roles) && node.roles.every((role) => role && typeof role === "object" && !Array.isArray(role) && Object.keys(role).length === 2 && typeof role.role === "string" && typeof role.value === "string") && claimValueValid && ["positive", "negative"].includes(node.polarity) && ["asserted", "possible", "conditional", "desired", "reported", "unknown"].includes(node.modality) && (typeof node.time === "string" || node.time === null)) continue;
    if (node.kind === "Act" && ["request", "question", "correction", "commitment", "quotation", "other"].includes(node.actType) && typeof node.content === "string") continue;
    return false;
  }
  return true;
}

function parseJsonOutput(text) {
  const trimmed = text.trim().replace(/^```(?:json)?\s*/i, "").replace(/\s*```$/, "");
  try { return JSON.parse(trimmed); } catch { return null; }
}

async function appendRecord(path, value) {
  await mkdir(dirname(path), { recursive: true });
  await appendFile(path, `${JSON.stringify(value)}\n`, "utf8");
}

async function summarizeRecords(mode, rows, metadata = {}) {
  const usageRows = rows.map((row) => row.usage).filter(Boolean);
  const usageTotals = usageRows.reduce((sum, usage) => ({
    totalTokens: sum.totalTokens + (usage.totalTokens ?? 0),
    inputTokens: sum.inputTokens + (usage.inputTokens ?? 0),
    cachedInputTokens: sum.cachedInputTokens + (usage.cachedInputTokens ?? 0),
    outputTokens: sum.outputTokens + (usage.outputTokens ?? 0),
    reasoningOutputTokens: sum.reasoningOutputTokens + (usage.reasoningOutputTokens ?? 0)
  }), { totalTokens: 0, inputTokens: 0, cachedInputTokens: 0, outputTokens: 0, reasoningOutputTokens: 0 });
  const output = {
    mode,
    measuredAtUtc: new Date().toISOString(),
    count: rows.length,
    firstDeltaP50Ms: percentile(rows.map((r) => r.firstDeltaMs), 0.50),
    firstDeltaP90Ms: percentile(rows.map((r) => r.firstDeltaMs), 0.90),
    completionP50Ms: percentile(rows.map((r) => r.completedMs), 0.50),
    completionP90Ms: percentile(rows.map((r) => r.completedMs), 0.90),
    successCount: rows.filter((r) => r.turnStatus === "completed").length,
    failures: rows.filter((r) => r.errorCode !== undefined).length,
    usageRows: usageRows.length,
    usageTotals,
    ...metadata
  };
  const outPath = resolve(reportRoot, `${mode}-summary.json`);
  await mkdir(dirname(outPath), { recursive: true });
  const { writeFile } = await import("node:fs/promises");
  await writeFile(outPath, `${JSON.stringify(output, null, 2)}\n`, "utf8");
  return output;
}

async function speechBatch(server, model, lang, runs) {
  const dataset = JSON.parse(await readFile(fixturesPath, "utf8")).cases.filter((item) => item.lang === lang);
  if (dataset.length < runs) throw new Error(`not enough ${lang} fixtures for ${runs} speech runs`);
  const runId = randomUUID();
  const rawPath = resolve(reportRoot, `raw/remote-speech-${lang}-${runId}.jsonl`);
  const rateLimitsBefore = await server.readRateLimits();
  const rows = [];
  for (let index = 0; index < runs; index += 1) {
    const sample = dataset[index];
    const instruction = lang === "zh" ? `请用中文用一句简短、自然的话回应，最多20字：${sample.text}`
      : lang === "ja" ? `日本語で20語以内の短い自然な一文で返答してください：${sample.text}`
        : `Reply in English with one short, natural sentence, at most 20 words: ${sample.text}`;
    try {
      const result = await server.runOne({ model, text: instruction });
      const row = { index: index + 1, caseId: sample.id, lang, model, turnStatus: result.turnStatus, firstDeltaMs: result.firstDeltaMs, completedMs: result.completedMs, outputChars: result.outputChars, usage: result.usage };
      rows.push(row);
      await appendRecord(rawPath, { runId, ...row });
    } catch (error) {
      const row = { index: index + 1, caseId: sample.id, lang, model, errorCode: error.rpcCode ?? null, errorClass: error.message };
      rows.push(row);
      await appendRecord(rawPath, { runId, ...row });
      if (error.message === "server_exited" || error.message === "spawn_error") throw error;
    }
  }
  const rateLimitsAfter = await server.readRateLimits();
  return summarizeRecords(`remote-speech-${lang}`, rows, { runId, rateLimitsBefore, rateLimitsAfter });
}

async function speechAll(server, model, runsPerLanguage) {
  const rateLimitsBefore = await server.readRateLimits();
  const languages = ["zh", "en", "ja"];
  const summaries = [];
  let rateLimitsAfter100 = null;
  for (const [index, lang] of languages.entries()) {
    summaries.push(await speechBatch(server, model, lang, runsPerLanguage));
    if ((index + 1) * runsPerLanguage === 100) rateLimitsAfter100 = await server.readRateLimits();
  }
  const output = {
    mode: "remote-speech-all",
    measuredAtUtc: new Date().toISOString(),
    calls: summaries.reduce((sum, summary) => sum + summary.count, 0),
    rateLimitsBefore,
    rateLimitsAfter100,
    rateLimitsAfter: await server.readRateLimits(),
    summaries
  };
  const outPath = resolve(reportRoot, "remote-speech-all-summary.json");
  await mkdir(dirname(outPath), { recursive: true });
  const { writeFile } = await import("node:fs/promises");
  await writeFile(outPath, `${JSON.stringify(output, null, 2)}\n`, "utf8");
  return output;
}

async function irlBatch(server, model, runs, schema) {
  const dataset = JSON.parse(await readFile(fixturesPath, "utf8")).cases;
  if (dataset.length < runs) throw new Error(`not enough fixtures for ${runs} IRL runs`);
  const runId = randomUUID();
  const rawPath = resolve(reportRoot, `raw/remote-irl-${runId}.jsonl`);
  const rateLimitsBefore = await server.readRateLimits();
  const rows = [];
  let rateLimitsAfter100 = null;
  for (let index = 0; index < runs; index += 1) {
    const sample = dataset[index];
    const prompt = `Convert this single synthetic user utterance into an experimental SemanticCandidate JSON object using the supplied schema. Preserve only what the sentence says. Use sourceRef ${sample.sourceRef}; sourceSpan is a half-open range of Unicode code points in the utterance. Do not fill missing facts. Language=${sample.lang}; utterance=${JSON.stringify(sample.text)}`;
    try {
      const result = await server.runOne({ model, text: prompt, outputSchema: schema });
      const candidate = parseJsonOutput(result.text);
      const schemaValid = validateSemanticCandidate(candidate, sample);
      const row = { index: index + 1, caseId: sample.id, lang: sample.lang, category: sample.category, model, turnStatus: result.turnStatus, firstDeltaMs: result.firstDeltaMs, completedMs: result.completedMs, outputChars: result.outputChars, schemaValid, nodeCount: schemaValid ? candidate.nodes.length : null, usage: result.usage };
      rows.push(row);
      await appendRecord(rawPath, { runId, ...row });
    } catch (error) {
      const row = { index: index + 1, caseId: sample.id, lang: sample.lang, category: sample.category, model, errorCode: error.rpcCode ?? null, errorClass: error.message, schemaValid: false };
      rows.push(row);
      await appendRecord(rawPath, { runId, ...row });
      if (error.message === "server_exited" || error.message === "spawn_error") throw error;
    }
    if (index === 99) rateLimitsAfter100 = await server.readRateLimits();
  }
  const rateLimitsAfter = await server.readRateLimits();
  const output = await summarizeRecords("remote-irl", rows, { runId, rateLimitsBefore, rateLimitsAfter100, rateLimitsAfter });
  const validated = rows.filter((row) => row.turnStatus === "completed");
  const { writeFile } = await import("node:fs/promises");
  await writeFile(resolve(reportRoot, "remote-irl-schema-summary.json"), `${JSON.stringify({ runId, total: rows.length, completed: validated.length, valid: validated.filter((row) => row.schemaValid).length, schemaValidity: validated.length === 0 ? null : validated.filter((row) => row.schemaValid).length / validated.length }, null, 2)}\n`, "utf8");
  return output;
}

async function catalog(server) {
  const [models, rateLimits] = await Promise.all([server.listModels(), server.readRateLimits()]);
  let cliVersion = null;
  try { cliVersion = execFileSync("codex", ["--version"], { encoding: "utf8" }).trim().replace(/^codex-cli\s+/, ""); } catch {}
  const output = { measuredAtUtc: new Date().toISOString(), cliVersion, modelCount: models.length, models, rateLimits };
  const outPath = resolve(reportRoot, "model-catalog.json");
  await mkdir(dirname(outPath), { recursive: true });
  const { writeFile } = await import("node:fs/promises");
  await writeFile(outPath, `${JSON.stringify(output, null, 2)}\n`, "utf8");
  return output;
}

async function controlProbe(server, model) {
  const controls = [];
  const longText = "Write a long numbered response in English, continuing for at least 400 numbered items unless interrupted. Do not use tools.";
  const threadId = await server.startThread({ model });
  const { turnId, startedAt } = await server.startTurn({ threadId, model, text: longText });
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 750));
  try {
    await server.request("turn/interrupt", { threadId, turnId }, 15_000);
    const completed = await server.waitForTurn(turnId, 30_000);
    controls.push({ control: "turn/interrupt", requestAccepted: true, turnStatus: completed.turn.status, firstDeltaMs: server.firstDeltas.has(turnId) ? Math.round((server.firstDeltas.get(turnId) - startedAt) * 100) / 100 : null });
  } catch (error) {
    controls.push({ control: "turn/interrupt", requestAccepted: false, errorCode: error.rpcCode ?? null, errorClass: error.message });
  }

  const steerThread = await server.startThread({ model });
  const steerTurn = await server.startTurn({ threadId: steerThread, model, text: longText });
  const firstDeltaAt = await server.waitForFirstDelta(steerTurn.turnId, 20_000);
  try {
    const steered = await server.request("turn/steer", {
      threadId: steerThread,
      expectedTurnId: steerTurn.turnId,
      input: [{ type: "text", text: "Stop the list and reply with exactly STEERED.", text_elements: [] }]
    }, 15_000);
    const completed = await server.waitForTurn(steerTurn.turnId, 60_000);
    controls.push({ control: "turn/steer", requestAccepted: true, firstDeltaSeen: firstDeltaAt !== null, turnStatus: completed.turn.status, outputContainsMarker: (server.messages.get(steerTurn.turnId) ?? "").includes("STEERED"), responseObserved: Boolean(steered) });
  } catch (error) {
    controls.push({ control: "turn/steer", requestAccepted: false, firstDeltaSeen: firstDeltaAt !== null, errorCode: error.rpcCode ?? null, errorClass: error.message });
  }

  const approvalDir = join(tmpdir(), `nemu-p0-08-approvals-${randomUUID()}`);
  const { mkdir: makeDir, rm } = await import("node:fs/promises");
  await makeDir(approvalDir, { recursive: true });
  const approvalPrompts = [
    { control: "command-approval", text: "Use the command execution tool to create a file named COMMAND-PROBE.txt in the current directory using shell output redirection, with only the text P0-08 approval probe. Do not do anything else." },
    { control: "file-change-approval", text: `Use the file editing tool to create a file named APPROVAL-PROBE.txt in the current directory containing only P0-08 probe. Do not use a shell command.` }
  ];
  const approvalStartIndex = server.approvals.length;
  for (const prompt of approvalPrompts) {
    try {
      const result = await server.runOne({ model, text: prompt.text, mode: "approval", cwd: approvalDir, approval: true });
      controls.push({ control: prompt.control, turnStatus: result.turnStatus, outputChars: result.outputChars });
    } catch (error) {
      controls.push({ control: prompt.control, requestAccepted: false, errorCode: error.rpcCode ?? null, errorClass: error.message });
    }
  }
  controls.push({
    control: "approval-routing",
    requests: server.approvals.slice(approvalStartIndex).map(({ method, decision }) => ({ method, decision })),
    commandFileCreated: existsSync(join(approvalDir, "COMMAND-PROBE.txt")),
    testFileCreated: existsSync(join(approvalDir, "APPROVAL-PROBE.txt"))
  });
  await rm(approvalDir, { recursive: true, force: true });
  const outPath = resolve(reportRoot, "controls.json");
  const { writeFile } = await import("node:fs/promises");
  await writeFile(outPath, `${JSON.stringify({ measuredAtUtc: new Date().toISOString(), model, controls }, null, 2)}\n`, "utf8");
  return controls;
}

function printHelp() {
  process.stdout.write([
    "P0-08 Codex app-server measurement probe",
    "",
    "Commands (run from the repository root):",
    "  node spikes/codex-app-server/src/probe.mjs catalog --code-home <isolated path>",
    "  node spikes/codex-app-server/src/probe.mjs smoke --code-home <isolated path>",
    "  node spikes/codex-app-server/src/probe.mjs speech --code-home <path> --model <id> --lang zh|en|ja --runs 50",
    "  node spikes/codex-app-server/src/probe.mjs speech-all --code-home <path> --model <id> --runs-per-language 50",
    "  node spikes/codex-app-server/src/probe.mjs irl --code-home <path> --model <id> --runs 200",
    "  node spikes/codex-app-server/src/probe.mjs controls --code-home <path> --model <id>",
    "",
    "All raw records omit response text, account identifiers, and credentials."
  ].join("\n"));
}

async function main() {
  const { mode, options } = parseArgs(process.argv.slice(2));
  if (!mode || mode === "help" || mode === "--help" || mode === "-h" || options.help) return printHelp();
  const codeHome = required(options, "code-home");
  const model = options.model;
  if (["speech", "speech-all", "irl", "controls"].includes(mode)) required(options, "model");
  const server = new AppServer(codeHome);
  try {
    await server.initialize();
    let result;
    if (mode === "catalog") result = await catalog(server);
    else if (mode === "smoke") {
      const threadId = await server.startThread({});
      result = { initialized: true, ephemeralThreadStarted: typeof threadId === "string" && threadId.length > 0, configuredModel: null, turnStarted: false };
      const outPath = resolve(reportRoot, "local-smoke.json");
      await mkdir(dirname(outPath), { recursive: true });
      const { writeFile } = await import("node:fs/promises");
      await writeFile(outPath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
    }
    else if (mode === "speech") {
      const lang = required(options, "lang");
      if (!["zh", "en", "ja"].includes(lang)) throw new Error("--lang must be zh, en, or ja");
      result = await speechBatch(server, model, lang, safeInteger(options.runs, 50, 1, 50));
    } else if (mode === "speech-all") {
      const runsPerLanguage = safeInteger(options["runs-per-language"], 50, 1, 50);
      if (runsPerLanguage !== 50) throw new Error("speech-all requires exactly 50 runs per language");
      result = await speechAll(server, model, runsPerLanguage);
    } else if (mode === "irl") {
      const schema = JSON.parse(await readFile(schemaPath, "utf8"));
      result = await irlBatch(server, model, safeInteger(options.runs, 200, 1, 200), schema);
    } else if (mode === "controls") result = await controlProbe(server, model);
    else throw new Error(`unknown command: ${mode}`);
    process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
  } finally {
    await server.close();
  }
}

main().catch((error) => {
  const detail = error.rpcCode === undefined ? error.message : `rpc_error_${error.rpcCode}`;
  process.stderr.write(`P0-08 probe failed: ${detail}\n`);
  process.exitCode = 1;
});
