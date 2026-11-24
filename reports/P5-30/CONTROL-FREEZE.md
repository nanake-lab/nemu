# P5-30 L0-control-1 冻结记录

第一版对照 / 非正式结论。尚未运行生活轨迹；本记录推送之后按完整清单执行。

- 预注册：已合并 PR #192，v0.2 §§1–7；基线 ecdec912a7b9f69def84dad61a7cad29f2518c44。
- 源码与只读分析：a0e0f680fd207f90588a75b47e843fa8447f9e6d。
- 私有执行包：[nemu-evidence / 7340ee79f](https://github.com/nanake-lab/nemu-evidence/tree/7340ee79f3e13a0058343cab9fcbaaa8d105c8c7/P5-30/v0.2/L0-control-1)。事件、候选、μ、探针、run list、参数、资源和 SHA 清单均已推送。
- sha256.json SHA-256：fb45bf48ca258cd02fd51c12cdda3ac0b0be061cd0428e978712a8f5f708e0c2。
- Windows release 二进制 SHA-256：5f1f80efa1d7195342b11608da6ca3157f8ce74f564ec17ab7d5d955e18533fa。
- rustc 1.99.0 (b940084d7 2026-09-28)，Python 3.12.10；cargo --release --offline --locked，日志明确 optimized，debug run/continue 会拒绝执行。
- 180 ready / 20 not_run；435810 个预定主事件；固定两种子，含完整敏感性和三年反馈。没有试跑成长、挑种子或回调参数。
- 阴性对照：同实现完整前缀重算；Q1 U/V → X/Y 与事件/原因 ID 重命名后全状态逆映射；Q2 双份相同完整状态共同续接逐步一致。
- 预计最慢：G 的历史分配校验/重算，其次整段重放和 Q2 多分支。预估来自代码路径，不来自提前试跑。实测分段时间随结果报告。
- 运行位置：私有证据树 `P5-30/v0.2/L0-control-1/run01`；只新建、串行 CPU、gzip 原始记录、2 GiB 磁盘保护。

## 必要检查

工作区 release 测试通过，既有 1 ignored 原样保留；最终适配 crate 的三项机械测试通过。工作区 clippy --all-targets -D warnings、fmt、Python 三项分析反例测试、四文件 mypy --strict 通过。全工作区测试后新增的阴性对照由适配 crate 最终测试覆盖，内核算法无改动。最初 debug 长测试中止和此前机械失败保留，不把它们记通过。

执行指示来自本轮用户明确授权；冻结后不再修改执行源码/输入，不重试失败单元，不以结果改变矩阵或参数。
