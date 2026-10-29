# P5-24 证据组装 — 报告

## 结果

**部分完成，见 [BLOCKED.md](BLOCKED.md)。** 已实现受治理输入上的证据组装纯函数及 30 项合成测试。它尚未生产接线：EV9 完整属性和部分语义/记录接口待 P5-06，卡外模块登记与正式验收入口列入请求。不能把这些合成测试算作生产 H、真实语义、完整实验谱系或整卡验收。

分支 `task/P5-24-evidence`；PR [#117](https://github.com/nanake-lab/nemu/pull/117)；起始/本机验证 main 为 `740c2289c2a86f400580b4e6d4aa07654c08ace6`（P5-23 / #49）。本次用户明确限定代码目录，并要求本机验证后转为可审查、写「请审查」，不等待 CI、不合并。

## 交付物

- `core/crates/core-kernel/src/evidence/`：`assemble(Prefix)`、记录校验、规范候选/提出次序、同源连通分组、条件可比性与撒谎排除、首次教导组、外源事件序列。
- 同目录 `types.rs`：仅含准许输入的进程内视图与 `Assembly::delta/step`。不发布 IDL，不写持久状态，不调用模型/时钟/存储。调用方负责证明 H 已提交、记录已接受和 m=0 来源。
- 同目录 `tests/`：正反规则测试、128 条符号翻转历史、别名/账号性质、真实 P5-23 G 消费、16 个完整前缀与增量的逐值对照、容量拒收后重提回溯。所有身份/数值均为合成，未读取用户数据。
- 同目录 `verify.ps1`：D 盘临时 crate 编译本模块，复用同一工作树的 core-kernel/core-types；将命令、工具链、基线、源码哈希、锁文件、原始输出保存在唯一运行目录。
- [卡外接口请求](../../tasks/requests/P5-24-inputs.md)：P5-06 属性/语义、组 μ/可核实归约、H 提出次序、C/重算接线，以及 lib.rs/验收登记。

## 行为与限制

1. 记录必须具备有效生成类型、唯一记录身份、已出现的 H 引用、当前提交绑定、匹配的规则版本和有效数值；来源认证仍由上游负责。
2. 对话、来源规范化、同一件事形成传递分组；成员按 H 位置排列。反应及匹配仅消费记录。组内重复由真实 G 验证不增计数/权重。
3. 自述/知识内容/维护不作为她的经历；阅读反应可以作为经历。A 降级无 a(e) 时不输出 C 反应。撒谎及其对话/parents 后继不进入 G/C 的证据输入。
4. 结果未知只排除依赖结果的匹配，已观察反应仍可保留；0 不自动解释成未知。晚到已知结果使用新事件。不生产 `EVIDENCE_UNKNOWN`。
5. 教导只消费关于她应怎样的受治理标注。说话人、规范倾向、正反形成键，首次位置及属性冻结，重复不刷新；信任权重由 G 计算。
6. 新提/重提保持规范身份并回溯完整有效历史。别名不占新增名额、不刷新提出次序。同一提交的多个新身份需要明确记录次序，无序输入返回错误。
7. 同组 μ/可核实值/合格倾向集合不同即返回 `Ambiguous`，未补写归约规则。否则共享组均值可能把对 k 未知、对 j 合格的成员反应错误用于 k；本模块不自行拆组或取交集。普通增量仅发送新增 C 后缀及变化组；旧组撤回/并合、旧反应改变等要求重新计算。生产 P5-26 重放不在本次结果之内。
8. 未添加 `lib.rs` 的模块登记，所以普通工作区构建**不包含本模块**；模块自身由 harness 编译测试。卡外修改请求待接续，不能用工作区通过冒充已生产启用。

## 依赖核对

- P5-23 已合并；复用其纯 G 输入、V3 状态、关系与候选容量接口。
- P5-06 / #111 当前 head 为 `650e9f138e6bf534450f5a5163571e88b3ce1303`，仍为草稿；完整属性规则/正式接口尚未交付。核对具体缺口后写入 `BLOCKED.md`，不在本卡重建语义治理。
- 已阅读 P5-24 全部追加及 ADR 0014 更正：不生产 `EVIDENCE_UNKNOWN` 或复用旧 `EvidenceRelation`。

## 验收

工具链：Windows 本机 `rustc 1.99.0 (b940084d7 2026-09-28)`，仓库锁定版本。缓存/目标/临时目录为 `D:/Dev/cache/P5-24/`，Cargo 缓存为 `D:/Dev/cache/cargo/`；仅设置进程环境，未改用户/系统设置。重负载串行。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `verify.ps1 -Mode Test -RunId test-06` | passed：30 passed，0 failed，0 ignored | `verification/test-06/` |
| `verify.ps1 -Mode Clippy -RunId clippy-02` | passed：harness 全目标，`-D warnings` | `verification/clippy-02/` |
| `verify.ps1 -Mode Format -RunId format-02` | passed：本模块递归 rustfmt check | `verification/format-02/` |
| `verify.ps1 -Mode WorkspaceFormat -RunId workspace-format-01` | passed：整个 Rust 工作区格式检查 | `verification/workspace-format-01/` |
| `verify.ps1 -Mode WorkspaceTest -RunId workspace-test-01` | passed：退出 0，288 passed / 0 failed / 1 原有 ignored；约 22.5 分钟 | `verification/workspace-test-01/` |
| `verify.ps1 -Mode WorkspaceClippy -RunId workspace-clippy-01` | passed：工作区全目标，`-D warnings` | `verification/workspace-clippy-01/` |
| I24 / EV10 结构核对 | 签名无 x/m；实现无存储、网络、模型、实时时钟调用 | `types.rs`、`mod.rs` 及实现源码 |
| TS/Python/Electron/GPU | not_run：未改相关代码或依赖，本次纯 Rust 范围不涉及 | 范围检查 |
| 生产 H、属性生成、实验谱系、SYS17 | not_run：接口/所属卡未接通 | `BLOCKED.md` |

命令从工作树根目录以 `pwsh -NoProfile -File core/crates/core-kernel/src/evidence/verify.ps1 ...` 运行。实际 cargo 参数见各 `result.json`；没有把临时 harness 注册进工作区。harness 锁中的依赖名称/版本均与 `core/Cargo.lock` 一致，仅多一个本地验证包。

仓库文本规则会统一日志换行，因此额外保存 `raw-evidence.zip`：包含全部运行原始字节、最终被测源码和逐文件 SHA-256 清单，已解压读取逐项核验。运行记录中的 SHA-256 对应当时字节；跨 checkout 校验时以此档案为准。展开的文本副本供直接审阅。

失败记录完整保留：`test-01` 为测试夹具的 Rust 借用错误（未运行测试）；`test-02` 为 23 passed / 4 failed（规律夹具未满足参数域）；`test-03` 为 25 passed / 2 failed（G 的变更摘要被测试适配器错误替换为全量）。分别修正借用、采用既有合法参数域、按 group_id 合并 G 摘要后，`test-04` 为 28/28，增加真实容量回溯测试后的 `test-05` 为 29/29。自查补上组内资格不一致的拒绝路径及回归后，最终 `test-06` 为 30/30。未放宽断言、阈值或改变 G。

工作区原有忽略项是 `dynamics::tests::reference::theta_1_original_500_streams_zero_violations`（V2-93），本卡未增删忽略项；该项不作为通过证据。core-kernel 原有测试为 169 passed / 1 ignored。工作区检查不编译未登记的 evidence；最后修改该目录后，单独重跑了 `test-06`、`clippy-02`、`format-02`。

## 规格对照

用户限制代码目录，因此对照表与编号证据先保存在本报告；技能所列卡外 traceability/正式验收登记已写请求。

| 条款 | 实现 | 本机证据与边界 |
| --- | --- | --- |
| EV1、EV6 | `records.rs`：引用、顺序、绑定、版本、生成 codec 和数值检查 | 缺失/未来引用/NaN/错误 schema 拒绝；不替代生产认证与已接受记录接口 |
| EV2、AC2-03 多账号、LP3 | `groups.rs` + `records::canonical` | 同来源/同对话/转述传递归组，多账号合一，G 重复不增权重 |
| EV3、AC2-10、V2-87/88 | `eligibility.rs`、`assemble.rs` | 自述和候选本身无证据，维护无外源学习；合成属性前提 |
| EV4、AC2-11 性质 | `eligibility::comparable` | 同上下文有不同做法才计；缺比较、未知结果、不同情境拒绝；正式语义映射待接口 |
| I27 | `assemble::deception` | 合成事件引用的撒谎本身、同一对话后继、跨对话 parents 后继排除；晚到标注要求重算。Claim/事实级 H 引用解析待正式接口，当前拒绝 |
| EV5 | `candidates.rs`、全前缀组装 | 128 条反应相对 b=0 翻转的合成历史，组身份/成员/顺序不变；旧事件回溯不改原评估 |
| EV7、AC2-12 性质、ADR 0014 第 2 条 | `groups::teaching` | 自报/请求/事实/引用不教导；重复首次冻结；真实 G 的信任加权与重复不刷新 |
| EV8 | `EventUse`、`MatchBasis::ReadContent` | 读到的内容不生成自己的证据，阅读 a(e) 正常进入经历 |
| EV9 | `EventAttributesRecord` 消费 | 合成 verifiable 折扣到真实 G；缺属性/混合属性拒绝。生产 G05 **未完成** |
| EV10、I24 | 输入视图与纯函数边界 | 无 x/m 数据；只将 a(e) 传给 G/C；不读取快状态 |
| ADR 0014 第 3 条及更正 | `Outcome::known`、匹配过滤 | UNKNOWN 只排除结果依据；已知 0 可计；晚到结果新事件；无旧 EvidenceRelation |
| ADR 0014 第 4 条 | 规范概念/人映射 | 1/2/7/31 个同义候选不改组、规范候选或保护次序 |
| V2-80/105、P5-23 容量接口 | `proposal_order`、完整回溯 | 缺同提交次序拒绝；容量 0 拒收后提高容量，重提同一身份并消费原历史证据 |
| I23 局部对接 | `Assembly::delta` + 真实 G | 16 个合成前缀：每步增量与完整组序列一致；不代替真实 H 重放和变化 θ 的全谱系 |

## main 增量影响核对

本机必要检查的实际基础 B 为 `740c2289c2a86f400580b4e6d4aa07654c08ace6`。2026-10-29 再次 fetch 的 M 与 B 相同，无需同步新代码。测试时 HEAD 为占卡提交 `96767757155af22f017f7bb67c462598048927db` 加未提交源码；每次运行记录源码 SHA-256，交付提交前核对最终内容相同。报告修改不冒充新的源码验证基线。

## 新增依赖

无新增运行时/工作区依赖、IDL、黄金向量、规律版本、存储格式或迁移。临时 harness 使用仓库已锁定的 dev 依赖 `serde_json 1.0.154`（MIT/Apache-2.0），仅用于读取合成黄金夹具。源码文件均低于 400 行。

## 待用户确认 / 上游接续

- [ ] （阻塞）P5-06 正式属性与 accepted-record/语义接口交付并完成生产验证。
- [ ] （阻塞）同组混合 μ/verifiable/合格倾向集合的正式输入或规则；需要规范扩展时先批准 ADR。
- [ ] （阻塞）同提交实际提出次序、生产重算与 C 接线，以及卡外模块/验收登记。
- [ ] （阻塞）高难度卡独立交叉审查。当前提交可审查不表示可合并。

## 后续建议（不在本任务范围内）

按接口请求接入 P5-26/P5-29 与 C；有效 H 重解释/重放、注意力驻留去重及全谱系行为验收由对应卡补齐。不要从本模块合成测试推断真实成长或语义正确率。

## 下一步

本机验证已完成，交付至 PR #117，按用户指示转为可审查并评论「请审查」。等待交叉审查及上述上游接口；不合并、不取下一张卡。
