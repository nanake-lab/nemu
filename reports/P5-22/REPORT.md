# P5-22 终端契约变更 — 报告

> 当前修订：Claude 首轮审查的 B1、B2、D1 已修改，完整验收和字节证据见 [REVISION-1.md](REVISION-1.md)。下文保留首轮交付记录；最新数量为 443 合法、457 非法、18 Unicode 向量，三语言各 10,702 例通过。合并最新 origin/main=47ee284 后完成本轮检查，仍等待 Claude 复审及用户批准。

## 结果

技术交付与自动验收完成；契约候选未合并。PR #57 保留 needs-user，全部本机与 CI 检查完成后转可审查，请 Claude 对契约及配套工具交叉审查，最后由用户批准合并。工作目录为 `D:\Dev\nemu-p522`，分支 `task/P5-22-terminal-contracts-v2`；从 `origin/main=91b31ed2d30aadd8cc1cf8086dd0aad8ffe03821` 新建。2026-10-22 最终检查前再次 fetch、merge，结果 Already up to date。

## 交付物

- `docs/CONTRACTS.md` v2.2：修订 4.1、4.3、5.3、第 6–8 节，明确状态显示、三项身体读出、主动边界、社交权限、通知、成长查询/控制及实际行动审批。
- `core/contracts/terminal.proto`：新增 BodyReadoutV2、ReadoutV2、RunStateV2、QueryV2、QueryResultV2、Notice、GrowthControl、ApprovalRequestV2、ApprovalDecisionV2、SocialAction；六种既有终端消息标为被后继取代，字段表和已发布字节不改。
- `packages/contracts/`：身体 v2、严格边界/通知/控制 zod 校验、后继审批总线事件、载荷摘要核对函数；旧舞台与旧总线入口保留。
- `config/opportunities.json`：只含允许种类、冷却、门控与预算，没有触发脚本。
- `core/contracts/golden/`：只追加 37 合法、39 非法向量；当前 433 合法、448 非法、18 Unicode。
- `core/contracts/baselines/p5-32/`：从上述已发布 main 原样冻结 7 个 proto 及黄金前缀摘要，补齐 P5-32 到本卡的基线。
- `docs/traceability/P5-22.md`：逐条规格对应；`tasks/requests/P5-22-oneof-support.md`：用户追加范围授权。

## 验收

最终命令、工作目录、起止 UTC 时间、退出码及原始 stdout/stderr：`final-checks/results.json` 和同目录各 `.log`。完整串行复现：`./reports/P5-22/run-checks.ps1 -Evidence reports/P5-22/new-checks`（使用尚不存在的证据目录，避免覆盖）。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| `pnpm test:contracts` | 325 测试通过；覆盖输出改放本卡目录，不修改 P0-02 报告 | final-checks/contracts.log；coverage/coverage-summary.json |
| `pnpm test:core-contracts` / D10.8 / SYS21 | Rust/Python/TypeScript 各 10,471 例：2,447 合法、8,024 拒绝；134 消息；protoc 433 互操作向量通过 | final-checks/core-contracts.log |
| `generate.py --check` | 0 stale | final-checks/generate.log |
| `check_compat.py --ref origin/main` | p5-01、p5-21、p5-32、current、origin/main 全部 0 违例，含 19 个定向自检 | final-checks/compat.log |
| 文本级 oneof 解析门禁 | 5 测试通过：未注解字段、未识别字段、reserved 区间、重复/乱序号、成员改动拒绝 | final-checks/parser.log |
| `pnpm lint` / `pnpm typecheck` / `pnpm test` / `pnpm build` | 通过；全仓 30 文件、456 测试 | final-checks/lint.log、typecheck.log、unit.log、build.log |
| Rust fmt / clippy `-D warnings` / `cargo test --workspace --locked` | 通过；保留原有数值与状态语义测试 | final-checks/rust-*.log |
| 契约工具 ruff / mypy strict | 通过 | final-checks/contract-ruff.log、contract-mypy.log |
| 三个 worker 的 ruff、mypy strict、pytest | 通过；生成的 Python 合同仍由同一源输出 | final-checks/{irl,drive,model-adapter}-*.log |
| 主动表没有 trigger/candidate；notice 没有回应字段 | 严格 schema 与定向测试通过 | terminal-v2.test.ts；final-checks/contracts.log |
| I26 摘要不同拒绝 | 请求/决定/实际执行字节、ID、有效期、T3 不可永久信任均有拒绝用例，T2 信任有成功用例 | terminal-v2.test.ts |
| v1 与 v2 读出可解码 | 新 wrapper 的两个分支有合法黄金向量；旧 Readout 原字节也保留 | golden/legal.json；core-contracts.log |
| Windows / Ubuntu CI | 以 PR #57 Checks 页和交付评论记录的最终 head 为准；全部完成前保持草稿 | https://github.com/nanake-lab/nemu/pull/57/checks |

