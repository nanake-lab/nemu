# ADR 0019 推论 6 方案 A 与长历史对照

## 当前状态

用户已确认（2026-10-23）方案 A：保留候选 B，接受失去固定名义 ρ 下的 E/C 总量上界。ADR 0019 §11 已勾选、写明当前版本的替代保证和证明条件，V2-60 追加登记。仅确认这一差异，ADR 整体仍为提议，PR #61 保持 `needs-user`，未合并。

本次工作区 `D:\Dev\nemu-wt\law-lp1-reference`，所有临时/uv/mypy缓存在 `D:\Dev\cache`。开始先 fetch，合并 main `382c0e7`（`07fe936`）；随后合并 main #68 `9f69648`（`5080c90`）后复跑；最后 fetch 合并仅P5-20卡追加5行的 main #69 `49069e17864a7667ac9684308da78b9601d4a9b4`（`5326077`）后再复跑；未使用 C 盘旧工作区。开始前读取第三轮[审查通过结论](https://github.com/nanake-lab/nemu/pull/61#issuecomment-6360876090)。本次没有改规律公式、现有八个 Python 模块、原 check_law.py、Rust、IDL、原规格或 PR #49 分支，没有浏览器、其他应用或 GPU 操作。

## 交付物

- [ADR 0019 §11.5](../../../docs/adr/0019-law-lp1-reference.md)：固定旧组权重趋零的充分条件、固定 h 的越线时间证明、当前版本范围与 P5-30 后续。
- [差异登记 V2-60](../../../docs/traceability/diff-register.md)：用户确认的保证变化；原推论6、LAW-6原文保留。
- [compare_law_v2_long_history.py](../../echo-core-v2-draft/compare_law_v2_long_history.py)：调用原/新参考实现逐组更新，无闭式捷径或随机采样。
- [counts.csv](long-history/counts.csv)、[端点](long-history/endpoints.json)、[summary](long-history/summary.json)：20组实验、参数、首次越线前后状态（含 binary64 hex）、Python版本、源SHA256及[源快照](long-history/source/)。
- [原始 stdout](long-history.stdout.txt)、[stderr](long-history.stderr.txt)、[命令/时间/退出码](long-history.meta.json)：exit0，无数值故障，失败记录没有覆盖。

## 比较协议与结果

单倾向，P=0、n₀=1、κ=0、W=1；先 N 组 S=+1，再连续 S=−1。h₀=0.1、n_h=10。固定 h 使用 `fixed_rho=0.9`；自适应调用现有估计，先用组前计数算 h、处理证据，最后更新符号翻转和组数。c、n_h 是本次明确固定的比较参数，不声称 Claude 的简化估算已指定它们。

计数为第一组使更新后的 π **严格小于 1/2** 的反对组序号；不是 π=0 的过零延迟，也不是另一套产品察觉门槛。相同支持前缀复用后复制状态，避免不同历史；全部实际递推，无闭式代算。20个组合均直接越线，无删失、无故障、不设门槛。两次main同步后分别另跑[long-history-post-sync](long-history-post-sync/)和[long-history-final](long-history-final/)，CSV与端点均逐字节相同，三次原始stdout/stderr/meta分别保存，未覆盖第一次输出。

| N | 固定 h 旧 | 固定 h 新 | 自适应 h 旧 | 自适应 h 新 |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 2 | 2 | 2 | 3 |
| 100 | 3 | 11 | 16 | 21 |
| 1,000 | 3 | 28 | 146 | 197 |
| 10,000 | 3 | 49 | 1,449 | 1,953 |
| 100,000 | 3 | 71 | 14,473 | 19,516 |

Claude 提供的 N=100/1,000/10,000 固定 h 新11/28/49、旧3，自适应新21/197/1953、旧16/146/1449，全部与实际参考实现相同，12项差值均为0。没有通过改变门槛、计数起点或公式来匹配估算。

固定 h 时，新形式的适应时间随前期一致支持变长而增加；ADR给出的无统一常数上界证明仅针对这一指定场景。自适应时，十万组历史下新旧两式都需要很多反例：此时首次反对的 h 约为9.999×10⁻⁶，不能把迟缓全归因于新形式。有限的五个 N 不用于证明所有自适应序列的增长率。

## 替代保证的限定

任意固定旧组权重趋零的证明限定为：不改该组历史份额，此后每步固定非零有效权重，名义固定0<h<1，或当前自适应 h 且 h₀n_h>0。符号可以不同；由 δ 的调和下界得 ∑δ发散，∏q趋零。它不保证无事件、全零信号、h=0或任意衰弱输入下也趋零；这是充分条件，不是新设证据准入下限。实数极限不保证有限 binary64 能无限运行，数值故障继续显式报告。

