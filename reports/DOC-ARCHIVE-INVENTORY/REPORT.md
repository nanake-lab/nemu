轻档：本 PR 只新增这一份报告（`reports/`），不移动、不删除、不改任何现有文档。

# DOC-ARCHIVE-INVENTORY E1 收尾文档归档清点 — 报告

## 当前状态

清点完成，等用户审查。范围是 `docs/` 全部 Markdown 与原文目录（含 `adr/`、`echo-core/`、`traceability/`、`learn/`、`journal/`）、`tasks/requests/` 全部 42 份，以及根目录的 `README.md`、`AGENTS.md`，共 124 份。核对基线为 `origin/main` `74f7bcaf41ec45bb31063fc079eeeae49927ac82`。

结论：

- 能明确判为**整体取代**的只有 2 份：[ADR 0006](../../docs/adr/0006-single-root-spec-conflict.md)（被 ADR 0009 取代），以及旧规格的转换版 [v0.6.4-rg-plasticity-spec.md](../../docs/echo-core/v0.6.4-rg-plasticity-spec.md)（全部内容落在 v2.0 第 15 节列为「取代」的各行）。后者能不能挪，取决于用户怎样理解「旧规格的原文与转换版作为历史保留」，标「待用户判断」。
- 其余旧说法大多散落在仍然现行的文档里，属于**部分取代**。多数已经有文内指引（DOC-PRESETS-01 的「有效性说明」、各 ADR 的「追加」），但有一批没有，见第 3 节。
- **状态标注不一致**共列出 50 处（第 3 节 A–G 组）。最集中的是 ADR 头部：16 份 ADR 已经由带 `needs-user` 的 PR 合并进 main，头部仍写「提议」「草案」或「PROPOSED」。

分类口径：

| 分类 | 含义 |
| --- | --- |
| 现行 | 现行规则在用，没有被取代的部分；或者只在文内追加里修订了个别读法、且已标明 |
| 部分取代 | 有具体的节或条被后来的决定取代，其余仍有效。表中写明是哪几节、被谁取代、文内有没有指引 |
| 整体取代 | 全文的内容都已被取代，现行规则不再使用 |
| 历史记录 | 报告、手记、审计、施工对照、已办结的请求与卡入口。按惯例不挪 |

「已合并」一律指 PR 合并进 main。判断依据是 `gh api` 读到的 PR 状态与标签；这批 PR 的合并者都是 `nanake-lab`。按 AGENTS.md 第 6 节，带 `needs-user` 的 PR 由用户合并、合并即批准，本报告据此推断「已采纳」。只有 ADR 0034 有用户的明确判定（[P0-12 报告](../P0-12/REPORT.md)第 2 行），其余都是推断，所以第 3 节统一标「待用户判断」。

## 1. 逐份清单

### 1.1 根目录

| 文件 | 分类 | 说明 |
| --- | --- | --- |
| [README.md](../../README.md) | 现行 | 2026-11 由 DOC-README-01（#153）重写 |
| [AGENTS.md](../../AGENTS.md) | 现行 | 有两处旧说法：第 4 节第 104 行写「birth 只由用户在 N3 执行」，N3 已改为 E4；第 2 节表格与第 10 节的执行者分工，见第 3 节 G 组 |

### 1.2 docs/ 顶层

| 文件 | 分类 | 被取代的部分（取代者） | 文内指引 |
| --- | --- | --- | --- |
| [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) | 部分取代 | 第 2 节第 6、8、11、30 项，第 3 节，第 5 节第 1 步，第 6.3–6.7、6.9 节的旧预设读法（ADR 0032、V2-117、ADR 0025、ADR 0026 §7）；第 5 节说完判定与打断目标（ADR 0017）；声音模型（ADR 0027） | 有：各节开头的 DOC-PRESETS-01 有效性说明、ADR 0017 追加；第 227 行「排练版（N2）」仍用旧里程碑名 |
| [CONTRACTS.md](../../docs/CONTRACTS.md) | 部分取代 | 第 3.3 节 `ttsStyle` 的「本地 Qwen3-TTS Base」（ADR 0027）；第 4.1 节 `desktop.observation`、第 4.2 节 `desktop` 观察、第 6 节 T1 表中「窗口标题需 T1 授权」（ADR 0025，V2-70 等）；第 5.3 节 `autonomousPerDay: 30` 与第 8 节「自主助手每日 30 次」（ADR 0033 §2）；第 7 节读出「参数来自 genesis，运行中不学习」（ADR 0032 §8）；第 8 节 180 秒、每小时 2 次等固定主动预算（V2-117）；「追加（2026-10-20）」（P5-22 已完成，现为历史授权） | **无**。DOC-PRESETS-01 当时因目录与 P6-17 重叠没有改这份文件，只在[索引](../../docs/traceability/DOC-PRESETS-01.md)里写明边界；ADR 0033 发生在索引之后，索引也没有覆盖 |
| [SETUP.md](../../docs/SETUP.md) | 部分取代 | 「Windows 自托管 runner」第 1 条「确认仓库保持私有」（2026-10-26 仓库公开，#99 收紧公开仓库的 runner）；「CI 额度」一节按私有仓库的托管分钟估算——公开后是否还适用，**待用户判断** | 无 |

### 1.3 docs/adr/（33 份）

