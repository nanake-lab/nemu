# P5-36 θ 估计器状态的存储 — 报告

> **当前结论（2026-10-28 实施）：部分完成。** 内嵌契约、最小桥接及真实逐位恢复/续算已完成并通过专项验收；完整工作区测试受新增 **B3 存储消费者冲突**阻塞，保持草稿 #108，不转可审查、不写“请审查”。最新交付/验收/下一步见文末“2026-10-28 实施报告”。下面的准备阶段正文与旧证据原样保留，不再代表当前目录占用或实现状态。

> 2026-10-28 恢复准备增量：#95/#65 已合并，当前等待 #110 释放契约目录；桥接请求已收窄为两个新文件与一个模块入口，见 [BLOCKED 当前更新](BLOCKED.md#2026-10-28-当前状态更新) 和 [接口请求](../../tasks/requests/P5-36-dynamics-roundtrip.md)。本轮不改实现、不重跑基线；以下保留 2026-10-27 的报告与原始验证口径，旧测试不代表最新 main 组合通过。

## 结果

部分完成，见 [BLOCKED.md](BLOCKED.md)：已认领 [草稿 PR #108](https://github.com/nanake-lab/nemu/pull/108)，完成 ADR 0029 提案、完整状态字段核对、恢复接口请求和既有契约基线检查；后继 IDL、生成物、语义检查、黄金向量及真实内存往返尚未实施。

分支 `task/P5-36-theta-estimator-idl`，工作区 `D:\Dev\nemu-wt\P5-36`，来源为 fetch 后的 `origin/main@3f8f98e5ac3cba2f2b4718ca242a5a0c226d0fbf`。发布准备前再次 fetch，main 未变。P5-25 / #77 已合并；#95、#65 仍占用本卡实现目录，且真实往返缺少 core-kernel 接口。未扩大实现范围。

已完整读完用户要求的四份文件、Echo Core 技能、v2.0 全文、README §5 的本卡相关保留章节、ADR 0005/0007/0008–0011、0019 §8、本卡全部内容、指定差异条目和 P5-25 请求/METHODS 的全部追加。旧规格按 v2.0 及最新差异登记的取代关系读取。

## 交付物

- [ADR 0029 提案](../../docs/adr/0029-theta-estimator-storage.md)：逐项列出真实递推字段、参数摘要绑定、V2-95/96/97 的关系、只增不改与恢复约束。未批准，精确 IDL 分配仍待实施。
- [恢复接口请求](../../tasks/requests/P5-36-dynamics-roundtrip.md)：记录私有状态字段及缺失接口，提出最小桥接路径与两种处理选项；未修改 core-kernel。
- [规格对照](../../docs/traceability/P5-36.md)：区分既有基线测试与新契约未执行项。
- [verification/](verification/)：原始命令输出、退出码、依赖环境、PR 核对和 31 个现有 proto/黄金文件摘要。`baseline-observation.json` 仅为现状观察，不是新冻结基线。

本轮源码及运行配置未改。新增 ADR、追踪报告、请求和本报告属于 AGENTS 允许的追加材料；未改既有任务卡、既有规格、原文或差异登记。V2-107 至 V2-109 尚未发布条目。

## 验收

| 命令或检查（含本卡验收） | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | passed；0 generated files stale | `verification/generate-check.log`、`light-checks.json` |
| `python core/contracts/check_compat.py --ref origin/main` | passed；p5-01、p5-21、p5-32、current、origin/main 均 0 违例；137 messages（42 superseded）、51 enums | `verification/compatibility.log` |
| `pnpm test:core-contracts` | passed，exit 0；三语言各 10,766 例（合法 2,527，非法 8,239），官方 protoc 445 传输向量、8 项 schema 测试通过；有下述依赖预检查告警 | `verification/core-contracts.log`、`core-contracts.json` |
| P5-25 真实状态与新消息逐位 round trip | not_run；新消息与桥接均未实施 | B1、B2；接口请求 |
| 每种新消息 ≥3 合法、≥3 非法向量 | not_run；不得把旧案例总数算作此项通过 | `verification/acceptance.json` |
| 新基线冻结、ADR/契约批准 | not_run；未实施，未获本次契约批准 | ADR 0029、BLOCKED.md |
| `git diff --check`、修改范围核对 | passed；仅新增设计/请求/报告材料及改本卡报告 | `verification/scope-check.json` |
| TS 全仓 lint/typecheck/test、Rust 全工作区、worker pytest | not_run；本轮未修改这些实现，契约命令只运行其自身编译和探针 | 不签发完整实现验收 |

全部命令结果只覆盖未改动的 main 契约基线。`pnpm` 在脚本开始前自动尝试依赖校验/安装，因 `node_modules` 为目录联接报错；其原有 warn 模式继续运行了 verifier。此告警原样保留，没有修改设置来消除它。当前工作区与复用依赖工作区的 pnpm/uv 锁文件 SHA-256 相同，实际 TypeScript 6.0.3、Zod 4.6.7；不能据此声称完整依赖安装检查成功，见 `dependency-context.json`。本轮不重复运行已经完成的契约测试。

构建、venv、protoc、TS 输出和临时目录位于 `D:\Dev\cache\P5-36`，依赖联接见 `build-paths.json`。主代理串行执行编译和测试；开始前未检出 Runner.Worker 或其他列出的重负载，未运行 Electron、视觉或 GPU 检查，未访问排练或正式存储。

## 规格对照

| 规格条款 | 本轮对应位置 | 测试 |
| --- | --- | --- |
| AGENTS §2 目录互斥 | 未修改 contracts/types/generated，保留草稿 | PR 状态、完整变更文件清单 |
| ADR 0007 | 后继消息设计，旧定义未动 | 现有全部兼容基线 0 违例 |
| V2-98/103、I18 | ADR 0029 全字段表 | 新契约与真实往返 not_run |
| V2-95/96/97/100、I23 | ADR 0029 时间/数值/资源/报警关系 | 后继语义、续算对照 not_run |
| I29 | 专用临时目录 | 本轮契约探针，无用户存储 |

完整条款和 AC2 验收边界见 [规格对照表](../../docs/traceability/P5-36.md)。本轮未修改或签发 AC2-05/14/20/21 的运行验收。

## 新增依赖

无新增运行时或锁定依赖；使用现有锁文件的工具。`uv sync --project core/contracts --locked --offline` 仅建立独立测试 venv，原始日志和退出码已保存。

## 待用户确认

- [ ] （阻塞 B1）按目录互斥顺序释放 #95、#65 的重叠目录；本轮没有批准例外。
- [ ] （阻塞 B2）由所属模块交付完整状态桥接，或明确批准请求中的最小 core-kernel 实现范围。
- [ ] （阻塞）最终实现完成后的 ADR 0029 与契约变更批准、独立交叉审查。当前提案不等于已批准的最终契约。

## 后续建议（不在本任务范围内）

H 来源/停机记录认证、长期时间域、P5-26 独立完整 H 重算和 P5-29 外层原子发布仍归既定后继任务；本轮不扩大范围代做。

## 证据与发布说明

轻量 PR 证据由子代理按最新 `.codex/agents/routine-reporter.toml` 的 **GPT-6.1 Terra / max 显式参数试用**收集；未声称按角色名自动加载已验证。子代理只查询和整理证据，未测试、修改仓库、发布或判断验收；主代理核对后复制摘要、完整变更路径和补充查询，并作出阻塞判断。原始返回的保存位置、SHA-256 见 `pr-raw-manifest.json`。

P5-25 #77 的标签接口返回 null，未将 null 解释为空标签；补充的带/不带 `unverified` 搜索支持目前无该标签，保留接口失败边界。开放 PR 扫描未发现 ADR 0029 的路径占用，相关 diff-register patch 未使用 V2-107 至 V2-109；#95 的全文只出现既有预留说明。编号实施前和合并前须再核对。

最初两次 git push 均因 schannel TLS 握手失败，认领通过 GitHub 连接器提交等价文件树并建立草稿。本地 `6ee2aeda` 与远端 `bbcf0a4d` 的 tree 均为 `92661d2c8cb8e4196c6cde451898904cf298472c`，提交身份不同。后续 fetch 恢复，用普通 merge `b0072b8f` 合入远端认领提交，没有 rebase、强推或修改 TLS/全局配置；详见 `publication-history.json`。

## 下一步

保持草稿和 `needs-user`。目录释放并具备桥接后，先 fetch 并 merge 最新 main，重查任务追加、ADR/类型编号和 PR 占用，再实施后继消息、三语言校验/向量、冻结基线及真实往返。全部验收完成后才转可审查并评论“请审查”；本轮不满足此条件，不取下一卡。

## 2026-10-28 实施报告

### 结果

部分完成（见 [BLOCKED.md](BLOCKED.md) B3）。用户批准的内嵌方案、三路径 kernel 最小桥接、契约和真实逐位恢复/续算均已完成；完整工作区验收未通过，尚不能转可审查。

继续原分支 `task/P5-36-theta-estimator-idl` / PR #108，已 merge main `f24f1256259c998bc213a9dc93103fcfb9cfe89a`，合并提交 `34c81aea38211c1fb0ebce307dbc2d3525008c25`。实施后再次 fetch，main 未变化。#95/#65/#110 和原接口范围阻塞均已解除。

### 交付物

- `core/contracts/theta.proto`：5 个新消息，ThetaSnapshotV2 内嵌完整参数、参数摘要和估计器；旧 ThetaSnapshot 标为被取代，旧字段表、标签和向量字节保留。保留旧 5–9 号，新增 10–12 号。
- `core/crates/core-types/src/semantic/theta.rs` 和 Python/TypeScript 生成的 `echo/theta` 模块：一致的显式语义检查；通用结构解码不能替代它。完整 law/fuses 参数摘要经现有 CanonicalV1 类型域分离 SHA-256 产生。
- 已批准的 kernel 三路径：新增 `dynamics/storage.rs`、`dynamics/storage/tests.rs`，`dynamics/mod.rs` 只注册/导出及更新说明。未修改递推算法、字段可见性、存储、迁移或运行时接线；未增加 Cargo 依赖。
- `core/contracts/verify_theta_restore.py`：真实 C → 字节 → Python/TS 解码复编码 → Rust 真正恢复并续算，已接入 `pnpm test:core-contracts`；所有数据都是临时合成轨迹。
- `baselines/p5-35/`：冻结合入 #110 后、本卡实施前 8 个 proto 和 500 legal / 528 illegal / 63 ingress / 18 unicode 的摘要。新增向量仅追加 15 legal / 69 illegal，详见 [逐消息覆盖](implementation/message-coverage.json)。
- ADR 0029、接口请求、规格对照均只追加实施差异；V2-107–109 已登记。历史 `verification/` 不改写、不充当本次结果；新增日志在 [implementation/](implementation/) 和 [V2-109/](V2-109/)。

`storage/tests.rs` 超过 400 行（478 行）：它包含独立的全字段位比较、真实轨迹、完整续算/报警/故障边界及跨语言交换验证。保留在用户批准的唯一测试文件内，未为拆文件扩围。DynamicsState、DynamicsLaw、Detector、Run、RunningTime 的字段以完整解构检查，新增字段会强迫测试更新。

### 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | passed，0 stale | `implementation/contracts-acceptance.log` |
| `python core/contracts/check_compat.py`（所有已冻结基线） | passed；p5-01/21/32/34/35/current，0 违例 | 同上 |
| `pnpm test:core-contracts` | 三语言各 13,224 例（2,983 legal、10,241 illegal）、官方 protoc 515 向量、14 schema 测试；真实交换验证见下行 | `implementation/contracts-acceptance.log`、`contracts-acceptance-result.json` |
| V2-109 真实逐位恢复 + 续算 | passed；65 前缀 × 2 种格式；Python/TS 返回 260 份状态逐一恢复；状态/ThetaChange/signals/StepBounds 一致 | `V2-109/result.json`、`actual-states.json`、`returned-states.json`；`implementation/three-language-restore-01.log` |
| 每新消息 ≥3 合法、≥3 非法 | passed；参数 3/18、时间 3/6、末事件 3/6、估计器 3/27、快照 3/12 | `implementation/message-coverage.json` |
| `cargo test -p core-kernel`（工作区独立执行结果） | 89 passed，1 个既有 V2-93 旧诊断 ignored；本卡 3 项均通过，包括报警、大 ordinal、次正规数 | `implementation/rust-other-crates.log` |
| `cargo test -p core-types --locked` | passed；34 个单元/集成测试，另含子进程冷启动检查 | `implementation/core-types-acceptance.log` |
| `cargo fmt --all -- --check`；`cargo clippy --workspace --all-targets --locked -- -D warnings` | passed | `implementation/rust-acceptance-results.json`、对应 acceptance 日志 |
| `cargo test --workspace --locked` | **failed**；core-store 31 failed / 15 passed，旧 θ 新写拒绝；未完成后续 crate | `implementation/rust-workspace-01.log`；B3 |
| `cargo test --workspace --locked --exclude core-store --no-fail-fast`（继续其余验收，不是完成门槛替代） | **failed**；fact-view 17 failed / 1 passed，同一 B3。core-types 当时 2 项失败后已定向修复并通过上行；其余执行结果在原始日志 | `implementation/rust-other-crates.log` |
| `pnpm lint`；`pnpm typecheck`；`pnpm test` | passed；52 文件、580 测试 | `implementation/pnpm-lint-01.log`、`pnpm-typecheck.log`、`pnpm-test.log` |
| 契约 Python ruff / mypy strict；三个 worker 的 src/tests/generated ruff / mypy strict / `uv run --locked pytest` | passed；各 worker 1 个既有骨架测试。新语义覆盖来自三语言契约套件 | `implementation/ruff-01.log`、`mypy-02.log`、`workers-results.json` 及各日志 |
| 迁移、运行时存储/发布/接续 | not_run；明确不在授权范围，不签发 SYS17 或整系统恢复 | B3；ADR 0029 实施追加 |

首轮契约命令的自动安装告警保留在 `contracts-01.log`。随后将复用的 node_modules 联接保留到本任务缓存备份，按锁文件独立 `pnpm install --offline --frozen-lockfile` 成功（239 包、0 下载），生成探针的 Node 依赖也改为本工作区；最终契约日志无该告警。未更改全局配置或关闭依赖检查。首次测试的 signed min 注解、探针语义边界、登记数量和测试导入问题均保留失败日志与修复后的定向验证，不隐藏失败或把试运行写成通过。

全部构建/测试由主代理串行；无 GPU、Electron 或视觉测试，无用户排练/正式存储。先前子代理仅作 PR/编号只读机械查询，按 GPT-6.1 Terra / max 显式参数试用，不作规格或通过判定，原始审计已复制到 `implementation/pr-audit-20261027T173056Z*.json`。

### 规格对照

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| ADR 0007；V2-107 | theta.proto、旧 superseded 标记、p5-35 基线 | 旧向量前缀不变、所有基线 0 违例；B3 说明消费者影响 |
| V2-94/95/98/103、I18 | 全字段估计器与四时间量、kernel storage | 真实前缀，正/零时长、负 log、停机、开放段、检测与慢漂；所有 f64 直接 to_bits |
| V2-100、V2-108 | 完整参数与摘要；restore 的可信资源匹配 | 资源/参数/摘要/未知版本/计数/质量/时间反例，无部分恢复或默认填值 |
| I23、V2-109 | verify_theta_restore + storage/tests.rs | 两种编码、两种返回语言、真实内存续算；variance 不等于 sigma² 的正控 |
| I11/I24、AC2-05/14 | 原 stage 不改 | 恢复后无事件/排除来源恒等，重复/倒序事件拒绝 |
| V2-96/97、AC2-20/21 | 原数值/报警算法不改 | StepBounds/ThetaChange/signals 同值，Jump/SlowDrift 正控；不声明运行时报警记录已接线 |
| I29 | 合成数据与任务缓存 | 未读取个体数据；不签发行为/校准有效性 |

完整对照见 [P5-36 追踪表](../../docs/traceability/P5-36.md) 的实施追加。

### 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| 无 | — | — | 复用既有 core-types、libm、serde_json 测试依赖与已锁定工具链；未改锁文件 |

### 待用户确认

- [x] 原 B1：#95/#65/#110 合并，目录释放。
- [x] 原 B2：用户批准三路径最小扩围及内嵌方案，桥接已交付。
- [ ] **（阻塞 B3）** 存储消费者后继接线的范围与安排。当前明确禁止扩围，按 BLOCKED 的两个选项留给后继协调；不通过放行旧写或改断言消除失败。
- [ ] （合并门槛）实现通过完整验收后的交叉审查及用户最终契约合并；当前未请求 ready 审查，未自行批准或合并。

### 后续建议（不在本任务范围内）

由明确拥有 core-store 的后继工作处理 θ 写入/读取代际；同步相关合成夹具，并判断旧存储读写边界是否要求单独迁移设计。H/资源来源认证、外层原子发布和独立完整 H 重算沿原后继任务处理。不能从本卡的纯内存恢复证据推断这些已经完成。

### 下一步

保持 PR #108 **draft / needs-user**，提交并推送已完成实现和证据。等待 B3 存储接线范围裁定或后继交付；后续合入相关变更后重跑受影响的完整工作区验收，全部通过才转可审查并写“请审查”。本轮不取下一卡。

### 冻结字节与 whitespace 说明

`git diff --cached --check` 只报新冻结的 `baselines/p5-35/growth.proto:300` 含末尾空行（exit 2）。该文件已逐字节核对等于 main 的原文件；为保留冻结字节没有删空行。这不是源码逻辑或测试失败，完整工作区的未通过仍按 B3 保留。源文件范围检查与所有冻结 proto 的 main 字节核对见 `implementation/scope-source-manifest.json`，原始 whitespace 输出见 `implementation/staged-diff-check.log`。

### 发布记录

实现与本轮证据已于提交 `6be258320b8683117bdc19d55afb88a92ed905f5` 推送至原分支；PR #108 的标题和描述已更新为实现成果及 B3，仍为 draft / needs-user。GitHub 连接器更新描述遇网络错误后，通过 `gh pr edit --body-file` 完成并重新读取确认。

固定实现提交的 CI 快照见 `implementation/ci-6be25832-snapshot.json`：Windows self-hosted 作业 skipped，汇总门禁 failure，注释明确为草稿 PR 不允许把 skipped 当通过；Ubuntu 当时仍 in_progress，不作通过结论。此快照只属于该 head。B3 已由本地完整测试证明，未转 ready 来规避草稿门禁，也未在已知存储接线未完成时额外触发 Windows 重负载。

本条发布记录为纯报告追加，不改变已验证源文件；不用为它重复本卡基线或真实恢复测试。后继解除 B3 时仍须核对最新 main 与实际验证基线，并补完整验收及当前 head CI。
