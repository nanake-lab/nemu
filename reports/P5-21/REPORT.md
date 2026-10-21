# P5-21 IDL 契约变更：v2.0 的状态与记录 — 报告

## 结果

完成，待审查与用户批准。v2.0 的状态与记录类型已进 IDL，旧的 R/G 类型标为取代；已发布的 78 种消息的字段表、类型标签与黄金向量逐字节不变。三种语言对 9,004 个用例结论一致。这是契约变更，PR #36 标 `needs-user`，合并前需要你批准本卡与差异登记 V2-29 至 V2-37，并由 Codex 交叉审查。

## 交付物

- `core/contracts/records.proto`（新）：解释记录。天性评估、情境吻合、候选倾向、教导、撒谎、同一件事、重新解释，以及事件属性记录。
- `core/contracts/ledger.proto`（新）：变化账本 L 的条目，包括 R 与 θ 的变化、整段重算、迁移、暂停与恢复、复核暂缓与复核结论。
- `core/contracts/law.proto`（新）：R（倾向、证据组、快照）、θ 的快照、成长的管理命令、规律版本 𝔏_v 与参数、保险丝、genesis v2。
- `core/contracts/base.proto`、`state.proto`、`terminal.proto`：枚举只增不改；新增共享值类型；改动的消息以带新类型标签的后继消息实现（V2-35），旧消息原样保留并标为取代。`terminal.proto` 只改 ModulationEnvelope：新增 ModulationEnvelopeV2，只被它用到的 HookCoefficients 随之标为取代。
- `core/contracts/growth.proto`：21 种消息、4 个枚举全部标为取代，字段不变。
- `core/contracts/generate.py`：解析 `@superseded`、`@supersedes`、`reserved`。生成的 Rust、TypeScript、Python 类型带取代说明，并生成 `WireMessage::SUPERSEDED`、`<Enum>::is_superseded`、`SUPERSEDED_MESSAGES`。未改动的 `.proto` 生成的描述符逐字节相同（先改生成器、后改 IDL 时 `schema.json` 没有变化）。
- `core/contracts/check_compat.py`（新）+ `baselines/p5-01/`：不复用检查。比较当前 `.proto` 与 P5-01 发布时的 `.proto`，以及黄金向量的冻结前缀。自检对 11 种故意违例逐一报错，结果不会空过。由 `verify.py` 调用，也可以用 `--ref` 比较任意 git 引用。
- `core/contracts/freeze_vectors.py`：改为只追加。先用 Rust 权威实现重编码全部已冻结的向量，逐字节一致才继续，然后只为还没有向量的消息加种子向量，再加 P5-21 的定向向量（`fixture_inputs.py`）。
- `core/contracts/golden/`：新增 137 个合法、145 个非法向量；原有 241 个合法、243 个非法、18 个 Unicode 向量逐字节不变。
- `core/crates/core-types/src/semantic.rs`（新）：编解码之外的语义检查。包括 d_k 的单位范数、μ > 0、π ∈ (−1, 1)、倾向的计数不变量、第 16 节参数间的不等式、genesis 的一致性，以及倾向 ID 的规范次序。
- `core/crates/core-types/tests/v2_contract.rs`（新）：I28 的类型检查、语义检查与取代登记的测试。
- 生成产物：`core/crates/core-types/src/generated.rs`、`packages/contracts/src/generated/echo/`、`workers/{irl,drive,model-adapter}/generated/echo/`。
- `docs/traceability/P5-21.md`（新，按 echo-core-work 技能；只新增文件）、`docs/traceability/diff-register.md`（追加 V2-29 至 V2-37）。
- `.github/workflows/ci.yml`：追加一步，对两个新脚本跑 `mypy --strict`。不改已有条目。
- `reports/P5-21/`：本报告、`IMPLEMENTED-MAP.md`（每条消息的规格、IDL、类型标签与字段）、`review_vectors.py`（独立的字节复核）、`verification/`（全部日志）。

## 验收

本地工具链：rustc / cargo 1.97.0（stable；`rust-toolchain.toml` 锁定的 1.99.0 下载被代理拒绝，按 AGENTS.md 第 10 节用环境内的 stable），Node 22.22.0（仓库要求 24），pnpm 12.11.2，系统 Python 3.13.16，各 Python 项目用 uv 0.11.32 装的 CPython 3.12.13，grpcio-tools 与 protobuf 按 `core/contracts/uv.lock`。结论以 CI 中按锁定版本运行的结果为准。

| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | 通过，0 个生成文件过期 | verification/generate-check.log |
| `pnpm test:core-contracts`（施工合同第一篇 10.8，SYS21） | 通过：Rust、Python、TypeScript 各 9,004 个用例（2,130 合法、6,874 拒绝），官方 protoc 互操作 378 条；120 种消息每种至少 3 合法、3 非法 | verification/core-contracts.log |
| SYS21：未知 schema、NaN、计数溢出、越界的 a(e)、μ、π | 通过。verify.py 按每个字段的范围注入 min − 1、max + 1，以及传输层的 NaN、±Inf 与计数超限；另有冻结的非法向量「a(e) above 1 / below -1」「mu above 1 / below 0」「pi above 1 / below -1」 | 同上；golden/illegal.json |
| d_k 的范数（`core-types` 语义校验） | 通过：单位向量通过，零向量、非单位、NaN 被拒 | semantic.rs 单元测试；v2_contract.rs |
| 不复用检查（比较合并前后的 `.proto`） | 通过：对 `baselines/p5-01/` 与 `origin/main` 均 0 违例；自检 11 种违例全部报出；人工注入三处复用（旧消息加字段、枚举值复用、后继复用被取代的字段号）时报 5 条违例并退出 1 | verification/check-compat.log |
| 黄金向量逐字节核对 | 通过：三个文件的冻结部分与 `origin/main` 文本前缀相同；新增的 137 个合法向量用按 ADR 0007 独立重写的编码器逐个重算规范字节与摘要，0 不一致 | verification/review-vectors.log |
| 结构检查：genesis 的类型里没有自由文本的性格、风格字段（I28 的类型部分） | 通过：genesis 可达的字符串只有 5 个标识字段，字节只有 32 字节摘要，三语文本类型不可达，顶层字段正好是第 13 节的清单 | v2_contract.rs `genesis_has_no_free_text_fields` |
| `cargo fmt --all -- --check` | 通过 | verification/rust-fmt.log |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 | verification/rust-clippy.log |
| `cargo test --workspace --locked` | 通过，46 个测试，0 失败（core-types 新增 9 个） | verification/rust-test.log |
| `pnpm lint`、`typecheck`、`test`、`build`、`test:contracts` | 全部通过（vitest 422 个测试） | verification/pnpm-*.log |
| 契约工具 ruff / mypy --strict（含 CI 新增的一步） | 通过 | verification/contract-*.log |
| 四个 Python 项目 ruff、mypy --strict、pytest；三个生成目录 mypy | 全部通过 | verification/workers-*.log、services-*.log |
| 规格对照表 | 见下节与 `docs/traceability/P5-21.md` | — |
| 你批准这次契约变更 | **待批准（阻塞）** | — |

Windows 作业没有在本地运行。本卡不改动桌面、隔离与视觉相关的代码，但 Windows 作业仍按规则在 PR 上运行。推送前会先通知你。

## 规格对照（Echo Core 相关的卡必填）