| ADR | 分类 | 被取代的部分（取代者） | 文内指引 |
| --- | --- | --- | --- |
| [0001](../../docs/adr/0001-baseline-architecture.md) | 部分取代 | 「Agent」「离线」两行、动画行为状态机、人设/记忆库/主动性/情绪模型（ADR 0002）；里程碑（ADR 0005）；「语音」一行的 Qwen3-TTS（ADR 0027） | 状态行写明了 0002、0005；没写 ADR 0027（ADR 0027 只点名取代 ARCHITECTURE 和 P4-02）。是否算 0027 取代，**待用户判断** |
| [0002](../../docs/adr/0002-echo-core-as-nemu-core.md) | 部分取代 | 决定 4（ADR 0010）；决定 7、8、10（ADR 0009）；NO-GROWTH 与资格（ADR 0009/0010、V2-15） | 有（2026-10-19、10-29 追加） |
| [0003](../../docs/adr/0003-speech-modes.md) | 部分取代 | 「genesis 校准时由用户挑选语气样例」（v2.0 §9.6–9.7）；模式 C「行动计划不变」（ADR 0026 §7） | 有 |
| [0004](../../docs/adr/0004-codex-helpers.md) | 部分取代 | 决策第 5 项的每日 30 次自主调用上限（ADR 0033 §2：改为防失控保险丝） | **无**：10-29 追加写的是「旧日额度列入 BLOCKED」，没有指向 10-30 的 ADR 0033 |
| [0005](../../docs/adr/0005-rehearsal-and-birth.md) | 部分取代 | 排练、谱系、N3（ADR 0010）；§4 里程碑 N0–N4 与承诺日期（2026-10-26 里程碑重排为 E1–E5，#93，[tasks/README.md](../../tasks/README.md) 第 5 节） | 有 ADR 0010 追加；**没有** E1–E5 重排的说明 |
| [0006](../../docs/adr/0006-single-root-spec-conflict.md) | **整体取代** | 全文（ADR 0009：「取代：ADR 0006」，v2.0 没有单根、多根资格） | 0006 本身没有标注；见第 2、3 节 |
| [0007](../../docs/adr/0007-initial-idl-canonical-encoding.md) | 现行 | 第 133 行「C01 remains unresolved; ADR 0006 remains PROPOSED」已过时（ADR 0009） | 无，见第 3 节 |
| [0008](../../docs/adr/0008-echo-core-math-clarifications.md) | 部分取代 | §1(4) 用于要求统一定时回落的读法（ADR 0032 §5） | 有 |
| [0009](../../docs/adr/0009-echo-core-v2-first-principles.md) | 部分取代 | 固定预设、主动强度、旧暂停与权限、出生锚定、教导与真实后果的旧读法（ADR 0032 及 DOC-PRESETS-01 索引） | 有 |
| [0010](../../docs/adr/0010-v1-scope-birth-and-lineages.md) | 部分取代 | 固定主动资源、合法变化自动暂缓、容量满暂停等旧读法（ADR 0032） | 有 |
| [0011](../../docs/adr/0011-network-life-boundaries.md) | 部分取代 | 社交审批与钱包（V2-89）；日联系人/消息上限（V2-104）；撒谎不带来成长（ADR 0032 §6）；决定 2「与 u* 共用账号」在微信场景（用户 2026-11-01 决定她有自己的微信号） | 前三项有；**微信一项没有**，只记在 [R-WECHAT-01 请求](../../tasks/requests/R-WECHAT-01.md)第 24 行，正式修订留给了后继卡 |
| [0012](../../docs/adr/0012-continuity-carrier-mobile.md) | 部分取代 | 决定 1「先试 Qwen3-4B-Instruct-2507」与第 4 条三组对比里的模型名（ADR 0034，P0-12 已判定通过）；连续性、移动端等其余条款现行 | **无**，见第 3 节 |
| [0014](../../docs/adr/0014-v2-amendments-from-research.md) | 部分取代 | 「已读未回必为负」的固定评价、§2 教导的直接证据读法（ADR 0032 §§1–2、§6） | 有 |
| [0015](../../docs/adr/0015-vrm-license-fields-and-p2-lint-split.md) | 现行 | — | — |
| [0016](../../docs/adr/0016-law-replay-fields.md) | 部分取代 | 四种 V2 后继消息在生产上由 V3 取代，V2 只读（ADR 0021） | ADR 0021 有，0016 自身没有 |
| [0017](../../docs/adr/0017-voice-endpoint-and-barge-in-targets.md) | 现行 | — | — |
| [0018](../../docs/adr/0018-law-lp1-reference-conflict.md) | 部分取代 | 「问题与两种方案」已由用户选 A、ADR 0019 落实；文末「用户选择 A」追加里 LP1 的新增审查要求仍是 ADR 0019 的授权依据 | 有（文末追加）。是否改判为整体取代，**待用户判断**；指向它的路径见第 2 节附表 |
| [0019](../../docs/adr/0019-law-lp1-reference.md) | 现行 | 存储字段见 ADR 0021；ADR 0035 定义的新规律版本发布后，相应部分被取代（发布时点**待用户判断**） | — |
| [0020](../../docs/adr/0020-fact-view-cpu-vector-runtime.md) | 现行 | — | — |
| [0021](../../docs/adr/0021-law-retention-fields.md) | 部分取代 | 第 7、9 节的存储扩围方案与第 10 节的完全撤回状态（同文末节「分阶段交付」） | 有（文内） |
| [0022](../../docs/adr/0022-body-readout-animation-v2.md) | 部分取代 | 固定线性映射作为终态的读法（ADR 0032 §8） | 有 |
| [0023](../../docs/adr/0023-desktop-focus-probe.md) | 现行 | — | — |
| [0024](../../docs/adr/0024-terminal-ingress.md) | 现行 | — | — |
| [0025](../../docs/adr/0025-desktop-perception-and-history.md) | 现行 | （它是 CONTRACTS「窗口标题需 T1」的取代者） | — |
| [0026](../../docs/adr/0026-continuous-cognition.md) | 部分取代 | §2(5)、§6.3 与 COG-A12 的统一定时回落（ADR 0032 §5）；§6.3 的结果期限 τ（ADR 0032 §7，ADR 0035 §5.6）；§6.3 念头与行动结果的 κ^n、κ^{n_发出} 递减（ADR 0035 §4.9，在新规律版本中） | 有 ADR 0032 部分；**没有** ADR 0035 部分 |
| [0027](../../docs/adr/0027-voice-model-breeze-target-gsv-interim.md) | 现行 | P4-03 登记 IndexTTS 1.5 为候选，没有改路线（#152） | — |
| [0028](../../docs/adr/0028-lifecycle-single-instance.md) | 现行 | — | — |
| [0029](../../docs/adr/0029-theta-estimator-storage.md) | 现行 | — | — |
| [0031](../../docs/adr/0031-carrier-engine-and-injection.md) | 部分取代 | 第 20 行「当前基线为 Qwen3-4B-Instruct-2507」（ADR 0034 已生效）；引擎、量化、注入与 E2 默认 m = 0 现行 | 同一行写了「P0-12 实测通过才改」，但没有更新为已通过 |
| [0032](../../docs/adr/0032-presets-as-starting-points.md) | 现行 | （大量取代关系的来源） | — |
| [0033](../../docs/adr/0033-helper-fuses-cursor-evidence.md) | 现行 | — | — |
| [0034](../../docs/adr/0034-qwen35-4b-conditional-carrier.md) | 现行 | 生效条件已满足（P0-12 #128，用户判定通过） | — |
| [0035](../../docs/adr/0035-law-definitions-d07-d09-d11.md) | 现行 | — | — |

