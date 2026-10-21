# Echo Core 2.0 评价与学习修订研究

- 状态：用户于2026-10-21直接委派；候选报告与实验提交范围已授权，架构采纳和合并未授权。
- 范围：新增 `reports/echo-core-v2-revision/`、`docs/adr/0013-grounded-experience-learning.md`、本请求；只追加 `docs/traceability/diff-register.md`。
- 分支：`codex/echo-core-v2-revision`。这是独立研究请求，不认领或完成任何 P5 实现卡，不修改任务目录或 cards.json。
- 目标：明确2.0保留/删除/替代的条款、给出可运行的最小研究实现、保留反例与失败边界，提交PR由Claude审查、用户决定合并。
- 不修改：现行规格原文、IDL、core/、workers/、生产配置、正式/排练存储、验收阈值。
- 验证：研究代码的 `mypy --strict`、`ruff check`、单元测试、预先固定的多种子合成实验、源码和结果哈希、复现比较。
- 资源：仅CPU合成实验；无GPU测量。主代理设计和裁决，routine_reporter仅运行既定命令和收集证据。
- 开放PR #36修改P5-21 IDL；本请求不修改其目录。本研究会说明未来契约影响，不替该PR裁决。
- 交付：[报告](../../reports/echo-core-v2-revision/REPORT.md)、[候选条文](../../reports/echo-core-v2-revision/PROPOSAL.md)、[ADR](../../docs/adr/0013-grounded-experience-learning.md)。
