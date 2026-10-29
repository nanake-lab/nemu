# P5-26 纯重算机制

`start / step / replay` 在显式规律、解释器和来源版本下暂存 H 的确定性结果。`Kernel` 由具体规律实现：提供 genesis、有效 H 投影、单步递推以及事件、投影输入、完整状态的逐位比较。普通浮点 `PartialEq` 不能替代缓存/历史身份比较；有符号零反例覆盖此处。重算层不定义任何规律内容，也不反推缺失的 q/V。`Command`、`Commit`、`Scope`、`Audit` 是进程内输入/输出，不能序列化为新的权威格式。

- 普通追加：投影仍以已处理输入为前缀时，接着执行同一递推；前缀改变则从 genesis 开始。加速状态不可直接构造或修改，可由 H 丢弃重建。
- 整段重算：`replay` 在每个需要计算的历史前缀从 genesis 折叠完整输入，按历史暂停及检查记录决定当时应用什么。不是只比较最终可见 b/g/λ。
- 来源/解释/规律变更：显式 `Rebuild` 选择已解析的新输入资源并强制从头。有效 H 投影须由所属层提供；没有记录时拒绝，不能重新调用模型或从文本猜测。说话撤回仍是普通经历。
- 手动暂停：记录事件继续追加，状态与已应用版本不变；目标版本可推进但不冒充已应用版本。恢复强制补算全部历史。没有自动暂停或结果否决命令。
- 检查：`CheckRecord` 绑定谱系、H 前缀、规律/解释/来源版本、报警 ID。相同结果、缺失、失败、超时都应用并暂存双接收方通知；明确不一致或计算故障拒绝完整候选，下一次计算从 H 重试。新发现的数学故障须先作为有绑定的记录进入输入，不能只写不可重放的日志。历史检查的耗时/结果不会在重放时重做。
- 复核：只接受命名既有报警的“规律正确/规律有错”；两种结论都不回滚结果或暂停成长。规律修复是另外一次版本事件。
- 探测：`ProbeSet` 从显式校准输入构造四通道各正反一次的八个中性向量；通过调用方的 F/J 投影取得 P5-12 数值输出。按系数身份比较并检查身体三项，严格大于阈值报警。校准版本、行序、资源、完整编译和有限性必须有效；没有默认幅度或默认阈值。

## 验证

从仓库根运行（独占编译窗口；每次使用新 RunId）：

```powershell
pwsh -NoProfile -File core/crates/core-kernel/src/recompute/verify.ps1 -Mode Test -RunId test-example
pwsh -NoProfile -File core/crates/core-kernel/src/recompute/verify.ps1 -Mode Clippy -RunId clippy-example
pwsh -NoProfile -File core/crates/core-kernel/src/recompute/verify.ps1 -Mode Format -RunId format-example
pwsh -NoProfile -File core/crates/core-kernel/src/recompute/verify.ps1 -Mode WorkspaceTest -RunId workspace-example
```

临时 crate 位于 D 盘，直接编译本目录及同树 P5-24，链接同树 G/C/J；不修改 `lib.rs` 或工作区清单。`tests/legacy.rs` 只在测试构建存在，显式使用旧规律；不是现行 ADR 0032 的生产适配器。全部身份与数值均为合成。

P5-23 的 Python 对照保留原入口：工作区测试执行 `law::tests::reference_parity::adr0019_python_legacy_and_extra_same_input_all_scalar_bits` 和 `law::tests::reference_events::adr0019_python_events_review_and_side_tapes_match_g_phase_machine`；可用进程变量 `P523_PYTHON` 指定 Python 3.12。原 Python 源、规模与断言不改。这是标量/份额/事件计划对照，不是完整有效 H/证据组装/G/C 第二实现。

## 未完成的接线

完整独立第二实现、真实 H 来源与解释投影、完整 C 规范导出、H/L 同事务持久化、校准资源与正式 F/J 探测投影、V2→V3 迁移/回退都仍需接续，见 `tasks/requests/P5-26-integration.md` 与 `reports/P5-26/BLOCKED.md`。`verify_candidate` 的成功测试只证明比较/分流机制，不能证明提供者独立。未提供真正第二实现时必须记 Missing，不能将 `replay` 再跑一次标为 Consistent。

当前实现优先可审计性，完整投影和历史副本有随 H 增长的成本；没有做生产长历史性能验收。未来缓存优化必须保持逐位结果，并仍可从 H 完全重建。不能用截断历史来消除成本。
