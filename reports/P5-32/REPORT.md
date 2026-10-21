# P5-32 契约变更：R 的典型幅度递推与分配重放字段 — 报告

## 结果

完成，等待审查与你的批准。按 ADR 0016 在 IDL 里补了 G 需要的五个字段：倾向的 `magnitude_sum`、`magnitude_weight`，分配的 `ordinal`、`mu`、`projection`。实现方式是后继消息 TendencyV2、EvidenceAllocationV2、EvidenceGroupV2、RSnapshotV2。已发布的消息、字段号、类型标签和黄金向量都没有变。三种语言各 9,696 个用例结论一致。

PR #52 标 `needs-user`，按你的要求由你合并。合并前需要 Codex 交叉审查（GPT-6 Astra，最高推理强度）。

## 交付物

- `docs/adr/0016-law-replay-fields.md`（新）：
  - 为什么从已有字段推不出来；
  - 五个字段与后继消息；
  - 两条语义检查；
  - 重放的完整依据与 I23；
  - 按 P5-23 D5 的三种情况，逐一核对字段是否最小且够用。
- `tasks/P5-echo-core.md`：末尾追加 P5-32 卡。
- `tasks/cards.json`：追加 P5-32 条目，你 2026-10-22 批准。
- `core/contracts/law.proto`：
  - 新增 TendencyV2、EvidenceAllocationV2、EvidenceGroupV2、RSnapshotV2；
  - 旧的 Tendency、EvidenceAllocation、EvidenceGroup、RSnapshot 标为取代；
  - EvidenceGroupV2 保留 7，RSnapshotV2 保留 4。
- 生成的代码：`core-types/src/generated.rs`、`core/contracts/generated/schema.json`、`packages/contracts/src/generated/echo/schema.ts`、`workers/{irl,drive,model-adapter}/generated/echo/`。
- `core/crates/core-types/src/semantic.rs`：
  - `check_tendency_v2`：原有的计数不变量，再加 U = 0 ⇒ M = 0；
  - `check_allocation_ordinals`：一个倾向的分配序号正好是 0…N − 1。
- `core/crates/core-types/tests/p5_32_contract.rs`（新）：后继消息的字段表、两条语义检查的正反例、往返编码。`v2_contract.rs` 的取代登记计数由 32 改为 36。
- `core/contracts/baselines/p5-21/`（新）：冻结 P5-21 发布的 `.proto`（main `9042c59`）和黄金向量摘要。以后 check_compat 也拿它比较。
- `core/contracts/fixture_inputs.py`：P5-32 的定向向量，5 个合法、9 个非法。
- `core/contracts/golden/`：只追加，新增 17 个合法、21 个非法向量。
- `docs/traceability/P5-32.md`（新）：规格对照表。`docs/traceability/diff-register.md` 追加 V2-50。
- `reports/P5-32/verification/`：全部日志。

## 验收

本地工具链：rustc / cargo 1.97.0 stable（锁定的 1.99.0 下载被代理拒绝，按 AGENTS.md 第 10 节用 `RUSTUP_TOOLCHAIN=stable`），Node 22.22.0（仓库要求 24），pnpm 12.11.2，uv 0.11.32。契约工具用 `core/contracts/uv.lock` 的 CPython 3.12。结论以 CI 按锁定版本运行的结果为准：本 PR 的 Ubuntu CI 在 `2eddd77` 上通过。

| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | 通过，0 个生成文件过期 | verification/core-contracts.log |
| `pnpm test:core-contracts`（施工合同第一篇 10.8，SYS21） | 通过：Rust、Python、TypeScript 各 9,696 个用例（2,248 合法、7,448 拒绝）；官方 protoc 互操作 395 条；124 种消息每种至少 3 合法、3 非法 | verification/core-contracts.log（本地用 venv 的 python 直接跑 verify.py：沙箱里 pnpm 的 shim 不能被 Python 子进程执行，命令内容相同） |
| 不复用检查 | 通过：对 `p5-01`、`p5-21` 两个基线和当前 IDL 都是 0 违例，自检不漏报 | 同上 |
| 不复用检查的人为违例 | 三种注入都报出：EvidenceGroupV2 去掉 reserved 7 并复用 7；TendencyV2 改字段 13；改已发布的 Tendency 字段 13（只有新的 p5-21 基线能抓到这一条） | verification/check-compat-mutations.log |
| SYS21：越界的 μ、负的 magnitude_weight、溢出的 ordinal、缺少必填字段、重复的集合键、嵌入被取代的类型 | 通过：9 个定向非法向量，加上每种新消息 3 个通用非法向量 | `golden/illegal.json` |
| 黄金向量逐字节核对 | 通过：冻结部分与 `origin/main` 的文本前缀相同；新增的 17 个合法向量用独立编码器（`reports/P5-21/review_vectors.py`）逐个重算，0 不一致 | verification/review-vectors.log |
| 语义检查的正反例 | 通过 | `p5_32_contract.rs`，verification/rust-test.log |
| `cargo fmt --all -- --check` | 通过 | verification/rust-fmt.log |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 | verification/rust-clippy.log |
| `cargo test --workspace --locked` | 通过，51 个测试，0 失败 | verification/rust-test.log |
| `pnpm lint`、`typecheck`、`test`、`build`、`test:contracts` | 全部通过（vitest 443 个、契约 312 个） | verification/pnpm-*.log |
| 契约工具 ruff、mypy --strict | 通过（11 个源文件） | verification/contract-*.log |
| 四个 Python 项目的 ruff、mypy --strict、pytest；生成目录的 mypy | 全部通过 | verification/py-*.log、generated-mypy.log |
| 规格对照表 | `docs/traceability/P5-32.md` | — |
| 你批准这次契约变更 | **待批准（阻塞）** | — |

