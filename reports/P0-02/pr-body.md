## P0-02 — Application contracts

Implements CONTRACTS 1, 2, 3, 4.1 and 7 within `packages/contracts`, with no new wire fields or handwritten Echo Core DTOs. Includes all 30 motion slots, channel-specific stage validation, all app-bus payloads, profile/readout schemas, and a lazy streaming performance parser. The generated directory remains a P5-01 placeholder.

Acceptance: 197 parser cases (minimum 60), 100% performance.ts line coverage (minimum 90%), positive/negative fixtures for every exported schema and every stage/app-bus payload. Full workspace has 324 passing tests. Install, lint, typecheck, build, format, Cargo workspace tests and all four Python tests pass locally. CI retains all previous checks and adds the contracts coverage gate/artifact.

Runtime dependency: Zod 4.6.7, MIT. Development dependency: @vitest/coverage-v8 5.0.4, MIT. No architecture change, ADR, baseline update or blocked confirmation. Parser warning sink is injected for the future consumer's @nemu/log integration; no text is logged.

Report and evidence: [reports/P0-02/REPORT.md](reports/P0-02/REPORT.md), [coverage](reports/P0-02/coverage/coverage-summary.json), [validation](reports/P0-02/validation/checks.json). API/edge-case details: [packages/contracts/README.md](packages/contracts/README.md).

After final-head CI is green and main is current, squash merge under AGENTS.md and continue to the next eligible task (expected P0-06). Final run links and merge disposition will be recorded in this PR.
