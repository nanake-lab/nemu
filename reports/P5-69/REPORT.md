重档：修改 core-kernel 的权威动力学计算路径。

# P5-69 C：生活基线的共同折扣 — 报告

## 当前状态
已实现 ADR 0035 D07-A，本机最终测试、clippy 与格式检查通过。PR #150 保持草稿、needs-user，等待独立交叉审查与用户合并。基线 origin/main `51298e9ce5d571e8753d047eaac864fce007b867`（P5-68 已合并）；最终受测代码提交 `b8fe03ac9760aaa2f78bfced43ea8a2d2dc1b2b1`。后续提交只更新报告与索引，不自动领取下一张卡。

## 交付物

- `dynamics/update.rs`、`types.rs`：显式新版本 genesis 与共享 P，按规范次序共同折扣；旧构造器保持旧规律，P 为 +0 也继续学习。
- `dynamics/bounds.rs`：含商缺陷的新单步外向包络；只在严格越界时拒绝，证明区间不能收紧时退回范围界。
- `dynamics/common_discount.rs`、`storage.rs`：直接使用 P5-68 的生成契约，完整位值导出/恢复、资源绑定；两版快照互不冒充，不从旧状态猜 P。
- `dynamics/tests/`：新增机制/边界/重算测试及 Python Fraction 独立 oracle、规范算术黄金夹具。`dynamics` 已由 lib.rs 无条件公开；新模块无条件登记，测试随常规 cargo test / CI 运行。
- [逐条规格对照](SPEC-MAPPING.md)：覆盖 ADR 0035 §9 P5-69 的十条及不变量、变异、证据边界。

## 验收

| 检查 | 当前结果 |
| --- | --- |
| 最终 `cargo test --manifest-path core/Cargo.toml -p core-kernel --locked` | passed：254 passed、0 failed、1 既有 ignored；329.79 秒；doc-tests 0 项 |
| D07 专项测试 `dynamics::tests::common_discount -- --nocapture` | passed：13 项、0 failed；4.71 秒；保存 1074 步 P/S/U/b 位值和 128 步动态出生差轨迹 |
| 有理数 oracle `d07_oracle.py --check` | passed：256 个出生份额前缀、1074 个 N/T 舍入包络前缀 |
| Python mypy --strict / ruff check | passed；仅标准库，无新增依赖 |
| `cargo clippy --manifest-path core/Cargo.toml -p core-kernel --all-targets --locked -- -D warnings` | passed |
| `cargo fmt --manifest-path core/Cargo.toml --all -- --check` | passed |
| [Ubuntu CI 42841073030](https://github.com/nanake-lab/nemu/actions/runs/42841073030)，受测代码提交 b8fe03ac | passed：工作区 fmt、clippy、tests（按既有工作流排除 core-store），可选 CPU 编译检查通过 |
| Windows CI / Node、旁路 Python 项目检查 | 本卡路径不触发这些检查；Windows 本机 Rust 结果见上表，oracle 自身 Python 静态检查单独执行 |
| 独立交叉审查 | not_run，完成后在 PR 写「请审查」，由用户安排 |

环境：Windows x86_64，rustc/cargo 1.99.0、Python 3.12.10；缓存隔离在 `D:/Dev/cache/P5-69/target`，本机重负载串行。原始日志包括失败日志归档至私有 `nanake-lab/nemu-evidence/P5-69/`；见 [证据索引](EVIDENCE.md) 的提交号、逐文件 SHA-256。主仓库只留报告、索引及代码所需测试夹具。

## 失败、未完成与结论边界

首次编译失败：旧逐字段穷举状态测试未包含新增 P，Rust E0027；已增加 P 的 Option/浮点位值断言，未删除旧断言，之后完整 kernel 通过。首次 CI 42838938760 因测试中的 `n % 2 == 0` 触发 clippy `manual_is_multiple_of` 失败，改用等价整数方法；修复后 CI 42839438093 通过，最终代码的 CI 另列上表。失败日志均保留。既有 `**2` 诊断依旧 ignored，不能当 passed，本卡未改它。中间版完整测试为 249 passed / 1 ignored，最终结果以上表为准。

真实时间、生产 H/L 原子写入、运行时来源认证/同次接触身份治理、暂停与更正编排、版本切换迁移：**not_run，P5-75**。本卡的 AC2-08/18/19 是纯 C 合成历史投影层结果，AC2-20 只提供新版本重算与 ThetaChange 前提，不声称实际写 L。E07 的动态 ρ 是固定外源序列机制对照；真实反馈闭环、生活参数校准及真实载体为 not_run（P5-71/76/30/77），不签发完整 AC2-25。

新数值路径登记为 `ADR0035-D07-gradual-underflow-v1`，规律 `law_id`、参数及报警资源由调用方显式提供；不填正式生活参数。ADR 0035 的组合规律生产启用仍需后继卡，不因本卡合并自动切换旧谱系。

## 新增依赖与后续

无新增运行时或测试依赖。σ₀、τ₀ 的固定先验质量、残差平方等路径的下溢故障保持现行规则（ADR 0035 §3.2–3.3）；它们属于已登记后续，不在本卡修订。未新增差异登记或 ADR。

保持原有估计器与边界层，未引入泛化框架或默认生活参数。已有 `dynamics/storage/tests.rs` 为 567 行；本卡只为旧逐字段状态比较补上 P，不借此拆分无关旧测试。新增 Rust 文件均不超过 400 行。

## 待用户确认
- [ ] 独立交叉审查通过后，由用户批准新规律版本并合并；PR 始终保持草稿与 needs-user。
