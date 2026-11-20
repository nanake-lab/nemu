# P5-69：导出规律状态数值版本标识

依据：PR #150 交叉审查非阻塞项 2；用户 2026-11-11 授权登记最小请求。

`core-types/src/semantic/theta.rs` 已定义 `LAW_STATE_NUMERICS_ID`，但 `theta` 是私有模块，`semantic.rs` 尚未重导出。因此 dynamics 暂时无法引用它。

请求契约所属卡在 `core/crates/core-types/src/semantic.rs` 公开重导出该既有常量，不改变字面值、IDL 或校验行为。导出后，将 dynamics 的 `COMMON_DISCOUNT_NUMERICS_ID` 改为该常量的别名重导出，保留现有公开名称；无需重构。当前 V3 语义校验仍强制检查标识一致。本请求不阻塞第 7 条的控制事件测试修复。
