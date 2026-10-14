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

## 2026-10-14 续报：用户批准后的实现与验收

前面的待批准/未实现段落是历史阶段记录。本续报为当前状态。

### 结果与交付物

实现已完成；本地验收通过，任务完成/合并仍以最新 main 上的复验和 PR #4 CI 为门槛。
ADR 0007 已按用户批准修改：独立冻结标签、36 字节手算示例（72 字节哈希输入）、嵌套 NCV1 头、N3/genesis 冻结点、独立版本化审计扩展。

- `core/contracts/{base,state,growth,terminal}.proto`：78 种消息，逐条来源和冻结标签，显式 presence、集合键、字段范围与变体约束。
- Rust DTO/严格 Protobuf/CanonicalV1/SHA-256；TypeScript 和三个 worker 的 Python DTO/编码器由同一 IDL 生成。
- `core/contracts/golden/`：241 个合法、243 个非法固定向量，每条消息至少 3+3。另有完整合法消息单字段破坏、官方 protoc 编码、集合/向量、UTF-8、边界等测试。
- `IMPLEMENTED-MAP.md`：78 条消息及所有字段的规格映射。`DEPENDENCIES.md`：精确版本、许可证及超过 400 行文件的说明。
- `core/contracts/README.md`：生成/验收命令、公开类型与 Node 编码器的边界、Python DTO 用法、冻结规则。
- 新根命令 `pnpm test:core-contracts`，新增 CI 验收步骤；不改变已有检查及阈值。

### 实测验收

`./reports/P5-01/verify-full.ps1` 的 27 项检查全部通过，见 `acceptance/checks.json` 与逐命令日志。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| D 第一篇 §10.8 | Rust/Python/TypeScript 各 5,228 用例通过，含 1,306 合法、3,922 拒绝用例 | acceptance/D10.8-SYS21.log |
| SYS21 | 未知 schema、非法枚举、NaN/Inf、u32/u64/i64 与二进制 count 溢出、消息/深度限额被拒绝 | 同上；canonical_boundaries.rs；numeric.rs |
| 官方 Protobuf 互操作 | 241 条官方编码输出经三个严格解码器还原；支持 packed/unpacked 数值向量 | 同上 |
| Rust | fmt、clippy -D warnings、workspace tests 通过；core-types 共 9 测试 | acceptance/rust-*.log |
| TypeScript | lint、typecheck、test、build、原 test:contracts 全通过 | acceptance/pnpm-*.log |
| Python | 三 worker 与 voice-sidecar 的 ruff/mypy/pytest；三个生成目录 mypy；契约工具检查通过 | acceptance/*mypy.log、*ruff.log、*test.log |
| 原规格、架构、CONTRACTS、ADR 0006 | 无差异 | acceptance/frozen-specs.log |

### 实现边界与待确认

- 本卡只发布类型和编码，不创建/读取排练或正式存储，不接入 G/C、资格判定、行动授权或任何运行路径。
- R/G 资格分支及单根账本字段均保留，C01 继续仅阻塞 W09/W10；ADR 0006 仍 PROPOSED；v1.0 NO-GROWTH。
- C02 仍为 SYS01–SYS25 全部保留，不修改原文。
- ObjectRef 是带摘要/类型/资源版本/长度的不可变可恢复引用；资源语义验证由所属后续卡执行，不以不透明引用伪称语义实现。未补写缺失的 Review_and_Remediation。
- 排练 LineageHeader 不包含 genesis 摘要；LIFE 必须包含摘要。这里只验证结构，不意味着创建或批准诞生。
- ADR 0007 批准已解除用户决策阻塞；没有新增待用户裁决项。合并前仍需最新 main 复验、CI 全绿。

### 下一步

同步最新 main（已发现 P0-03 提交 78c8deb），完整复验并更新 PR #4。全部满足后按规则 squash 合并；之后按任务顺序取 P0-07。未通过前不合并、不同时启动下一张卡。
