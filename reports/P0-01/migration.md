# Legacy UI inventory and migration

Source confirmed as the candidate by the user in this task:
`C:\Users\nanake\Documents\ChatGPT\10-10\R1_recv_20261010_01\nemu-agent`.
It declares itself a React + TypeScript + Vite + Electron Mock Agent prototype.
The source was inspected read-only; no personal content, logs, recordings, credentials,
session data, or character assets were imported.

| Inspected source | SHA-256 | Decision |
| --- | --- | --- |
| src/components/TaskParts.tsx | 473fbf0f2dc207e74a195f62d093941d019747fbdac8d83e3541300ac15e1f7d | Adapt Composer only |
| src/components/CompanionView.tsx | 1be9b36c85ad65de7777b87c79acbb06768badcf14f8f1bae86fe82b45686da0 | Do not migrate its session/controller/Live2D dependencies |

The old Composer reads a session store and calls submitTask on a controller.
The migrated apps/panel/src/legacy/Composer.tsx retains controlled draft input,
empty-input gating, form submission and IME-safe Enter handling, expressed solely
as UI props/callbacks. Shift+Enter creates a newline. The mounted P0 panel keeps
sending disabled because no Echo Core chat contract is implemented yet.

Verification: Composer.test.tsx checks escaping and submission gating;
Electron smoke checks the rendered draft field and disabled send button.
No claim of end-to-end chat or action execution is made.

Excluded: mock agent and scenarios; session stores/reducers/persistence;
legacy IPC contracts and approval decisions; task execution UI; character state;
Live2D runtime/assets; prototype build/dependency configuration.
No extra runtime dependency was introduced by migration (React 19 was already required).
Original local files remain untouched. Nothing was deleted from the prototype.
