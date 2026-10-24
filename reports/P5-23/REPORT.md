# P5-23 规律引擎 G — 报告

## 当前状态

按 PR #49 审查评论 6379618075 修复非有限中间量被裁剪隐藏的问题，新增七项回归，等待独立复审。分支已合并 `origin/main` 的 `7f962b2`，仍按 ADR 0019 实施；没有改变公式、参数下限或容差。当前提交的实际检查结论与逐条修复回复见 [PR #49](https://github.com/nanake-lab/nemu/pull/49)，不使用旧提交 a3d2ff2 的通过结果代替本轮验证。写「请审查」后停下，不合并、不取下一张卡。

## 本轮修复

- `support.rs`：依原计算顺序检查 `2*p`、除法和减法，再执行规定的支持度裁剪；逐成员投影先检查再取峰值；强度商和 cap 先检查再取 min。
- `replay.rs`：M、U、r_max 及非零 U 时的 M/U 在裁剪前检查；保留 U=0/M≠0、分别舍入和有限值的正常裁剪。
- `tests/finite_contract.rs`：审查给出的正负 helper 最小复现；完整 G 最小复现及前置教导已成功暂存后的晚阶段失败；强度、成员投影、M/U 溢出和回滚；正常裁剪、最小正 r_min 与 U=0/M≠0 的成功对照。
- [同类排查表](FINITE-AUDIT.md)：逐一核对 law 生产代码的 clip/min/max 及 core-kernel 其余生产模块；没有修改卡外实现。

## 验证与证据

本机重负载被连续 Windows CI 占用时不并行编译；本轮完整验证使用当前 PR head 的 Ubuntu CI。检查会执行新增七项回归及原有全部 core-kernel/workspace 测试；具体退出码、测试数量和日志由 PR 对应提交的 Checks 与修复回复给出。本机仅格式检查不等于编译或测试通过。

| 检查 | 当前提交证据入口 |
| --- | --- |
| `cargo fmt --all -- --check` | PR Ubuntu CI 的 Rust checks；本机格式检查原日志在 `verification/finite-review-20261024/` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 同提交 Rust checks |
| `cargo test --workspace --locked`，含 `finite_contract` | 同提交 Rust checks；必须保留 helper/G 故障与原子性断言 |
| legacy/extra、80,000 整步、events/review/sides 同输入逐位比较 | `reference_parity`、`reference_events` 随完整测试执行，不跳过 |
| 原 LAW-6 | 原判据失败继续保留；诊断测试要求 3,064 个状态违例，不能当作 LAW-6 通过。先前独立命令 result.json 与退出 1 的原证据保留在 `LAW-6/adr0019-20261024/` |
| Windows CI | 以 PR 对应提交的检查与路径范围判定为准；不操作其他应用启动或干预 runner |

修复前审查原始失败材料及源码复制为 `verification/finite-review-20261024/reviewer-fault-contract.*`，原测试为 1 passed / 1 failed、退出 101；这不是本轮新跑的本机红测。原 a3d2ff2 的本机与 CI 证据全部保留；它们仍不能证明本轮新代码通过。没有覆盖或删除失败日志。

## 规格对照与边界

见 [P5-23 规格对照](../../docs/traceability/P5-23.md) 的 ADR 0019 追加节和本轮有限性修复追加说明。I03 的故障原子性由暂存返回和 G 回归检查；I10/I18/I24 的受限输入与显式状态、I23 的重放、I25 的精确份额、I29 的合成夹具边界保持不变。

q/V 仍在 law 内临时结构承载，P5-34 合并前没有生产编码；完整运行时、真实有效 H 的来源更正及暂停补算不由本卡证明。原 Python 参考文件、IDL、黄金向量及冻结规格未改动。标量 Python 模型只保存原始 M/U，不能替代本轮 G 读出商的非有限故障检查。

## 新增依赖

无。

## 待用户确认与下一步

- [ ] 独立复审评论 6379618075 的阻塞问题；执行者不自行发布审查通过结论。
- [ ] V2-38–V2-47 白话清单仍由 Claude 整理后供用户一次确认，本卡不代做。
- [x] 原 LAW-6 按原判据继续报告失败；ADR 0019 / V2-60 的保证变化不改写为通过。

在原 PR #49 逐条回复并写「请审查」后停下；保留 needs-user，不合并。