AC2-20、AC2-23 仅覆盖本卡的接口部分。没有运行成长/社交运行时集成、GPU 或外观测试；这些属于 P5-05、P5-12、P5-19、P5-29、P6-14/P6-15 等卡，本卡不签发其验收。未访问正式或排练存储；全部数据为合成夹具。

## 已有编码的逐字节与哈希证据

复现：`python reports/P5-22/verify-frozen.py reports/P5-22/new-frozen-comparison.json`。结果在 `frozen-byte-comparison.json`、原始输出在同名 `.log`。

| 基准列表 | 原有数量 | 当前原前缀 | SHA256（sort_keys JSON 完整列表） |
| --- | --- | --- | --- |
| legal | 396 | 逐条完全一致 | acb4046c8ad79fabc7b7e11c35502b0dc94b1c92eb31c1fb62f058a8b07707d5 |
| illegal | 409 | 逐条完全一致 | 9b66105363052b09fbd966193013d5eaf474e87a903f22d5ebfd9fcbc27d0aca |
| unicode | 18 | 逐条完全一致 | 0e3e6d848402361540fef0b798337c970a598a8c6e78fe7d2ca52a7e5a900f31 |

对这些旧向量逐一执行 JSON 编码、Canonical 解码、Protobuf 解码，并在每条成功结果中比较 value、canonical、protobuf、digest 的完整内容；三语言各 1,627 查询、421 拒绝、0 差异。成功输出列表的 SHA256 在三种语言及基准中一致：

- Canonical 十六进制列表：`0f86a6f48ad191d7523ebb219aeeafec843ebbb7ee31f33ff2b3bd0a114a3883`
- Protobuf 十六进制列表：`31502ad9a70d47d64365ac7ea3fb858502eabfe8459b9305a5a58f982d0b5aff`
- 对象摘要列表：`77890ccb9dfe4e377dc57a4de4a0233b6147c64b5bd13548dd12e2cfefcb6bff`

routine_reporter 按角色名调用，只读独立核对了旧前缀、数量与列表哈希，返回结果与上述主代理复核一致；未委派规格解释或通过判定，未授权其测试/编译。角色文件配置为 gpt-6-luna/max；运行时模型身份未独立核验。

## oneof 的独立与拒绝用例

只实现 ADR 0007 §6：选中成员字段号 u32 LE，再写该值，没有各成员 presence 字节，不改其他编码规则。只支持本卡使用的必填 oneof；不支持的声明失败关闭。

- ReadoutV2：4 个合法黄金向量，覆盖 v1/v2，另有 TypeScript 按字面字段手工拼接的独立中性 v2 规范字节 oracle。
- 冻结的定向非法向量覆盖 JSON 缺失/多选、规范路径缺失/多选（尾随第二选择）、未知字段号、尾随字节，以及 Protobuf 缺失/多选/未知号。旧消息仍按旧路径编码。
- compatibility 自检分别证明字段号重复、成员表变化、成员换组能被门禁拒绝；真实 proto 文本测试防止声明被正则静默漏掉。

## 配套修改

用户于本聊天追加授权（详见 requests 文件），和契约一样交 Claude 审查：

| 工具文件 | 必须修改的原因 |
| --- | --- |
| core/contracts/generate.py | 从真实 Protobuf oneof 导出成员与必填信息；强制声明可解析、字段号升序与成员连续；旧字段 descriptor 不增语义 |
| core-types/src/contract.rs、canonical.rs | Rust 权威实现 oneof 互斥/必填、选中号和值的编解码；原有字段走原分支 |
| templates/python/runtime.py、templates/typescript/runtime.ts | 与 Rust 完全同一 oneof 规则；Protobuf 通用解码保留不同成员后由校验拒绝多选，无 last-value-wins |
| fixture_inputs.py、golden/*.json、freeze_vectors.py 的既有调用 | 合法分支选取、只追加定向向量；freeze 工具本身未修改 |
| check_compat.py、test_oneof_schema.py、verify.py | 成员/字段号门禁自检与文本级反例进入日常验收 |
| baselines/p5-32/ | 冻结最新已发布格式，不把尚待审批的 P5-22 当已批准基线 |
| core-types/tests/v2_contract.rs | 已取代注册表从原来的 36 项增加为精确 42 项，并逐项检查六个新增前任；不删除旧断言 |
| core-types/src/generated.rs、workers/*/generated/echo、packages/contracts/src/generated/echo | 单一 IDL 自动生成的类型/descriptor/runtime，未手工复制 wire 类型 |

