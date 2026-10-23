# P5-22 首轮审查修订

## 当前状态

**完成。** B1、B2、D1 已在前轮关闭，后续 A1 经用户另开的 Codex 新会话[复审通过](https://github.com/nanake-lab/nemu/pull/57#issuecomment-6360980179)，按 2026-10-23 规则代替 Claude 审查要求。Ubuntu 与 Windows CI 均通过，用户已于 2026-10-23 合并 #57，合并即批准契约、主动边界候选和 V2-51–54。当前完成状态及最终 CI 链接见 REPORT.md；下文保留本轮修订的实现及验收记录。

审查来源：https://github.com/nanake-lab/nemu/pull/57#issuecomment-6341178269 。工作目录 D:\Dev\nemu-p522。已 fetch 并 merge origin/main=47ee2845c4731969242fe0a809ccc7637e1cf55c（合并提交 4658996），全部检查在合并后完成；提交前再次 fetch/merge 为 Already up to date。

## 修订与回归

| 项目 | 修改 | 回归 |
| --- | --- | --- |
| B1 异步审批竞态 | 移除尚未发布的 approvalDecisionMatches 布尔 API；validateApproval 在 await 前复制请求、决定及实际载荷，哈希后拒绝源变化，返回不可变载荷快照。consume 只能消费一次、检查新鲜时间，执行回调收到同一十六进制载荷及冻结绑定；过期/抛错也消耗 | 10 种等待期间篡改拒绝；返回后原缓冲区变化只能执行获批 A；构造键不可伪造、过期、时间倒退、二次消费、异常重试拒绝 |
| B2 成长结果丢失 L | 新增后继 core.queryResult.v2；queryResultToPanel 核对查询关联并按 IDL 校验完整 LedgerEntry，保留普通条目。growth/items 与其他查询/growthEntries 互斥，旧事件保留 | 非空 tendency_change 及证据引用贯穿 IDL→主进程映射→面板；普通三种查询、全部六种状态；非法 L、Infinity、额外投票、关联不符、混合数组拒绝 |
| D1 主动边界缺少 IDL 权威源 | 新增 ProactivityBoundaries、ProactivityBudgets、ProactivityCooldown 及 ProactiveKind/Gate。IDL 元数据定义 JSON 名称、枚举别名、cooldownS 字典投影；生成类型和 descriptor 驱动的浏览器 zod 校验替代手写边界 schema，配置 JSON 外形/值不变 | 资源→wire→规范字节往返、非空冷却表；未知枚举、重复集合、空键、非有限值、uint32 溢出、安静因子范围拒绝；元数据漂移被兼容门禁捕获 |

审批快照仅是载荷校验结果，身份认证、权限及实际执行仍属 P5-05。浏览器模块负责结构校验；规范排序、编码与字节预算仍由三语言编解码器负责。面板 L 使用 IDL 原生 bigint/Uint8Array，经结构化克隆传递，不做有损 JSON 转换。

## 验收

复现：`./reports/P5-22/run-checks.ps1 -Evidence reports/P5-22/<新的目录>`。26 项命令、时间、工作目录和退出码见 revision-1-checks/results.json，原始日志同目录保留。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| pnpm lint / typecheck / test / build | 全通过；32 文件、474 测试 | revision-1-checks/{lint,typecheck,unit,build}.log |
| pnpm test:contracts | 6 文件、343 测试；语句/行/函数 100%，分支 97.46% | revision-1-checks/contracts.log、coverage/coverage-summary.json |
| pnpm test:core-contracts / D10.8 / SYS21 | Rust/Python/TypeScript 各 10,702 例通过：2,511 合法、8,191 拒绝；protoc 443 通过 | revision-1-checks/core-contracts.log |
| generate.py --check | 0 stale；137 消息、51 枚举 | revision-1-checks/generate.log、core-contracts.log |
| check_compat.py --ref origin/main | p5-01、p5-21、p5-32、current、origin/main 0 违例；原 oneof 自检保留 | revision-1-checks/compat.log |
| oneof / JSON 资源 schema 回归 | 8 测试通过 | revision-1-checks/parser.log |
| Rust fmt / clippy -D warnings / workspace tests | 通过 | revision-1-checks/rust-*.log |
| 契约工具及三个 worker 的 ruff / mypy strict / pytest | 全通过 | revision-1-checks/contract-*.log、{irl,drive,model-adapter}-*.log |
| 旧字节往返与哈希 | 三语言各 1,777 查询、460 拒绝、0 差异 | revision-1-frozen-byte-comparison.json / .log |
| CI | 最终 head 的 Ubuntu/Windows 均通过，用户已合并；链接见 REPORT.md | https://github.com/nanake-lab/nemu/pull/57/checks |

原首轮施工失败和历史证据全部保留。本轮中间输出在 revision-1-targeted.log、revision-1-targeted-2.log，最终以 revision-1-checks 为准；未放宽验收或删除断言。

## 已有编码与黄金向量

比较基准为被审查 head=16711b0da9e839d98d4e80a790363391b28248fa。freeze_vectors.py 先重新验证旧 433 合法、448 非法向量，再只追加 10 合法、9 非法（revision-1-freeze.log）。目前 443 合法、457 非法、18 Unicode；三个新消息分别有 4/3、3/3、3/3 合法/非法向量。原 oneof 合法和缺失、多选、未知字段号、尾随字节拒绝向量不变。

| 列表 | 原数量 | 原前缀 SHA256（基准与当前相同） |
| --- | --- | --- |
| legal | 433 | 4792b83b53cc0f26271962c748e85f6f7576ee32f31dc60c45efc565fc142af7 |
| illegal | 448 | 2b12270dfaccddb21d5e73b08b01f0ea8dcd604ddd16a4b5a189f20bf2d8d7ff |
| unicode | 18 | 0e3e6d848402361540fef0b798337c970a598a8c6e78fe7d2ca52a7e5a900f31 |

执行 JSON 编码、Canonical/Protobuf 解码并逐项比较 value、canonical、protobuf、digest。每语言 1,317 成功、460 拒绝、0 差异；成功列表哈希在三语言及基准中一致：

- Canonical：ce4696e01e394f3259dcca9d1e28531d1e7975e5a96126cb5901fec893df8412
- Protobuf：ee80e61fb60fe803e3915395317343b0166fa7196b65c93eb3a349fda99fd565
- 对象摘要：0f5af7e0697a36267f2486f1cd3548ba312d25c5af4ddd4fcb41c1e3b5994102

已发布 main=91b31ed 的 396/409/18 比较保留在 frozen-byte-comparison.json。本轮还覆盖首轮新增向量。复现：`python reports/P5-22/verify-frozen.py reports/P5-22/<新文件>.json 16711b0`。routine_reporter 仅只读核对前缀/数量/哈希，主代理独立核对上述结果；未委派修复或通过判定。

## 配套修改

以下配套修改与契约一并纳入交叉审查；最终按 2026-10-23 规则由新 Codex 会话复审通过，没有改变任何 wire 编码规则。

| 文件 | 必须修改的原因 |
| --- | --- |
| core/contracts/generate.py、generate_browser.py | 解析权威 IDL 的资源/JSON 投影元数据、生成资源接口，避免手写跨边界类型 |
| templates/typescript/browser.ts | 不加载 Node 编解码依赖，由 descriptor 派生严格 zod 结构校验/资源投影，供面板 L 与主动边界使用 |
| templates/typescript/runtime.ts | 仅扩展 Schema/Field 元数据类型；本轮未改编解码逻辑 |
| check_compat.py、test_browser_schema.py、verify.py | 冻结 JSON 投影标记、字段元数据与枚举别名，新回归纳入日常检查 |
| fixture_inputs.py、golden/*.json | 三个新消息向量只追加；freeze 工具未改 |
| core-types/src/generated.rs、workers/*/generated/echo、packages/contracts/src/generated/echo | 从同一源生成，含新增资源接口与浏览器模块 |
| reports/P5-22/run-checks.ps1、verify-frozen.py | 纳入新工具检查、覆盖率按证据目录保存、支持指定被审查 head 做比较 |

generate.py 415 行：保留既有解析和多语言输出入口，新增资源接口生成拆到 generate_browser.py；未为行数拆散既有解析。TypeScript runtime 原已超过 400 行，集中承载双编解码与预算校验以保持单一参考实现，本轮仅增加元数据类型。新增实现模块均小于 400 行，生成产物大小由 IDL 决定。

## 规格对照、依赖和边界

逐条规格对应见 docs/traceability/P5-22.md；V2-52–54 补充在 diff-register.md 末尾，已随用户合并 #57 获批。无新增依赖；未使用 GPU，未打开用户正式/排练谱系，未签发后续卡的运行时或人格验收。

继承的非阻塞 C1(a) 仍在：单向 superseded 声明尚可绕过部分后继检查；本轮不扩大到历史 supersession 治理，不声称已解决。

## 待用户确认与下一步

- [x] 交叉审查完成：B1/B2/D1 已关闭；A1 经新 Codex 会话复审通过，按 2026-10-23 规则代替 Claude 审查。
- [x] 用户已合并 #57，批准契约、主动边界候选和 V2-51–54。

本卡已完成；本次仅收尾报告，按用户要求开 PR 后停下，不认领下一张卡。
