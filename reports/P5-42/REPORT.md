档位：重档，改动 Echo Core IDL、生成类型和跨语言语义检查。

# P5-42 J 资源与作用回执 — 报告

## 当前状态

PR #164 已同步 main。差异登记改号为 V2-185，已审契约与生成物无变动。原审查者已通过 `02a4dc4c` 的 B1 修复及其 Windows 证据。本轮完整工作区在 Windows 默认并发下被网关 SYS01 超时阻塞，同一失败已在纯 main 复现。PR 保持 draft / needs-user，等待同步增量审查与用户批准 ADR 0036 / V2-185。见 [BLOCKED.md](BLOCKED.md)。

## 2026-11-19 main 增量核对

第一轮同步目标 M₁ 为 `19f1ec4e168d59ff62b440211491b7d8c8433839`，合并提交为 `97087d5d87866084a7eed2a411217b4a18b2b582`，第一父提交为旧 PR head `2200121f109db00e6f30d801adcdd19f465474df`，第二父提交为 M₁。

| 最近有效的验证 | 实际验证对象 | 基线 B 与增量 |
| --- | --- | --- |
| B1 本机三语言契约与 Rust 局部检查，见 [REVIEW-B1](REVIEW-B1.md) | `02a4dc4c4c0dfe0a5897424523970f0e2b1107b1` | `a6a0567f02efa5a183fa9fdb449c064afe11e661`，经 `5e04340f` 合入，B..M 共 53 个提交 |
| [Ubuntu 43067969814](https://github.com/nanake-lab/nemu/actions/runs/43067969814) | PR 合成提交 `6d777b4cb7ee7ddabbabb4da8d3397113e1c3680`，另一父提交为 `02a4dc4c` | main 父提交 `02372ff6114d1f173db01283b71f05b9b1495132`，B..M 共 49 个提交 |
| [Windows 全量 43361480191](https://github.com/nanake-lab/nemu/actions/runs/43361480191) | 手动派发直接 checkout `02a4dc4c4c0dfe0a5897424523970f0e2b1107b1` | `a6a0567f02efa5a183fa9fdb449c064afe11e661`，B..M 共 53 个提交 |

两个 B 都是 M₁ 的祖先。核对了两段提交清单和文件差异，Ubuntu 运行时已包含前 4 笔隐私文档、Docs guard 和独立研究变动。

| 增量 | 对 P5-42 的影响 |
| --- | --- |
| #194 FIX-STORE-01 | 改动 Rust 工作区内的持久化派生索引、见证、恢复和 `core-kernel` 的趋向正反证据数值分解。后者属于和本卡数值往返测试共用的 crate，改动点在 `law/recurrence.rs`，与 J 调制和身体线性读出分属不同模块。存储仍消费原提交对象；需要重跑 Rust 工作区和 Windows 存储、网关检查。 |
| #154 P5-71 | 增加 `core-kernel` 测试代码和研究入口，改变工作区检查所覆盖的目标，J 计算源码未变。 |
| #221 与 #222–225 | `task-layer` 新增来由投影及演示，Rust 锁文件为该 crate 记录 serde 和 serde_json。合并后的锁文件同时保留 P5-42 的 `core-types → core-kernel` 测试依赖，需跑锁定工作区检查。后续 #226、#228 是工具演示，模式 B 投影与本卡 J 回执互不调用。 |
| 契约、J 资源与身体读出 | `core/contracts/`、生成契约的消费者、`core-policy/`、`packages/body/`、J 数值模块、`CONTRACTS.md` §7 和 Rust 工具链均无 main 增量。#144 等试验数据及脚本只供独立载体研究，本卡生成与验收不读取它们。 |
| ADR、验收与工作流 | ADR 0042 描述模式 B 的来由投影，明确保留严格 J 的取值边界。ADR 0030 的状态确认、ADR 0035 的发布时点说明与标题勘误、P5-30 的实验验收追加均未改 P5-42 的 IDL 规则或验收。Ubuntu 和 Windows 工作流定义未变，Docs guard 的增量作用于文档检查。 |

唯一冲突在 `docs/traceability/diff-register.md`。以 M₁ 的完整文件为前缀，追加 #164 自有条目。main 对原号 V2-151 的命中均为 #164 的历史占号引用，未发现第二条同号登记。按同步任务要求将 #164 的现行登记与报告引用由 **V2-151 改为 V2-185**。main 最大 V2-178，开放 PR 已占 V2-179 至 V2-184。main 的历史行完整保留。V2-116 由 P5-12 登记，合并前后原行逐字节一致。PR 相对 main 的空白检查仅报告冻结基线 `growth.proto` 的既有 EOF 空行，其 blob 与原件相同。

第二轮同步目标 M₂ 为 `27a7acf3c9320cb8e4fd39c06766a9ae8e234c29`，合并提交为 `80e204e430cdf25f9b1f752e6f55ac4561cf6815`，无冲突。B..M₂ 分别累计 57 和 53 个提交。M₁..M₂ 的 4 笔增量分别是 #229 的独立 Python 演示，#230 的 Ubuntu/Windows 路径分流，#220 的研究范围说明，以及 #207 的技能说明。#230 把 `tools/cause-demo/` 送入 Python 演示检查并排除桌面组。P5-42 的变更仍匹配 Ubuntu 的 Node、Python、Rust、存储与契约检查。Windows 手动 `full=true` 仍覆盖桌面、W02 和 W18。#207 明确了本地注入与模式 B 的边界，P5-42 保留现有的结构检查与外层义务。此 4 笔增量没有改变本机 Rust 或三语言契约的输入，CI 在最终提交上运行。

### 同步后验收

| 命令或检查 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过，复核两次合并后的源树 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 通过，checkout `97087d5d`。第二次合并无本机编译输入变化。 |
| `cargo test --workspace --locked`（Windows，本机） | 失败。默认并发两次同一 SYS01 超时，后一次 checkout `80e204e`，退出码 101。未执行到的后续目标无本机结果。 |
| 同一 SYS01 用例单独运行 / 纯 main 默认并发网关对照 | 单用例通过。纯 main `27a7acf3` 的 `cargo test -p gateway --locked --lib` 为 40 通过、2 失败，含相同 SYS01 超时。 |
| `python core/contracts/generate.py --check` / `python core/contracts/check_compat.py` | 通过，0 个生成文件过期、0 条兼容违例 |
| `pnpm test:core-contracts`（D10.8 / SYS21） | 通过，Rust、Python、TypeScript 各 16,773 个用例，另有 646 个 Protobuf 传输向量。P5-42 的资源绑定、B1 和版本恢复检查通过。 |
| P5-12 J Rust 数值往返 / 契约 Ruff / 严格 mypy | 通过。J 数值往返在本机工作区测试退出前已执行。 |
| 当前 PR 的 Ubuntu、Windows | 运行链接和实际 checkout 见本轮 PR 说明与审查评论。草稿 Windows 自动门禁与手动全量运行分别记录。 |

原始日志及逐文件 SHA-256 见 [本轮证据索引](EVIDENCE.md#本轮同步证据)。

首次本机工作区测试加入了 `RUST_TEST_THREADS=1`，退出码 101。P5-71 的子进程把首个 `P571-PREFIX:` 输出接在 libtest 的测试名之后，按行前缀读取的工具丢了首帧。另一个子用例等待首条响应卡住，终止了该测试子进程。两项失败均保留在原始日志中。随后对同一测试二进制清除该环境变量，4 项 P5-71 测试均通过，其中完整工具检查覆盖 28 个短协议夹具。

第二次以默认测试线程运行，P5-71、J 数值往返和 core-store 通过。gateway 的 `sys01_authenticated_upload_dispatch_reconnect_gamma_and_reverse_receipts` 在 `service.rs:316` 报 `gateway deadline exceeded`（41 通过，1 失败，退出码 101）。该次测试与 [main 的 Windows W02 CI](https://github.com/nanake-lab/nemu/actions/runs/44463774308) 重叠。该作业结束后，同一 SYS01 用例单独复跑通过（20.37 秒），完整工作区仍复现 SYS01 超时（41 通过，1 失败，退出码 101）。纯 main 对照同样在 `service.rs:316` 超时，另有上传预约用例失败。此结果把问题定位到 Windows 默认并发下的网关测试组，具体超时根因仍待诊断。

## 交付物

- `core/contracts/j_resources.proto`：10 个新增消息、8 个枚举，方向集/验证记录/身体参数/作用回执；GenesisManifestV3 退出主动读出引用并保留旧 V2。
- 三语言 `j_resources` 语义模块：单对象、已解析资源、既有信封绑定检查；不签发资格、不发布生产信封。
- 生成类型、黄金向量、`baselines/p5-37/`、`verify_j.py`：接入正常 `test:core-contracts`，原 616 合法 / 860 非法向量不改字节；新增 30 合法 / 41 非法。
- `core-types/tests/j_numeric_roundtrip.rs`：调用 P5-12 纯数值实现，检查系数、保守总界、身体读出逐位往返与排除清单。
- ADR 0036、V2-185、`docs/traceability/P5-42.md`：字段来源、消费者义务、兼容/迁移/回退边界。

## 初轮验收

本机最终源提交 `5e04340f407b904903133be70ed5ef22635aee19`，Windows / Python 3.12.10 / Node 24.21.0 / pnpm 12.11.2 / Rust 1.99.0。原始证据见 EVIDENCE.md，逐文件路径与 SHA-256 见 EVIDENCE-INDEX.csv。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | 通过，无生成漂移 | generate-check-final |
| `python core/contracts/check_compat.py` | 所有既有基线 0 违例；新增 p5-37 的 11 个 IDL 与原 Git blob 相同 | compat-final、baseline-source |
| `pnpm test:core-contracts`（D10.8 / SYS21） | 190 消息；每语言 16,773 用例通过；646 官方 protobuf 传输向量通过 | conformance-final |
| P5-42 跨资源与恢复 | 每语言 3 组合、15 资源拒绝、12 同版本恢复 / 12 跨版本拒绝通过；另含信封与保守舍入、β=0 身体正反例 | conformance-final / verify_j |
| `cargo test -p core-types --locked --offline` | 通过，含实际 P5-12 数值逐位往返 | core-types-final |
| `cargo fmt --all -- --check`；`cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 通过 | fmt-final、clippy-final |
| 契约 Ruff / `mypy --strict`（与 CI 范围一致） | 通过 | ruff-final、mypy-final |
| Node lint/typecheck/test/build；四个 Python 项目的 Ruff/mypy/pytest/build；Rust 工作区及独立 core-store | 通过：[Ubuntu CI 43044637186](https://github.com/nanake-lab/nemu/actions/runs/43044637186) | CI 原始记录与 PR 最终验收评论 |
| Windows 桌面/隔离/视觉、W02、W18 | 通过：[手动全量 43047851482](https://github.com/nanake-lab/nemu/actions/runs/43047851482) | CI 原始记录与 PR 最终验收评论 |

本机完整 `cargo test --workspace` 在主干 Windows CI 启动时中断，不计为通过；最终工作区通过结论依赖上述 CI，不用局部测试代替。

## 失败与未完成项

- 新冻结基线 growth.proto 保留源文件 EOF 空行，`git diff --check` 对该逐字节副本提示空行；已核对原 Git blob 一致，其余改动无空白错误，未为消除提示修改冻结文件。
- 初轮静态检查暴露 Python 类型标注/导入、Rust 单元素循环问题，已修正；首次冻结向量因 echo 导入路径失败，已修正显式路径。原始失败日志保留。
- 自查纠正了「注入系数全零 ⇒ 身体中性」的错误推断；普通模式身体值由外层依据快状态重算，只有无快状态排练强制中性，并加入三语言正反例。
- 早期草稿 Windows 汇总门禁因自托管步骤跳过而失败。#157 隐私修复合入后，B1 的旧源码提交已通过手动 Windows 全量验证，原审查者确认解除该轮验证阻塞。
- 初轮本机完整工作区测试与 main Windows CI 重叠后中断（exit 130），初轮通过结论使用当时的 Ubuntu CI。
- 真实方向提取、三语能力无损、实际注入/缓存、已提交快照/原子发布、真实 genesis 迁移和 SYS17 均 not_run。所有夹具仅合成、未访问她的排练或正式存储；没有采用 R-STEER-01 研究结论或选定部署层位、hook、数值。

## 规格对照

见 [完整字段与验收对照](../../docs/traceability/P5-42.md) 与 [ADR 0036](../../docs/adr/0036-j-resources-and-effect-receipt.md)。外层需验证已提交快照、R⁺ 清单、数值重算、资格证据及已批准迁移链；接口通过不证明这些事实。

## 新增依赖

无新增运行时或外部依赖。仅给 core-types 添加仓库内 core-kernel dev-dependency（0.0.0，仓库内部代码，未新增第三方许可证），以调用真实 P5-12 数值实现。生成 schema/类型和黄金数据沿用机器生成的大文件，不手工拆分。

## main 增量核对

开工及早期本机测试基线 B=`6b9c9b0b136e6c3cd4483adf9e35a3298cf55093`。#155/P5-37 与 #101/P5-12 已合并且无 unverified；开工无其他开放契约 PR；ADR 最大 0035、差异登记最大 V2-150，故占用 0036/V2-185。

先核对 `f9c6ce74` 的 #162 状态登记，语义无新增影响。之后 M=`a6a0567f02efa5a183fa9fdb449c064afe11e661` 合入 #157，修改 Windows 证据脚本/工作流，已读操作性完整 diff 并 merge 为 `5e04340f407b904903133be70ed5ef22635aee19`；最终检查以这个组合运行。旧 B 的中断工作区结果不冒充新组合验证。再次 fetch 的 M=`88be8131d1240209da0a1c9428f2e0875c0e4380` 只增加独立研究/隐私审计报告和证据索引，无代码、契约、工作流或规范变化；相关运行/构建目录未引用这些报告，不影响本卡，未再 merge。

## 待用户确认

- [ ] （阻塞）ADR 0036 / V2-185 契约批准（用户合并）。
- [ ] （阻塞）同步后的增量交叉审查。旧源码的 B1 复审已通过。

## 后续建议（不在本卡范围内）

P5-08/19 接快照与发布；P5-13/31 接真实载体与验证；迁移接线在临时谱系通过 SYS17 后才可使用。字段约束不替代这些后继验收。

最终报告/证据索引提交只改 reports/P5-42/，相对被测源码提交无代码、生成物、依赖或工作流变化；按 AGENTS §6/9 核对文档收尾，不把不同提交号混称为同一次 checkout。Ubuntu 实际 checkout 为 e164c4af35541271c5e95e627008a49cd9b1a67e（父提交 a6a0567f、5e04340f），Windows 直接测试 5e04340f。

初轮 Windows 43047851482 的 E2E 为 3 passed / 2 skipped。B1 修复后的 Windows 43361480191 同样为 3 passed / 2 skipped。两次未测项均是 click-through 50+50 和 voice-idle 60 秒。