用户指出的“长期稳定后连续几次反例应被很快察觉”仍未满足，明确记录为现有 h 估计的已知不足。替代保证仅适用于当前参考版本；由 P5-30 对照变化检测（包括变点检测写法），再决定是否作为新规律版本替换。本次不选检测器、不改公式或添加隐藏状态。最终同步的main #68把参考实现预研安排到P5-33，P5-30以其结论在真实实现复测；本PR不实施这些卡。

## 同步 main 后复跑

| 命令 / 检查 | 结果 | 原始证据 |
| --- | --- | --- |
| `python -B reports/echo-core-v2-draft/check_law.py` | exit0，原13项通过 | [old](../verification/attempt-10/old.stdout.txt)，同名stderr/meta |
| `python -B reports/echo-core-v2-draft/check_law_v2.py --suite legacy` | **exit1，原LAW-6失败：3,000序列、3,064状态违例**；其他九项通过 | [new-legacy](../verification/attempt-10/new-legacy.stdout.txt)，同名stderr/meta |
| `python -B reports/echo-core-v2-draft/check_law_v2.py --suite extra` | exit0；80,000整步、三条LP1及R1/R2回归无违例 | [new-extra](../verification/attempt-10/new-extra.stdout.txt)，同名stderr/meta |
| `python -B reports/echo-core-v2-draft/compare_law_v2.py --out …/attempt-10/worlds` | exit0；四世界全部输出与attempt-09/08/07相同，无新门槛 | [worlds](../verification/attempt-10/worlds/)，[stdout](../verification/attempt-10/worlds.stdout.txt) |
| `python -B reports/echo-core-v2-draft/compare_law_v2_long_history.py --out reports/law-lp1-reference/decision-20261023/long-history` | exit0，20组合 | 本目录stdout/stderr/meta与long-history目录 |
| mypy 1.18.2 `--strict --follow-imports=silent`，八个既有模块及新增对照 | exit0，9文件无问题 | [static-10](../verification/static-10/)；meta列出全部文件 |
| 原始证据/公式保护 | 既有8模块与96a899e逐字节相同，原check_law.py与origin/main相同；最终10与09/08/07的四命令stdout/stderr、四世界4文件均相同 | [provenance](provenance.json)、[SHA256清单](SHA256SUMS.txt) |

原 LAW-6 的种子20261019、3,000序列、判据、退出码全部保留。用户接受差异不等于该判据通过。原失败见证、审查前后输出以及attempt-01至07未删改。

初次生成的清单与provenance在最后同步前形成，原字节保留为inventory-before-final-sync.sha256、provenance-before-final-sync.json，仅记录当时工作区；当前内容以本目录SHA256SUMS.txt、provenance.json为准。

全部原/新/extra/四世界检查由 `routine_reporter`（GPT-6 Luna、max，按角色名复用）单次串行执行，主代理核对原始日志；标量实验与类型检查由主代理执行。首次attempt-08的子代理外层PowerShell会话最后一条轮询未获得wrapper退出码，四个内部meta完整，未为此重跑；随后因main #68/#69更新才分别另跑attempt-09/10。三次attempt的内部meta分别保留退出码0/1/0/0及时间，各四份stderr为空。Python3.12.10、Windows19045。mypy使用D盘已有缓存离线解析固定版本；static-08的stderr记录缓存包安装，最终static-10的stderr为空；无新增运行时依赖。

## 规格对照与待办

| 条款 / 用户指示 | 位置 | 证据 |
| --- | --- | --- |
| 推论6方案A、版本范围、原LAW-6失败保留 | ADR§7/10/11，V2-60 | legacy原始失败与新旧脚本字节核对 |
| 固定旧组让位、长历史适应边界 | ADR§11.5实数证明 | 20组参考递推及端点 |
| 整步LP1、反事实、新候选首次分配、分侧修复、I23/I25 | 既有8模块不变 | extra原始输出；同上轮逐字节一致 |
| P5-30含变点检测的对照 | ADR§11.5已知不足 | 后续工作，本次未实施、未声称验收 |

仍等待用户处理整份ADR/PR，`needs-user`保留；本次不合并、不操作其他执行者或应用。CI按仓库docs/reports路径过滤预期不触发，推送后由主代理核实实际checks/runs；无检查时明确报告为空，不称CI全绿。未运行Rust/TS/真实存储/θ或行为验收，不拿这些标量实验代替它们。
