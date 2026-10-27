# P5-35 终端接入契约 — 报告

## 当前状态

实现及本地定向验证完成，草稿 PR #110 保留 `needs-user`，等待当前 head 的 CI、独立审查和用户批准。最终 CI 状态以 PR 描述/评论为准；本报告不通过补交 CI 结果制造提交循环。本轮只做 P5-35，不写网关服务、客户端、面板，不领取下一卡，不自行合并。

开工/本地验证基线 B：`32a5888ec6f1114d6341291bc15e865d58a6d51d`（已合并 #65/#95/#100）。提交前再次 fetch 的 M 与 B 相同。报告内本地验证针对本提交所含实现；最终 PR 会记录准确 head、CI 实际 checkout 与基线。

## 交付物

- `core/contracts/terminal_ingress.proto`：12 个新消息、5 个枚举，认证/会话预算、事件提交、上传、分派配对、ReceiptV2、拒绝审计；旧 Receipt 保留内部/历史用途，新字段不复用 5 号 reason。
- 生成器增加 `@transient` 和 `@retains`；瞬态认证不可持久化，Rust Debug/Python repr 不回显凭据；兼容门禁禁止复活凭据持久化及嵌套持久记录，校验保留的前代字段。
- Rust/Python/TypeScript 纯验证器：认证/dispatch 配对、连接会话/能力/帧预算、上传分片/对象/总预算、连续偏移和声明固定、域分离摘要；不实现连接认证服务。
- core-store：从完整 D 尾部分配位置/时间的接收 API；独立 append-only 拒绝域及 reject 见证；记录/字节容量类型化原因与安全拒绝映射，SQL/I/O 保留结果未知。
- 物理 SQLite format=1↔2 显式事务迁移，普通 open 不隐式迁移；审计为空才允许回退。已有审计拒绝破坏性回退，旧数据库回滚被见证识别。旧 V2 head 保持原字节只读/拒写。
- `baselines/p5-34` 冻结开工 main 的 IDL 与 464 合法/492 非法/18 Unicode 向量摘要；当前 500 合法/528 非法，新增每种消息 3 正 3 负，另有 38 条上下文向量。
- ADR 0024、[规格映射](../../docs/traceability/P5-35.md)、V2-64–69；按用户批准同步修订 P5-35 任务卡迁移与验收范围。

## 范围及目录核验

2026-10-27 只读检查 29 个本地 worktree 与所有开放 PR 当前 diff；本卡相关契约/core-types/core-store/生成目录无实际未提交占用。#108、#97 保留草稿且未施工；本轮按用户指示优先占用。ADR 0024 未占用，V2-64–69 为本卡预留。原工作区的 content、远程附件与研究报告不动。

## 验收

| 命令/检查 | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check`；compat 四基线及 current | 通过，0 stale/0 违例；另对 origin/main 为 0 违例 | SYS21/contracts-release-2.log |
| `pnpm test:core-contracts`（SYS21、D10.8） | 14 个 schema 单测；protoc 500 项；三语言各 12,356 项；保留浏览器 schema 106 项 | SYS21/contracts-release-2.log |
| `cargo test -p core-store terminal_ingress -- --nocapture` | 10 通过；包括 10 个进程退出注入窗口、容量/失败/回滚保护 | W02/store-ingress-final.log |
| `cargo test -p core-store p534_v2_uncheckpointed -- --nocapture` | 1 通过，平铺/分段 V2 的 DB 和未 checkpoint WAL 都逐字节不变，迁移正反向均拒写 | W02/legacy-wal-final.log |
| SYS17 物理格式子项 | 正反向迁移、回退、StorageFull、进程退出与数据不变通过；**不是完整 SYS17** | SYS17/result.json；W02 日志 |
| 凭据存储/日志扫描 | 存储测试检查 token 原字节/hex；Rust debug 脱敏；26 份开发日志扫描通过 | W02/token-log-scan.json；W02 日志 |
| Rust clippy（两改动包、all-targets、-D warnings） | 通过 | static/clippy-release.log |
| Python ruff；mypy --strict（13 文件，含生成包与夹具） | 通过 | static/ruff-final-2.log；static/mypy-release-2.log |
| ESLint（改动 TS）；空白检查 | ESLint 通过；冻结 p5-34/growth.proto 原样保留 main 已有 EOF 空行（git diff --cached --check 的唯一提示），不改冻结源 | 提交前命令 |
| Ubuntu 全工作区、Windows W02/相关桌面检查 | 留给 CI 执行，本机不重复全量 | 当前 head 的 PR 检查与最终评论 |

环境：Rust/Cargo 1.99.0（仓库锁定版）、Node 24.21.0、pnpm 12.11.2；系统 Python 3.12.10，uv 合同工具 Python 3.12.14。所有构建缓存、venv 和 TemporaryRoot 在 D 盘缓存目录，没有打开用户排练或正式存储。

开发失败日志保存在 `development/*.log.gz`，未删除或用成功日志覆盖：生成器调试代码位置、probe 类型推断、测试快照 SQL 的单列表排序/SQLite 可重建 shm、lint/Copy 克隆、TS 新元数据声明和 Python 夹具推断均已修正；未改变既有黄金前缀或降低验收阈值。SHA256 清单为 `EVIDENCE-SHA256.json`。超过 400 行的生成器/生成文件与原有协议夹具是已有集中生成/兼容入口；新手写生产模块均小于 400 行。

## 规格对照

见 [P5-35 映射](../../docs/traceability/P5-35.md)：I01/I02/I04 经 writer 类型、持久事务与故障测试；I18 经无凭据独立审计和完整权威行比较；I29 经 TemporaryRoot；SYS21/D10.8 经跨语言/官方 protobuf；SYS17 仅覆盖物理格式子项。

## 迁移范围决定

用户 2026-10-27 已批准区分物理格式与状态消息代际。当前 V3 可写库的 format=1→2 结构迁移/回退/故障测试属于本卡；旧 V2 不补 q/V、不借结构迁移解锁写入。V2→V3 历史转换、G/H 重放及其迁移回退验收按 ADR 0021 §11 为 **P5-26 后继未完成**，不作为本卡合并阻塞；本卡自身结构迁移失败仍阻塞。未沿用 #95 的免迁移例外。

## 新增依赖

无新增运行时依赖；沿用已锁定的编译、SQLite、protobuf 和测试工具。

## 待用户确认

- [ ] （阻塞合并）ADR 0024、新契约及 V2-64–69 的批准。
- [ ] （阻塞合并）用户另安排的独立审查；执行者只在 PR 写“请审查”。
- CI 最终结果在 PR 维护，未通过不得合并。暂无未决的实质规格矛盾。

## 后续建议（不在本任务范围内）

P5-03 按契约完成真实认证、连接生命周期、上传缓冲/GC 接线和 dispatch；P5-26 完成历史状态代际重放。没有运行真实断电、服务端系统验收或行为验收。
