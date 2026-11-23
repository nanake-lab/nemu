# P5-06 阻塞

本卡尚未完成，PR 保持草稿，不合并。可独立实现的部分是结构预检，不是完整语义核心。

## 已查明

P5-04/P5-21 已合并不代表所有 ObjectRef 指向的对象已有正式类型。主线 contracts 顶层 proto 中没有 SourceSpan、provenance、Concept Registry/alias 快照及 Claim operand 的对象定义；只有对应引用。详见 [接口请求](../../tasks/requests/P5-06-formal-semantic-boundaries.md) G01–G05。

已经核对生成类型、正式 proto、fact-view 的来源读取与 core-types codec。fact-view 可读取 UserText 等正式对象，但不能据此把任意 UserText 视作已经验证的跨度。现有数值与规范编码检查直接复用，不新增数值阈值。

## 影响

- 完整 IRL 阶段链、ACCEPTED/⊥sem 对编排的正式接口、provisional 原子登记、完整属性生产、确定性语义图适配尚未实现。
- preflight 的 Ok/输出图均不是 source/span 已验证、事实正确、证据合格或可提交的证明；不提供提交权限。
- SYS02、SYS04、SYS03 结构部分及 16.15/16.16 的整项验收尚不能签发。特别是越界 span 不能用普通 ObjectRef 尺寸检查冒充完成。
- 新的差异登记本身属于技能规定的阻塞项；不把合成测试当正式验收。

## 可选后续

1. 由独立契约卡补足 G01–G05 并交叉审查，合并后 P5-06 消费正式类型继续。本卡不进入 #108/#64 的范围。
2. 用户明确批准将本 PR 缩为“语义结构预检”，其余要求另立后继卡；当前未获此授权，所以仍按完整 P5-06 标为部分完成。

当前独立部分继续完成测试和代码检查；不需要用户授权即可进行这些检查。
