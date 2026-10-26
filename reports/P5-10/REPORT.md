# P5-10 快状态 F（v2.0 第 6 节）— 报告

## 当前状态

2026-10-26：已修复最新复审指出的 λ_k 混合量级偏差。本轮验收命令全部通过：定向回归、45 项 core-kernel 测试、工作区 100 项单元/集成测试与 2 项 compile-fail 文档测试，以及工作区 fmt/Clippy。保留 PR #94 草稿与 `needs-user`，等待独立复审与当前 head 的 CI；本轮不合并、不取下一张卡。接入 process_head 仍属 P5-19。

分支 `task/P5-10-fast-state` 已 fetch，并通过 merge `20ec9ae` 合入 `origin/main` 的 `a0c9ed8`；未 rebase 或强推。上游仅更新 CI 与 README，任务卡和规格未变。本轮自有修改只涉及 fast 模块、fast 结构夹具、本卡报告与规格对照条目。

## 审查修复

针对 [最新复审](https://github.com/nanake-lab/nemu/pull/94#issuecomment-6410467710)：λ_k 不再先把原始权重除以最大方向分量。选择最大 |d_i| 对应的 λ 作为初始均值（并列取首通道），用原始最大权重初始化累计质量；其余通道依次执行 `mass += |d_i|`，`mean += ((λ_i - mean) * |d_i|) / mass`。先乘原始权重、后除累计质量，保留次正规方向与大速率的贡献。对获准单位方向，非首项权重小于 1，正有限速率之差不会溢出，每次更新至多为剩余差的一半；累计质量、更新量、派生速率和输出继续检查有限性及合法范围。

新增 133 个 binary64 位型参考向量：包含复审反例、24 种通道排列、符号变化、五种次正规方向权重、1e308/MAX 速率、直接分子溢出、次正规输出、共同速率及单位范数容差内略大于 1 的首分量。Python 标准库 Fraction 从 binary64 输入精确求参考商，最后舍入一次；生产实现仍为有限 binary64，参考生成不调用生产均值算法。

复审反例修复值与精确参考均为 `3.52904032743748543e-16`，偏差 0 ULP。133 例预定舍入界为 4 ULP，实测最大 2 ULP；共同速率要求逐位相同。另通过公开 project_time 检查 Δ=0 逐位恒等及连续 100,000 次一秒投影，每一步均与参考速率使用同一 libm/expm1 的结果逐位相同。原阈值、断言、测试全部保留。

上一轮驱动/η 边界、负零规范化、V2-115 及直接事件映射已获复审确认闭合，本轮实现不变。V2-115 的多人最高亲近度、无人 w=1、κ^n 下溢取 0 仍为用户已确认（2026-10-26）。

## 交付物

- `core/crates/core-kernel/src/fast/`：状态校验、Φ_x、F_in/F_out、距离、快照、事件分类驱动、共情权重与恢复纯函数。
- `fast/tests.rs` 与 `fast/tests/{drive_bounds,rates,state_boundaries,event_map}.rs`：33 项 fast 测试，本轮新增 2 项；core-kernel 共 45 项。原测试文件超过 400 行，保留既有验收与共用夹具便于复核，新增案例放在较小子模块。
- `profiles/structural_fixture/fast/fixture.json` 保持不变；新增 `rate-reference-vectors.json` 存放 133 个参考向量。均仅证明结构性质，不作为行为有效或正式谱系证据。其 [生成器](review-2-2026-10-26/generate_rate_references.py) 无第三方依赖。
- [规格对照表](../../docs/traceability/P5-10.md)、[差异登记 V2-115](../../docs/traceability/diff-register.md)、[复验脚本](verify-review-fixes.ps1)、[本轮原始日志](review-2-2026-10-26/)、[测试输入哈希](review-2-2026-10-26/tested-inputs.json) 与按验收编号整理的结果。

## 验收

卡片列出验收性质，没有独立命令行。core-kernel 命令覆盖本卡全部测试；另运行 Rust 工作区格式、静态检查及测试。主代理串行执行重负载；routine_reporter 只做路径、日志摘要和测试名的只读核对。

| 命令 | 结果 | 原始证据 |
| --- | --- | --- |
| `cargo test -p core-kernel --locked --offline learned_rate_ -- --nocapture` | 3 通过，反例 0 ULP、133 例最大 2 ULP | [定向回归](review-2-2026-10-26/regression-20261026-165451.log) |
| `cargo fmt --all -- --check` | 通过 | [fmt](review-2-2026-10-26/fmt-20261026-165505.log) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 通过 | [clippy](review-2-2026-10-26/clippy-20261026-165506.log) |
| `cargo test -p core-kernel --locked --offline` | 45 通过，0 失败/忽略 | [kernel](review-2-2026-10-26/kernel-20261026-165501.log) |
| `cargo test --workspace --locked --offline` | 100 项单元/集成测试及 2 项 compile-fail 文档测试通过，0 失败/忽略；1050.86 秒 | [workspace](review-2-2026-10-26/workspace-20261026-165514.log) |

环境：Windows，锁定 Rust 1.99.0；`CARGO_HOME=D:\Dev\cache\cargo`，构建与临时存储位于 `D:\Dev\cache\P5-10\`，jobs=2、测试线程=1。等待 Windows runner 作业结束后再启动本机重负载，串行运行。未打开用户排练或正式存储。

失败记录：先前 head `1e7644d` 的混合量级反例偏小约 20%，由上述复审复现，本轮回归已覆盖。第一轮曾误用 `Evaluation::default()` 编译失败，修正后通过，[失败原始日志](review-fixes-2026-10-26/kernel-20261026-131414.log) 保留。旧轮次日志、结果索引及输入哈希均未删改。

| 验收项 | 结果与范围 | 证据 |
| --- | --- | --- |
| AC2-15 / I05 | 通过：200 组 × 30 步含事件区间收缩、无事件回落、10,000 组 Δ = 0 直接非扩张及独立正例 | [AC2-15](AC2-15/review-2-result.json) |
| F01、施工合同 10.6 | 通过：时间分段一致性原容差 1e-12、同锚点读取不累计衰减、合法域 Δ = 0 逐位恒等、恢复后快照位型一致 | [F01](F01/review-2-result.json) |
| F02 | 通过：20,000 组投影/驱动后在 X 内；错误总和、learned 超界、非有限 η/贡献、极端及混合量级 λ、非规范零拒绝路径 | [F02](F02/review-2-result.json) |
| F03（结构） | 通过：近期历史停止驱动后按 λ_min 遗忘，λ = 0 拒绝 | [F03](F03/review-2-result.json) |
| F05（纯函数） | 通过：F_out 保留/删除/补零，同事件产生同一单次脉冲；不等同于事务去重验收 | [F05](F05/review-2-result.json) |
| v0.6.4 16.17（结构） | 通过：盒空间界、时间分段与近期扰动收缩由 F01–F03 覆盖；learned 维度跟随 R⁺ | [16.17](16.17/review-2-result.json) |
| 施工合同 10.5 | 通过本卡部分：A 缺失只去掉天性评估项，非法驱动拒绝且不伪造脉冲；降级审计归 P5-19 | `missing_appraisal_keeps_only_the_tendency_effects`、`f02_*` |
| D 不读 x̄ | 通过：函数签名无 FastState，编译期函数指针约束 | `drive_signature_has_no_fast_state` |
| ADR 0026 §6.3、V2-115 | 通过纯函数检查：事件权重、n 转移、多人/无人、κ 下溢 | `self_caused_*`、`internal_count_*`、`empathy_weight_*`、`kappa_power_*` |
| 快照与恢复 | 通过：CanonicalV1 逐位往返、未知版本拒绝、固定 x_init 重置及 washout 判据 | `snapshot_*`、`state_boundaries::*`、`recovery_reset_and_washout` |

固定求值次序与锁定 libm 是确定性基础；跨目标逐位兼容仍需黄金向量/回放证据，不能推及任意平台。κ 下溢用例不等于 COG-A12 硬件时间包络验收。系统记录/超期结果绕过 F、Q 绑定、实际 n 持久化、事务去重与降级审计仍属编排。当前 head 的 CI 与复审以 PR 检查/结论为准，旧 head 的通过不替代新 head。

## 规格对照与不变量

完整条款、实现与测试映射见 [docs/traceability/P5-10.md](../../docs/traceability/P5-10.md)。

| 不变量 | 本卡检查方式 |
| --- | --- |
| I02 / I03 | 只读输入与返回值，无数据库写能力；只产出 x 或 drive，外层事务负责提交 |
| I04 / I21 | 同锚点纯投影；F_out 不再驱动，重复读取一致；事务去重留给 P5-19 |
| I05 | D 签名无 x̄；AC2-15 直接映射与含事件区间测试 |
| I06 | 沿用 P5-27 的 R⁺ 编译，原 `i06_candidates_have_no_effect_on_x_or_m` 随测试运行 |
| I18 | 构造/检查为无持久状态纯函数，不增加缓存或隐藏状态 |
| I29 | 结构夹具、固定随机种子、临时存储 |

本次未改 IDL、跨进程契约、黄金向量或快照格式，表示版本仍为 `echo-core-v2.0/fast-state/1`。修复对象是未合并 PR 内的数值求值与输入边界。无新增差异编号，仅落实 V2-115 确认。

## 新增依赖

本次无新增依赖、清单或锁文件改动；参考向量生成只使用 Python 标准库。原 PR 的 `libm =0.2.16`（MIT/Apache-2.0）用于 expm1/exp，为 P5-09 已用版本；`serde_json =1.0.154`（MIT/Apache-2.0）为工作区已有依赖，仅测试使用。

## 待用户确认

无未决规格读法；V2-115 已由用户于 2026-10-26 确认。仍等待独立交叉复审与当前 head 的 CI；实现方不代写“审查通过”。

## 后续建议（不在本任务范围内）

- P5-19：外层事务、时钟/锚点、内部计数 n、降级/连续性中断审计、事件去重；系统记录与超期行动结果不调用 F。
- P5-38：行动发出时的 n 随 Q 的在途行动保存，传给 `EventClass::ActionResult`。

## 下一步

本轮修复与证据提交 PR #94，回复最新复审并请求“请复审”。等待当前 head 的 CI 与用户安排的独立交叉复审；本轮不合并、不认领下一张卡。
