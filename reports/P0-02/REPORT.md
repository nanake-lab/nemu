# P0-02 应用契约包 — 报告

## 结果

实现与本机验收完成；等待本 PR 的 Ubuntu / Windows CI 后按 AGENTS.md 合并。此处记录是合并前快照，最终 CI / 合并状态同时记录在 PR 中。

## 交付物

- `packages/contracts/src/asset.ts`：NemuProfile v2、资产许可清单 schema 与推导类型。
- `motions.ts`：30 个稳定槽位、6 类、10 个可表演和 16 个可请求槽位；没有独立手写 allowlist。
- `stage.ts`、`geometry.ts`、`stage-events.ts`：26 种舞台消息、物理状态、锚点、身体动作、三通道验证；只允许第 2.3 节的六种总线事件转发给舞台。
- `performance.ts`：五类提示、情绪/语言、流式剥离、三语分句、句子类型与 schema。
- `appbus.ts`、`vocabulary.ts`：第 4.1 节全部 22 种事件、公共枚举、查询和延迟字段；没有手写第 4.2–4.4 节的核心消息。
- `readout.ts`：BodyReadout 全部取值范围，不计算读出、不积累状态。
- `src/generated/index.ts`：仅占位导出；P5-01 才生成 IDL 类型。
- `packages/contracts/README.md`：API、分句边界处理、使用约束；`vitest.config.ts` 和新增根命令 `pnpm test:contracts` 固定行覆盖率门槛 90%。CI 只追加覆盖率检查和证据上传，不改变已有检查。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| frozen pnpm install、lint、typecheck、build | 通过 | `validation/checks.json`、对应日志 |
| pnpm test | 324 项通过（契约包 312 项） | `validation/unit.log` |
| 解析器 ≥ 60 用例 | 197 项通过，含每个分块位置、48/49 字符、未知标记、预算、跨句、三语标点、20/40 字、8/25 词、18 种缩写及大小写、小数/省略号/空句/连续标点、取消及错误传播 | `src/performance.test.ts`、`validation/coverage.log` |
| performance.ts 行覆盖率 ≥ 90% | 100%，104/104 可执行行；分支 97.05% | `coverage/coverage-summary.json` |
| 全部 schema 正反样例 | 通过；动态枚举导出 schema 与样例集必须完全一致；每种舞台/总线 payload 也分别测试合法、非法、未知字段及字段 null；另测范围端点、NaN/Inf、版本、麦克风类型/长度、通道隔离、完整映射 | `src/schemas.test.ts` |
| 定向 Prettier 检查 | 通过 | `validation/format.log` |
| Cargo workspace 与四个 uv pytest | 通过；仍为 P0-01 骨架测试，不宣称核心业务验收 | `validation/rust-test.log`、四个 Python 日志 |
| 远程完整 CI | 提交后检查；结果链接记录于 PR | Ubuntu 执行静态/单元/覆盖率/构建，Windows 执行原有隔离/Electron/视觉回归 |

复现：仓库根目录用 PowerShell 7 执行 `./reports/P0-02/verify.ps1`；只需要契约验收时执行 `pnpm test:contracts`。

## 规格对照

| CONTRACTS 条款 | 实现 | 测试 |
| --- | --- | --- |
| 1.6、1.7 | asset.ts | 完整映射、v2、胶囊长度、未知字段、非法值 |
| 1.8 | motions.ts | 全部 30 个 id 与两套使用权限逐项断言 |
| 2.1–2.3 | stage.ts / geometry.ts / stage-events.ts | 每种消息、通道隔离、总线转发白名单 |
| 3.1–3.3 | performance.ts | 197 项 parser 测试与 schema 正反样例 |
| 4.1 | appbus.ts | 每种事件、ULID、嵌套 payload、镜像 schema 共用 |
| 7 | readout.ts | 每个闭区间端点及越界拒绝、部分表情映射 |

仅实现现有应用契约，不修改规范或核心 IDL。实现细节（无逗号时硬切、Unicode 计数、标点运行的一 token 前视、未闭合标记 EOF 处理、语言平局与同类提示取末项）见包 README，并有测试。没有新增消息字段、情绪状态、数据库或用户内容。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| zod | 4.6.7 | MIT | 契约运行时验证 |
| @vitest/coverage-v8 | 5.0.4 | MIT | 开发依赖；与现有 Vitest 同版本 |

Context7 技能用于核对 Zod 4 的 strictObject、枚举完整 record 与 partialRecord、判别联合与类型推导；资料为官方 Zod 文档。精确版本与许可经 npm registry 核对，锁文件已更新。未引入其他框架。

## 待用户确认

- 无阻塞项；没有 ADR、阈值降低或视觉基准修改。
- 本包要求调用方注入 warning sink，并只发固定脱敏信息；`@nemu/log` 目前仍为空骨架。生产消费方接入时绑定统一日志，不在本卡越权修改日志包。

## 后续建议（不在本任务范围内）

- P5-01 生成核心 DTO；后续消费方使用包导出的判别联合与分通道 schema，不能绕过验证或复制事件定义。
- `schemas.test.ts` 超过 400 行（约 430 行）：为了把 schema 全量样例与覆盖完整性断言集中核对，包含 26 个舞台/22 个总线 payload 的数据表，无生产模块超限。
- VRM 几何、骨骼数量和文件系统路径的实际校验属于 asset-lint，不在纯 JSON 结构包里访问模型或文件。

## 下一步

当前分支 `task/P0-02-contracts`；完整 CI 通过、最新 main 已包含、无阻塞项时 squash 合并。之后按 tasks/README.md 取下一张满足依赖和目录互斥的卡，预期为 P0-06；不得提前开展其实现。