逐条表格在 `docs/traceability/P5-21.md`；每条消息的规格出处与类型标签在 `IMPLEMENTED-MAP.md`。摘要：

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| v2.0 §1、§3、EV6：a(e) ∈ [−1,1]⁴，固定次序 | `records.proto` NatureAppraisalRecord；`base.proto` NatureVector | 编解码范围；冻结非法向量 |
| v2.0 §1、§2.4：μ_k(e) ∈ [0,1]，只列 μ > 0，按倾向 ID | SituationMatchRecord、TendencyMatch | 冻结非法向量；`check_situation_match` |
| v2.0 §7、EV3、§5.2：候选倾向（三语概念、情境谓词、单位方向、关系） | TendencyCandidateRecord、TendencyConcept、TrilingualText、PersonRelation | `check_direction` |
| v2.0 EV2、EV4、EV7、I27：教导、撒谎、同一件事 | TeachingAnnotation、LieAnnotation、SameMatterAnnotation | 黄金向量 |
| v2.0 EV5、§5.4：重新解释，旧记录不改 | ReinterpretationLink | 黄金向量 |
| v2.0 §1 外源、EV2、EV3、EV9 | EventAttributesRecord | 黄金向量 |
| v2.0 §5.2、§5.3 推论 7、§7、§10：R | Tendency、EvidenceGroup、EvidenceAllocation、RSnapshot | `check_tendency` 八种语义违例 |
| v2.0 §8：θ 与 C 的统计状态 | ThetaValues、ThetaSnapshot | `check_theta`、`check_theta_within` |
| v2.0 §6：x 的空间 | FastStateSnapshotV2、LearnedComponent | 规范次序测试 |
| v2.0 §2.4、§2.5：事务与 L | CoreTransactionV2、CoreCommitResultV2、LedgerEntry 及其载荷 | 变体非法向量；`check_tendency_change` |
| v2.0 §5.4、§10；ADR 0010 第 5 条：成长的管理命令 | GrowthCommand | 冻结非法向量 |
| v2.0 §1 𝔏_v、§16：规律版本 | LawVersion、LawParameters | `law_parameter_inequalities` |
| v2.0 §1 Ω：实现资源 | VersionedEnvironmentV2 | 冻结非法向量；check_compat |
| v2.0 §2.3、§13、I28：genesis | GenesisManifestV2、FuseConfig | I28 结构测试；`genesis_semantics` |
| v2.0 §9.1、§9.2：调制 | ModulationEnvelopeV2、NatureReal、LearnedCoefficient | check_compat |
| v2.0 §13、§14；ADR 0010 第 2 条：EXPERIMENT 谱系 | `Lineage.EXPERIMENT`、LineageHeaderV2 | 五个谱系变体非法向量 |
| v2.0 §15；ADR 0007 第 2 条：取代且不复用 | 各 `@superseded` 标注 | check_compat 规则 1–7 |

涉及的不变量（v2.0 §17.1）：I28 的类型部分由结构测试检查；I03、I13、I23、I25、I27、I29 在这里只提供记录类型（L 条目、快照、规律版本、证据组的份额、撒谎标注、谱系头部），由 P5-23 至 P5-26、P5-02、P5-15 执行。

### 新增与改动的 IDL 消息

新增 42 种消息、7 个枚举；枚举 Lineage、RunStatus、FailureCode 只增加新值；32 种消息标为取代。完整列表见 `IMPLEMENTED-MAP.md`。改动的消息都有后继，旧消息的字节与摘要不变：

| 旧消息 | 后继 | 保留 / reserved / 新字段号 |
| --- | --- | --- |
| VersionedEnvironment | VersionedEnvironmentV2 | 1–3、5–9 保留；4（scope_interpreter）reserved；10–13 新增，可缺省 |
| ProposalBinding | ProposalBindingV2 | 1–15、17–19 保留；16 reserved；20 为新环境 |
| SemanticCandidate、ConceptProposal | …V2 | binding 的 2 reserved；新 binding 在 7、8 |
| FastStateSnapshot | FastStateSnapshotV2 | 4、5 reserved；7 天性通道，8 学得分量 |
| FastDriveProposal | NatureAppraisalRecord | 由天性评估记录取代 |
| CoreTransaction | CoreTransactionV2 | 2 reserved；15 新 binding，16 暂存的 L 条目 |
| CoreCommitResult | CoreCommitResultV2 | 1–7 保留；8 已发布的 L 条目 |
| ModulationEnvelope | ModulationEnvelopeV2 | 10（hooks）reserved；13 方向集，14 天性层系数，15 学得层系数；Σβ 仍是 11 |
| LineageHeader | LineageHeaderV2 | 1–4 保留；5、6 合成世界标识与种子 |

### 原因码与运行状态逐个判定（v2.0 §12、§15）

