import { AppServer } from "./probe.mjs";
import { writeFile } from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { resolve } from "node:path";
import { createInterface } from "node:readline";

const arg = name => {
  const index = process.argv.indexOf(`--${name}`);
  if (index < 0 || !process.argv[index + 1]) throw Error(`Explicit --${name} required`);
  return process.argv[index + 1];
};
const codeHome = arg("code-home");
const catalogPath = arg("catalog");
const output = resolve(arg("output"));
const runId = randomUUID();
const records = [];
const isolatedOnly = process.argv.includes("--isolated-only");
for (const [disableOrchestratorMcp, isolateAppContext] of isolatedOnly ? [[true, true]] : [[false, false], [true, false], [true, true]]) {
  const server = new AppServer(codeHome, { catalogPath, strictTools: true, disableOrchestratorMcp, isolateAppContext });
  let currentItems = [];
  const observer = createInterface({ input: server.child.stdout });
  observer.on("line", line => {
    let message;
    try { message = JSON.parse(line); } catch { return; }
    const item = message.params?.item;
    if (message.method === "item/completed" && item?.type === "mcpToolCall") {
      currentItems.push({ tool: item.tool, status: item.status, hasError: item.error != null });
    }
    if (message.method === "rawResponseItem/completed" && item?.type === "function_call_output") {
      currentItems.push({ rawOutput: true, unsupportedToolError: typeof item.output === "string" && /unsupported|unknown tool|unrecognized tool/.test(item.output) });
    }
  });
  try {
    await server.initialize();
    const config = await server.request("config/read", { includeLayers: false });
    const mcp = await server.request("mcpServerStatus/list", { limit: 100 });
    for (const tool of ["list_mcp_resources", "list_mcp_resource_templates"]) {
      currentItems = [];
      const threadId = await server.startThread({ model: "gpt-6-luna", experimentalRawEvents: true,
        baseInstructions: "This is an isolated capability audit. Only enumerate the requested resource catalog if that helper is available. Never read any resource, file, or network content. Do not call any other tool or work around a denial. Do not report catalog entries; reply AUDIT-DONE."
      });
      const { turnId } = await server.startTurn({ threadId, model: "gpt-6-luna", effort: "low",
        text: `If available, call ${tool} once with empty arguments. Do not call any other function. Never read any listed resource. If unavailable, stop. Reply AUDIT-DONE without listing entries.`
      });
      let status = null;
      let errorClass = null;
      try { status = (await server.waitForTurn(turnId, 180000)).turn.status; }
      catch (error) { errorClass = error.message; await server.request("turn/interrupt", { threadId, turnId }).catch(() => {}); }
      records.push({ disableOrchestratorMcp, isolateAppContext, requestedTool: tool, status, errorClass,
        configuredOrchestratorMcpEnabled: config.config?.orchestrator?.mcp?.enabled ?? null,
        configuredPluginsEnabled: config.config?.features?.plugins ?? null,
        configuredCloudSkillsEnabled: config.config?.cloud?.skills?.enabled ?? null,
        mcpServerCount: mcp.data?.length ?? null,
        rawToolCallNames: server.rawToolCalls.get(turnId) ?? [],
        observedItemTypes: server.observedItems.get(turnId) ?? [],
        completedToolItems: currentItems,
        mcpServerCountAfter: (await server.request("mcpServerStatus/list", { limit: 100 })).data?.length ?? null,
      });
      await server.request("thread/unsubscribe", { threadId });
      await writeFile(output, JSON.stringify({ runId, measuredUtc: new Date().toISOString(), model: "gpt-6-luna", effort: "low", records, method: "Capability audit, not a latency sample. No output text, resource entries, or credentials persisted. Arms: baseline; orchestrator MCP disabled only; full app-context isolation (clean inherited CODEX environment plus orchestrator MCP/local plugins/cloud skills/skill MCP installation disabled)." }, null, 2) + "\n");
    }
  } finally { observer.close(); await server.close(); }
}
const isolated = records.filter(record => record.isolateAppContext);
if (isolated.length !== 2 || isolated.some(record => record.errorClass || record.rawToolCallNames.length || record.mcpServerCountAfter !== 0 || record.configuredOrchestratorMcpEnabled !== false || record.configuredPluginsEnabled !== false || record.configuredCloudSkillsEnabled !== false)) process.exitCode = 2;

