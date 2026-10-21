import { readFile, writeFile, mkdir } from "node:fs/promises";
import { createHash } from "node:crypto";
import { basename, dirname, resolve, relative, isAbsolute, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const arg = (name) => {
  const index = process.argv.indexOf(`--${name}`);
  if (index < 0 || !process.argv[index + 1]) throw new Error(`Explicit --${name} required`);
  return resolve(process.argv[index + 1]);
};
const source = arg("source");
const output = arg("output");
const report = arg("report");
if (basename(source) !== "models_cache.json") throw new Error("Read only models_cache.json capability metadata, never auth files");
const within = relative(root, output);
if (within !== ".." && !within.startsWith(`..${sep}`) && !isAbsolute(within)) throw new Error("Keep the whole provider catalog outside Git");
const raw = await readFile(source);
const cache = JSON.parse(raw.toString("utf8").replace(/^\uFEFF/, ""));
if (!Array.isArray(cache.models)) throw new Error("Invalid model metadata catalog");
const models = structuredClone(cache.models);
const matches = models.filter((model) => model.slug === "gpt-6-luna");
if (matches.length !== 1) throw new Error("Expected exactly one gpt-6-luna entry");
const model = matches[0];
const fields = ["slug", "tool_mode", "apply_patch_tool_type", "experimental_supported_tools", "supports_search_tool"];
const before = Object.fromEntries(fields.map((field) => [field, model[field] ?? null]));
Object.assign(model, { tool_mode: "direct", apply_patch_tool_type: null, experimental_supported_tools: [], supports_search_tool: false });
const derived = Buffer.from(JSON.stringify({ models }, null, 2) + "\n");
await mkdir(dirname(output), { recursive: true });
await writeFile(output, derived, { flag: "wx" });
const hash = (data) => createHash("sha256").update(data).digest("hex");
await mkdir(dirname(report), { recursive: true });
await writeFile(report, JSON.stringify({
  createdUtc: new Date().toISOString(), generator: "spikes/codex-app-server/src/prepare-talk-catalog.mjs",
  source: "Authenticated profile models_cache.json capability metadata; no auth file read",
  sourceSha256: hash(raw), derivedSha256: hash(derived), derivedLocalPath: output,
  before, after: Object.fromEntries(fields.map((field) => [field, model[field] ?? null])),
  otherModelMetadataChanged: false, selectedModelIdChanged: false, reasoningEffortChanged: false,
  note: "Supported model_catalog_json configuration narrows client tool declarations. Other model fields remain unchanged; the whole provider catalog stays outside Git. Use with the explicit strict feature profile and empty environments."
}, null, 2) + "\n");
process.stdout.write(JSON.stringify({ output, report, derivedSha256: hash(derived) }) + "\n");