| 枚举值 | 判定 | 依据 |
| --- | --- | --- |
| RunStatus NORMAL | 保留 | — |
| NO_GROWTH | 保留，只用于排练谱系，配 GROWTH_OFF_REHEARSAL | §13 |
| CORE_UNMODULATED | 保留：载体失效时 m = 0 | §12 结构故障 |
| CORE_QUARANTINED | **取代** | §12「不再整体隔离」 |
| AX_DEGRADED | **取代**，后继 A_DEGRADED | V2-17 |
| IRL_DEGRADED | 保留 | §2.4 |
| TEXT_PROJECTED | 保留 | §9.7、ADR 0003 |
| MODULATION_NOT_ENABLED | 保留：排练谱系、F 尚未实现时 | E13、ADR 0010 第 3 条 |
| STORAGE_RECOVERY_FAULT | 保留 | §12 结构故障 |
| 新增 GROWTH_PAUSED、GROWTH_REVIEW_PENDING、A_DEGRADED、M_DEGRADED | 新状态，本身说明原因 | §5.4、§2.4、V2-10、V2-17 |
| FailureCode GROWTH_NOT_ENABLED_V1 | **取代**，后继 GROWTH_OFF_REHEARSAL | ADR 0010 第 1 条 |
| AX_UNAVAILABLE | **取代** | V2-17；A 的失败用 A_DEGRADED 加通用原因 |
| IRL_FAILED、CARRIER_UNAVAILABLE | 保留 | §2.4、§9 |
| EVIDENCE_INCOMPLETE | **取代** | §15：它守护的 C 凝固查询计划已被取代 |
| UNSUPPORTED_SLICE、UNKNOWN_OVERLAP | **取代** | 卡片列明；§15 |
| DEPENDENCY_FAULT、STRUCTURAL_FAULT、VERSION_MISMATCH、NUMERIC_INVALID、RESOURCE_LIMIT | 保留：通用故障，也用作 A、M 失败的原因 | §12 |
| MODULATION_DISABLED | 保留：MODULATION_NOT_ENABLED 的原因 | E13 |
| 新增 GROWTH_OFF_REHEARSAL | 新原因码 | §13、V2-17 |

另有整枚举取代：QualificationBranch、EffectEndpoint、EffectOwner、CouplingPhase、TrialOutcome，以及 `growth.proto` 的 DistinctionForm、DistinctionStatus、TraceLifecycle、EvidenceRelation。

### 用到这些类型的卡：字段够不够

| 卡 | 需要 | 由哪些类型提供 |
| --- | --- | --- |
| P5-06 | 校验 a(e)、μ、d_k、ProposalBinding、生产者与版本；属性规则表；候选的概念登记 | 各记录类型；`semantic`；EventAttributesRecord；ConceptProposalV2 |
| P5-08 | 各阶段的接口类型；按谱系的阶段配置；新的状态与原因码；事务 | 记录类型、FastStateSnapshotV2、RSnapshot、ThetaSnapshot、LedgerEntry、CoreTransactionV2；VersionedEnvironmentV2 的四项可缺省，正式与实验谱系启动时检查；RunStatus、FailureCode、LineageHeaderV2 |
| P5-10 | X 的空间；D 的输入 a、b、g、λ、μ、s_k、r_k、d_k；共情的亲近度；w_min、g_max、r_max | FastStateSnapshotV2；NatureAppraisalRecord（含所涉之人）、ThetaValues、SituationMatchRecord、Tendency（π、typical_magnitude、direction、relation）、LawParameters；`semantic::tendency_order` |
| P5-23 | 倾向的证据统计；证据组；n₀、κ₁、w_max、w_teach、v_u、r_min、r_max、h₀、n_h；关系倾向与 A_att；R 的容量；L 的 R 变化 | Tendency、EvidenceGroup、LawParameters、PersonRelation、GenesisManifestV2.attachment_prior、FuseConfig.r_capacity、TendencyChange |
| P5-24 | 输入记录与属性；输出证据组与外源事件序列 | 各记录类型、EventAttributesRecord、EvidenceGroup |
| P5-25 | θ 与 C 的统计状态；θ 的先验与范围；Δθ_max、缓慢漂移；L 的 θ 变化 | ThetaSnapshot、LawParameters、GenesisManifestV2（g₀、b₀、σ₀、τ₀）、FuseConfig、ThetaChange、ReviewTrigger |
| P5-26 | 整段重算、来源更正、新规律版本、重新解释、暂停与复核、跳变检查 | RecomputeRecord、LawVersion、ReinterpretationLink、LedgerEntry、GrowthCommand、FuseConfig 的跳变阈值与探测情境 |
| P5-28 | 输出吻合、候选、标注与回溯记录 | SituationMatchRecord、TendencyCandidateRecord、三种标注；记录的 event_id 可以不同于 binding 的事件 |
| P5-29 | 接入事务；管理命令的执行；实验谱系；验证前的方向不进入 m | CoreTransactionV2、GrowthCommand、LineageHeaderV2、ModulationEnvelopeV2（只列已验证的学得系数） |

两处留给后续卡，不在本卡加字段：