编号空缺：0013 在仓库任何地方都没有用过；0030 由开着的 PR #155（P5-37）占用。挪动时不要复用这两个号。

### 1.4 docs/echo-core/

v2.0 第 15 节把旧规格逐项分成「保留」与「取代」。下表按该表定位；没有出现在该表里的节，冲突处以 v2.0 为准，本次没有逐条判定。转换版和原文都不改，所以旧规格文件里都没有文内指引。

| 文件 | 分类 | 被取代的部分（取代者） | 仍保留 |
| --- | --- | --- | --- |
| [README.md](../../docs/echo-core/README.md) | 部分取代 | §1 优先级（v2.0 排到最前）；§2 缺失增补与 E12（ADR 0009：消解）；§3 v1.0 范围（ADR 0010）；§4 中 E5、E6、E10、E13、E14（ADR 0009/0010）；§5 阅读建议（[echo-core-work 技能](../../.agents/skills/echo-core-work/SKILL.md)第 1 节，2026-11-11） | 文末三段追加都已写明；E1–E14 登记表仍是总入口 |
| [echo-core-v2.0-first-principles.md](../../docs/echo-core/echo-core-v2.0-first-principles.md) | 现行（最高优先） | 局部读法由 ADR 0014、0019、0032、0033、0035 修订 | 文内已有 2026-10-29 决定指引，头部状态见第 3 节 |
| [ada-core-operators-spec-v1.0.md](../../docs/echo-core/ada-core-operators-spec-v1.0.md)（O） | 部分取代 | §§6–11（G、C、资格、试验、交接；v2.0 第 4、5、7、8、12 节）；§5.2 A_x（改为「乎」评估）；§3.3 scope DSL、§10.2、§15 的 q 包络（v2.0 LP5，P5-27）；§18 的 I07、I08、I09、I12、I19、I20、I22（v2.0 §17.1）；§20 的 G、C 系列验收 | §2 状态与写权限、§4 事件顺序、§5 F 的结构（ADR 0008 §1）、§§5.3–5.4 降级、§§12–14 J 与载体（方向改为从模型提取，签名按 ADR 0008 §3）、§16 事务与重放、§17.3 行动 |
| [echo-core-system-plan-v1.0.md](../../docs/echo-core/echo-core-system-plan-v1.0.md)（S） | 部分取代 | §§14–18；§12 的 scope DSL 部分（P5-27）；§4、§26 中的 W06–W10（ADR 0010）；§27「分工 排期与第一批任务」是否已被 E1–E5 任务体系整体取代，**待用户判断** | §§1–13、§§19–25 的结构；§20 机会规则只保留边界（v2.0 §9.4）；§21 结构故障路径；§22 重放与谱系 |
| [v0.6.4-frozen-architecture.md](../../docs/echo-core/v0.6.4-frozen-architecture.md)（F） | 部分取代 | §6 R、χ、K；§§7–9 凝固、账本更新、更新审查（v2.0 第 8、12 节）；§16、§17 中的 G、C 系列验收 | §2 状态、§§3–4 H 与来源、§5 事务、§10 J（方向从模型提取）、§11 行动（强度来自「乎」）、§13 重放与身份 |
| [v0.6.4-integrated-implementation.md](../../docs/echo-core/v0.6.4-integrated-implementation.md)（D） | 部分取代 | 第二篇全文（文件第 336 行起，R、χ、K、G、shadow、C/θ；v2.0 第 4、5、7、8 节）；第一篇 §4.5 中的 NO-GROWTH 读法（ADR 0010）；§5.3 A_x（v2.0 §15） | 第一篇其余（IRL、x、IDL、事务、权限矩阵） |
| [v0.6.4-rg-plasticity-spec.md](../../docs/echo-core/v0.6.4-rg-plasticity-spec.md)（R） | **整体取代**（挪不挪**待用户判断**） | 全文。正文与 D 第二篇相同（文件首行注释）；第 0–11 部分讲的都是 v2.0 第 15 节列为取代的 R、χ、K、G 状态机与 C/θ 凝固。数学核验也曾建议在合并稿里删去这份（[核验报告](../../docs/traceability/echo-core-math-audit.md)「删除」一条） | 无独有内容。原文 docx 保留在 `original/` |
| `original/` 下 5 份 docx | 现行（原文） | — | 永远不改、不挪（AGENTS.md 第 2 节，Docs guard 检查） |

