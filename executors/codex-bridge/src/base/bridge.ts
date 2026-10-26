import { z } from 'zod';
import { mkdtemp, realpath, rmdir } from 'node:fs/promises';
import { join } from 'node:path';
import { checkCatalog, strictThread } from './isolation.js';
import { RpcClient } from './rpc.js';
import { inventoryResult, object, threadResult, turnResult, usage, type Message } from './protocol.js';
import type { JsonValue } from './generated/serde_json/JsonValue.js';
import type { ReasoningEffort } from './generated/ReasoningEffort.js';
import type { TokenUsageBreakdown } from './generated/v2/TokenUsageBreakdown.js';

export type Tier = 'everyday' | 'work' | 'hard';
const models = z.object({ everyday: z.string().min(1), work: z.string().min(1), hard: z.string().min(1) });
const completion = z.object({
  threadId: z.string(), turn: z.object({
    id: z.string(), status: z.enum(['completed', 'interrupted', 'failed', 'inProgress']),
    error: z.object({ codexErrorInfo: z.unknown() }).passthrough().nullable(),
  }),
});
const delta = z.object({ threadId: z.string(), turnId: z.string(), delta: z.string() });
const item = z.object({ threadId: z.string(), turnId: z.string(), item: object });
const tokenEvent = z.object({ threadId: z.string(), turnId: z.string(), tokenUsage: z.object({ last: usage }) });

export interface RunInput {
  tier: Tier; instructions: string; text: string;
  effort: ReasoningEffort; outputSchema: JsonValue; deadlineMs: number;
  maxOutputBytes: number; onDelta?: (text: string) => void;
}
export interface TurnReceipt {
  threadId: string; turnId: string; tier: Tier; model: string;
  text: string; tokens: TokenUsageBreakdown | null;
  completedMs: number; firstDeltaMs: number | null;
  toolCalls: 0; mcpServersAfter: 0;
}
export interface UsageRecord {
  tier: Tier; model: string; threadId: string; turnId: string | null;
  tokens: TokenUsageBreakdown | null; succeeded: boolean;
}

/** Talk-only foundation. Helper execution and approvals remain in P6-06. */
export class Bridge {
  private active: { threadId: string; turnId: string } | null = null;
  private busy = false;
  private poisoned = false;
  private ready = false;
  private exhausted = false;
  private readonly modelMap: z.infer<typeof models>;
  private readonly ledger: UsageRecord[] = [];

  constructor(private readonly rpc: RpcClient, config: unknown, private readonly catalog: unknown, private readonly rpcTimeoutMs: number, private readonly maxUsageEntries: number, private readonly workRoot: string) {
    this.modelMap = models.parse(config);
    if (!Number.isSafeInteger(maxUsageEntries) || maxUsageEntries < 1) throw new Error('usage_capacity');
  }

  async initialize(): Promise<void> {
    if (this.ready || this.poisoned) throw new Error('bridge_lifecycle');
    await this.rpc.request('initialize', {
      clientInfo: { name: 'nemu-codex-bridge', title: 'Nemu', version: '0.0.0' },
      capabilities: { experimentalApi: true, requestAttestation: false },
    }, object, this.rpcTimeoutMs);
    this.rpc.initialized();
    await this.checkInventory();
    this.ready = true;
  }

  private async checkInventory(threadId?: string): Promise<void> {
    const result = await this.rpc.request('mcpServerStatus/list', {
      ...(threadId === undefined ? {} : { threadId }), limit: 100,
    }, inventoryResult, this.rpcTimeoutMs);
    if (result.data.length !== 0 || result.nextCursor !== null) throw new Error('mcp_inventory_not_empty');
  }

  usage(): readonly UsageRecord[] { return structuredClone(this.ledger); }

  async steer(text: string): Promise<void> {
    if (!this.active || this.poisoned) throw new Error('no_active_turn');
    await this.rpc.request('turn/steer', {
      threadId: this.active.threadId, expectedTurnId: this.active.turnId,
      input: [{ type: 'text', text, text_elements: [] }],
    }, object, this.rpcTimeoutMs);
  }

  async interrupt(): Promise<void> {
    if (!this.active) throw new Error('no_active_turn');
    await this.rpc.request('turn/interrupt', this.active, object, this.rpcTimeoutMs);
  }