TypeScript 通用 runtime 502 行：原文件已承载完整双编解码与预算校验，本卡只加约 22 行 oneof 支持，维持单一参考实现；生成文件行数由生成器决定。其余本卡新增模块均小于 400 行。

## Claude P5-01 回溯审查的处理

已读取 [2026-10-22 审查评论](https://github.com/nanake-lab/nemu/pull/4#issuecomment-6332237760)。其结论为通过，未发现改变已发布字节的问题。本卡落实相关 C1(b,c)：不再静默忽略 oneof 或未注解声明；unsupported syntax/reserved 区间拒绝；字段号必须升序；补文本级测试。P5-32 基线也已冻结，对应 C1(d)。D2 在本卡改为按 ADR 0007 实现 oneof，符合用户此次明确授权，不采用「永不支持」的建议。

没有顺手修改 B1 的既有 cases 异常类型、C1(a) 历史语义取代与编码后继的关系、C2 全局拒绝原因分类、C3 JSON 桥宽松度、D3 旧时间/played_chars 单位。这些不影响本卡 oneof 的已测路径，列作独立后续；本卡未声称已消除全部 P5-01 工具问题。

## 失败记录与修正

- `typecheck-initial.log` 保留施工中 TS 可空 descriptor 下标及新总线测试样本缺失的错误；补充分组存在性信息与样本后，全量最终 typecheck 通过。
- `checks/rust-test.log` 保留固定旧注册表数量的失败；增加六个明确前任检查并保持精确总数断言，最终 Rust workspace 测试通过。
- 其余中间日志原样保留；没有删失败、放宽阈值或用新运行覆盖旧证据。

## 规格对照

见 `docs/traceability/P5-22.md`。涉及 I18（无终端持久状态）、I26（实际载荷审批）、I29（终端排除 EXPERIMENT）、AC2-20 接口（无投票入口）、ADR 0007（冻结字节）。差异登记新增 V2-51–54，表示与边界候选均随本 PR 请求批准，不代表已获批准。

## 新增依赖

无。SHA256 使用已有 Rust/Python 编解码依赖及平台 Web Crypto；未新增运行时或开发依赖。

## 待用户确认

- [ ] （阻塞合并）Claude 对 CONTRACTS、IDL、生成器、三语言编解码与证据交叉审查通过。
- [ ] （阻塞合并）用户批准本次契约、opportunities 边界候选和 V2-51–54 的具体表示，由用户 squash 合并；执行者不合并。
- [ ] （不另阻塞）BodyReadout v1 的兼容类型与生产停用标记方式在 V2-51 明示；P5-19 只发 v2，P3-06 迁移旧舞台。

## 迁移与回退

没有运行存储迁移或打开任何用户谱系。旧对象字节与摘要保持；后继使用新类型标签。部署新协议前按 ADR 0007/AGENTS 做本机备份，双方升级后才启用新生产者；回退恢复旧应用/备份，保留新对象原标签，不把新字节解释成旧消息。新审批不能降级到旧的无载荷/摘要入口。P5-19、P3-06、P5-05 分别承担生产、舞台迁移和权限执行。

## 后续建议（不在本任务范围内）

- P5-12 实现固定身体/主动读出；P5-19 起生产 v2；P3-06 改舞台输入。此卡不选择审美系数。
- P5-04/P5-29 实现 growth 查询、L 摘要通知与管理命令执行；P5-05 完成身份、一次性与实际载荷绑定。
- P6-14/P6-15 完成原样载荷格式、社交承诺分类和执行。
- config/helpers.json 目前不存在；其所属卡创建时与本边界自主助手 30/天一致。cooldownS 为空候选，未从旧触发规则推导新冷却。
- P5-01 回溯审查其余项目另卡治理，避免扩大本次编码规则。

## 下一步

等待 Claude 交叉审查和用户批准合并；不自行合并，不取依赖未获批准的下一张卡。