### 1.5 docs/traceability/（29 份）

| 文件 | 分类 | 说明 |
| --- | --- | --- |
| [diff-register.md](../../docs/traceability/diff-register.md) | 现行 | 只追加。条目的状态散在后续追加行里，本次没有逐条核对（见「后续建议」） |
| [DOC-PRESETS-01.md](../../docs/traceability/DOC-PRESETS-01.md) | 现行 | 预设决定的有效读法索引；不含 ADR 0033 之后的决定 |
| [law-lp1-reference.md](../../docs/traceability/law-lp1-reference.md) | 现行 | ADR 0019 的对照；头部「设计状态：提议」见第 3 节 |
| [baseline.md](../../docs/traceability/baseline.md) | 部分取代 | 别名与哈希表现行（数学核验和 `reports/P0-06/verify-baseline.ps1` 都在用）；第 33 行「W00-C01 待用户确认、ADR 0006 未批准」与第 37 行「E12 待用户提供」已过时（ADR 0009；第 39 行已追加「不可得」，但没提 ADR 0009） |
| [scope-v1.md](../../docs/traceability/scope-v1.md) | 部分取代 | W06–W10 行、G/C 验收行、成长相关不变量行（文内 2026-10-19 更新已标为历史）；2026-10-20 路由表里 AC2-11、AC2-12、AC2-15、AC2-25 须按 ADR 0032 重定义（DOC-PRESETS-01 索引），后来新拆的规律卡（P5-66 至 P5-76）也没有进表。`reports/P0-06/verify-baseline.ps1` 读取本文件 |
| [continuous-cognition.md](../../docs/traceability/continuous-cognition.md) | 部分取代 | COG-06、COG-07 的递减加权，COG-09 至 COG-11 的期限，COG-A12 的统一回落（ADR 0032 §§5、7，ADR 0035 §§4.9、5.6、8）；文内第 13 行只注明 COG-06/07 取代了更早的写法 |
| [echo-core-math-audit.md](../../docs/traceability/echo-core-math-audit.md) | 历史记录 | 2026-10-19 审计；文末「追记」已列出针对被取代机制的条目 |
| [build-lock.md](../../docs/traceability/build-lock.md) | 历史记录 | P0-06 在 2026-10-14 实测的构建基线 |
| 各卡施工对照 21 份：[P0-01](../../docs/traceability/P0-01.md)、[P0-06](../../docs/traceability/P0-06.md)、[P5-01](../../docs/traceability/P5-01.md)、[P5-02](../../docs/traceability/P5-02.md)、[P5-03](../../docs/traceability/P5-03.md)、[P5-04](../../docs/traceability/P5-04.md)、[P5-09](../../docs/traceability/P5-09.md)、[P5-10](../../docs/traceability/P5-10.md)、[P5-12](../../docs/traceability/P5-12.md)、[P5-21](../../docs/traceability/P5-21.md)、[P5-22](../../docs/traceability/P5-22.md)、[P5-23](../../docs/traceability/P5-23.md)、[P5-25](../../docs/traceability/P5-25.md)、[P5-27](../../docs/traceability/P5-27.md)、[P5-32](../../docs/traceability/P5-32.md)、[P5-34](../../docs/traceability/P5-34.md)、[P5-35](../../docs/traceability/P5-35.md)、[P5-36](../../docs/traceability/P5-36.md)、[P5-65](../../docs/traceability/P5-65.md)、[P6-08](../../docs/traceability/P6-08.md)、[P6-17](../../docs/traceability/P6-17.md) | 历史记录 | 技能第 1 节把它们列为「按需查找」，按惯例不挪。其中有局部取代：P5-09 中 O §3.3、§10.2、§15 与 S §12 的 scope DSL、q 包络、ValidityContract、接管与资格被 P5-27 取代（P5-27 首行写明，**P5-09 自身没有指向**）；P5-02、P5-23、P5-36 在文末追加里取代了前文的旧状态；P5-04 的 V2-14 旧读法被 V2-88 取代（文内已注）。状态行过时的见第 3 节 |

### 1.6 docs/learn/、docs/journal/

| 文件 | 分类 | 说明 |
| --- | --- | --- |
| [learn/README.md](../../docs/learn/README.md) | 现行 | 第 27 行仍让读者「从 docs/echo-core/README.md 第 5 节的顺序开始」，该节已改为分级读法，见第 3 节 |
| [learn/01-echo-core-and-the-model.md](../../docs/learn/01-echo-core-and-the-model.md) | 现行 | 2026-11-07，载体已按 Qwen3.5-4B 写 |
| [journal/README.md](../../docs/journal/README.md) | 历史记录 | 开发手记 |

### 1.7 tasks/requests/（42 份）

「已办结」指请求的事已经做完或已被决定，文件按惯例作为历史记录保留；「现行」指仍有卡或代码在引用、或仍在等待。

