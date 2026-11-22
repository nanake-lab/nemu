# P5-30 L0 首版对照 — 规格对照

仅对应预注册 v0.2 §7 的合成对照，不签完整 P5-30 或诞生验收。

| 条款 | 实现 | 检查 |
|---|---|---|
| v2.0 §2.4、I03/I04 | experiment-world/src/adapter.rs 调用 Φ/F_in/G/C/F_out，完整阶段成功后返回新状态 | mechanical_stage_replay_and_probe_isolation；既有核心阶段测试 |
| v0.2 §2/§5，EV2/EV6/I24 | world.rs 的固定脚本、独立原因/时间，合成原始 a 与候选/μ 表；G/C 仅消费原始评估 | world_tables_have_fixed_counts_and_q1_sequence；源码边界 |
| v0.2 §3.1 | probes.rs 从同一冻结前态移除原始项后调用原 compute_drive/apply_drive；不从舍入总量减项 | 探针隔离、零边际归因的分析反例 |
| v0.2 §3.2–3.4 | analyze.py 同倾向同探针 30 日，末 90 日分类，配对逐坐标轨迹与种子分报 | 三项 fabricated-row 分析测试 |
| v0.2 §6 | q2.py 严格固定候选库、完整字段差、精确/近似分层；continuation.rs 克隆合成检查点作对称分支 | 机械检查点恢复逐位一致；未来轨迹状态由运行记录提供 |
| I16/I23 | 正常构建 feature 导出的 P5-26 start/step/replay 与真实 G/C/F；主线另从 genesis 重算 | 三事件机械重放；每运行单元的 same_implementation_recompute；独立第二实现 unavailable |
| LP5/AC2-05 | 日检查点只读投影，W5 不提交事件 | no_event_growth_and_zero_marginal_rounding；W5 轨迹 |
| I18/I29 | 无权威存储 crate、无模型/网络依赖；实验目录 create-new，资源标记 EXPERIMENT，缓存不持久化为权威 | 源码审查、类型边界；不声称生产谱系隔离联测 |
| v0.2 §4/§7 | 全量表/参数/分析代码先推送冻结；故障保存原前缀、重放失败见证，余项原样记录 | 冻结提交与私仓索引、result.json、原始失败日志 |

未更改 IDL、黄金向量、G/C/F 公式、容量或验收阈值。新 crate 的直接第三方依赖仅复用工作区既有锁定 serde 1.0.229、serde_json 1.0.154、sha2 0.11.1（MIT/Apache-2.0）；Python gzip 为标准库，无额外安装。没有新增规律或待选择 U 项。