- **P5-12 的「回执里列出」未验证的方向：** P5-12 卡给出的 ModulationEnvelope 字段清单里没有这一项。回执的形式由 P5-12 定；如果要放进 IDL，就新增后继消息。
- **P5-28 的输入视图：** Tendency 里有 π 等强度信息，不能直接交给 M。worker 的输入 DTO 由 P5-28 定义，只含概念定义与情境谓词（EV6）。

### 黄金向量的变化

- 原有 241 个合法、243 个非法、18 个 Unicode 向量逐字节不变：文本前缀与 `origin/main` 相同，`baselines/p5-01/golden.json` 记录了它们的条数与 SHA-256，由 check_compat 在 CI 中复核。
- 新增 137 个合法向量：42 种新消息各 3 个；LedgerEntry 另加 5 个，覆盖全部 8 种条目；另有 6 个定向向量，覆盖新的状态码与原因码，以及真实取值的倾向、天性评估和情境吻合。
- 新增 145 个非法向量：每种新消息 3 个（缺版本、未知版本、未知字段），另有 19 个定向向量（越界的 a(e)、μ、π，谱系与账本的变体，重复的集合键，已删除的字段名，未知状态码）。
- 手算核对示例：NatureVector 全 0 的规范字节是 `4e435631`（NCV1）、`01000000`（schema 1）、`12000000`（标签长 18）、`6563686f2e6e61747572652d766563746f72`（`echo.nature-vector`），接 4 个 binary64 零，各 8 字节，共 62 字节。与冻结的向量一致。

### 迁移说明（ADR 0007 第 11 条）

没有任何已发布的字节被重新解释：改动都是新消息、新标签、新枚举值，或者对旧消息加取代标注，标注不进字节。目前还没有排练存储或正式存储，P5-02 尚未开工，所以没有需要迁移的已存对象，也不需要备份。以后的生产者改用后继类型；读旧数据时仍按旧类型解码。从旧类型到新类型的映射见上表。在 N3 之前再改编码，仍按 ADR 0007 第 11 条先备份、写迁移说明。

### 超过 400 行的文件

只有生成的 `generated.rs`、`schema.json`、`schema.ts` 和黄金向量 JSON，都由同一份 IDL 机器生成，与 P5-01 相同。

## 新增依赖

无。没有新增运行时或开发依赖，锁文件不变。

## 待用户确认

- [ ] （阻塞）批准本次契约变更（P5-21 卡，ADR 0009、0010）。
- [ ] （阻塞）差异登记 V2-29：第 16 节的参数按第 13、16 节分放在规律版本、genesis 与保险丝中，而不是全部放进规律版本。
- [ ] （阻塞）V2-30：L 条目与保险丝的最小字段集（重建）。
- [ ] （阻塞）V2-31：速率以 1/秒、持续时间以秒计。
- [ ] （阻塞）V2-32：语义检查的读法，单位范数容差 1e-12，成事的初始增益严格最弱。
- [ ] （阻塞）V2-33：解释记录的外形（重建）。
- [ ] （阻塞）V2-34：R、θ 的存储字段（重建）。其中 θ 的递推状态是一个由 P5-25 定格式的对象引用。
- [ ] （阻塞）V2-35：以后继消息实现改动。ProposalBinding 换代后，SemanticCandidate、ConceptProposal 也随之有后继，卡里没有写到这两条。
- [ ] （阻塞）V2-36：原因码与状态的判定。RunState 在 P5-22 收紧之前，编解码层也接受 EXPERIMENT。
- [ ] （阻塞）V2-37：genesis v2 的读法（不含名字与 x 的初值，读出与边界以引用给出）。

## 后续建议（不在本任务范围内）

- **P5-22：** 用后继消息或 `@rules` 把终端 RunState 的谱系限制在 REHEARSAL、LIFE（V2-36）。
- **P5-26：** 来源更正事件的载荷类型（事件是假的、几个账号是同一个人、内容被篡改）还没有 IDL。P5-26 开工前需要一张契约卡，或在该卡里登记。
- **P5-27：** 不再使用 EffectContract 等取代类型后，可以让 core-kernel 不再依赖它们。`generated::SUPERSEDED_MESSAGES` 可用来做静态检查。
- **P5-21 之后的契约卡：** 在 `baselines/` 下新增一个基线目录，冻结本卡发布的 `.proto` 与黄金向量摘要，check_compat 会自动比较全部基线。

## 下一步