| 文件 | 分类 | 说明 |
| --- | --- | --- |
| [DESIGN-ASSIST-EV-01](../../tasks/requests/DESIGN-ASSIST-EV-01.md) | 历史记录（卡入口） | #124 已合并；状态行过时，见第 3 节 |
| [DESIGN-EV-01](../../tasks/requests/DESIGN-EV-01.md) | 历史记录（卡入口） | #119 已合并；用户选择记在 ADR 0033 |
| [DESIGN-LAW-01](../../tasks/requests/DESIGN-LAW-01.md) | 历史记录（卡入口） | #131 已合并；认领状态行过时，见第 3 节 |
| [DESIGN-VRAM-01](../../tasks/requests/DESIGN-VRAM-01.md) | 历史记录（卡入口） | #130 已合并；末段「用户选择 D1–D4 后另开 ADR 与实现范围」是否都已登记成卡，本次没有核对 |
| [DOC-PRESETS-01](../../tasks/requests/DOC-PRESETS-01.md) | 历史记录（卡入口） | 已由 #122 执行；头部仍写「待排期，未认领」，见第 3 节 |
| [P0-04-scope-and-version-bootstrap](../../tasks/requests/P0-04-scope-and-version-bootstrap.md) | 历史记录（已办结） | SETUP.md 已有 P0-04 一节；Blender 锁 5.2.2 LTS 已记录 |
| [P0-08-config-scope](../../tasks/requests/P0-08-config-scope.md) | 部分取代 | `config/helpers.json` 仍未建立，[N0 决定](../P0-09/DECISION.md)第 33 行把它列为用户 ADR 事项，所以仍在等；其中「每日预算」候选已被 ADR 0033 §2 改为防失控保险丝 |
| [P2-05-review-spring-inspector](../../tasks/requests/P2-05-review-spring-inspector.md) | **待用户判断** | 针对自制 VRoid 模型的弹簧骨；2026-10-26 已决定购买外观模型，自制路线暂停。这项还要不要做，请用户定 |
| [P2-05-spring-fold-continuity](../../tasks/requests/P2-05-spring-fold-continuity.md) | **待用户判断** | 同上；末行「PR #48 继续保持草稿」已过时（#48 已合并） |
| [P2-05-spring-runtime-stability](../../tasks/requests/P2-05-spring-runtime-stability.md) | **待用户判断** | 同上；保护已由 #67 划给 P3-01 的 L4 |
| [P2-05-tail-stage-integration](../../tasks/requests/P2-05-tail-stage-integration.md) | 历史记录（已办结） | 用户 2026-10-23 批准并实施，#48 已合并 |
| [P3-06-panel-successor-preset](../../tasks/requests/P3-06-panel-successor-preset.md) | 历史记录（已办结） | 用户批准，#76 已合并 |
| [P4-01-to-P0-09-latency-budget](../../tasks/requests/P4-01-to-P0-09-latency-budget.md) | 历史记录（已办结） | 由 ADR 0017 与 N0 决定第 4 条落定 |
| [P5-02-acceptance-registration](../../tasks/requests/P5-02-acceptance-registration.md) | 现行 | 等 P5-18（W16） |
| [P5-03-gateway-interfaces](../../tasks/requests/P5-03-gateway-interfaces.md) | **待用户判断** | P5-03（#64）已合并，P5-35（#110）已提供契约与存储接口；完整 daemon 的认证上传分派等接口是否还有缺口，本次没能确认 |
| [P5-04-core-store-prefix](../../tasks/requests/P5-04-core-store-prefix.md) | 历史记录（已办结） | 用户授权，#65 已合并 |
| [P5-04-core-types-codec-cache](../../tasks/requests/P5-04-core-types-codec-cache.md) | 历史记录（已办结） | 同上 |
| [P5-04-memory-page-transport](../../tasks/requests/P5-04-memory-page-transport.md) | 现行 | 等契约/集成卡提供续页字段 |
| [P5-04-windows-temp-acl](../../tasks/requests/P5-04-windows-temp-acl.md) | 历史记录（已办结） | 2026-10-26 改按 main 工作流，原选项不再执行 |
| [P5-05-permission-integration](../../tasks/requests/P5-05-permission-integration.md) | 部分取代 | 契约与联合验收移交 P5-40（#102）；其余已在 #100 交付 |
| [P5-07-ipc-contract](../../tasks/requests/P5-07-ipc-contract.md) | 现行 | P5-07 整卡仍是部分交付 |
| [P5-07-model-and-human-gold](../../tasks/requests/P5-07-model-and-human-gold.md) | 现行 | 同上 |
| [P5-08-result-time-semantics](../../tasks/requests/P5-08-result-time-semantics.md) | 现行 | 被 ADR 0035 §5 与多张卡引为来源；头部「待后继设计」见第 3 节 |
| [P5-08-runtime-publication](../../tasks/requests/P5-08-runtime-publication.md) | 现行 | `core-runtime/src/lib.rs` 与卡引用 |
| [P5-08-stage-inputs-and-law](../../tasks/requests/P5-08-stage-inputs-and-law.md) | 现行 | `core-runtime/src/stages.rs` 引用 |
| [P5-12-resources-and-receipt](../../tasks/requests/P5-12-resources-and-receipt.md) | 部分取代 | 主动函数、单位、中性值、上界（V2-117，文内已注）；IDL 部分移交 P5-42 |
| [P5-22-oneof-support](../../tasks/requests/P5-22-oneof-support.md) | 历史记录（已办结） | #57 已合并 |
| [P5-23-capacity-inputs](../../tasks/requests/P5-23-capacity-inputs.md) | 现行 | 等 P5-24、P5-26、P5-29 等接线 |
| [P5-23-law-state-idl](../../tasks/requests/P5-23-law-state-idl.md) | 历史记录（已办结） | 由 P5-32（ADR 0016，#52）办结，文内已追加 |
| [P5-24-inputs](../../tasks/requests/P5-24-inputs.md) | 现行 | `core-kernel/src/evidence/mod.rs` 引用；文内「历史请求」一节已标明受后续取代限制 |
| [P5-25-dynamics](../../tasks/requests/P5-25-dynamics.md) | 现行（部分办结） | V2-98 的存储已由 P5-36（#108）交付；H 时间投影等仍在等 |
| [P5-26-integration](../../tasks/requests/P5-26-integration.md) | 现行 | 2026-11-01 一节已取代前文状态 |
| [P5-34-core-store-v3](../../tasks/requests/P5-34-core-store-v3.md) | 历史记录（已办结） | 用户授权在 #95 实施；头部「待用户安排」见第 3 节 |
| [P5-34-supervisor-test](../../tasks/requests/P5-34-supervisor-test.md) | **待用户判断** | 根因未定位；Windows 退出确认测试现在是否还失败，本次没有运行 |
| [P5-36-dynamics-roundtrip](../../tasks/requests/P5-36-dynamics-roundtrip.md) | 历史记录（已办结） | #108 已合并；头部「状态：待处理」见第 3 节 |
| [P5-67-d05-design](../../tasks/requests/P5-67-d05-design.md) | 现行 | 记有用户 2026-11-05 的选择，并列出后继卡 P5-74、P5-77 等 |
| [P5-69-law-state-numerics-export](../../tasks/requests/P5-69-law-state-numerics-export.md) | 现行 | 2026-11-11 新登记 |
| [P6-08-desktop-sense-integration](../../tasks/requests/P6-08-desktop-sense-integration.md) | **待用户判断** | scratch 修复已获批准并实施；主入口接线与设置接口是否已由 P6-19 等卡接手，本次没能确认 |
| [R-CHOICE-01-state-selection](../../tasks/requests/R-CHOICE-01-state-selection.md) | 历史记录（研究卡入口） | #113 已合并 |
| [R-CHOICE-02-measurement](../../tasks/requests/R-CHOICE-02-measurement.md) | 历史记录（研究卡入口） | #114 已合并 |
| [R-KAPPA-01](../../tasks/requests/R-KAPPA-01.md) | 历史记录（研究卡入口） | #112 已合并 |
| [R-WECHAT-01](../../tasks/requests/R-WECHAT-01.md) | 现行 | 记有用户 2026-11-01 选定的路线与 E5 后继卡清单 |

