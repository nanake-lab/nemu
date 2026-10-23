# P5-02 单写事务与对象存储（W02）— 认领

- 基线：origin/main 91b31ed2d30aadd8cc1cf8086dd0aad8ffe03821。
- 分支：task/P5-02-store；实现范围 core/crates/core-store/，共享清单与只追加证据按 AGENTS.md。
- 状态：实施中。全部验收完成前保持草稿；完成后同步 main 并重跑检查，请 Claude 交叉审查，通过前不合并。
- 所有测试、基准与故障注入仅使用新建临时目录，不打开用户排练库或正式库。
- 不使用 GPU；重负载串行调度。
