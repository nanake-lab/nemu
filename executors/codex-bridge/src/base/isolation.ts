import { z } from 'zod';
import type { ThreadStartParams } from './generated/v2/ThreadStartParams.js';

export function isolatedEnvironment(environment: NodeJS.ProcessEnv, codeHome: string): NodeJS.ProcessEnv {
  return {
    ...Object.fromEntries(Object.entries(environment).filter(([key]) =>
      !key.toUpperCase().startsWith('CODEX_') &&
      !['OPENAI_API_KEY', 'OPENAI_BASE_URL'].includes(key.toUpperCase()))),
    CODEX_HOME: codeHome,
  };
}

const catalog = z.object({ models: z.array(z.object({
  slug: z.string(), tool_mode: z.literal('direct'), apply_patch_tool_type: z.null(),
  experimental_supported_tools: z.array(z.unknown()).length(0),
  supports_search_tool: z.literal(false), multi_agent_version: z.literal('disabled'),
}).passthrough()) });

export function checkCatalog(value: unknown, model: string): void {
  const raw = z.object({ models: z.array(z.record(z.string(), z.unknown())) }).parse(value);
  const selected = raw.models.filter((entry) => entry.slug === model);
  if (selected.length !== 1) throw new Error('missing_or_duplicate_model');
  catalog.parse({ models: selected });
}

export const strictArguments = [
  '-c', 'features.apps=false', '-c', 'features.remote_plugin=false', '-c', 'features.hooks=false',
  '-c', 'orchestrator.mcp.enabled=false', '-c', 'features.plugins=false',
  '-c', 'features.skill_mcp_dependency_install=false', '-c', 'cloud.skills.enabled=false',
  '-c', 'multi_agent_v2.disable_native_tools=true',
];

export function strictThread(model: string, cwd: string, instructions: string): ThreadStartParams {
  return {
    model, cwd, baseInstructions: instructions, approvalPolicy: 'untrusted',
    approvalsReviewer: 'user', sandbox: 'read-only', personality: 'none',
    ephemeral: true, dynamicTools: [], environments: [], selectedCapabilityRoots: [],
    experimentalRawEvents: true,
    config: {
      orchestrator: { mcp: { enabled: false } }, cloud: { skills: { enabled: false } },
      multi_agent_v2: { disable_native_tools: true }, web_search: 'disabled',
      tools: { update_plan: { enabled: false }, experimental_request_user_input: { enabled: false } },
      features: Object.fromEntries([
        'apps', 'hooks', 'multi_agent', 'remote_plugin', 'shell_tool', 'plugins',
        'skill_mcp_dependency_install', 'multi_agent_v2', 'stable_environment_tools',
        'view_image', 'image_generation', 'request_permissions_tool', 'tool_search',
        'tool_suggest', 'deferred_executor', 'token_budget', 'sleep_tool',
        'current_time_reminder', 'send_message_to_user_async', 'goals', 'code_mode', 'code_mode_host',
      ].map((name) => [name, false])),
    },
  };
}
