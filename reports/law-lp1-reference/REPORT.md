# LP1 参考形式研究 — 当前报告

用户已于2026-10-23选择推论6方案A：保留候选B，接受失去固定名义ρ下的E/C总量上界。ADR0019仍为提议，PR #61保留needs-user，等待用户处理；没有改规律公式或解除P5-23实施阻塞。

本次决定、替代保证证明、长历史对照及复跑的完整证据见[决定报告](decision-20261023/REPORT.md)。ADR§11勾选方案A，差异登记追加V2-56。固定旧组权重趋零有持续非零证据等充分条件；固定h下指定支持后反对场景的适应时间随历史增长、无统一常数上界。保证仅适用于当前参考版本；用户提出的长期稳定后应很快察觉几次反例，作为现有h估计的已知不足，交P5-30比较包括变点检测的写法后决定新版本。

| 本次交付 / 检查 | 结果 | 证据 |
| --- | --- | --- |
| 长历史对照，N=10/100/1k/10k/100k，固定/自适应h，新/旧 | 20组合完成，无门槛；Claude的12项估算全部复现 | [CSV](decision-20261023/long-history/counts.csv)、端点/参数/源快照/原始输出同目录 |
| 十万组支持后的反对越线组数（旧/新） | 固定h：3/71；自适应h：14,473/19,516 | 同上；P=0、c=1、W=1、h₀=.1、n_h=10 |
| 同步main 49069e1后原检查 | exit0，13项通过 | [attempt-10](verification/attempt-10/) |
| 新形式原LAW-1–10判据 | **exit1，仅LAW-6失败：3,000序列、3,064状态违例** | [原始失败](verification/attempt-10/new-legacy.stdout.txt) |
| 额外完整事件/反事实/分侧/I23/I25检查 | exit0；80,000整步、三条LP1、R1/R2回归无违例；与上轮输出逐字节相同 | [extra](verification/attempt-10/new-extra.stdout.txt) |
| 四世界对照 | exit0，全部输出与attempt-07相同；只报告，无门槛 | [worlds](verification/attempt-10/worlds/) |
| strict mypy 1.18.2 | exit0，9源文件无问题 | [static-10](verification/static-10/) |

新增脚本[compare_law_v2_long_history.py](../echo-core-v2-draft/compare_law_v2_long_history.py)直接调用未修改的原/新递推。现有8模块与已审查96a899e逐字节相同，原check_law.py与main相同；不改Rust/IDL/原规格/PR #49。工作区D盘，缓存D盘；无浏览器、其他应用、GPU或用户存储操作，无新增运行时依赖。CI的docs/reports过滤及实际推送后状态在最终交付中说明，无检查不称全绿。

最初候选比较、推论1–7影响、数值次序与反例保留在[ADR0019](../../docs/adr/0019-law-lp1-reference.md)。第一轮[修复报告](review-20261023/REPORT.md)、第二轮[修复报告](review-20261023-r2/REPORT.md)与全部失败证据仍保留；第三轮[审查已通过](https://github.com/nanake-lab/nemu/pull/61#issuecomment-6360876090)。旧报告中的当时未选择状态是历史，当前以ADR§11和决定报告为准。

原LAW-6原文和失败不变，不把用户接受差异说成通过原判据。本次只交付设计和参考实验；Rust、真实存储、θ与产品行为验收未运行。下一步等待用户处理PR，停止工作，不另取卡、不自动请求其他执行者审查。
