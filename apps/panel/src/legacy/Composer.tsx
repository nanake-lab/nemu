// Adapted from the user-owned nemu-agent TaskParts.tsx Composer; see migration report.
// UI props only: no legacy controller, persistence, mock agent, or IPC contract.
import type { KeyboardEvent } from 'react';

export interface ComposerProps {
  draft: string;
  setDraft: (value: string) => void;
  onSubmit: (text: string) => void;
  disabled?: boolean;
}

export function Composer({ draft, setDraft, onSubmit, disabled = false }: ComposerProps) {
  const submit = () => {
    if (!disabled && draft.trim()) onSubmit(draft);
  };
  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === 'Enter' && !event.shiftKey && !event.nativeEvent.isComposing) {
      event.preventDefault();
      submit();
    }
  };
  return (
    <form
      className="composer"
      onSubmit={(event) => {
        event.preventDefault();
        submit();
      }}
    >
      <label htmlFor="chat-draft">Message</label>
      <textarea
        id="chat-draft"
        value={draft}
        rows={2}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={onKeyDown}
        autoComplete="off"
      />
      <button type="submit" disabled={disabled || !draft.trim()}>
        Send
      </button>
    </form>
  );
}
