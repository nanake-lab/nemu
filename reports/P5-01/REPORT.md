# P5-01 IDL、规范编码与黄金向量 — 报告

## 结果

部分完成：数值基础类型已实现并验证；初始字节契约 ADR 0007 待批准，完整 IDL 与三语言黄金向量尚未完成。见 BLOCKED.md。

## 交付物

- `core/crates/core-types/src/numeric.rs`：有限 binary64、显式限额的纳秒区间与计数；私有字段、受检构造，无隐式截断/饱和/回绕。
- `docs/adr/0007-initial-idl-canonical-encoding.md`：提议中的初始编码契约，明确字节布局、存在性、集合、哈希域与拒绝策略。
- `reports/P5-01/initial-wire-candidate.proto`：供评审的三个示例消息，不编译、不生成、不供运行时导入。
- `reports/P5-01/SOURCE-MAP.md`：任务卡全部消息类别与来源清单，逐项保留未完成标记。
- `docs/traceability/P5-01.md`：实现与验收边界。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| cargo fmt --all -- --check | 通过 | verification/fmt.log |
| cargo clippy --workspace --all-targets --locked -- -D warnings | 通过 | verification/clippy.log |
| cargo test --workspace --locked | 通过，新增 5 个数值单元测试 | verification/test.log |
| SYS21 数值子集 | 有限值、NaN/Inf、负零、限额与溢出用例通过 | numeric.rs；test.log |
| SYS21 完整验收 | **not_run**，未知 schema/枚举/消息解码未实现 | BLOCKED.md |
| D 第一篇 §10.8 三语言一致性 | **not_run** | 未发布 IDL 与编码器 |
| 每条消息 ≥3 合法 +3 非法向量 | **not_run** | 待完成 |
| 原规格与 ADR 0006 未修改 | 通过 diff 范围检查 | verification/checks.json |

命令在 core/ 执行。没有读取用户排练或正式存储；测试不创建存储。
这五个单元测试不代表完整 SYS21、W01 或 Echo Core 验收通过。

## 规格对照

见 `docs/traceability/P5-01.md` 与本目录 SOURCE-MAP.md。遵循
echo-core-work 技能，逐项区分已实现、待实现与待批准。新增登记仅 W01-D01。
资格分支和单根账本字段的来源与类型限定已列入清单，尚未发布消息或接入运行路径。

## 新增依赖

无。仅使用 Rust 标准库；没有新增运行时依赖、锁文件变更或部署阈值。

## 待用户确认

- [ ] **阻塞：** ADR 0007 初始编码契约（AGENTS §§2、7 与 CONTRACTS 修改规则要求）。
- C01 继续未解决、只阻塞 W09/W10；ADR 0006 保持 PROPOSED，v1.0 NO-GROWTH。
- C02 已决定，SYS01–SYS25 全部保留。

## 后续建议（不在本任务范围内）

后续存储/权限卡需要验证这些类型在入口处的强制使用；本卡不接入提交路径。

## 下一步

等待 ADR 0007 的编码契约决定；继续本卡，不将部分交付标为完成或合并。
批准后完成所有 IDL、生成器、严格解码与黄金向量，再运行整卡验收和 CI。
本分支从 PR #3 squash 后的 main `fdea8e1` 创建；没有启动其他卡。
