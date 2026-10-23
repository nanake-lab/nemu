# PR #61 审查修复 — 2026-10-23

## 结果

两处实现阻塞已修复并补充检查，待复审；新规律仍为提议，`needs-user` 保留，不合并。推论6仍未满足，原LAW-6继续失败。ADR 0019 §11单列影响与可选做法，**没有选择放弃或修改原推论6**。

审查对象为 `f10d8530d6694bebf53daaed4486172e6ea62af8`，[审查评论](https://github.com/nanake-lab/nemu/pull/61#issuecomment-6356460860) 完整 JSON 保存在 [review-comment.json](review-comment.json)。本轮先 fetch，merge `origin/main=876cff7211389f7058668dfb2b9592634d517abc`；规则与相关规格没有变化。修复后再 fetch/merge 确认仍为该 main，以下全部检查都在已经包含它的分支上运行。main 新增的 core-store 与本轮研究修复无关，未修改那些 Rust 文件或 PR #49 分支。

## 阻塞1：反事实漏掉新候选自身

修复前按审查方原脚本复现一次，原始 [stdout](before/stdout.txt)、[stderr](before/stderr.txt)、[命令元数据](before/meta.json) 与 [源副本](before/source/) 都保留。它返回 cf[b]=after[b]=0.4794520547945206，而独立预期 cf[b]=0；旧随机检查没有遍历该首次分配。早期“零违例”不能覆盖此问题，也不能证明事件定义正确。

`law_v2_events.py` 现在分开处理：旧分配仅改份额并重放；回溯产生的所有首次分配进入 `FirstAllocation` 计划；初始化候选、捕获 cf；最后才加入这些首次分配和当前新组。份额计划保持相同。cf是不可提交的审计暂态，只有全部首次分配完成且I25守恒后才原子发布。

`law_v2_event_checks.py` 保留原250×24随机带，新增3,000个回溯候选目标。独立oracle仅遍历事件前已经存在的分配身份，用实际修订后的份额折叠，**不调用 prepare/revise/apply**，从构造上省去全部首次分配。新建倾向的 cf 由无证据初值独立给出；已有其他证据的倾向则保留那些旧证据，不一律当成0。

`law_v2_review_checks.py` 另加150×24随机事件，包括仅回溯1,800次、回溯+新组1,800次，其中1,200次目标倾向已经有其他组的证据。检查首次分配前后方向、完整新增边际、独立cf、重放与守恒。修复后的最小反例：cf={a:0.6122448979591837,b:0}，after={a:0.6122448979591837,b:0.4794520547945206}，新候选自身的支持边际严格为正。

## 阻塞2：份额乘法的正数下溢

修复前同一脚本记录了 min-subnormal×0.5 被接收，a/b均 π=0、组数1。该失败输出保留，不用修复后的结果覆盖。

`nonnegative_product` 在乘法之前保留两个正因子的信息；有限正数乘积若舍入为0，显式 `UnderflowEvidenceWeight`。它用于首次插入和旧分配重放的 weight×share；标量和重放的 W×|S| 同样检查，后者故障为 `UnderflowEvidence`。没有提高最低权重或μ、没有夹紧、没有把弱证据算作合法零信号。

`apply`、`append` 在独立暂存对象执行，所有阶段和I25检查完成后才发布；`replay_tendency`也在临时倾向完整折叠成功后才替换。新增七个边界均检查显式故障与**全快照不变**：

| 用例 | 实测故障 | 说明 |
| --- | --- | --- |
| 新正支持各半份额 | UnderflowEvidenceWeight | 事件入口 min-subnormal×0.5 |
| 新反对各半份额 | UnderflowEvidenceWeight | 同上，反向也不能静默丢失 |
| 回溯中新候选首次分配 | UnderflowEvidenceWeight | 由合法较早事件构造一单位的正份额 |
| 旧份额调整后的重放 | UnderflowEvidenceWeight | 较早事件合法，份额减小后才发生下溢 |
| 混合事件晚阶段失败 | UnderflowEvidenceWeight | 旧份额已改且候选已暂存，仍不发布任何部分变化 |
| 存储行注入：重放share下溢 | UnderflowEvidenceWeight | 单独覆盖fold防线，不称合法历史 |
| 存储行注入：重放signal下溢 | UnderflowEvidence | 同上，覆盖W×|S|防线 |

真零权重、真零支持仍合法，π=0且按原约定记组；旧状态遗忘及M/U分别舍入的合法下溢语义不变。原7个数值故障用例和M=2^-1074、U=0合法例也继续通过。

## 推论6：只补决定所需材料

[ADR 0019 §11](../../../docs/adr/0019-law-lp1-reference.md#11-推论-6失去的保证行为影响与待选做法2026-10-23) 单列三件事：

1. 失去的是折算证据总量的统一上限，不是H历史保留规则；单个旧组渐渐变轻不保证总E/C有界。
2. 长期一致经历可能增加惯性、降低偶然误翻转，也可能延迟真实变化察觉；512步E2不足以保证更长历史的速度。总量过大还可能更早触发binary64精度故障。
3. LP1并未被证明与有限上界本身互斥，但硬截断和强制固定折扣不能直接解决。给出开放上界新增步的实数可行构造，并明确它没有解决回溯份额增加、固定历史q、完整重放与其他性质。

提供A–D待选做法：是否接受当前候选的保证变化；坚持有限上界继续研究；重新设计π/统计参数化；或仅研究有界察觉量（后者**不恢复推论6**）。没有默认选择，没有把探索公式写入实现，没有改旧判据。L过小可能令复杂倾向永远停在候选，或使依恋无法改变，代价也明确列出。

## 全部复跑与范围

执行者为 `routine_reporter`（按角色名调用，GPT-6 Luna/max），主代理提供确切命令与CPU窗口、核对证据并作判定。Python 3.12.10 / Windows，无GPU、用户数据或新运行时依赖；不启动Rust/TS/真实存储测试。

| 命令 | 内部退出码 / 结果 | 原始证据 |
| --- | --- | --- |
| `python -B reports/echo-core-v2-draft/check_law.py` | 0；原13项failed:none | [old.stdout](../verification/attempt-06/old.stdout.txt)、stderr/meta |
| `python -B reports/echo-core-v2-draft/check_law_v2.py --suite legacy` | **1；仅LAW-6失败，3,000序列、3,064状态违例** | [new-legacy.stdout](../verification/attempt-06/new-legacy.stdout.txt)、stderr/meta |
| 同上 `--suite extra` | 0 | [new-extra.stdout](../verification/attempt-06/new-extra.stdout.txt)、stderr/meta |
| `compare_law_v2.py --out <attempt-06/worlds>` | 0；四世界每种128×512 | [worlds.stdout](../verification/attempt-06/worlds.stdout.txt)、四份原始世界数据 |
| mypy 1.18.2，strict，7文件 | 0，7文件无问题；stderr为工具下载信息 | [static-06](../verification/static-06/) 含完整命令与元数据 |

旧LAW的种子/规模/判据不改；原check_law.py字节不改。四命令都在 `run_checks.ps1 -Attempt attempt-06` 下串行运行，外层0只表示继续收集，不代替内部退出码。源快照与运行文件逐字节匹配，见 [provenance.json](provenance.json)。

| 检查 | 样本 | 违例 / 差异 / 正常域故障 |
| --- | --- | --- |
| 完整同号新增步 | 80,000步、279,592组 | 新0；旧同带21,406；新故障0 |
| 原随机事件：无修订 / 含回溯新增边际 / 撤回 | 9,000 / 12,000 / 5,019 | 各0；含3,000个此前漏测的候选目标；oracle差异0 |
| 原完整H前缀 | 6,000 | 重放0，守恒0，故障0 |
| 新仅回溯 / 回溯加新组 | 1,800 / 1,800 | 候选总边际0、3,600次首次分配方向0、oracle差异0 |
| 新完整H前缀 | 3,600 | 重放0，守恒0，故障0 |
| 标量序列化与M/U重放 | 8,000 | 位差0 |
| 精确份额 / 匹配前态性质 | 20,000组 / 10,000次 | 0 |

其余审查意见：LP3的LAW-9不是完整组装器验收；LP4/LP6/LP7的控制条件保持明确；I23不等于事件定义必然正确，本次独立cf检查补足已知遗漏。真实G、θ、容量、来源更正、暂停、CanonicalV1、存储与行为验收仍未运行或被这些参考检查证明。CI按docs/reports路径过滤，目前没有检查，不能称“CI全绿”。

早期attempt-01至05、原审查脚本/失败、旧清单完整保留；新的根SHA256清单用于当前树，旧清单副本在 `before/source/SHA256SUMS.f10d853.txt`。修复不触碰Rust/IDL/原规格/PR #49，也不解除它的阻塞。

## 下一步

在PR逐条回复并写「请审查」，等待复审和用户对推论6等设计选项的决定。本轮没有批准新规律，没有选择推论6方案，不合并PR。
