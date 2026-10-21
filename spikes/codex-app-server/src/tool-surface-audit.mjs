import { AppServer } from "./probe.mjs";
import { createInterface } from "node:readline";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve, join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { randomUUID } from "node:crypto";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const option = (name) => process.argv[process.argv.indexOf(`--${name}`) + 1];
if (!process.argv.includes("--code-home")) throw new Error("Explicit isolated --code-home required");
const model = "gpt-6-luna";
const catalogPath = process.argv.includes("--catalog") ? resolve(option("catalog")) : undefined;
const server = new AppServer(option("code-home"), { catalogPath, strictTools: Boolean(catalogPath) });
const dirs = [];
const records = [];
const runId = randomUUID();
const output = resolve(root, `reports/P0-08/part2/tool-surface-audit-${runId}.json`);
let current = null;
const observer = createInterface({ input: server.child.stdout });
observer.on("line", (line) => {
  if (!current) return;
  let message;
  try { message = JSON.parse(line); } catch { return; }
  if (message.method) current.methods.add(message.method);
  const inspect = (value) => {
    if (!value || typeof value !== "object") return;
    if (["function_call", "custom_tool_call"].includes(value.type) && typeof value.name === "string") {
      current.calls.add(value.name);
      const args = typeof value.arguments === "string" ? value.arguments : value.input ?? "";
      if (/clock|curr_time|current_time/.test(args)) current.clockIdentifierSeen = true;
    }
    for (const child of Object.values(value)) if (child && typeof child === "object") {
      if (Array.isArray(child)) child.forEach(inspect); else inspect(child);
    }
  };
  inspect(message.params);
});
try {
  await server.initialize();
  for (const request of [
    { name: "clock", text: "Use the available built-in current-time/clock tool exactly once to read UTC. Do not estimate time or simulate a call. Then reply AUDIT-DONE. Do not run commands, read files, or ask questions." },
    { name: "file-change", text: "Use only the file editing tool to create TOOL-SURFACE-AUDIT.txt in this temporary current directory with text P0-08 synthetic audit. Do not use a shell or read other files. If denied, stop and reply AUDIT-DONE." }
  ]) {
    const cwd = await mkdtemp(join(tmpdir(), "nemu-p008-tool-surface-"));
    dirs.push(cwd);
    current = { methods: new Set(), calls: new Set(), clockIdentifierSeen: false };
    let record;
    try {
      const threadId = await server.startThread({ model, cwd, experimentalRawEvents: true,
        baseInstructions: "You are an isolated capability exposure audit. Perform only the one explicitly requested harmless synthetic test. Never work around a denial. Do not access other files, network, or user data."
      });
      const turn = await server.startTurn({ threadId, model, text: request.text });
      const result = await server.waitForTurn(turn.turnId, 180000);
      record = { test: request.name, status: result.turn.status,
        observedItemTypes: server.observedItems.get(turn.turnId) ?? [],
        approvalRequests: server.approvals.map(({ method, decision }) => ({ method, decision })),
        fileCreated: existsSync(join(cwd, "TOOL-SURFACE-AUDIT.txt")) };
      await server.request("thread/unsubscribe", { threadId });
    } catch (error) { record = { test: request.name, errorClass: error.message, errorCode: error.rpcCode ?? null, errorDetail: error.rpcDetail ?? null }; }
    records.push({ ...record, notificationMethods: [...current.methods], rawToolCallNames: [...current.calls], clockIdentifierSeen: current.clockIdentifierSeen });
    current = null;
    await writeFile(output, JSON.stringify({ runId, measuredAtUtc: new Date().toISOString(), model, effort: "max", catalogRestriction: catalogPath ? "derived empty-tools catalog + strict feature flags" : "baseline catalog", scope: "Same read-only/never speech capability flags; only base instructions and raw-event reporting changed to request harmless capability tests. Does not count as a latency sample.", records }, null, 2) + "\n");
  }
} finally {
  observer.close();
  await server.close();
  await Promise.all(dirs.map((dir) => rm(dir, { recursive: true, force: true })));
}
process.stdout.write(JSON.stringify({ output, records }, null, 2) + "\n");
