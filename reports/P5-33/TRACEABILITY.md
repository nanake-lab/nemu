# P5-33 规格对照与范围

表放在本卡唯一授权目录；未改`docs/traceability/`，未新增正式差异编号。

| 规格条款 | 原文要点 | 研究位置 | 证据/边界 |
| --- | --- | --- | --- |
| v2.0§0.4、5.4 | 换参考写法是新规律版本 | ADR-待定.md | 只建议，不发布；编号待分配 |
| v2.0§5 LP1–LP8 | 方向、分摊、去重、变化率、无时间变化、复杂度、惯性、可重算 | integration.py、check_properties.py | checks-03；参考范围见报告，非产品AC2通过 |
| ADR0019§2、5–6 | 冻结历史q、独立cf、撤回和新增分别检查 | 原Engine+ResearchEngine | old-identities oracle、40前缀/方法完整重放 |
| ADR0019§4 LP4 | 匹配同前态与新证据时h大使q不增 | check_properties.py、sensitivity.py | 200对/方法；lp4_worlds.csv，固定h不是学习候选 |
| ADR0019§8 | binary64唯一运算次序、严格精度故障、精确份额 | 原Tendency/exact_shares、detectors.py | 无容差通行；下溢后全部状态未提交 |
| ADR0019§11 | 已选择保留候选B，失去原推论6上界 | REPORT.md、original-LAW-6.log | 原判据3064违例、failed，未替用户改该决定 |
| v2.0§5.3推论1/2/3/5/7 | 门槛、相反证据条件、无事件不变、依恋先验 | REPORT.md逐条说明 | 代数不改，轨迹改变；依恋保留证明条件 |
| I18 | 影响未来的状态不能隐藏 | Controlled.detector、METHODS状态表 | 所有研究状态参与快照；生产编码未实施 |
| I23、LP8 | 增量与全重算逐位一致 | full_state、事件前缀重放 | Python位模式/检测器状态；非CanonicalV1 |
| I25、LP2/LP3 | 同源一次、每一边份额和为1 | 原Engine及exact_shares、competition | 重复拒绝不变；同侧各.5、异侧各自1；组装器未实现 |
| I29、AGENTS黄金规则14 | 不含用户数据，不打开任何存储 | 全部脚本 | 只读源码及报告输出，内存合成状态 |
| ADR0014§3、5–6 | 未知不评负、判断偏差单独测、四世界对照 | METHODS、REPORT | 缺席负证据仅假设；判断偏差没有被本实验验证 |
| v2.0§8；P5-25 | θ变化率区分噪声/变点，反馈稳定须另验 | REPORT后续建议 | 未改θ；不把R二元后验宣称为θ方案 |
| v2.0§14；P5-30追加 | 生活情景和E2对照 | scenarios.py、run_study.py | 逐种子数据；无产品门槛/完整长时反馈结论 |

不变量检查机制：本卡没有生产类型能力或权威运行时断言。研究中I23/I25用可复现测试，I18用显式完整快照，I29靠合成输入及仅源码/报告文件I/O范围。真实谱系互不可读、原子数据库提交、CanonicalV1、来源更正、容量和θ均为not_run，不由本报告代签。
