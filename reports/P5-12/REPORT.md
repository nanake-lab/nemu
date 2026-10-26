# P5-12 J 编译与读出 — 报告

## 结果

**完成用户 2026-10-26 选择 A 后的范围：J 数值编译、绝对预算检查、身体读出、进程内数值视图与未验证方向清单；等待独立交叉审查。** 不提供生产信封发布入口。两项原阻塞均已有决定和去向，见 [BLOCKED.md](BLOCKED.md)。

- 方向集、验证记录、身体读出参数、J 作用回执 IDL 移交 **P5-42**，只改 tasks/ 的 [PR #105](https://github.com/nanake-lab/nemu/pull/105) 已标 `needs-user`；E2，依赖 E1 的 P5-12，在契约线中排在 P5-37 后。P5-40/P5-41 已由 #102 占用，核对全部开放 PR 后顺延 P5-42。
- 主动不设强度函数，本卡不实现主动读出。时机由持续认知中的决定产生，只受用户边界、防失控保险丝与行动生命周期执行前复核约束；独立文档 [PR #103](https://github.com/nanake-lab/nemu/pull/103) 已合并为 **V2-117**，实施与验收由 P5-38/P5-39/P5-20 承担。

用户从 Claude 改派给 Codex。分支 `task/P5-12-j-compiler`，工作树 `D:\Dev\nemu-wt\P5-12`；先开 [草稿 PR #101](https://github.com/nanake-lab/nemu/pull/101) 认领。初始基线 `b4eb7fd`，先后 merge `86779b1`（P5-25）和 `317dde9`（V2-117），最近合并提交 `1415050`；没有 rebase/强推。冲突仅为差异登记末尾追加，保留本卡 V2-116 和 main 的 V2-117 全文。收尾没有修改数值实现。

## 交付物

- `core/crates/core-kernel/src/modulation/`：纯函数 `compile_numeric`，直接复用既有 effects 与 fast，按记录 t 投影一次；四天性固定顺序，学得项按规范倾向顺序。未验证/缺失方向排除并列出，不缩放旧项。
- `config.rs`、`numeric.rs`：进程内配置和资源引用绑定、个别与总 β 预算；绝对和向上保守舍入，拒绝 NaN/Inf/溢出，无 epsilon 宽限。不证明真实方向或验证证书内容。
- `readout.rs`：四通道固定线性组合，中性 `(0.5, 0, 0.5)` 和范围沿用 CONTRACTS §7，先检查有限乘加再截断。无审美默认参数；测试参数全是结构夹具。
- `rehearsal_without_fast`：仅接受 REHEARSAL，无 x 输入，零系数与中性身体读出；运行标记和自主行动停止仍须外层接入。
- 20 项新增测试、308 对精确 Fraction 预算向量；[J06 证据](J06/README.md)、`verification/` 原始日志与逐文件输入哈希；[收尾核对](J06/closeout-20261026.json)。
- [规格对照](../../docs/traceability/P5-12.md)、[接口移交](../../tasks/requests/P5-12-resources-and-receipt.md)。V2-116 保留原始缺口登记，解除情况以本报告、用户决定和 V2-117 为准。

生产代码仅在卡内目录；`lib.rs` 只追加模块登记（共享例外）。未重写 fast/effects，未改 IDL、协议黄金向量、Cargo 清单/锁文件或正式配置。其余为报告、接口请求和规格追溯。所有 Rust 文件小于 400 行；1542 行 JSON 是完整的 308 对算术向量。无新依赖。

`NumericReceipt` 无 Serialize/WireMessage 或自造摘要，`NumericCompilation` 不是生产信封。输入的已提交快照一致性、资源真实内容和 genesis 来源不由此进程内接口证明；该边界属于已移交契约和后续发布接线。

## 验收

本机 Windows，锁定工具链 `rustc 1.99.0`；进程内设置 `CARGO_HOME=D:\Dev\cache\cargo`，target/TEMP/TMP 为 `D:\Dev\cache\P5-12`，jobs=2、测试线程=1。测试只用合成内存状态或临时存储。没有打开用户的排练/正式存储。

以下执行日志均原样保存。测试在实现提交前运行，sidecar 的 revision 是认领提交 `2147461`，逐文件 inputs 才是实际受测源码；[tested-git-blobs.json](tested-git-blobs.json) 将它绑定到实现 `ad6c523` 的 Git blob。收尾只读核验：26 个原受测文件中 25 个 blob 相同，唯一变化为 `lib.rs` 增加上游 `pub mod dynamics;`；modulation、fast、effects、Cargo 依赖与 IDL 均无变化。所有 20 个新增测试逐项存在于成功日志，向量仍为 308 对。按角色名使用 routine_reporter 做机械只读复核，未委派验收裁决或交叉审查。

| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `cargo test -p core-kernel --locked --offline` | passed：65 项，含新增 20 项；0 失败/忽略 | `verification/kernel-20261026-183537-213.log` / `.json` |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | passed | `verification/clippy-20261026-183849-036.log` / `.json` |
| `cargo test --workspace --locked --offline` | passed：120 项单元/集成 + 2 项 compile-fail 文档测试；0 失败/忽略；1193.17 秒。此为合入 P5-25 前的完整回归 | `verification/workspace-20261026-183938-846.log` / `.json` |
| `cargo fmt --all -- --check` | passed | `verification/fmt-20261026-190029-027.json`（无命令输出） |
| `python -B reports/P5-12/generate_bound_vectors.py --check` | passed：308 对精确参考向量复算一致 | `verification/oracle-20261026-190106-829.log` / `.json`；本轮复核见 closeout |
| 生成脚本 `mypy --strict`、Ruff E4/E7/E9/F/I | passed | `verification/{mypy,ruff}-20261026-190106-829.log` / `.json` |
| J06 数值结构：非零两层调制、R⁺/方向资格门控、绝对预算、符号相消拒绝、旧项不缩放 | passed，限定本卡数值范围 | `J06/result.json`；308 对向量各测两个顺序、1024 组固定二进制有理数样本 |
| 记录 t 重放、确定性、输入不变、规范排序、零态/静息点、F 未接通排练路径、身体中性/截断/溢出拒绝 | passed | 20 项新增测试；64 次逐位重复 |
| 契约资源、规范回执、genesis 解码及生产信封发布 | not_run，已移交/卡外；不作为缩小后范围的通过项 | P5-42 / #105；BLOCKED B1 |
| 主动读出 | 实现义务已由用户取消，不签发旧验收的 passed | #103 / V2-117；BLOCKED B2 |
| 真实载体/方向行为、J01–J05、J07 | not_run，不在本卡数值范围 | P5-13/P5-31 及相应接线卡 |

保留的资源冲突记录为 `fmt-20261026-190000-978.json` 及同步后的 `fmt/clippy-20261026-190547-*.json`：均因其他重负载而未启动命令，结果是 `not_run`，不能算通过。已执行检查没有失败结果。收尾仍有其他重负载，本轮仅核对未变源码与已有结果并复算轻量预算向量，没有把旧日志冒充合并后的本机全量回归。上游新增 dynamics 的全工作区复测仍须在合并前按 AGENTS §6 执行；当前交付是可审查结果，不是合并结论。

原始 `J06/result.json` 的 `task_complete=false` 和旧 pending 反映用户决定前状态，未删改；本次范围、移交与证据核对单独记录在 closeout。未改阈值、断言、夹具参数或协议；Python 工具静态检查已完成，没有 TS/UI 改动。

## 规格对照

完整映射见 [docs/traceability/P5-12.md](../../docs/traceability/P5-12.md)。

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| v2.0 §9.1–9.2；ADR 0008 §3 | `modulation/compile.rs`，调用 effects/fast | 两层非零系数、候选/未验证排除、记录 t 重放 |
| O §13.1；数学核验 §7.4；S §6；J06 | `config.rs`、`numeric.rs` | 绝对预算、相消、零态超额、相邻/次正规/MAX 浮点上界 |
| v2.0 §9.3；CONTRACTS §7 | `readout.rs` | 三轴、中性、截断、乘加溢出拒绝 |
| O §13.2；v2.0 §16；用户选择 A | 进程内视图与轨迹；IDL 移交 P5-42 | 已有生成系数/身体类型 CanonicalV1 往返；未冒充生产发布 |
| v2.0 §9.4、§16 的后续读法 V2-117 | 不实现主动读出 | 已合并 #103；持续认知/保险丝由所属卡验收 |

I03/I05：收窄只读 DTO，无 H/z/o/p/c_task/叙事接口；I06：复用 R⁺ 编译并排除未验证方向；I18/I21：无持久状态、IO 或隐式时钟，借用输入；I23：固定顺序、有限 binary64、正零与逐位重放；I29/AT12：合成夹具不签发生产资格。跨 CPU/编译器逐位兼容没有签发。未新增 IDL 或修改协议黄金向量，算术向量仅验证本模块预算算法。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| 无 | — | — | 现有 Rust 依赖；预算参考生成只用 Python 标准库 |

## 待用户确认

- [x] B1：用户选择 A，收窄本卡，IDL 移交 P5-42 / #105。
- [x] B2：取消主动强度函数与读出，#103 / V2-117 已记录。
- [ ] （合并前）高难度独立交叉审查；实现方不自签审查通过。
- [ ] （P5-42，不阻塞本卡审查）#105 的任务定义及未来 ADR/IDL 仍由用户批准；本卡未代为批准具体契约。

## 后续建议（不在本任务范围内）

P5-42 提供正式资源/回执类型后，所属运行时/载体卡接入已提交快照、摘要、genesis、方向资格与生产信封；运行状态和结构故障的自主行动门控仍须端到端验收。真实模型、方向提取、能力无损及行为通路由 P5-13/P5-31 及相关卡验证。

## 下一步

#101 推送后转为可审查并写「请审查」，等待独立交叉审查；不合并，不接下一张卡。#105 保持 `needs-user` 等用户处理任务定义。未决定的实现阻塞为零；合并前仍须同步当时最新 main 并完成规定复测。
