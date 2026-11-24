import { createHash, timingSafeEqual } from 'node:crypto';
import { appBusEventSchema, EchoCore, type AppBusPayloads } from '@nemu/contracts';
import { wireInputSchema } from '../../../../packages/contracts/src/generated/echo/browser.js';
import type { ChatEvent, TerminalClient } from '@nemu/terminal';
import type { AppBus } from '../bus/bus.js';

const outputChunk = wireInputSchema('OutputChunk');
const outputEnd = wireInputSchema('OutputEnd');
const runState = wireInputSchema('RunStateV2');
const approvalRequest = wireInputSchema('ApprovalRequestV2');
const outputCancel = wireInputSchema('OutputCancel');
const {
  ApprovalChoice,
  IngressKind,
  OutputEndReason,
  Autonomy,
  RunStatus,
  SpeechMode,
  TerminalLineage,
} = EchoCore.schema.enums;

function enumName(values: Readonly<Record<string, number>>, value: number): string {
  return Object.keys(values).find((name) => values[name] === value) ?? String(value);
}

/** Application-side consumers for existing IDL messages. P5-53 supplies the
 * transport demultiplexer; the current gateway sends only paired replies. */
export function bindTerminal(
  bus: AppBus,
  terminal: Pick<TerminalClient, 'enqueue' | 'sendControl'>,
  report: (reason: string) => void,
) {
  const outputs = new Map<string, { last: bigint; stopped: boolean }>();
  const approvals = new Map<string, EchoCore.ApprovalRequestV2>();
  function publish<K extends keyof AppBusPayloads>(type: K, payload: AppBusPayloads[K]) {
    bus.publish(bus.createEvent('main', type, payload));
  }

  const unsubscribe = bus.subscribe(['chat.send', 'approval.decision.v2'], (input) => {
    const parsed = appBusEventSchema.safeParse(input);
    if (!parsed.success || parsed.data.source !== 'panel') return;
    const event = parsed.data;
    if (event.type === 'chat.send') {
      if (event.payload.text.trim()) terminal.enqueue(event satisfies ChatEvent);
    } else if (event.type === 'approval.decision.v2') {
      const request = approvals.get(event.payload.approvalId);
      if (!request || request.expires_at_ns <= BigInt(Date.now()) * 1_000_000n) return;
      if (!timingSafeEqual(request.payload_digest, event.payload.payloadDigest)) return;
      if (
        request.tier === EchoCore.schema.enums.ToolTier!.T3 &&
        event.payload.decision === 'allowAlways'
      )
        return;
      const choices = {
        allowOnce: ApprovalChoice!.ALLOW_ONCE,
        allowAlways: ApprovalChoice!.ALLOW_ALWAYS,
        deny: ApprovalChoice!.DENY,
      };
      approvals.delete(request.approval_id);
      void terminal.sendControl({
        kind: IngressKind!.INGRESS_APPROVAL as EchoCore.IngressKind,
        approval: {
          schema_version: 1,
          approval_id: request.approval_id,
          decision: choices[event.payload.decision] as EchoCore.ApprovalChoice,
          payload_digest: request.payload_digest,
        },
      });
    }
  });

  return {
    close: unsubscribe,

    disconnected() {
      approvals.clear();
      for (const [id, output] of outputs) {
        if (!output.stopped) {
          output.stopped = true;
          publish('output.state', { outputId: id, state: 'cancelled' });
        }
      }
    },

    async cancel(input: unknown) {
      const parsed = outputCancel.safeParse(input);
      if (!parsed.success) return;
      const id = parsed.data.output_id;
      const output = outputs.get(id);
      if (!output || output.stopped) return;
      // Freeze the visible prefix immediately, even while the wire ACK is in flight.
      output.stopped = true;
      publish('output.state', { outputId: id, state: 'cancelled' });
      await terminal.sendControl({
        kind: IngressKind!.INGRESS_CANCEL as EchoCore.IngressKind,
        cancel: parsed.data,
      });
    },

    chunk(input: EchoCore.OutputChunk) {
      const chunk = outputChunk.parse(input);
      if (chunk.chunk_index > BigInt(Number.MAX_SAFE_INTEGER)) {
        report('Echo Core supplied an unsupported output chunk index');
        return;
      }
      const old = outputs.get(chunk.output_id);
      if (old && (old.stopped || chunk.chunk_index <= old.last)) return;
      outputs.set(chunk.output_id, { last: chunk.chunk_index, stopped: false });
      if (!old) publish('output.state', { outputId: chunk.output_id, state: 'streaming' });
      publish('output.chunk', {
        outputId: chunk.output_id,
        chunkIndex: Number(chunk.chunk_index),
        text: chunk.text,
        cues: [],
      });
    },

    end(input: EchoCore.OutputEnd) {
      const end = outputEnd.parse(input);
      const old = outputs.get(end.output_id);
      outputs.set(end.output_id, { last: old?.last ?? -1n, stopped: true });
      if (end.reason === OutputEndReason!.ERROR) report('Echo Core reported an output failure.');
      if (!old?.stopped) {
        publish('output.state', {
          outputId: end.output_id,
          state: end.reason === OutputEndReason!.DONE ? 'done' : 'cancelled',
        });
      }
    },

    runState(input: EchoCore.RunStateV2) {
      const state = runState.parse(input);
      const speechMode = enumName(
        SpeechMode!,
        state.speech_mode,
      ) as AppBusPayloads['core.runState']['speechMode'];
      publish('core.runState', {
        states: state.states.map((s) => enumName(RunStatus!, s)),
        speechMode,
        strict: state.strict,
        lineage: state.lineage === TerminalLineage!.TERMINAL_LIFE ? 'LIFE' : 'REHEARSAL',
        ...(state.born_at !== undefined ? { bornAt: state.born_at } : {}),
        autonomy: state.autonomy === Autonomy!.ACTIVE_AUTONOMY ? 'active' : 'paused',
        reasons: state.reasons,
      });
    },

    approval(input: EchoCore.ApprovalRequestV2) {
      const request = approvalRequest.parse(input);
      const digest = createHash('sha256').update(request.action_payload).digest();
      if (
        !timingSafeEqual(digest, request.payload_digest) ||
        request.expires_at_ns <= BigInt(Date.now()) * 1_000_000n
      )
        return;
      approvals.set(request.approval_id, {
        ...request,
        payload_digest: Uint8Array.from(request.payload_digest),
      });
      publish('approval.request.v2', {
        approvalId: request.approval_id,
        actionId: request.action_id,
        tier: enumName(
          EchoCore.schema.enums.ToolTier!,
          request.tier,
        ) as AppBusPayloads['approval.request.v2']['tier'],
        title: request.title,
        detail: request.detail,
        expiresAt: Number(request.expires_at_ns / 1_000_000n),
        actionPayload: Uint8Array.from(request.action_payload),
        payloadDigest: Uint8Array.from(request.payload_digest),
      });
    },
  };
}
