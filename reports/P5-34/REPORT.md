# P5-34 分配的实际保留率与倾向的证据量 — 报告

## 当前状态

部分完成（见 BLOCKED.md）。新草稿 [PR #95](https://github.com/nanake-lab/nemu/pull/95) 标 needs-user，由用户合并。原 PR #72 因目录冲突关闭；本次在原分支上依次 merge 最新 main，没有 rebase 或强推。已合入 main 2bc04bad4902657eef49d3225cafa7945e7cfae6，合并提交 da945d521ab5d2a2dcfa9d21930a47e6c6d24e82。当前保留草稿，尚不满足转可审查条件。

本次只保留任务卡列明的契约、core-types、生成代码，以及要求的 ADR、差异登记、规格对照、请求和报告。原 PR 的 core-store 扩围已恢复为 origin/main，净差异为零；原实现仍在 Git 历史，原始证据完整保留。卡外存储消费者尚未从 V2 切换至 V3，不能用旧 PR 的成功日志证明当前兼容。

## 交付物

- core/contracts/law.proto：四种 V3 后继消息；EvidenceAllocationV3.9 actual_retention 保存历史 q，TendencyV3.18 evidence_absolute 保存独立 V。EvidenceGroupV3 保留 7/8，RSnapshotV3 保留 4/5。V2 标为取代，原定义、标签和字节保留。
- core-types 的编解码与纯语义检查：q 有限且在 (0,1]，V 有限且 ≥|A|；绑定规律参数核对 B、π、r 的有限 binary64 计算次序；序号恰为 0…N−1。JSON 位值通过既有 serde_json 的 float_roundtrip 保留。
- Rust/Python/TypeScript 生成类型、运行时和浏览器安全 Zod；只增加两个权威字段，不设计 G、检测器或 θ 状态。
- 黄金向量只追加 19 合法、35 非法；现有总数 464 合法、492 非法、18 Unicode。四种 V3 的合法/非法数量分别为分配 6/10、倾向 5/11、证据组 4/7、快照 4/7。
- ADR 0021 §10、docs/traceability/P5-34.md 当前范围说明，以及 tasks/requests/P5-34-core-store-v3.md 的接续请求。沿用 V2-62/63，不新占编号。
- verify-resumed.ps1 与 summarize_resumed.py：独立记录本轮命令、开始/结束、HEAD/main、退出码、日志 SHA-256、生成代码和冻结基线哈希。不会覆盖旧证据。

## 验收

当前证据根目录为 verification/resumed-20261026/。attempt-01 内各检查均有同名 .log/.json。日志保留原始失败；汇总不把失败或未执行改成通过。验证工具链为 Rust/Cargo 1.99.0、系统 Python 3.12.10、uv 项目 Python 3.12.14、uv 0.12.15、Node 24.21.0、pnpm 12.11.2。缓存、target、临时存储在 D 盘；重负载由主代理串行执行，routine_reporter 仅做只读向量/哈希/CI 核验。命令 JSON 的 jobs/testThreads 表示 Cargo/Rust 环境约束；Vitest 依仓库原配置运行。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| python core/contracts/generate.py --check | 通过，0 stale | attempt-01/generate.log |
| pnpm test:core-contracts；D10.8 / SYS21 的本卡范围 | 通过；12 项 schema，三语言各 11592 用例（2659 合法/8933 非法）、protoc 464 向量、浏览器 Zod 106 项 | attempt-01/contracts.log |
| python core/contracts/check_compat.py --ref origin/main | 通过，p5-01、p5-21、p5-32、current、origin/main 均 0 违例 | attempt-01/compatibility.log |
| 旧黄金及新增字节独立复核 | 旧 445/457/18 项和文本前缀相同；新增 19 合法向量独立复算 0 差异 | attempt-01/bytes.log |
| 定向输入控制与 p5-32 冻结基线 | 7 合法/23 非法、6 非有限有效对照通过；7 proto 与发布版本 91b31ed 相同 | attempt-01/inputs.log |
| cargo test -p core-types；语义正反例 | 31 项通过，含 P5-34 的 7 项；0 失败/忽略 | attempt-01/types.log |
| cargo test --workspace --locked -j1 -- --test-threads=1 | 失败，退出 101；core-kernel 12 通过，core-store 30 通过/1 失败；Cargo 在该包后终止 | attempt-01/workspace.log |
| 全工作区失败后未运行到的 5 个脚手架 crate 及 doc-tests | 未运行：补充调度被其他卡重负载挡住，未并行启动；不替代或改判全工作区失败 | resource-deferred.json、guard-remaining-*.json |
| cargo clippy --workspace --all-targets -- -D warnings | 通过 | attempt-01/clippy.log |
| cargo fmt --all -- --check | 通过 | attempt-01/fmt.log |
| pnpm lint | 通过 | attempt-01/lint.log |
| pnpm typecheck | 通过 | attempt-01/typecheck.log |
| pnpm test | 失败，退出 1；579 通过/1 失败，51 文件通过/1 文件失败 | attempt-01/test.log |
| IDL/生成 Python 的 ruff、mypy --strict | 通过；mypy 12 个文件 | attempt-01/ruff.log、mypy.log |
| 三个 worker 的 ruff、mypy --strict | 全部通过，各 2 个源/测试文件；生成模块另有上述检查 | attempt-01/irl-*、drive-*、model-adapter-* |
| 三个 worker 的 uv run pytest | 全部通过，各 1 项脚手架测试；不代表真实模型效果 | attempt-01/*-pytest.log |
| Ubuntu CI，先前 head b84d0bf | 失败，Rust checks 复现相同 V2 新写入拒绝；退出 101，core-store 30/1 | ci-b84d0bf.json；[run 40271343235](https://github.com/nanake-lab/nemu/actions/runs/40271343235) |
| Windows CI | 草稿条件跳过；本轮检查时 Runner.Listener/Worker 均未在线，尚不能提供通过结论 | environment.json；PR checks |
| 真实 G 重放、AC2-08/19/21、I23、SYS17 | not_run；后续集成范围 | 本报告、BLOCKED.md、summary.json |

工作区测试开始于 17ef44d / main 716f7f0；运行期间合入的 main 2bc04ba 只涉及看板、报告和 CI 追加步骤，core、packages、workers 没有差异。后续检查各自记录实际 HEAD；不会把旧提交的 CI 说成最终提交全绿。ci-b84d0bf.json 保存原始错误摘录和 job 链接，明确不是完整 CI 日志；完整记录可在对应不可变 run 查阅。

两项失败分别为：core-store/src/tests/objects.rs:172 的 V2 新写入被取代规则拒绝，以及 apps/desktop/src/supervisor/supervisor.test.ts:199 的退出清理断言观察到 PID 87292。后者随后已退出，不能据此改判通过；根因尚未确定。两个卡外目录都没有本卡净修改，未改断言、未放宽阈值、未靠重跑覆盖失败。完整诊断与接续选项见 BLOCKED.md 和两份 tasks/requests/P5-34-*.md。

## 规格对照

完整表见 docs/traceability/P5-34.md，当前有效范围为首表、JSON 位值回归及 2026-10-26 追加节；其中 2026-10-24 的存储映射只描述已关闭 PR #72。

| 规格条款 | 当前实现 | 检查 |
| --- | --- | --- |
| v2.0 §5.2；ADR 0019 §5/6/8.5；ADR 0021 §2–4 | actual_retention；保存首次 q，当前 S=0 不能倒推 q=1 | q 边界、次正规数、修改支持度后 q 仍合法、序号正反例 |
| v2.0 §5.2；ADR 0019 §1/8.2/8.5；ADR 0021 §1–3 | evidence_absolute；V 不含 c/P，不以 B−c 恢复 | V<|A| 双符号、非有限、B 舍入丢失、小量 M/U、依恋分母括号 |
| ADR 0007；ADR 0016；ADR 0021 §2 | 四种后继消息、继承 reserved、V2 取代标记 | 三基线/当前兼容门、旧字节独立复核、保留字段号拒绝 |
| SYS21；施工合同 10.8 | 三语言同一描述符、严格有限 binary64 及浏览器 Zod | 黄金向量、对抗输入、protoc 对照与语义检查 |

涉及不变量：I18 由显式版本化 IDL 和字段最小性表约束，不引入隐藏持久状态；I23 只验证所需 q/V 信息和计算分组能无损表达，完整重算仍由 G 集成验证；I25 的份额/同源算法不在本卡实现，序号和重复容器检查只覆盖契约局部；I02/I29 通过纯检查器、无权威库写入口及 TemporaryRoot/临时输入验证，未打开用户排练或正式存储。既有 LAW-6 和验收阈值未改变。

## 新增依赖

无新增依赖或锁文件变更。既有 serde_json 1.0.154（MIT OR Apache-2.0）启用 float_roundtrip。超过 400 行的 semantic.rs 保留旧 V1/V2 检查并追加 V3，生成文件按原生成器组织；未另造第二份跨进程类型。

## 待用户确认

- [ ] （阻塞）core-store 消费者适配的归属与卡目录安排；不得以取消 V2 取代标记或默认填 q/V 绕过。
- [ ] （阻塞）桌面进程监督器测试失败的所属模块诊断与复验；不由 P5-34 扩围修复。
- [ ] （阻塞）ADR 0021、V2-62/63 与具体后继契约交叉审查、用户批准合并。
- ADR 0019、V2-55–59 已由 V2-84 确认，不重复请求；V2-80/105、81–83 的算法决定留给所属 G 实现。

## 历史证据与后续建议

verification/run-01、after-main-0b5412e、core-store-v3、idle-20261024、review-20261024 和顶层各验收编号 result.json 均保留为 PR #72 历史证据，不代表本轮结果。旧 record_verification.py 面向旧运行目录，不用于生成本次结果。旧 JSON 位值失败、CI 超时和存储复审记录未覆盖或删除。

后续由有权修改存储的卡接入 V3 并保留 V2 历史读取，再对真实 G 的完整 H 重放、来源更正和迁移/回退独立验收。P5-35 依赖 P5-34，不能把等待 P5-35 当作消除当前合并阻塞的安排。新变化检测器/θ 契约由后续卡处理。

## 下一步

本卡指定的契约验收命令均已执行并通过；全部要求的工作区命令已实际执行，Rust/TypeScript 两项失败保留。资源占用挡住的补充覆盖不作为通过证据。等待所属目录解决两项失败后，重跑完整验收，再转为可审查；当前保持 draft/needs-user，由用户安排独立交叉审查及最终合并。本卡不领取下一张卡。
