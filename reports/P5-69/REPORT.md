重档：修改 core-kernel 的权威动力学计算路径。

# P5-69 C：生活基线的共同折扣 — 报告

## 当前状态
已实现 ADR 0035 D07-A；正在完成最终验证。PR #150 保持草稿、needs-user，等待独立交叉审查与用户合并。基线 origin/main `51298e9ce5d571e8753d047eaac864fce007b867`（P5-68 已合并）。不自动领取下一张卡。

## 交付物

- `dynamics/update.rs`、`types.rs`：显式新版本 genesis 与共享 P，按规范次序共同折扣；旧构造器保持旧规律，P 为 +0 也继续学习。
- `dynamics/bounds.rs`：含商缺陷的新单步外向包络；只在严格越界时拒绝，证明区间不能收紧时退回范围界。
- `dynamics/common_discount.rs`、`storage.rs`：直接使用 P5-68 的生成契约，完整位值导出/恢复、资源绑定；两版快照互不冒充，不从旧状态猜 P。
- `dynamics/tests/`：新增机制/边界/重算测试及 Python Fraction 独立 oracle、规范算术黄金夹具。`dynamics` 已由 lib.rs 无条件公开；新模块无条件登记，测试随常规 cargo test / CI 运行。
- [逐条规格对照](SPEC-MAPPING.md)：覆盖 ADR 0035 §9 P5-69 的十条及不变量、变异、证据边界。

## 验收

| 检查 | 当前结果 |
| --- | --- |
| 中间版本 `cargo test -p core-kernel --locked` | passed：249 passed、0 failed、1 ignored；299.61 秒。最终补充测试仍待运行 |
| 有理数 oracle `d07_oracle.py --check` | passed：256 个出生份额前缀、1074 个 N/T 舍入包络前缀 |
| Python mypy --strict / ruff check | passed；仅标准库，无新增依赖 |
| 最终 core-kernel 测试 / clippy / fmt / CI | 待运行/核验 |
| 独立交叉审查 | not_run，完成后在 PR 写「请审查」，由用户安排 |

原始日志包括失败日志留在 `D:/Dev/cache/P5-69/`，收尾归档至私有 `nanake-lab/nemu-evidence/P5-69/` 并生成提交号、逐文件 SHA-256 索引。主仓库只留报告、索引及代码所需测试夹具。

## 失败、未完成与结论边界

首次编译失败：旧逐字段穷举状态测试未包含新增 P，Rust E0027；已增加 P 的 Option/浮点位值断言，未删除旧断言，之后完整 kernel 通过。原始 `initial-kernel.log` 保留。既有 `**2` 诊断依旧 ignored，不能当 passed，本卡未改它。

真实时间、生产 H/L 原子写入、运行时来源认证/同次接触身份治理、暂停与更正编排、版本切换迁移：**not_run，P5-75**。本卡的 AC2-08/18/19 是纯 C 合成历史投影层结果，AC2-20 只提供新版本重算与 ThetaChange 前提，不声称实际写 L。E07 的动态 ρ 是固定外源序列机制对照；真实反馈闭环、生活参数校准及真实载体为 not_run（P5-71/76/30/77），不签发完整 AC2-25。

新数值路径登记为 `ADR0035-D07-gradual-underflow-v1`，规律 `law_id`、参数及报警资源由调用方显式提供；不填正式生活参数。ADR 0035 的组合规律生产启用仍需后继卡，不因本卡合并自动切换旧谱系。

## 新增依赖与后续

无新增运行时或测试依赖。σ₀、τ₀ 的固定先验质量、残差平方等路径的下溢故障保持现行规则（ADR 0035 §3.2–3.3）；它们属于已登记后续，不在本卡修订。未新增差异登记或 ADR。

## 待用户确认
- [ ] 独立交叉审查通过后，由用户批准新规律版本并合并；PR 始终保持草稿与 needs-user。
