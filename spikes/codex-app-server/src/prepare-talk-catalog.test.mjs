import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, readFile, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const script = fileURLToPath(new URL("./prepare-talk-catalog.mjs", import.meta.url));
const invoke = (source, output, report, extra = []) => spawnSync(process.execPath, [script, "--source", source, "--output", output, "--report", report, ...extra], { encoding: "utf8" });

test("catalog derivation changes only the four Luna tool fields and refuses overwrite", async () => {
  const dir = await mkdtemp(join(tmpdir(), "nemu-catalog-test-"));
  try {
    const source = join(dir, "models_cache.json");
    const output = join(dir, "derived.json");
    const report = join(dir, "report.json");
    const models = [{ slug: "gpt-6-luna", tool_mode: "code_mode_only", apply_patch_tool_type: "freeform", experimental_supported_tools: ["clock"], supports_search_tool: true, supported_reasoning_levels: ["max"], opaque: { keep: true } }, { slug: "other", tool_mode: "code_mode_only" }];
    await writeFile(source, JSON.stringify({ models }));
    const result = invoke(source, output, report);
    assert.equal(result.status, 0, result.stderr);
    const derived = JSON.parse(await readFile(output, "utf8"));
    assert.deepEqual(derived.models, [{ ...models[0], tool_mode: "direct", apply_patch_tool_type: null, experimental_supported_tools: [], supports_search_tool: false }, models[1]]);
    const record = JSON.parse(await readFile(report, "utf8"));
    assert.equal(record.selectedModelIdChanged, false);
    assert.equal(record.reasoningEffortChanged, false);
    assert.equal(record.derivedSha256.length, 64);
    assert.notEqual(invoke(source, output, report).status, 0);
    const strictOutput = join(dir, "strict-derived.json");
    const strictReport = join(dir, "strict-report.json");
    const strict = invoke(source, strictOutput, strictReport, ["--disable-collaboration"]);
    assert.equal(strict.status, 0, strict.stderr);
    const isolated = JSON.parse(await readFile(strictOutput, "utf8"));
    assert.deepEqual(isolated.models, [{ ...derived.models[0], multi_agent_version: "disabled" }, models[1]]);
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test("refuses an authentication filename before opening it", () => {
  const result = invoke(join(tmpdir(), "auth.json"), join(tmpdir(), "unused-catalog.json"), join(tmpdir(), "unused-report.json"));
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /never auth files/);
});

test("refuses the whole provider catalog inside the repository", () => {
  const result = invoke(join(tmpdir(), "models_cache.json"), resolve(script, "../forbidden.json"), join(tmpdir(), "unused-report.json"));
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /outside Git/);
});
