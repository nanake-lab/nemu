#!/usr/bin/env node
import { createHash } from "node:crypto";
import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import { dirname, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const generatedRoot = resolve(repoRoot, "spikes/codex-app-server/generated");
const outputPath = resolve(repoRoot, "spikes/codex-app-server/drafts/cli-protocol-candidate.json");

async function collectFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) files.push(...await collectFiles(path));
    else if (entry.isFile()) files.push(path);
  }
  return files;
}

const cliVersion = execFileSync("codex", ["--version"], { encoding: "utf8" }).trim().replace(/^codex-cli\s+/, "");
const files = await Promise.all((await collectFiles(generatedRoot)).map(async (path) => ({
  path: relative(generatedRoot, path).split(sep).join("/"),
  sha256: createHash("sha256").update(await readFile(path)).digest("hex")
})));
files.sort((left, right) => left.path < right.path ? -1 : left.path > right.path ? 1 : 0);
const manifestHash = createHash("sha256").update(files.map(({ path, sha256 }) => `${path}\0${sha256}\n`).join(""), "utf8").digest("hex");
const candidate = {
  status: "candidate_only",
  cliVersion,
  generator: "codex app-server generate-ts --experimental",
  generatorVersion: cliVersion,
  generatedDirectory: "spikes/codex-app-server/generated",
  generatedFileCount: files.length,
  generatedManifestSha256: manifestHash,
  measuredAtUtc: new Date().toISOString(),
  scopeNote: "Candidate only; config/models.lock.json is outside P0-08 directory scope."
};

await mkdir(dirname(outputPath), { recursive: true });
await writeFile(outputPath, `${JSON.stringify(candidate, null, 2)}\n`, "utf8");
process.stdout.write(`${JSON.stringify(candidate, null, 2)}\n`);