## 2. 整体取代的文档与指向它们的链接

用 `git grep` 在全仓库（15,756 个已跟踪文件）里搜文件名。只写编号（如「ADR 0006」）而不写路径的地方，挪走后不会断，下面不列，只给数量。

### 2.1 ADR 0006 → 被 [ADR 0009](../../docs/adr/0009-echo-core-v2-first-principles.md) 取代

| 文件:行 | 形式 | 挪走后要不要改 |
| --- | --- | --- |
| `docs/traceability/baseline.md:33` | Markdown 链接 `../adr/0006-single-root-spec-conflict.md` | **要改**：链接会断。这一句本身也过时（第 3 节） |
| `reports/P0-06/BLOCKED.md:13` | Markdown 链接 `../../docs/adr/0006-…` | 会断；报告允许改写，改不改**待用户判断** |
| `reports/P0-06/REPORT.md:14` | 反引号路径 | 不是链接，不会断；改不改**待用户判断** |
| `reports/P5-01/verify-full.ps1:36`、`reports/P5-01/verify-main-20261015.ps1:36`、`reports/P5-01/verify-numeric.ps1:19` | 历史验证脚本里 `git diff --exit-code origin/main -- … docs/adr/0006-…` 的路径参数 | 不改。它们是固定在旧提交上的验证脚本；路径不存在时 `git diff` 不报错，只是不再检查这个文件 |
| `reports/P5-01/acceptance/checks.json:134`、`reports/P5-01/acceptance/main-d523e0b/checks.json:134`、`reports/P5-01/acceptance/post-main/checks.json:134` | 原始证据里记录的命令 | **不能改**（原始证据不得删改） |
| `reports/P5-25/evidence/environment/source-manifest.json:1895` | 原始证据的文件清单 | **不能改** |

另有只写「ADR 0006」不带路径的：`docs/` 内 15 行（ADR 0007、0009，echo-core README，traceability 的 P0-06、baseline、diff-register、数学核验；不含 0006 自身），`reports/` 内 11 个文件。

### 2.2 v0.6.4-rg-plasticity-spec.md → 被 [v2.0](../../docs/echo-core/echo-core-v2.0-first-principles.md) 第 4、5、7、8、12 节取代（v2.0 第 15 节、ADR 0009）

| 文件:行 | 形式 | 挪走后要不要改 |
| --- | --- | --- |
| `docs/echo-core/README.md:9` | 转换版对照表的一行（反引号文件名） | **要改**：表里的文件会不存在 |
| `docs/traceability/baseline.md:22` | 别名 R 对应的转换版文件名（反引号） | **要改**：`verify-baseline.ps1` 不读这一句，但读者会按它找文件 |
| `reports/P5-25/evidence/environment/source-manifest.json:2033` | 原始证据的文件清单 | **不能改** |

另有 2 处按标题「R/G 可塑机制实现规格」提到它（echo-core README 第 9 行、integrated-implementation 第 23 行），不带路径，`reports/` 内 1 个文件。

### 附：ADR 0018（如果用户改判为整体取代）

`tasks/P5-echo-core.md:1066`（反引号路径，卡面正文，改要按 `card-edit`）；`reports/law-lp1-reference/provenance.json:11`、`reports/P5-36/implementation/pr-audit-20261027T173056Z.json:1147`、`reports/P5-36/main-sync-20261030/main-files.txt:103`、`reports/P5-36/verification/pr-open-pr-changed-filenames.json:769`（原始证据，不能改）。没有 Markdown 链接指向它；只写「ADR 0018」的有 `docs/` 内 12 行（ADR 0019，traceability 的 P5-23、diff-register、law-lp1-reference；不含 0018 自身）、`reports/` 内 13 个文件。

## 3. 状态标注与实际不一致

下列都只是记录，本 PR 一处都没有改。「推断」指依据 `needs-user` PR 已被合并（AGENTS.md 第 6 节：合并即批准）；要不要据此统一改成「已采纳」，**待用户判断**。

### A. ADR 头部仍写提议或草案，但已合并

