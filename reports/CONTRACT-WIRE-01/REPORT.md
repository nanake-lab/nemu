档位：重档。Echo Core 契约变更，待交叉审查。

这张卡定义了怎样记录云端念头、原话作者、记录之间的关系，以及实际投递的文字。

接线以后，她可以区分自己采纳过的念头、别人转述的话，以及一段被打断的话究竟说到了哪里。

批准 ADR 0046，合并即批准。

# CONTRACT-WIRE-01 说话与独处思考的契约缺口

## 当前状态

实现及本机契约验收完成。代码提交 `c9425a2e` 基于 `fea7d2fe`。PR 保持草稿，当前提交的 Ubuntu 和 Windows 检查以 PR 和手动作业关联的 head SHA 为准。Windows runner 在准备交付时离线；作业状态与解除条件见 [BLOCKED.md](BLOCKED.md)。[PR #234](https://github.com/nanake-lab/nemu/pull/234)

用户更正了卡中前驱继续写入时使用的标记。按 #164 与 CoreTransactionV3 的先例，四个后继均用 `@retains`，继承或保留全部旧字段号，新字段号均大于前驱最大号。`@supersedes` 的双向退役及生产者切换条件见 [ADR 0046 §4](../../docs/adr/0046-speak-think-contract-gaps.md#4-迁移与回退)。ADR 0046 和 V2-187 在认领与交付核对时均无撞号。

## 缺口与交付物

| 缺口 | 判定与交付 | 接线方 |
| --- | --- | --- |
| A 云端念头 | 新增 RemoteCognitiveReceipt，活动记录直接选本地或云端分支。本地沿用原结果。后继为 InternalActivityRecordV2、InternalActivityCommitV2、CoreTransactionV4。 | P5-38、P5-39 |
| B 作者与引语 | 内部产物及教导者可按已有已接受记录还原。终端文字、语音和转述需要可信绑定，新增 RecordVersion、Utf8Range、QuoteLayer、SpeakerAttribution。 | P5-43、P5-53 |
| C 记录关系 | 新增 RecordRelation、RelationSet、RelationQueryReceipt。28 个非零事实种类对照原映射九行。按端点、用途和前沿核对权威清单摘要及计数，包含反向更正和撤权。 | P5-43、P5-53 |
| D 部分投递 | 新增后继 SpeechDeliveryV2，复用有版本的端点和字节区间。旧音频字符数按样本比例估算，无法证明 UTF-8 的精确投递边界。 | P6-04 |

共 8 个新消息、4 个后继、5 个枚举。IDL、三端生成物和两类黄金向量均已提交。逐字段出处和迁移表见 [ADR 0046](../../docs/adr/0046-speak-think-contract-gaps.md)，类型见 [speak_think.proto](../../core/contracts/speak_think.proto)。

**穷举与入口。** 卡外手写消息穷举表补行数为 0。检查所有冻结向量的 Rust 冷缓存、热缓存和无缓存探针，以及三端契约探针，各新增一次语义校验分派。终端上传的三端资格过滤排除了本卡 IDL；12 类均有首块拒绝覆盖，保留已有 UserText 对照。[缓存探针](../../core/crates/core-types/src/lib.rs#L171) · [三端用例](../../core/contracts/verify.py) · [上传入口](../../core/crates/core-types/src/terminal_ingress.rs#L181)

## 验收

| 检查 | 结果 |
| --- | --- |
| `python core/contracts/generate.py --check`；`python core/contracts/check_compat.py --ref origin/main` | 0 份过期文件，全部冻结基线、当前 IDL 及 main 比较均为 0 项违规。 |
| `pnpm test:core-contracts` | Rust、Python、TypeScript 各 18,623 例，包括 4,095 个正例和 14,528 个反例；官方 Protobuf 往返 726 条。 |
| `cargo test -p core-types` | 通过。首次测试在反例 940 接受了结构合法的全零摘要，定位到缓存测试探针漏调语义校验。补入与实际契约探针相同的检查后，全部冻结正反例在三种缓存路径通过。 |
| `pnpm test:contracts` | 8 个测试文件、354 项全部通过。 |
| Rust fmt/clippy、契约 Ruff 与严格 mypy | 通过。首次 fmt 要求调整新函数的换行；首次 mypy 指出测试夹具的字典类型推断过宽。修正后复验通过。 |
| 冻结前后比对 | 旧 646 个合法向量的 CanonicalV1 合计 342,120 字节，Protobuf 合计 209,092 字节，两种编码各有 0 个变更。旧反例 901 条、入口 63 条、Unicode 18 条逐条相同。本卡追加 80 个正例、87 个反例。[分编码字节数及 SHA-256](bytes.json) · [冻结基线](../../core/contracts/baselines/p5-42/golden.json) |
| 新增反例 | 云端携带选择或行动字段、零摘要、缺关系端点、截断声称完整采纳、重算计数掩盖遗漏、前沿清单不匹配、越界引用、重叠投递及未知投递均有对应结构或语义约束；已绑定字节切片另以含四字节字符的 Rust 测试核验。[夹具](../../core/contracts/fixture_speak_think.py) · [切片和上传测试](../../core/crates/core-types/tests/speak_think.rs) |
| 当前 head 的 CI | Ubuntu 由 PR 自动触发；草稿的 Windows 自动作业按仓库配置跳过，手动检查覆盖 desktop、storage、gateway。按本 PR 当前 head 核对 [PR Checks](https://github.com/nanake-lab/nemu/pull/234/checks) 和 [分支 Windows 记录](https://github.com/nanake-lab/nemu/actions/workflows/windows.yml?query=branch%3Atask%2FCONTRACT-WIRE-01)。 |
| 空白与文件范围 | Git 识别原样复制时 `git diff --check --find-copies-harder` 通过。默认模式会报告复制自旧 growth.proto 的末尾空行；冻结副本与原文件 Git blob 相同。 |

原始检查输出含初次失败，存放在私有证据仓库，按提交与 SHA-256 定位。[证据索引](EVIDENCE-INDEX.csv)

## 规格对照与限制

| 条款 | 实现及验证 |
| --- | --- |
| ADR 0045 §4–5、v2.0 I02、I18 | 云端只生成内部文字，实际返回、获准产物与核心事务分开；本卡阻止终端上传这些类型，原子 H/Q 写入与迟返治理由 P5-38/P5-39 验收。 |
| ADR 0045 §3、§4；ADR 0042 来源及投递闭包；I16 | 稳定端点、逐层身份依据、同前沿完整性、UTF-8 呈现范围均由黄金向量和语义反例验证。对真实来源、授权、前沿治理及对象字节的核验属于后继消费者。 |
| ADR 0007、0030、0036；SYS21 | 冻结旧前缀、拒绝未知字段与越界编码、三端往返。兼容测试覆盖契约层；部署迁移或恢复按 SYS17 另验。 |

本卡没有接入任何生产路径，也没有执行模型调用、谱系发布或说话与独处的联合验收。仅使用合成夹具。已有大文件包括生成的三端描述符、类型与追加式黄金向量；其中的历史条目按冻结规则保存。无新增运行时依赖。

## 待用户确认

- [ ] （阻塞）交叉审查后由用户批准 ADR 0046 / V2-187，合并即批准。
- [ ] （CI）Windows 手动作业须在本 PR 当前提交完成。runner 状态见 [BLOCKED.md](BLOCKED.md)。
