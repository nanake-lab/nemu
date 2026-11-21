重档。交付线，涉及正式 IDL、生成器输出、终端上传边界与 SYS21 三端验收。

# P5-43 五项正式语义边界

#235 已合并，任务分支以 merge 同步到 main `e79dce88`。在该版本重新运行生成器、冻结基线并补齐 22 个消息的黄金向量。契约实现 `76385337` 通过本机验收，等待交叉复审和用户批准 ADR 0048、V2-190 与字段编码。[PR #244](https://github.com/nanake-lab/nemu/pull/244)

| 验收 | 结果 |
| --- | --- |
| 基线与兼容性 | 14 个 IDL 文件逐字节与 main 相同。四组旧向量的前缀与 SHA-256 一致。生成器检查 0 个过期文件，`check_compat.py --ref origin/main` 检查 238 个消息、90 个枚举，0 项违规。 |
| 黄金向量与 SYS21 | 新增 77 条合法、132 条非法向量。22 个语义反例各有对象输入及两种合法编码的错误记录。官方 protoc/protobuf 通过 856 条传输向量；Rust、Python、TypeScript 各通过 21,541 例，其中 4,819 例应接受、16,722 例应拒绝。 |
| 三端上传入口 | 三端各接受合法完整分块，仅替换对象标签后各拒绝 22/22 个新标签。 |
| Rust 与 Python | Rust fmt、core-types Clippy 与全包测试通过，含 10 项本卡语义测试；schema 单测 14 项通过。Ruff 和严格 mypy 通过。 |
| TypeScript | 根 lint、typecheck 和 620 项测试通过；契约包另有 354 项通过。 |
| 编号 | 开 PR 前已核对。交付复核 main 及另外 19 个开放 PR 的提交；0048、V2-190 均无冲突。 |

[逐消息向量数](VECTOR-COVERAGE.md) · [规格、实现与后继验收对照](../../docs/traceability/P5-43.md) · [私有原始证据索引](EVIDENCE-INDEX.csv) · [CI 检查](https://github.com/nanake-lab/nemu/pull/244/checks)

首阶段因新向量数为零而失败的 SYS21 记录保留在证据索引。本轮修正过夹具、mypy 类型及导入路径、Ruff 命名和合并处的 Rust 排版。冻结文件 `growth.proto` 逐字节继承 main 的尾部空行，`git diff --check origin/main` 仅报告该行。Windows 自托管作业在草稿 PR 事件上跳过，草稿门卫显示失败；手动运行及提交状态见 PR 的最终审查请求。

新增运行时依赖为零。六个后继保留旧标签。生产存储回退（SYS17）、真实身份、调度及事务接线仍由 P5-06/P5-53 等后继卡验证，本卡这些项目记为 `not_run`。
