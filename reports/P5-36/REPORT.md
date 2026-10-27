# P5-36 θ 估计器状态的存储 — 报告

## 结果

部分完成，见 [BLOCKED.md](BLOCKED.md)：已认领 [草稿 PR #108](https://github.com/nanake-lab/nemu/pull/108)，完成 ADR 0029 提案、完整状态字段核对、恢复接口请求和既有契约基线检查；后继 IDL、生成物、语义检查、黄金向量及真实内存往返尚未实施。

分支 `task/P5-36-theta-estimator-idl`，工作区 `D:\Dev\nemu-wt\P5-36`，来源为 fetch 后的 `origin/main@3f8f98e5ac3cba2f2b4718ca242a5a0c226d0fbf`。发布准备前再次 fetch，main 未变。P5-25 / #77 已合并；#95、#65 仍占用本卡实现目录，且真实往返缺少 core-kernel 接口。未扩大实现范围。

已完整读完用户要求的四份文件、Echo Core 技能、v2.0 全文、README §5 的本卡相关保留章节、ADR 0005/0007/0008–0011、0019 §8、本卡全部内容、指定差异条目和 P5-25 请求/METHODS 的全部追加。旧规格按 v2.0 及最新差异登记的取代关系读取。

## 交付物

- [ADR 0029 提案](../../docs/adr/0029-theta-estimator-storage.md)：逐项列出真实递推字段、参数摘要绑定、V2-95/96/97 的关系、只增不改与恢复约束。未批准，精确 IDL 分配仍待实施。
- [恢复接口请求](../../tasks/requests/P5-36-dynamics-roundtrip.md)：记录私有状态字段及缺失接口，提出最小桥接路径与两种处理选项；未修改 core-kernel。
- [规格对照](../../docs/traceability/P5-36.md)：区分既有基线测试与新契约未执行项。
- [verification/](verification/)：原始命令输出、退出码、依赖环境、PR 核对和 31 个现有 proto/黄金文件摘要。`baseline-observation.json` 仅为现状观察，不是新冻结基线。

本轮源码及运行配置未改。新增 ADR、追踪报告、请求和本报告属于 AGENTS 允许的追加材料；未改既有任务卡、既有规格、原文或差异登记。V2-107 至 V2-109 尚未发布条目。

## 验收

| 命令或检查（含本卡验收） | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | passed；0 generated files stale | `verification/generate-check.log`、`light-checks.json` |
| `python core/contracts/check_compat.py --ref origin/main` | passed；p5-01、p5-21、p5-32、current、origin/main 均 0 违例；137 messages（42 superseded）、51 enums | `verification/compatibility.log` |
| `pnpm test:core-contracts` | passed，exit 0；三语言各 10,766 例（合法 2,527，非法 8,239），官方 protoc 445 传输向量、8 项 schema 测试通过；有下述依赖预检查告警 | `verification/core-contracts.log`、`core-contracts.json` |
| P5-25 真实状态与新消息逐位 round trip | not_run；新消息与桥接均未实施 | B1、B2；接口请求 |
| 每种新消息 ≥3 合法、≥3 非法向量 | not_run；不得把旧案例总数算作此项通过 | `verification/acceptance.json` |
| 新基线冻结、ADR/契约批准 | not_run；未实施，未获本次契约批准 | ADR 0029、BLOCKED.md |
| `git diff --check`、修改范围核对 | passed；仅新增设计/请求/报告材料及改本卡报告 | `verification/scope-check.json` |
| TS 全仓 lint/typecheck/test、Rust 全工作区、worker pytest | not_run；本轮未修改这些实现，契约命令只运行其自身编译和探针 | 不签发完整实现验收 |

全部命令结果只覆盖未改动的 main 契约基线。`pnpm` 在脚本开始前自动尝试依赖校验/安装，因 `node_modules` 为目录联接报错；其原有 warn 模式继续运行了 verifier。此告警原样保留，没有修改设置来消除它。当前工作区与复用依赖工作区的 pnpm/uv 锁文件 SHA-256 相同，实际 TypeScript 6.0.3、Zod 4.6.7；不能据此声称完整依赖安装检查成功，见 `dependency-context.json`。本轮不重复运行已经完成的契约测试。

构建、venv、protoc、TS 输出和临时目录位于 `D:\Dev\cache\P5-36`，依赖联接见 `build-paths.json`。主代理串行执行编译和测试；开始前未检出 Runner.Worker 或其他列出的重负载，未运行 Electron、视觉或 GPU 检查，未访问排练或正式存储。

## 规格对照

| 规格条款 | 本轮对应位置 | 测试 |
| --- | --- | --- |
| AGENTS §2 目录互斥 | 未修改 contracts/types/generated，保留草稿 | PR 状态、完整变更文件清单 |
| ADR 0007 | 后继消息设计，旧定义未动 | 现有全部兼容基线 0 违例 |
| V2-98/103、I18 | ADR 0029 全字段表 | 新契约与真实往返 not_run |
| V2-95/96/97/100、I23 | ADR 0029 时间/数值/资源/报警关系 | 后继语义、续算对照 not_run |
| I29 | 专用临时目录 | 本轮契约探针，无用户存储 |

完整条款和 AC2 验收边界见 [规格对照表](../../docs/traceability/P5-36.md)。本轮未修改或签发 AC2-05/14/20/21 的运行验收。

## 新增依赖

无新增运行时或锁定依赖；使用现有锁文件的工具。`uv sync --project core/contracts --locked --offline` 仅建立独立测试 venv，原始日志和退出码已保存。

## 待用户确认

- [ ] （阻塞 B1）按目录互斥顺序释放 #95、#65 的重叠目录；本轮没有批准例外。
- [ ] （阻塞 B2）由所属模块交付完整状态桥接，或明确批准请求中的最小 core-kernel 实现范围。
- [ ] （阻塞）最终实现完成后的 ADR 0029 与契约变更批准、独立交叉审查。当前提案不等于已批准的最终契约。

## 后续建议（不在本任务范围内）

H 来源/停机记录认证、长期时间域、P5-26 独立完整 H 重算和 P5-29 外层原子发布仍归既定后继任务；本轮不扩大范围代做。

## 证据与发布说明

轻量 PR 证据由子代理按最新 `.codex/agents/routine-reporter.toml` 的 **GPT-6.1 Terra / max 显式参数试用**收集；未声称按角色名自动加载已验证。子代理只查询和整理证据，未测试、修改仓库、发布或判断验收；主代理核对后复制摘要、完整变更路径和补充查询，并作出阻塞判断。原始返回的保存位置、SHA-256 见 `pr-raw-manifest.json`。

P5-25 #77 的标签接口返回 null，未将 null 解释为空标签；补充的带/不带 `unverified` 搜索支持目前无该标签，保留接口失败边界。开放 PR 扫描未发现 ADR 0029 的路径占用，相关 diff-register patch 未使用 V2-107 至 V2-109；#95 的全文只出现既有预留说明。编号实施前和合并前须再核对。

最初两次 git push 均因 schannel TLS 握手失败，认领通过 GitHub 连接器提交等价文件树并建立草稿。本地 `6ee2aeda` 与远端 `bbcf0a4d` 的 tree 均为 `92661d2c8cb8e4196c6cde451898904cf298472c`，提交身份不同。后续 fetch 恢复，用普通 merge `b0072b8f` 合入远端认领提交，没有 rebase、强推或修改 TLS/全局配置；详见 `publication-history.json`。

## 下一步

保持草稿和 `needs-user`。目录释放并具备桥接后，先 fetch 并 merge 最新 main，重查任务追加、ADR/类型编号和 PR 占用，再实施后继消息、三语言校验/向量、冻结基线及真实往返。全部验收完成后才转可审查并评论“请审查”；本轮不满足此条件，不取下一卡。