| ADR | 头部写的 | 实际 |
| --- | --- | --- |
| 0012 | 提议；用户合并即采纳 | #38 合并（推断已采纳） |
| 0014 | 提议；用户合并即采纳 | #41 合并（推断） |
| 0015 | 提议 | #47 合并（推断） |
| 0016 | 提议；用户合并 P5-32 即采纳 | #52 合并（推断） |
| 0017 | 提议 | #56 合并（推断）；N0 决定也已按它执行 |
| 0020 | 提议；PR 保留 needs-user | #65 合并（推断） |
| 0021 | 提议，待用户合并批准 | #95 合并（推断） |
| 0022 | PROPOSED，待交叉审查和用户合并 | #76 合并（推断） |
| 0024 | 实施提案，等待用户与独立审查 | #110 合并（推断） |
| 0026 | PROPOSED，用户合并才采纳 | #82 合并（推断） |
| 0027 | 提议 | #86 合并（推断） |
| 0029 | 「B3 阻塞完整验收，待交叉审查及用户最终合并」 | #108 于 2026-11-01 合并；P5-36 对照第 38 行写明用户已批准存储适配 |
| 0031 | 草案；第 20 行「当前基线为 Qwen3-4B-Instruct-2507」 | #115 合并（推断）；载体已按 ADR 0034 换为 Qwen3.5-4B |
| 0032 | 草案，待交叉审查与用户合并 | #118 合并（推断）；v2.0 与 echo-core README 都已按「已合入 main」引用它 |
| 0033 | 草案，待交叉审查与用户合并 | #124 合并（推断） |
| 0034 | 草案；「模型切换须待 P0-12 实测通过才生效」 | #127 合并；**用户已判定 P0-12 通过、条件切换生效**（P0-12 报告第 2 行）。即用户提到的已知一例 |
| 0035 | 提议（PROPOSED） | #138 合并（推断）；后继卡 P5-69、P5-70 已按它实现 |

### B. ADR 头部与取代关系不一致

1. ADR 0006 头部仍写「PROPOSED — requires user approval」，实际已被 ADR 0009 取代，自身没有任何标注。
2. ADR 0018 头部仍写「提议，待用户决定」，同一文件末尾已追加「2026-10-22 用户选择 A」。另外合并它的 #49 没有 `needs-user` 标签。
3. ADR 0007 第 133 行「C01 remains unresolved; ADR 0006 remains PROPOSED; W09/W10 remain blocked」已过时：C01 已由 ADR 0009 消解，W09/W10 已由 v2.0 成长工作取代（ADR 0010）。

### C. 规格与对照文件

1. v2.0 规格头部「状态：草案……随 PR 合并冻结」：#34 已合并，按它自己的说法和 ADR 0009 已经冻结。
2. `docs/traceability/baseline.md:33`「W00-C01 remains pending user confirmation; ADR 0006 is proposed」：已由 ADR 0009 消解。
3. `docs/traceability/baseline.md:37`「E12 remains 待用户提供」：第 39 行已追加「不可得」，但没有写 ADR 0009 之后已不再需要。
4. `docs/traceability/law-lp1-reference.md` 头部「设计状态：提议」：ADR 0019 已采纳（#61）。
5. `docs/traceability/P5-22.md:3`「契约与配套工具均等待 Claude 交叉审查和用户批准」：#57 已合并。
6. `docs/traceability/P5-34.md:3`「V2-62、V2-63 待用户批准」：#95 已合并。
7. `docs/traceability/P5-36.md` 标题「（准备阶段）」与第 3 行「后继契约未实现」：#108 已合并，文末已追加取代说明，但标题和开头没改。
8. `docs/traceability/P5-09.md` 没有指向取代它部分内容的 `P5-27.md`。
9. `docs/traceability/P5-01.md:3`「Status: partial」：P5-01（#4）已合并，后继契约卡（P5-21、P5-22 等）也已完成。是否仍算部分交付，**待用户判断**。

### D. 请求文件的状态行

1. `tasks/requests/DOC-PRESETS-01.md:3`「待排期，未认领」：#122 已执行并合并。
2. `tasks/requests/DESIGN-ASSIST-EV-01.md:10`「交叉审查与用户合并仍待完成」：#124 已合并。
3. `tasks/requests/DESIGN-LAW-01.md:6`「交叉审查阻塞 1 已响应、待原方复审」：#131 已合并。
4. `tasks/requests/P5-36-dynamics-roundtrip.md:3`「状态：待处理」：#108 已合并，文末已追加。
5. `tasks/requests/P5-34-core-store-v3.md:3`「状态：待用户安排范围与执行卡」：第 29 行已追加取代说明，头部没改。
6. `tasks/requests/P2-05-spring-fold-continuity.md:9`「PR #48 继续保持草稿」：#48 已合并。
7. `tasks/requests/P4-01-to-P0-09-latency-budget.md` 末段「当前 P4-01 仍保持草稿与 needs-user」：#44 已合并。
8. `tasks/requests/P5-08-result-time-semantics.md:3`「待后继设计」：设计已由 ADR 0035 §5（D11-A）完成，实现仍在后继卡。改不改这一行，**待用户判断**。

### E. 缺少指向后来决定的文内指引

这些不是状态字段写错，而是正文还写着旧说法，读者看不到它已被取代。

1. CONTRACTS.md 第 3.3、4.1、4.2、5.3、6、7、8 节（见 1.2 表），正文没有任何指引。
2. ADR 0004 决策第 5 项（每日 30 次）没有指向 ADR 0033。
3. ADR 0005 §4 里程碑 N0–N4 与日期，没有指向 E1–E5 重排。
4. ADR 0011 决定 2「与 u* 共用账号」，没有指向微信独立账号的决定（只记在 R-WECHAT-01 请求里）。
5. ADR 0012 决定 1 的载体型号，没有指向 ADR 0034。
6. ADR 0026 §6.3 的 κ^n 与 τ，没有指向 ADR 0035。
7. ADR 0016 没有指向 ADR 0021 的 V3 后继消息。
8. continuous-cognition.md 中 COG-09 至 COG-11、COG-A12，没有指向 ADR 0032、0035。
9. `docs/learn/README.md:27` 让读者从 echo-core README 第 5 节开始读，该节已于 2026-11-11 改为分级读法。
10. SETUP.md runner 第 1 条「确认仓库保持私有」：仓库已于 2026-10-26 公开。