### 字段是否最小且够用：自查

ADR 0016 第 5 节按 P5-23 D5 的三种情况逐一列出：重放读什么、本步重算什么、需要哪些新字段、不需要什么。P5-23（PR #49，提交 `2568f8f`）已经用与这五个字段一一对应的临时结构实现了重放，作为可执行的核对：

- **重放不读 θ：** `law/replay.rs` 的 `replay(t, allocations, law, caretaker)` 只接受倾向的定义、存下的分配与组的可信、可核实，以及规律参数与 u* 的依恋先验，参数里没有 b、σ̂。
- **份额只取本步的 π：** 回溯分摊与容量移出后的重新分摊都在本步按当前的 π 计算（`plan_shares`、`fuse::capacity`），不读 π 的历史。
- **三种情况都有测试：**
  - 回溯分摊：`ev5_back_collection_reshares_old_groups_and_keeps_i25`；
  - 后来的成员：`late_member_updates_the_group_without_a_second_count`；
  - 容量移出：`ac2_21_capacity_removes_the_weakest_candidate_and_reshares_its_groups`。
- **随机世界里两条恒等式：** 40 个随机世界各 120 个事件，共 5,702 个组、7,498 个分配。每一步之后，每个倾向按存下的值重放都与存下的倾向逐位相同，每一边的份额之和都是 1。
- **测试能抓到错：** 把重放去掉时，这些测试失败。

## 规格对照（Echo Core 相关的卡必填）

逐条见 `docs/traceability/P5-32.md`。摘要：

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| v2.0 §5.2 典型幅度 r_k 的加权平均；ADR 0016 §1 | TendencyV2 字段 16、17 | `successors_are_live_and_predecessors_superseded`；非法向量 |
| v2.0 §5.2 计算次序与回溯分摊；P5-23 D1、D5；ADR 0016 §1、§4 | EvidenceAllocationV2 字段 6、7、8 | 往返测试；非法向量 |
| ADR 0007 §2；check_compat 规则 5、6 | EvidenceGroupV2、RSnapshotV2 | check_compat 0 违例；人为违例 |
| ADR 0016 §3 | `semantic::check_tendency_v2`、`check_allocation_ordinals` | `p5_32_contract.rs` |
| ADR 0007 §11（迁移） | 没有任何已发布字节被重新解释；R 还没有接入存储 | 黄金向量前缀逐字节相同 |

涉及的不变量（v2.0 第 17.1 节）：I23、I25 由本卡提供它们需要的存储字段，执行在 P5-23（G）和 P5-26（重算）。I28 不受影响，genesis 不可达这些新类型。

### 新增与改动的 IDL 消息

| 旧消息（标为取代） | 后继 | 字段 |
| --- | --- | --- |
| Tendency | TendencyV2（`echo.tendency-v2`） | 1–15 相同；16 `magnitude_sum`，17 `magnitude_weight` |
| EvidenceAllocation | EvidenceAllocationV2（`echo.evidence-allocation-v2`） | 1–5 相同；6 `ordinal`，7 `mu`，8 `projection`（可缺省） |
| EvidenceGroup | EvidenceGroupV2（`echo.evidence-group-v2`） | 1–6 相同；7 reserved；8 `allocations` |
| RSnapshot | RSnapshotV2（`echo.r-snapshot-v2`） | 1–3 相同；4 reserved；5 `tendencies` |

### 迁移说明（ADR 0007 第 11 条）

没有重新解释任何已发布的字节：改动都是新消息、新标签，以及对旧消息加取代标注，标注不进字节。R 还没有接入存储（P5-02、P5-29 未开工），排练存储和正式存储里都没有 R 的对象，所以不需要迁移，也不需要备份。`CoreTransactionV2.staged_r` 是对象引用，从此指向 RSnapshotV2 对象，消息本身不变。

## 新增依赖

无。

## 待用户确认

- [ ] （阻塞）批准本次契约变更与 ADR 0016。
- [ ] （阻塞）差异登记 V2-50（解决 V2-48）。

## 后续建议（不在本任务范围内）

- **P5-23：** 本卡合并后，把最新 main 合并进 P5-23，换成生成的类型，删去 `law/pending.rs`，重跑全部测试，然后转为可审查。
- **P5-02、P5-29：** 重放要读出一个倾向的全部分配，存储需要按倾向索引证据组。
- **下一张契约卡：** 照惯例把本卡发布的 IDL 冻结为基线 `baselines/p5-32/`。

## 下一步

等待 Codex 交叉审查和你的批准（合并由你做）。转为可审查后 Windows 作业会运行：本卡改了 `packages/contracts/src/generated/`，在 `windows.yml` 的路径列表里。接下来取的卡：无。本卡合并后，回到 P5-23 换上生成的类型。

## 2026-10-22 合并 main 后重跑（追加）

合并最新 main（#51：needs-user 的 PR 由用户合并，提交信息不带跳过 CI 的标记），merge，不 rebase，不强推。没有冲突，变化只在 `AGENTS.md` 与 `tools/status/`。合并后重跑：`verify.py`（三种语言各 9,696 个用例，protoc 395 条，check_compat 0 违例）、cargo fmt、clippy、test（51 个），以及 pnpm lint、typecheck、test、build、test:contracts，全部通过。日志在 `verification/after-main-merge-20261022/`。

按 #51 后的 AGENTS.md 第 6 节：本卡带 `needs-user`，由你合并，合并即批准。按你 2026-10-22 的指示，本卡转为可审查，请 Codex 交叉审查；审查通过后由你合并。
