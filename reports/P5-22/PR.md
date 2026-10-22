P5-22 将终端契约升级到 CONTRACTS v2.2：身体读出 v2、主动边界、成长查询/通知/暂停恢复、载荷摘要绑定审批，以及社交行动权限。旧消息和旧字节保留，变更使用新标签的后继消息；舞台迁移留给 P3-06，核心 P5-19 起只生产读出 v2。

按用户追加授权补齐 ADR 0007 已规定的 required oneof：u32 选中字段号后接值，缺失/多选/未知号/尾随字节均拒绝；同步 Rust/Python/TypeScript、生成器和兼容门禁。结合 Claude 的 P5-01 回溯审查，补了文本解析与字段升序断言，冻结已发布的 P5-32 基线。未改变其他编码规则。

验证：
- 本机最终 26 项命令全部退出 0：pnpm lint/typecheck/test/build、test:contracts（325 测试）、Rust fmt/clippy/workspace tests、三个 worker 的 ruff/mypy/pytest。
- pnpm test:core-contracts：三语言各 10,471 例通过，protoc 433 互操作向量通过。
- check_compat：p5-01、p5-21、p5-32、current、origin/main 全部 0 违例。
- 原有 396 合法、409 非法、18 Unicode 向量逐条前缀不变；三语言各 1,627 个旧向量编解码查询，字节/摘要 0 差异。新增 37 合法、39 非法向量；ReadoutV2 有 4 个合法向量与 12 个非法向量。
- 最终检查前 fetch/merge 最新 origin/main，Already up to date（91b31ed）；全部证据含失败记录在 reports/P5-22/。

报告：reports/P5-22/REPORT.md；配套修改单列章节；规格对照：docs/traceability/P5-22.md；字节哈希：reports/P5-22/frozen-byte-comparison.json。

**请 Claude 交叉审查。** 请一并审查 CONTRACTS、terminal.proto、oneof 生成器/三语言编解码器、兼容门禁与黄金向量证据，特别核对 V2-51–54 的具体表示。实现方不自行给出交叉审查结论。

保留 `needs-user`：等 Claude 审查通过，由用户批准并 squash 合并。config/opportunities.json 的空冷却表候选、缺失 helpers.json 时沿用 30/天预算、原样 bytes 及 SHA256 绑定、v1 作为兼容 oneof 分支而保持类型可引用均在报告与差异登记中明示。无 GPU、无用户谱系访问、无新增依赖。