### F. 旧里程碑名

1. `AGENTS.md:104`「birth 只由用户在 N3 执行」：现为 E4（诞生）。
2. `docs/ARCHITECTURE.md:227`「排练版（N2）」：现为 E2。

### G. 执行者分工（待用户判断）

AGENTS.md 第 2 节目录表、第 10 节，以及 `tasks/cards.json` 仍把 `core-policy`、`core-kernel`、`experiment-world` 的实现卡建议给 Claude（例如尚未完成的 P5-30、P5-71、P5-74、P5-76）。开发手记 2026-10-26 一行写的是「Claude 转为协调者，实现交给 Codex」。两边哪个是现行分工，请用户定。

## 4. 挪动之前要注意的事

为下一步准备，本次都没有动：

1. **`docs/echo-core/original/` 永远不挪**（AGENTS.md 第 2 节，`docs-guard.yml` 检查）。
2. **「历史保留」的措辞：** v2.0 头部写「`docs/echo-core/` 中已有的文件不改，作为历史保留」，ADR 0009 写「旧规格的原文与转换版作为历史保留」。把 R 转换版挪进 `docs/archive/` 算不算违背这句，**待用户判断**。
3. **看板的受保护路径：** `tools/status/build_status.py` 第 64–74 行的 `GUARDED` 含 `docs/adr/` 和 `docs/echo-core/`，挪动这两个目录下的文件，看板会按需交叉审查处理。新目录 `docs/archive/` 不在其中；要不要加进去，**待用户判断**。
4. **原始证据不能改：** 第 2 节标「不能改」的 JSON 和日志会留下失效路径，这是预期的。
5. **报告里的链接：** AGENTS.md 允许改写报告，但 `reports/` 下的旧链接要不要一并更新，**待用户判断**。
6. **代码里的引用：** `core-runtime/src/lib.rs`、`core-runtime/src/stages.rs`、`core-kernel/src/evidence/mod.rs`、`executors/codex-bridge/src/base/README.md`、`tools/asset-pipeline/tail/README.md` 引用了 `tasks/requests/` 下的文件；看板的 `tools/status/delivery.py` 会从报告里解析指向 `tasks/requests/` 的链接。本清单没有建议挪任何请求文件；以后要挪，这些都得同步改。
7. **开着的 PR：** #155 新增 `docs/adr/0030-…`、`docs/traceability/P5-37.md` 并追加差异登记；#111、#45 追加差异登记并新增请求；#144、#116 新增请求。它们都不碰本清单里整体取代的两份文件。
8. **挪动 PR 本身：** 改了 `docs/` 里已有的文字（加「已被 xx 取代」、改链接），按 AGENTS.md 第 2 节需要 `needs-user`。

## 检查

| 检查 | 结果 |
| --- | --- |
| `git diff --check origin/main` | 通过，无输出 |
| 本报告中的 Markdown 相对链接逐个确认文件存在 | 通过：130 个链接，0 个缺失（Python 脚本按报告所在目录解析） |
| 改动范围：`git diff --name-only origin/main` | 只有 `reports/DOC-ARCHIVE-INVENTORY/REPORT.md` |
| PR 合并状态、标签与合并者 | 用 `gh api repos/nanake-lab/nemu/pulls` 与逐个 `pulls/<n>` 读取 |
| 指向整体取代文件的引用 | `git grep -n` 按文件名全仓库搜索（第 2 节） |

没有运行测试：轻档，只改 `reports/`。

## 待用户判断（汇总）

- [ ] （不阻塞）R 转换版 `v0.6.4-rg-plasticity-spec.md` 挪不挪，与「历史保留」的措辞是否冲突（第 1.4 节、第 4 节第 2 条）。
- [ ] （不阻塞）ADR 0018 算部分取代还是整体取代（第 1.3 节）。
- [ ] （不阻塞）第 3 节 A 组：是否把已合并的 ADR 头部统一改为「已采纳（PR #n，用户合并）」，格式可参考 ADR 0019。
- [ ] （不阻塞）P2-05 三份弹簧骨请求在购买外观模型之后还要不要（第 1.7 节）。
- [ ] （不阻塞）P5-03、P5-34-supervisor-test、P6-08 三份请求是否已办结（第 1.7 节）。
- [ ] （不阻塞）S §27「分工 排期」与 SETUP「CI 额度」是否已被整体取代（第 1.2、1.4 节）。
- [ ] （不阻塞）ADR 0001「语音」一行算不算被 ADR 0027 取代。
- [ ] （不阻塞）P5-01 对照是否仍算部分交付；P5-08 时间语义请求的状态行改不改。
- [ ] （不阻塞）执行者分工以 AGENTS.md 为准还是以开发手记为准（第 3 节 G 组）。
- [ ] （不阻塞）`docs/archive/` 要不要加进看板的受保护路径；报告里的旧链接要不要一并改（第 4 节）。

## 后续建议（不在本任务范围内）

- 差异登记 533 行，条目状态靠后续追加行改变，「待用户确认」出现 58 次。本次没有逐条核对哪些已经决定，适合另开一张卡专门清点。
- `tasks/README.md` 与各阶段任务卡（`tasks/P*.md`）不在本次范围内，没有清点。
- `reports/` 下有些报告的「当前状态」也已过时（例如 P5-03 报告仍写「PR #64 保持草稿」）。报告按惯例不挪，但要不要更新状态段，可在归档那一步一起决定。
