# P0-06 Echo Core baseline and difference register — report

## 结果

部分完成：independent W00 deliverables and local verification complete; **user confirmation of W00-C01/W00-C02 is blocking** (see [BLOCKED.md](BLOCKED.md)). No new growth semantics adopted. P0-02 was completed and merged as PR #2, main `ab3c6d175f0c4ea156aa9a8baff429e741d3eb49`; post-merge [CI passed](https://github.com/nanake-lab/nemu/actions/runs/38504208518). P0-06 started on that clean synchronized main.

## 交付物

- `docs/traceability/baseline.md`: append-only verified five-original baseline, precedence and missing-source limits.
- `docs/traceability/diff-register.md`: append-only E1–E14 and two unresolved conflicts, original file/section references.
- `docs/traceability/build-lock.md`: measured Windows/compiler environment and bundled SQLite fingerprint.
- `docs/traceability/scope-v1.md`: all W packages, operator/system/invariant IDs, earlier acceptance and attack-example routing; explicit v1.x boundaries.
- `docs/traceability/P0-06.md`: specification mapping.
- `docs/adr/0006-single-root-spec-conflict.md`: proposed options, not an adopted decision.
- `reports/P0-06/verify-baseline.ps1`: repeatable hash/routing/build verification with new evidence directory per run.
- `reports/P0-06/sqlite-probe/`: locked isolated build probe, not an application dependency or persistence implementation.

## 验收

| Command / check | Result | Evidence |
| --- | --- | --- |
| Get-FileHash for five DOCX originals | Passed, matched baseline; originals unchanged | verification-20261014T031717034Z/originals.json |
| Original title and conflict inspection (read-only ZIP/XML) | Passed; both single-root wordings confirmed in originals | originals.json; baseline and register source sections |
| E1–E14 presence and original citations | Passed scripted presence and manual file/section review | diff-register.md; verify-baseline.ps1 |
| All W00–W23, F/G/C/J/X, SYS01–SYS25, I01–I22 routed | Passed inventory; not execution of future acceptance | verification-20261014T031717034Z/checks.json; scope-v1.md |
| cargo +1.99.0 fmt … --check | Passed (no output) | verification-20261014T031717034Z/checks.json |
| cargo +1.99.0 clippy --locked … -- -D warnings | Passed | verification-20261014T031717034Z/probe-clippy.log |
| cargo +1.99.0 run --locked … | Passed: SQLite 3.53.2, WAL, FULL, rollback, integrity, temporary directory removed | verification-20261014T031717034Z/probe-run.log |
| git diff --check | Passed (no output) | verification-20261014T031717034Z/checks.json |
| User confirms new conflicts | **Pending / blocking** | BLOCKED.md; proposed ADR 0006 |
| Current PR full repository CI | Pending submission; final checkpoint records observed result | PAUSE_CHECKPOINT.md |

P0-06 changes no application, Rust core or Python project code. Existing repository language checks run in PR CI; the added report probe was independently formatted, linted and executed locally. No core behavioral acceptance is claimed by this card, and no coverage threshold is substituted for future acceptance. P0-02 evidence remains intact: 197 parser cases, 312 contract tests, performance.ts lines 104/104 (100%), all required checks and CI passed in its report.

## 规格对照

See [P0-06 mapping](../../docs/traceability/P0-06.md). I01–I22 enforcement responsibility is listed in scope-v1.md; none is newly implemented or certified here. No IDL messages/golden vectors changed. Original documents and prior reports are preserved. The Echo Core skill influenced this work by requiring conservative no-growth handling and blocking review of ambiguity rather than an agent-selected resolution.

## 新增依赖

No application runtime dependency added. Isolated report probe only:

| Name | Version | License | Purpose |
| --- | --- | --- | --- |
| rusqlite | 0.40.2 | MIT | Verify selected bundled SQLite build |
| libsqlite3-sys | 0.38.2 | MIT | Bundled SQLite 3.53.2 (upstream public domain) |

All transitive names/versions/licenses are recorded in `verification-20261014T031717034Z/probe-dependencies.json`; Cargo.lock pins checksums. Build probe is not a durability/concurrency acceptance suite. No personal files, credentials or user storage are included. No file added by this task exceeds 400 lines.

## 待用户确认

- [ ] **阻塞 W00-C01:** approve an option in proposed ADR 0006 concerning the frozen single-root branch versus later multi-root-only closure.
- [ ] **阻塞 W00-C02:** confirm treating S §32's “24” as a count typo while preserving all 25 SYS IDs.
- Existing E5/E8/E13/E14 approvals are retained, not reopened.

## 后续建议（不在本任务范围内）

- P5-02 should adopt the locked SQLite selection into core-store and perform crash/commit/pressure testing; this probe cannot establish those claims.
- Future growth work requires upstream Review_and_Remediation material and an explicit resolution of W00-C01.

## 下一步

PR must carry `needs-user` and remain unmerged. Await the two decisions above; record approval append-only, update branch against latest main, rerun relevant checks and CI, then merge only if all AGENTS conditions hold. Next queue card after P0-06 merge is P5-01 (also depends on already merged P0-02). Do not bypass this approval by starting its dependent work.
