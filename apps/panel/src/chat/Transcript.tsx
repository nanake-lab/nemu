import type { AppBusPayloads } from '@nemu/contracts';

export type ChatMessage = { id: string; speaker: 'You' | 'Nemu'; text: string };

export function appendChunk(
  rows: ChatMessage[],
  { outputId, text }: AppBusPayloads['output.chunk'],
): ChatMessage[] {
  if (!rows.some((row) => row.speaker === 'Nemu' && row.id === outputId)) {
    return [...rows, { id: outputId, speaker: 'Nemu', text }];
  }
  return rows.map((row) =>
    row.speaker === 'Nemu' && row.id === outputId ? { ...row, text: row.text + text } : row,
  );
}

export function Transcript({ messages }: { messages: ChatMessage[] }) {
  return (
    <ol className="chat-messages" aria-label="Messages" aria-live="polite">
      {messages.map((message) => (
        <li key={message.speaker + message.id}>
          <strong>{message.speaker}</strong>
          <div>{message.text}</div>
        </li>
      ))}
    </ol>
  );
}