等待两件事：Codex（GPT-6 Astra，最高推理强度）交叉审查，以及你批准契约变更与 V2-29 至 V2-37。在 P5-21 合并之前，Codex 不能开 P5-07，因为两者都改 `workers/irl/`。接下来取的卡：P5-27（依赖 P5-09，已合并；目录 `core/crates/core-kernel/src/effects/`、`core/crates/core-policy/`，与本卡不重叠）。按「一次只做一张卡」，本卡合并后再开工，或者等你另行指示。

## 2026-10-21 合并 main 与登记重编号（追加）

- 合并最新 main（含 #27、#37、#38、#39、#41、#42），不 rebase、不强推。两处冲突：
  - `.github/workflows/ci.yml`：#39 把 Windows 作业移到 `windows.yml`。本卡追加的契约工具检查步骤放在 Ubuntu 作业末尾，内容不变；
  - `docs/traceability/diff-register.md`：保留双方。
- 差异登记编号：本卡原用 V2-19 至 V2-27，与先合并的 ADR 0012（V2-19 至 V2-23）、ADR 0014（V2-24 至 V2-28）重号，改为 V2-29 至 V2-37（V2-19→29 … V2-27→37），内容不变。本报告、`docs/traceability/P5-21.md`、`law.proto` 与 `ledger.proto` 的注释、`core-types/src/semantic.rs` 的文档注释随之改号。注释不进入生成代码，已发布的字节与摘要不受影响。
- ADR 0014 用到的取值（`USER_SELF_REPORT`、`UNKNOWN_ACTION`、`UNKNOWN_DELIVERY`、`EVIDENCE_UNKNOWN`、`TeachingAnnotation`）在本卡契约中都已存在，不需要改动。
- 合并后的复跑结果见 `verification/after-main-merge-20261021/`。


## 2026-10-21 Codex 审查后的修正（追加）

Codex 交叉审查（[PR #36 评论](https://github.com/nanake-lab/nemu/pull/36#issuecomment-6310131751)）提出 2 项 P2。已合并最新 main（含 #46，ADR 0014 的更正），merge，不 rebase，不强推。

- **更正（第 1 项）：** 上一节「2026-10-21 合并 main 与登记重编号」说 ADR 0014 用到的取值都已存在、不需要改动，并把 `EVIDENCE_UNKNOWN` 列在其中。这一说法不成立：`EVIDENCE_UNKNOWN` 属于整体标为取代的 `EvidenceRelation`（v2.0 第 15 节），生产者不能再用。以 ADR 0014 的「更正」一节为准：结果是否未知按 H 中已有的记录判定，不需要证据关系取值，也不需要契约变更。本卡不恢复这个取值，`EvidenceRelation` 的取代标注不变。ADR 0014 仍依赖的 `USER_SELF_REPORT`、`UNKNOWN_ACTION`、`UNKNOWN_DELIVERY`、`TeachingAnnotation` 都是有效取值。新增测试 `v2_contract.rs::adr_0014_values_are_live` 断言这一点：生成的 Rust 标志与三种语言共用的 `schema.json` 都要求它们没有取代标注，并锁定 `EVIDENCE_UNKNOWN` 仍为取代。临时把 `USER_SELF_REPORT` 标为取代时，这个测试失败。
- **修正（第 2 项）：** `check_compat.py` 的后继规则原先只检查前任的实际字段。前任已保留的号虽然并入了比较集合，却没有要求后继继承，所以跨两代时可以被复用。现在后继必须完整继承前任的 `reserved`，也就是前任保留的号，加上前任字段中没有沿用、被取代的号；后继的字段也不得与自己的 `reserved` 相交。自检新增三种跨两代的负例：
  - Codex 的反例：VersionedEnvironmentV3 继承 V2，把 V2 的 reserved 4 改作新字段；
  - V3 不再保留 4；
  - V3 保留 4 的同时又声明字段 4。

  另有一个对照组：正确继承 reserved 4、新增字段 14 的 V3 必须通过，证明三种负例失败是因为 reserved 规则，而不是别的噪声。原有 11 种自检照旧全部报出，共 14 种。把检查规则换回修复前的版本时，新增的三种全部漏报。本卡的 IDL 本身没有复用任何号，check_compat 对 P5-01 基线与 `origin/main` 仍为 0 违例。
- 复跑结果见 `verification/review-fixes-20261021/`。
