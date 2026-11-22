import { useEffect, useRef, useState } from 'react';
import { appBusPayloads, type AppBusPayloads } from '@nemu/contracts';
import { globalDebugBusClient } from '../debug/bus-client.js';
import { Composer } from '../legacy/Composer.js';
import type { ChatPanelChannel, ChatStatus } from './channel.js';
import './chat.css';

import { appendChunk, Transcript, type ChatMessage } from './Transcript.js';
type Approval = AppBusPayloads['approval.request.v2'];
type Choice = AppBusPayloads['approval.decision.v2']['decision'];

export function Chat({ channel = window.nemuChat }: { channel?: ChatPanelChannel } = {}) {
  const [draft, setDraft] = useState('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const stopped = useRef(new Set<string>());
  const [active, setActive] = useState<string[]>([]);
  const [status, setStatus] = useState<ChatStatus>({
    canSend: false,
    reason: 'Echo Core is not connected.',
  });
  const [coreReason, setCoreReason] = useState('');
  const [approvals, setApprovals] = useState<Approval[]>([]);

  useEffect(
    () =>
      channel?.onStatus((next) => {
        setStatus(next);
        if (next.disconnected) {
          setCoreReason('');
          setApprovals([]);
        }
      }),
    [channel],
  );

  useEffect(
    () =>
      globalDebugBusClient.subscribe((event) => {
        if (event.direction !== 'in' || event.source !== 'main') return;
        if (event.type === 'output.chunk') {
          const result = appBusPayloads['output.chunk'].safeParse(event.payload);
          if (!result.success) return;
          if (stopped.current.has(result.data.outputId)) return;
          setMessages((rows) => appendChunk(rows, result.data));
        } else if (event.type === 'output.state') {
          const result = appBusPayloads['output.state'].safeParse(event.payload);
          if (!result.success) return;
          const { outputId, state } = result.data;
          setActive((ids) =>
            state === 'streaming'
              ? [...ids.filter((id) => id !== outputId), outputId]
              : ids.filter((id) => id !== outputId),
          );
        } else if (event.type === 'core.runState') {
          const result = appBusPayloads['core.runState'].safeParse(event.payload);
          if (result.success) {
            setCoreReason(
              result.data.reasons
                .map((reason) =>
                  reason === 'IRL_FAILED'
                    ? 'Echo Core could not interpret the input (IRL_FAILED).'
                    : reason,
                )
                .join(' · '),
            );
          }
        } else if (event.type === 'approval.request.v2') {
          const result = appBusPayloads['approval.request.v2'].safeParse(event.payload);
          if (result.success) {
            setApprovals((items) => [
              ...items.filter((item) => item.approvalId !== result.data.approvalId),
              result.data,
            ]);
          }
        }
      }),
    [],
  );

  function send(text: string) {
    const event = globalDebugBusClient.sendStageCommand('chat.send', { text });
    setMessages((rows) => [...rows, { id: event.id, speaker: 'You', text }]);
    setDraft('');
  }

  function decide(request: Approval, decision: Choice) {
    if (request.expiresAt <= Date.now()) {
      setCoreReason('This approval request has expired.');
    } else {
      const payload: AppBusPayloads['approval.decision.v2'] = {
        approvalId: request.approvalId,
        decision,
        payloadDigest: request.payloadDigest,
      };
      globalDebugBusClient.sendStageCommand('approval.decision.v2', payload);
    }
    setApprovals((items) => items.filter((item) => item.approvalId !== request.approvalId));
  }

  const outputId = active.at(-1);
  return (
    <section className="chat" aria-label="Conversation">
      <Transcript messages={messages} />
      {approvals.map((request) => (
        <section key={request.approvalId} aria-label="Approval request">
          <h2>{request.title}</h2>
          <p>{request.detail}</p>
          <details>
            <summary>Action payload (hex)</summary>
            <code>
              {Array.from(request.actionPayload, (byte) => byte.toString(16).padStart(2, '0')).join(
                '',
              )}
            </code>
          </details>
          <button type="button" onClick={() => decide(request, 'allowOnce')}>
            Allow once
          </button>
          {request.tier !== 'T3' && (
            <button type="button" onClick={() => decide(request, 'allowAlways')}>
              Always allow
            </button>
          )}
          <button type="button" onClick={() => decide(request, 'deny')}>
            Deny
          </button>
        </section>
      ))}
      <p className="chat-status" role="status">
        {status.reason ?? coreReason}
      </p>
      {channel && outputId && (
        <button
          type="button"
          onClick={() => {
            stopped.current.add(outputId);
            setActive((ids) => ids.filter((id) => id !== outputId));
            channel.cancel({ schema_version: 1, output_id: outputId });
          }}
        >
          Stop
        </button>
      )}
      <Composer draft={draft} setDraft={setDraft} onSubmit={send} disabled={!status.canSend} />
    </section>
  );
}
