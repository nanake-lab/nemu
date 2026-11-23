# PR #49 非有限中间量排查

范围：`core/crates/core-kernel/src/law/` 的生产 Rust；另搜索 core-kernel 其余生产模块，未修改卡外代码。依据审查评论 6379618075、ADR 0019 §8 和既有有限 binary64 契约。

| 位置 | 排查结论与处理 |
| --- | --- |
| `support::experience` 支持度 | 原先 `clip(2*p/r_tilde-1)` 会隐藏 Inf；现在依原运算次序分别检查 `2*p`、除法、减法，再执行既有裁剪与 μ 乘法 |
| 同函数逐成员峰值 | 每个 `dot(dev,d)` 在 `abs/max` 前检查；组均值有限不代表每个成员投影有限 |
| 同函数强度 | `max(r_tilde,peak)/sigma_k` 在 `w_max.min` 前检查；`w_max` 自身也先检查有限性；有限但大于上限的商仍正常裁到 w_max |
| `replay::typical_magnitude` | M、U、r_max 及非零 U 时的 M/U 在裁剪前检查；合法 U=0/M≠0 仍返回 0，分别舍入语义保留 |
| `plan::experience_plan` 的 r_tilde max | 两个操作数来自已校验的存储 r 与 `Law::new` 的有限参数；max 之前没有可能溢出的算术 |
| `change_rate::before_group` 的 h 下限 | 算术结果已先 `finite`；下限参数由 `Law::new` 校验，未增加新下限 |
| `recurrence::strength`、`relation::closeness_weight` | 既有合法输入 π∈(-1,1)，`2π-1` 和 `(1+π)/2` 有界且有限；保持只读函数签名及原裁剪 |
| `num::clip` | 保持“输入有限”的原前提；相关计算调用点如上逐一核对，不让 clip 替代数值校验 |
| core-kernel `effects/compile.rs` 的 min/max | 对有效期/时间值做排序，不是 binary64 数值裁剪，无本类中间算术 |

新增 `tests/finite_contract.rs` 七项回归：helper 的正负支持度溢出、2p 溢出、完整 G 最小复现与晚阶段回滚、helper/G 强度溢出、成员投影与非法非有限 cap、M/U 商溢出与标量 fold 回滚，以及有限裁剪和次正规数正向对照。G 的前置教导组另作成功对照，确保后续失败确实覆盖已经暂存的候选、分配、计数及 L 载荷，而非一个本就不能执行的前缀。

未增加参数下限、容差、输入跳过或替代公式。未修改原 Python 参考文件、IDL 或黄金向量。M/U 商溢出的独立故障回归补充 G 读出的有限性要求；它不由只保存原始 M/U 的 Python 标量模型替代。

修复前的审查原始失败材料复制到 `verification/finite-review-20261024/reviewer-fault-contract.*`，保留其原命令、源码和输出；这不是本轮新运行的本机红测。当前代码的实际验证以 PR 对应提交 CI 和修复回复列出的日志为准，不能使用旧提交的通过结果代替。
