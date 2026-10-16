# E12 状态追加与 Unicode 黄金测试 — 报告

## 结果

按用户授权追加记录并补充两组黄金测试。独立分支 `codex/docs-test-e12-unicode`，基于 main `d93f97c`；与 P0-07 的 `workers/model-adapter/probe/` 无目录重叠。

## 交付物

- `docs/traceability/diff-register.md`：追加 E12 当前状态，明确旧表格及历史段落中的“待用户提供”已被取代。
- `docs/traceability/baseline.md`：在原第 37 行说明之后追加当前状态。
- `docs/echo-core/README.md`：第 2 节追加当前状态。
- `core/contracts/golden/unicode.json`：两组共 18 个冻结用例。
- `core/contracts/verify.py`：现有三语言验收加载上述用例；不生成或覆盖黄金数据。

三处当前状态均为 **用户确认不可得（2026-10-14 决定）**，指 F 第 20–27 节、D 附录 B。原文每一行仍按原顺序保留。没有重建缺失内容，没有修改冻结规格、IDL、编码实现、C01 或 ADR 0006；C01 仍未决且仅阻塞 W09/W10，ADR 0006 仍为 PROPOSED，v1.0 仍为 NO-GROWTH。

## 黄金用例与规格对照

依据已批准 ADR 0007 第 5、7、8 条及 D10.8 / SYS21；本次只验证已有编码合同，不是新的契约变更。

1. 拒绝非法 Unicode：孤立高/低代理项作为 JSON 编码输入；UTF-8 编码的高/低代理项及 2/3/4 字节过长编码分别作为完整 CanonicalV1 和 Protobuf 消息，共 12 个拒绝用例。其余字段和长度均有效，避免因其他缺字段错误而虚假通过。
2. NFC/NFD 不归一化：`U+00E9` 与 `U+0065 U+0301` 的编码、CanonicalV1 解码、Protobuf 解码，共 6 个合法用例；分别断言完整值、规范字节、传输字节和摘要，禁止归一化或碰撞。

黄金值独立于实现逐字段构造：沿用 ADR 的 OutputEnd 例子，将 output_id 的 `41` 替换为 `c3a9`（长度 2）或 `65cc81`（长度 3）。NCV1 头、类型标签和 reason=1 不变；Protobuf field 2 的长度同步为 2 或 3。摘要由 ADR 给出的域前缀、类型标签长度及完整规范字节用 SHA-256 计算，作为固定字面值保存。CI 不重算预期值。非法串 `c0af`、`e080af`、`f08080af` 均为 `/` 的非最短形式，不能被宽松解码后接受。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| 三处文档原文行顺序保留检查 | passed | 对比 Git HEAD 与工作区，全部旧行仍依序存在 |
| `git diff --check` | passed | 本地执行 |
| `uv run --project core/contracts --locked ruff check core/contracts` | passed | 本地执行 |
| `uv run --project core/contracts --locked mypy --strict core/contracts/verify.py` | passed | 本地执行，1 个文件无错误 |

## 新增依赖

无。独立 worktree 使用现有锁文件安装验收工具。没有启动本地 Electron、Playwright 或视觉测试。

## 待用户确认

无新增决策。E12 的本次记录不重新打开任何已裁决事项。

## 下一步

作为独立 docs/test 小 PR 交付，全部 CI 通过前不合并。P0-07 保留在原工作区继续；此分支不占用其目录。Windows CI 应与 P0-07 GPU/渲染测量错开，统一安排 runner。

## 验收完成追加

三语言完整验收已通过：Rust、Python、TypeScript 各 5246 个用例（原 5228 + 新增 18）；合法 1312、非法 3934，78 种消息。官方 protoc 对照 241 个传输向量通过。命令：python core/contracts/verify.py；原始日志见 conformance.log。