  async run(input: RunInput): Promise<TurnReceipt> {
    if (!this.ready || this.poisoned) throw new Error('bridge_unavailable');
    if (this.busy) throw new Error('bridge_busy');
    if (this.exhausted) throw new Error('quota_exhausted');
    if (this.ledger.length >= this.maxUsageEntries) throw new Error('usage_capacity');
    if (!Number.isFinite(input.deadlineMs) || input.deadlineMs <= 0 || input.deadlineMs > 2_147_483_647 ||
      !Number.isSafeInteger(input.maxOutputBytes) || input.maxOutputBytes < 1) throw new Error('run_budget');
    const model = this.modelMap[input.tier];
    checkCatalog(this.catalog, model);
    this.busy = true;
    let workdir: string | null = null;
    let threadId: string | null = null;
    let turnId: string | null = null;
    let eventTurnId: string | null = null;
    let output = '';
    let finalText: string | null = null;
    let firstDeltaMs: number | null = null;
    let tokens: TokenUsageBreakdown | null = null;
    let succeeded = false;
    let notificationFailure: Error | null = null;
    let turnStarted = 0;
    let completedMs = 0;
    const started = performance.now();
    const remaining = (): number => {
      const value = input.deadlineMs - (performance.now() - started);
      if (value <= 0) throw new Error('turn_timeout');
      return Math.min(value, this.rpcTimeoutMs);
    };
    let completed!: () => void;
    let rejected!: (reason: Error) => void;
    const done = new Promise<void>((resolve, reject) => { completed = resolve; rejected = reject; });
    // Notifications may arrive before turn/start's response. Keep their original IDs.
    void done.catch(() => {});
    const checkIdentity = (actualThread: string, actualTurn: string): void => {
      if (actualThread !== threadId || (eventTurnId !== null && actualTurn !== eventTurnId) ||
        (turnId !== null && actualTurn !== turnId)) throw new Error('turn_identity_mismatch');
      eventTurnId = actualTurn;
    };
    const checkOutput = (text: string): void => {
      if (Buffer.byteLength(text) > input.maxOutputBytes) throw new Error('output_limit');
    };
    const listen = (event: Message): void => {
      try {
        if (event.id !== undefined || event.method === 'bridge/disconnected') throw new Error('unsafe_or_disconnected_server');
        if (event.method === 'rawResponseItem/completed') {
          const raw = object.parse(event.params?.item);
          if (typeof raw.type !== 'string' || raw.type.endsWith('_call') || raw.type === 'mcp_list_tools') throw new Error('tool_surface_violation');
        }
        if (event.method === 'model/rerouted') throw new Error('model_rerouted');
        if (event.method === 'item/started' || event.method === 'item/completed') {
          const parsed = item.parse(event.params);
          checkIdentity(parsed.threadId, parsed.turnId);
          if (!['agentMessage', 'reasoning'].includes(String(parsed.item.type))) throw new Error('tool_surface_violation');
          if (event.method === 'item/completed' && parsed.item.type === 'agentMessage' &&
            (parsed.item.phase === 'final_answer' || parsed.item.phase === null || parsed.item.phase === undefined)) {
            if (typeof parsed.item.text !== 'string') throw new Error('invalid_final_message');
            checkOutput(parsed.item.text); finalText = parsed.item.text;
          }
        }
        if (event.method === 'item/agentMessage/delta') {
          const parsed = delta.parse(event.params); checkIdentity(parsed.threadId, parsed.turnId);
          if (firstDeltaMs === null) firstDeltaMs = performance.now() - turnStarted;
          output += parsed.delta; checkOutput(output); input.onDelta?.(parsed.delta);
        }
        if (event.method === 'thread/tokenUsage/updated') {
          const parsed = tokenEvent.parse(event.params); checkIdentity(parsed.threadId, parsed.turnId);
          tokens = parsed.tokenUsage.last;
        }
        if (event.method === 'turn/completed') {
          const parsed = completion.parse(event.params); checkIdentity(parsed.threadId, parsed.turn.id);
          if (['usageLimitExceeded', 'rateLimitExceeded', 'sessionBudgetExceeded'].includes(String(parsed.turn.error?.codexErrorInfo))) this.exhausted = true;
          if (parsed.turn.status !== 'completed') throw new Error(this.exhausted ? 'quota_exhausted' : `turn_${parsed.turn.status}`);
          completedMs = performance.now() - turnStarted;
          completed();
        }
      } catch (error) {
        notificationFailure = error instanceof Error ? error : new Error('invalid_notification');
        rejected(notificationFailure);
      }
    };
    const unsubscribe = this.rpc.subscribe(listen);
    const timer = setTimeout(() => rejected(new Error('turn_timeout')), input.deadlineMs);
    try {
      workdir = await mkdtemp(join(await realpath(this.workRoot), 'irl-'));
      const thread = await this.rpc.request('thread/start', strictThread(model, workdir, input.instructions), threadResult, remaining());
      threadId = thread.thread.id;
      if (notificationFailure !== null) throw notificationFailure;
      turnStarted = performance.now();
      const response = await this.rpc.request('turn/start', {
        threadId, model, effort: input.effort, approvalPolicy: 'untrusted', approvalsReviewer: 'user',
        sandboxPolicy: { type: 'readOnly', networkAccess: false }, environments: [],
        input: [{ type: 'text', text: input.text, text_elements: [] }], outputSchema: input.outputSchema,
      }, turnResult, remaining());
      turnId = response.turn.id;
      if (eventTurnId !== null && eventTurnId !== turnId) throw new Error('turn_identity_mismatch');
      this.active = { threadId, turnId };
      await done;
      await this.checkInventory(threadId);
      // Include post-turn inventory in the request deadline, but keep inference timing separate.
      remaining();
      const text = finalText ?? output;
      if (!text) throw new Error('empty_output');
      const receipt: TurnReceipt = {
        threadId, turnId, tier: input.tier, model, text, tokens,
        completedMs, firstDeltaMs,
        toolCalls: 0, mcpServersAfter: 0,
      };
      succeeded = true;
      return structuredClone(receipt);
    } catch (error) {
      // No implicit resubmission: an uncertain/disconnected request is not run twice.
      this.poisoned = true;
      await this.rpc.close();
      throw error;
    } finally {
      if (threadId !== null) this.ledger.push({ tier: input.tier, model, threadId, turnId: turnId ?? eventTurnId, tokens, succeeded });
      clearTimeout(timer); unsubscribe(); this.active = null; this.busy = false;
      // Remove only our empty scratch directory; unexpected contents remain for diagnosis.
      if (workdir !== null) await rmdir(workdir);
    }
  }

  async close(): Promise<void> { this.ready = false; await this.rpc.close(); }
}
