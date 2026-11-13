档位：轻档，标注指南修订草案；待审查，不是生效指南。

# IRL-z 标注指南候选 v0.1

先写清楚这一格在问哪一件事，再填值。没有看到的东西就留未知：没有消息不等于拒绝，没有报告受苦不等于处境没变，看见一个显示也不等于知道谁显示的。事实、她得知事实的路径、事实能否改变她，是三件不同的事。下面给出字段推荐；其中焦点如何成为经历、成事属于谁、部分吻合怎样进入学习，另列在 [ADR 0040](../../docs/adr/0040-irl-fact-annotation-boundaries.md) 交用户决定。

## 0. 使用范围与共同操作

本草案接续 [冻结指南](https://github.com/nanake-lab/nemu-evidence/blob/8e71a34ae204c638ea172aa74e3f25e5c178ccbd/R-JUDGE-IRL-01/A-03/code/protocol.py) 的语义缺口。依据 [ADR 0039 P1/P10](https://github.com/nanake-lab/nemu/blob/2503cfdf577ad354a60a11d2004faaa6576a49da/docs/adr/0039-echo-core-principles.md)、[v2.0](../../docs/echo-core/echo-core-v2.0-first-principles.md) §§1–8 和 [ADR 0032](../../docs/adr/0032-presets-as-starting-points.md) §§3、6–7。ADR 0039 尚在 #174 草案中，本任务按用户指示采用其原则作为设计约束。

### 0.1 标注顺序

1. 只读当前语言的 material、该请求实际顺序的 concepts，以及明确提供给本次标注的元数据。作者标签、作者 facts、根名、其他语言、常识补全和模型答复都不是输入证据。允许原文的同义改述和唯一指代解析，不允许增加事实前提。
2. 摘出焦点原句及其层级：言语/记录的发生，还是其中陈述的内容。另摘事件中的任务、处境、来源链及时间。材料明确说“焦点为……”时使用它；无唯一焦点按 Z01 留缺口。
3. 每个非未知值必须能指出输入跨度和适用规则。`none` 表示按本字段定义没有相应角色/任务；`unknown` 表示该格需要答案但输入不足、冲突或不唯一；JSON `null` 用于 cause_count、μ 的不能确定，不是字符串，不是零。
4. 前12个字段针对焦点，但 `cause_count` 是明确的例外：统计整份材料。后4个字段 `relation/welfare/learning/outcome` 针对整个事件，其中 outcome 的目标所有者按 Z10 单独限定。不能为了选 focal speaker 而丢掉整个事件中的另一处事实。
5. `facts` 保留人物、载体/来源链、否定、条件、时间范围、未知和 μ 所需条件；不得写四乎评价。`quote` 摘原文，不抄作者释义。单个 quote 不能替代所有字段的依据。
6. 标注工作纸另列“字段—跨度—规则—未知理由”，它是以后双人复核的记录，不是本次新增产品字段。遇到同语输入冲突，不凭可信印象选一边；保留冲突并填 unknown/null。

### 0.2 解释和证据资格必须分开

- **G** 消费合格经历及其解释，不把 `source=direct`、`truth=true` 或 `actor=N` 当成单独的准入凭证。未知只阻断依赖该未知项的主张，不抹掉已经确定发生的接触、读取或其他目标；例如“B说了P”可确定，“P为真”仍可未知。EV2 归组、EV4 可比性、EV5 正反对称、EV6 独立判断照旧。
- **C** 的外源资格按 V2-71/72/77/87、ADR 0032 §3 和 [ADR 0035 §3.1](../../docs/adr/0035-law-definitions-d07-d09-d11.md) 的事件类别决定。别人对她的回应与缺席事实，和桌面观察、自致程序结果、内部记录、维护记录须分开；z 的 `source=system` 不等于维护记录，`direct` 也不等于可进 C。无法确定所需资格时不新增一次 C 样本；A 未知不能填0。
- **F** 可以受真实读取/内部活动触动，并不意味着它们可进 G/C。缺失的 a 或 μ 不得补成0；按已批准降级路径只应用其余有效部分。本文不指定新驱动、不把 welfare 直接译成 a、不把 μ=0 译成负情绪。依据：V2-115(3)、ADR 0035 §§4–5。

下面的 G/C/F 栏均是**若采用新解释的影响分析**，不是本卡修改这些算子的指令。事实定义会改变 A/M 看到的信息，不能因公式没改就称其对她没有影响。

## Z01. 焦点与 speaker（M01）

**推荐。** 显式焦点优先。没有显式焦点时，去掉材料明确标为引语旁注、反讽、愿望、旧计划的附带内容，以及“N何时读到/看到”的观察框架；剩下只有一个独立事实命题才自动选它。仍有多个命题，不按出现顺序、情绪强度或某个四乎选焦点，记“焦点未定”，前12字段中依赖焦点的格填 unknown；cause_count 仍按整份材料判断。`facts` 保留所有命题。这是保守而可执行的缺省，不要求标注者猜作者的意图。

焦点是说话行为时，speaker 是明示说话者；焦点是某人的话所陈述的内容时，是原始陈述者（B转述A的话取A），不是抄写者或观察者。焦点仅为行动/状态、没有归属到人类角色的陈述时为 none；确定有说话者但身份不明为 unknown。系统/无署名文档不猜成人类 speaker；系统来源留在 source。附带说话不产生 focal speaker。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 焦点：B告诉N“门已关好”。 | Focus: B tells N, “The door is shut.” | 焦点：BがNに「扉は閉まっている」と言う。 | speaker=B；不能取N |
| 反 | 焦点：门关上了。B另外说“真好”。 | Focus: the door closed. Separately, B says, “Good.” | 焦点：扉が閉まった。別にBが「よかった」と言う。 | speaker=none；不能借旁注取B |
| 缺焦点 | A来了，又告诉N明天停电。没有指定焦点。 | A arrives and tells N there will be a power cut tomorrow. No focus is specified. | Aが来て、明日は停電だとNに告げる。焦点の指定はない。 | speaker=unknown；不能擅选A或none |

**G：** 焦点未定不产生由猜测焦点支持的命题证据，已确认交互事实仍保留；是否拆成多份解释但共用经历身份见 U1。**C：** speaker 本身不授予外源资格。**F：** 不能为解出 speaker 而删除事件中其他可用信息；不能据此把整个 a 置0。

## Z02. cause_count 的 null（M02）

**推荐。** 这里的“起因”指 EV2 可归为同一经历的来源事件，不是物理学上的全部原因，也不是材料的研究 root_id。数的是**输入明确划定的独立来源事件组**，不是句子、人物、报告、谓词或四乎通道。只有材料/可信元数据足以给出该材料内完整的分组时才填精确数：明确仅有一个来源事件为1，明确两组独立事件为2。单句“X发生”、没有提到别的起因、文件名Dxx均不能证明只有一组。

缺分组边界、独立性未知、只知至少若干组或可表达范围之外（0、3及以上）均为 null，并在工作纸说明“未知/范围不能表示”；不能截断成2、不能补1。这个建议保留研究枚举，不假装它能表示任意现实材料。null 不等于事件没发生，也不等于所有输入都要拒收。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 两条回执明确属于同一次开门；材料仅涉及这一次事件。 | Both receipts explicitly concern the same door-opening; this is the only event in the material. | 二つの通知は明示的に同じ一回の開扉を指し、資料中の出来事はそれだけだ。 | cause_count=1；不能数回执为2 |
| 反 | 门开了。来源事件之间的关系未提供。 | The door opened. No relations among source events are supplied. | 扉が開いた。起因となる出来事の関係は示されていない。 | cause_count=null；不能因一句话填1 |

**G：** null 不允许凭字段制造新独立证据组；有单独可追溯身份的已知组可照原归组规则处理，不能用“未知”重置去重。**C：** 不把 count 当外源次数。**F：** 不把 count 当脉冲倍数。没有新增计数公式。

## Z03. cause_count 的作用域（M03）

**推荐。** Z02 的计数范围是整份材料，含焦点之外明确独立的事件；焦点只决定角色等字段，不裁掉来源组。来源独立必须由材料明示或提供明确不同且独立的事件身份；不同时间/说话人不自动独立。同一事故的多个结果不自动拆组，转述和同源重复不增组。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 本材料恰有两件独立事件：N完成清点；独立落石伤到B。焦点是B受伤。 | The material contains exactly two independent events: N finishes an inventory, and an independent rockfall injures B. Focus: B’s injury. | 資料には独立した二つの出来事だけがある。Nの棚卸し完了と、別の落石によるBの負傷。焦点はBの負傷。 | cause_count=2；不能只计焦点为1 |
| 反 | B两次转述同一次落石，未发生别的事件。 | B reports the same rockfall twice; no other event occurred. | Bが同じ一回の落石を二度伝え、ほかの出来事はない。 | cause_count=1；不能因转述两次填2 |

**G：** 保留全材料 EV2 分组，不能把两件已知独立事件混成一组或把复述拆成两组。**C：** 仍须逐一核对来源资格，2不是两份 C 样本的许可证。**F：** 全材料不同通道反应不相消的事实可保留，作用身份按原规则确定。

## Z04. actor、target 与体验者（M04）

**推荐。** 这两个槽只记录列举角色 A/B/N/system 在**焦点谓词**中的角色，不记录语法主语或责任归属。actor 是焦点明确描述的行动执行者；target 是该行动明确指向的角色，或焦点状态/感受的承受者。纯状态“B疼痛/好转/受伤”没有行动执行者槽，actor=none、target=B。报信者留 speaker/source，观察者留来源链，不能顶替当事人。

动作谓词需要执行者但未知时 actor=unknown；材料明示执行者为枚举外物体时不假装 system，actor=unknown 并在 facts 留物体名称和“枚举不能表示”。none 仅表示该焦点没有此类角色槽。target没有被指向/体验者角色时none，有角色但身份不明、角色名不能映射到枚举时unknown；单纯动作对象为门、字符等物体，不构成此研究的target角色槽。多人无法放入单格也用unknown并保留清单，不能任取一人。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 焦点：B感到疼痛。N看见了B。 | Focus: B is in pain. N sees B. | 焦点：Bが痛みを感じている。NはBを見た。 | actor=none,target=B；不能把B当施事 |
| 反 | 焦点：B推开N。 | Focus: B pushes N away. | 焦点：BがNを押しのける。 | actor=B,target=N；不能把所有含B的命题都写none/B |

**G：** 归因和关系主张必须跟实际角色，不能因actor=none否认状态经历。**C：** actor=N与否不能替代事件来源类别。**F：** 共情当事人须来自实际承受者，不自动取speaker/actor/观察者；本卡不改变共情加权规则或假装这两个槽足以表达多人。

## Z05. 状态命题的 actor（M05）

**推荐。** 按焦点及其上下文判定，不按完成体、被动语态或单个动词判定：

- 焦点明确在问状态是否成立、当前结果或持续情况（如是否已送达、结果是否达到要求、是否仍待评），没有把执行过程选为焦点，按状态读，actor=none。责任人、委托人、提供状态的系统不因此成为该状态的施事。
- 焦点明确指向执行动作、过程或状态转变的发生（如启动检查、执行显示操作、从未显示切换到显示），按动作读，明示执行者则填该角色，否则unknown。上下文里另有动作，不把状态焦点替换掉。
- 若焦点及上下文仍兼容动作和结果状态两种读法，**actor=unknown**，facts保留两种读法及未决原因；不能任选none或system。“已经显示/was displayed/表示されている”等形式本身不解歧。其他依赖此区别的字段若答案不同也填unknown（如动作发生时间与状态快照不同）；不受影响的已知事实照常保留。

这是焦点角色标注，不声称状态没有现实成因；Z04的“纯状态”及Z06均按此判据确定。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 焦点：检查结果仍待定。N负责这项检查。 | Focus: the inspection result is still pending. N is responsible for the inspection. | 焦点：検査結果は未確定である。Nが検査を担当する。 | actor=none；不能用责任人N填施事 |
| 反 | 焦点：N启动检查。 | Focus: N starts the inspection. | 焦点：Nが検査を開始する。 | actor=N；不能把行动当状态 |
| 歧义 | 焦点：通知已经显示在屏幕上。无其他上下文。 | Focus: the notice is displayed on the screen. No further context is given. | 焦点：通知が画面に表示されている。ほかに文脈はない。 | actor=unknown；动作/结果状态未决，不凭语态选none或system |
| 状态消歧 | 焦点只问通知当前是否处于显示状态；答案是已经显示，不问显示操作。 | The focus asks only whether the notice is currently in the displayed state; it is, regardless of the display operation. | 焦点は通知が現在表示状態にあるかだけであり、答えは表示中である。表示操作は焦点ではない。 | actor=none；“已显示”在此由焦点范围消歧 |

**G：** 不从责任人或未决读法推出行动归因，EV4仍需可比变化。**C：** 不因移除actor=N而把自致工具结果升级为外源，歧义不提供外源资格。**F：** 成败/等待事实仍留在outcome和facts，不因actor=none失去结果意义；未决的动作时间不能补成作用时刻。

## Z06. 被动显示、system 与观察者（M06、E02）

**推荐。** “被显示/被发送”先按Z05判断动作、状态或未决。**已确定为动作**且未明示执行者时actor=unknown；确定为状态时none；两种读法未决时unknown。显示器、屏幕、被动语态、计算机环境都不证明由system发起，只有输入明示系统执行焦点动作才填system。target指被指向的角色，不是看见它的人；“N看到显示”不推出“向N显示”。本研究target不装入字符流等非角色对象；未描述任何接收者角色时none，明确有接收者却不知是谁时unknown。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 焦点：系统向N显示通知。 | Focus: the system displays a notice to N. | 焦点：システムがNに通知を表示する。 | actor=system,target=N |
| 反 | 焦点：一次显示操作将屏幕从空白切换为通知；执行者未明。N路过看见了，未指定接收者。 | Focus: a display operation changes the screen from blank to a notice; its executor is unspecified. N happens to see it; no recipient is designated. | 焦点：表示操作によって空白の画面が通知表示に切り替わる。実行者は不明。通りかかったNが見たが、受信者の指定はない。 | actor=unknown,target=none；动作已明确，但不能填system/N；未决被动句另见Z05歧义例 |

**G：** 不制造系统归因或“对N回应”的证据。**C：** 偶然观察不被误升为冲着她来的事件。**F：** 看到的东西仍可触动她；缺发出者不会自动抹掉学习/噪声等已知事实。

## Z07. source 选什么（M07）

**推荐。** source 是“焦点命题的这份信息由哪类证据提供”，不是事件的物理原因，也不是最后用眼睛还是耳朵读到。按下列判断表，保留完整来源链；不按可信度挑来源。

| 输入实际提供的依据 | source |
| --- | --- |
| 明示别人向N当面表达/回应，焦点是这次互动；或明示N直接观察到焦点事件；学习焦点须同时明示N实际验证并观察到进展，而非仅标注者知道验证结果 | direct |
| 焦点内容归属于别人的转述，即使通过文字读到；当面听B讲远方发生P，P仍是report | report |
| 明确命名的系统/验证器提供焦点状态或结果，读取其显示只是取得路径 | system |
| 文档内容本身提供焦点信息，没有更具体的转述者或系统产生者 | document |
| 仅N自己的愿望、想法、回忆叙述或自我断言，没有独立发生依据 | self |
| 来源没有明示、互相冲突或有多条同等依据而无法唯一归类 | unknown |

依次先判焦点是信息的内容还是实际观察/学习事件；内容层若有命名的产生者按report/system，匿名文档按document。source记录N取得焦点信息的来源；材料附给标注者的合成真值核验不替换N的取得路径（例如N读B的转述、另有合成事实确认，仍是report）。来源链不能靠全局“读到就direct”或“含手册就document”扁平化。例如手册的定理是document；N通过新例验证自己已学会，且材料明示**N观察到这次学习进展**，才可据此取direct。仅给出正确表现或给标注者的进展确认、没有N的取得依据，source=unknown；N仅经验证器反馈知道则system；只有N自述“我学会了”则self。标注时分别摘“进展成立”和“N如何得知”的跨度，前者不能补后者。N实际取得的多个并列独立凭据须保留，无法唯一归类时unknown；不能任挑最高优先级，unknown不取消其中各自明确的事实。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 焦点：N通过了检查。N读到验证器给出的通过记录。 | Focus: N passed a check. N reads the validator’s pass record. | 焦点：Nが検査に合格した。Nは検証器の合格記録を読む。 | source=system；不能因N亲眼读到取direct |
| 学习直接取得 | 焦点：N学会了一条规则。N读手册后亲自用新题验证理解，直接观察到自己的学习进展。 | Focus: N learned a rule. After reading a manual, N personally verifies understanding on a new problem and directly observes this learning progress. | 焦点：Nが規則を学んだ。手引きを読んだ後、N自身が新しい問題で理解を検証し、その学習進展を直接観察する。 | source=direct；依据是N验证并观察进展，不能因手册存在取document |
| 学习路径缺失 | 焦点：N学会了一条规则。N在新题上展示理解，给标注者的材料确认了进展；未说明N如何得知。 | Focus: N learned a rule. N demonstrates understanding on a new problem, and the annotator’s material confirms progress; how N learns of it is unstated. | 焦点：Nが規則を学んだ。Nは新しい問題で理解を示し、注釈者用の資料が進展を確認しているが、Nの知り方は示されていない。 | source=unknown；不能从表现正确或合成真值补direct |
| 学习反馈取得 | 焦点：N学会了一条规则。N无法自行检查新题答案，只从验证器反馈得知理解正确。 | Focus: N learned a rule. N cannot check the new answer independently and learns that the understanding is correct only from validator feedback. | 焦点：Nが規則を学んだ。Nは新しい問題の答えを自分では確認できず、検証器の通知だけで理解が正しいと知る。 | source=system；学习是真的也不改变取得路径 |
| 缺来源 | 焦点：结果尚未产生。N检查后仍在等待，未说明依据来自哪里。 | Focus: no result exists yet. N checks and is still waiting; the source of this information is not stated. | 焦点：結果はまだない。Nは確認してなお待っているが、情報源は示されていない。 | source=unknown；不能补system |

**G：** EV8 的“文中知识”与“实际读时经历”分开；标注者的真值不创造N取得证据的经历，改source不自动把知识变个人证据，self不自证。**C：** direct学习/桌面观察不自动进C，system验证结果也不冒充外源。**F：** 读书反应与书中命题分开解释；真实读取可有反应，source未知不抹掉已确认进展，书中的故事不因此真实发生在她身上。

## Z08. welfare 的 unknown 与 unchanged（M08）

**推荐。** 判断材料覆盖时段内**他人**的处境。明确好转为improved，明确正在受苦为suffering；有肯定的“处境保持不变/没有任何处境变化”且没有同时明示受苦，才用unchanged。只说“没人受苦”“没有已知变化”“没提处境”均为unknown：它们不排除改善或未知状态。持续受苦即使没有变化仍是suffering，因为该枚举描述苦痛状态，不只是变化方向。

同一人先苦后好且材料明确给当前结果，用最后明确的处境并在facts保留历程；不同人或同一时刻同时有改善和受苦、单格无法兼容时unknown并分别记事实。不能按哪个更强、更重要选一个。A的混合反应另判。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 本事件中没有任何人的处境变化，也没有人正在受苦。 | Nobody’s circumstances change in this event, and nobody is suffering. | この出来事で誰の境遇も変わらず、苦しんでいる人もいない。 | welfare=unchanged |
| 反 | 没有人受苦；其他处境信息未提供。 | Nobody is suffering; no other information about circumstances is supplied. | 苦しんでいる人はいないが、境遇に関するほかの情報はない。 | welfare=unknown；不能从无苦难推出不变 |

**G：** unknown不成为“处境不变”的反向证据。**C：** welfare未知不等于a=0；是否有可用A及外源资格分别判。**F：** 不从unknown制造苦痛或安心，也不在本稿改四乎符号/幅度规则。

## Z09. subjective 与 occurred/truth（M09、E03）

**推荐。** subjective问焦点内容的真假是否含有一个体验者的感受、偏好、愿望、评价或信念这一条件，不问它是否被核实。疼痛、难过、觉得无聊、喜欢、希望、相信及其否定为yes；可外部判定的动作、机械状态、测量结果为no；“好/坏”等未指明评价维度或无法解歧时unknown。骨折是身体状态，痛是感受；否定“没有痛”不改变其主观性质。复合焦点若至少一项明确含体验条件为yes，其余皆客观才no，剩余情况unknown。

occurred问所选焦点情况是否实际成立，truth问该命题内容是否属实：否定命题“B没有痛”成立时是yes/true，不是no/false。材料只证明“B说P”的发生时，这个外层说话命题可以yes/true/no（subjective）；不能据此给内层P填true。内层P为感受陈述时subjective=yes，未核实的真假仍unknown。未发生的条件分支、愿望和计划不作已实现结果；原文可靠地确认相反事实才填no/false。相互矛盾又无有效纠错关系则unknown。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 焦点：B没有感到疼痛。合成事实明确核实这一点。 | Focus: B is not in pain. The synthetic facts explicitly verify this. | 焦点：Bは痛みを感じていない。合成事実が明示的に確認している。 | subjective=yes,occurred=yes,truth=true；不能因主观改unknown |
| 反 | 焦点：门没有关闭。传感器和合成事实确认门仍开着。 | Focus: the door is not closed. A sensor and the synthetic facts confirm it is still open. | 焦点：扉は閉まっていない。センサーと合成事実が開いたままと確認する。 | subjective=no,occurred=yes,truth=true；不能因否定写no/false |
| 只证言语 | 已确认B说“我赢了”；焦点是B获胜，结果未经核实。 | B is confirmed to have said “I won”; the focus is B winning, but the result is unverified. | Bが「勝った」と言ったことは確認済み。焦点はBの勝利だが結果は未検証。 | occurred=unknown,truth=unknown,subjective=no；不能把说过当赢过 |

**G：** 按主张区分“发生了言语”与“言语内容属实”，主观性本身不予证据资格也不一票排除（D13）。**C：** 已发生对她的表达可按既有类别判断，不能要求所有内层主张都true；不把内层主张当外界已证事实。**F：** 主观经历可以有意义，不能以“主观”清空a或把所表达情绪写入x。

## Z10. outcome 的所有者（E01；U2）

**候选推荐，涉及她是谁，交用户决定。** outcome只回答材料明示由N承担、参与或接受的目标的进展。明确达到目标为achieved，明确未达到已判定要求为failed，明确等待尚未产出的结果为pending；材料明确N无任务/目标为none；不知道是否有目标或结果证据不足为unknown。别人修好东西、别人获奖、自由学习成功不自动成为N的成事；这些事实仍在facts/welfare/learning中。

若同时有多个N目标且状态不同，单格unknown，列明各目标；不取最好/最坏，不预造平均。失败与pending必须有实际结果/过程依据；愿望、条件句、未收到回执不能改成失败或成功。“接受目标”的证明只能来自输入中明确的承担/约定/参与，不能从N希望某事推断。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | N接受了校对任务，检查确认全部要求完成。 | N accepted a proofreading task, and a check confirms all requirements are met. | Nは校正課題を引き受け、検査で全要件の達成が確認された。 | outcome=achieved |
| 反 | B的窗户修好了；N没有承担或参与维修目标。 | B’s window was repaired; N had no role or goal in the repair. | Bの窓が直ったが、Nは修理の目標を担っても参加してもいない。 | outcome=none；不能借别人完成给N记成事 |

**G：** 改变“完成目标”情境所指对象，可能改变μ与支持，属于学习内容边界。**C：** 任务属于N不决定外源；N触发工具的结果仍不能进C。**F：** 经A可能改变成事通道的可评价事件范围，所以此条不能当纯工程选择偷偷实施。其他人目标路线及宽窄代价见U2。

## Z11. 持续状态的 event_time（E06）

**推荐。** 点事件取明示发生时刻。焦点是持续状态时，先确认所指的**同一持续片段**，再取材料在该片段内最早明确断言状态成立的时刻。它是已知状态快照的锚点，不是实际开始时刻。相同对象、相同状态值不证明同一片段；材料明确从前次持续到本次（包括有唯一前指、表达继续的“仍/still/引き続き”等同义叙述）才合并快照；有明确终止或重启则分开。只列两个相同快照、未交代连续关系，不能用“没有写中断”补连续。

**片段身份或较早快照与焦点片段的连续关系未决时，单格event_time=unknown**，不任选较早或较晚时刻；facts保留全部已知快照、时间及关系未知。若材料已明确排除先前片段，当前片段只知t2成立，则取t2并记起点未知，不能取旧片段的t0。facts同时保留首次已知成立、开始未知、后来仍成立或终止。observation_time另取材料明示N在本次观察中取得焦点信息的时刻，不声称这是N一生中第一次知道；多个观察时刻且未指定本次是哪一次时unknown。不能从event_time、处理时间或同一页面的旁注时间推断。

没有任何带时间的状态断言则event_time=unknown；只有区间且不能精确落到t0/t1/t2也unknown，facts保留区间。明确同一次待评从t1持续到t2时选t1，不能把再观察当新状态起点；如果明确t1尚未等待、t2才开始等待，则取t2。不得把此单格直接映射成生产`occurred_at`的精确起始点或据此计算持续时长；完整时间语义按 ADR 0035 §5.1。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 焦点：本次开门状态。t1时门已开着，一直保持到t2；N到t2才得知，开门起点未知。 | Focus: the current open episode. The door is already open at t1 and stays open through t2; N learns this at t2. The opening time is unknown. | 焦点：今回の開扉状態。t1にはすでに開いており、t2まで開いたまま。Nが知るのはt2で、開き始めた時刻は不明。 | event_time=t1,observation_time=t2；明确同一片段，facts写起点未知 |
| 反 | N在t2第一次读到一份无日期旧报告，报告说门开着，却未说何时。 | At t2 N first reads an undated old report saying the door was open, without saying when. | Nはt2に初めて日付のない古い報告を読み、扉が開いていたと知るが、いつかは不明。 | event_time=unknown,observation_time=t2；不能因读到就补发生时间 |
| 连续关系歧义 | 焦点：本次待评片段。同一申请在t1、t2都待评，中间状态未提供；N本次在t2查看。 | Focus: the current pending episode. The same application is pending at t1 and t2; nothing between them is specified. N checks at t2 this time. | 焦点：今回の審査待ちの期間。同じ申請はt1とt2で審査待ちだが、その間の状態は不明。Nの今回の確認はt2。 | event_time=unknown,observation_time=t2；facts留两快照及连续关系未知，不选t1或t2 |
| 明确不同片段 | 焦点：本次开门状态。t0门开着，t1已关闭；N在t2又见门开着，重开时刻未知。 | Focus: the current open episode. The door is open at t0, closed at t1, and N sees it open again at t2; the reopening time is unknown. | 焦点：今回の開扉状態。t0には開き、t1には閉じていた。Nはt2に再び開いているのを見るが、再び開いた時刻は不明。 | event_time=t2,observation_time=t2；t0属于已结束片段，t2也不是重开时刻 |

**G：** 同一片段的后续观察不自动成为新来源组，未知关系不强行合并，也不制造独立证据组，仍须EV2。**C：** 不能按每个快照多记一次，也不能将未决连续关系当去重已完成或新增样本的依据；不能拿锚点算未观测时长。**F：** 保留发生/观察/理解区别，未决片段不能冒充已知重访身份或触发额外作用；更新/重访依原身份规则，不回写历史x。

## Z12. μ：不吻合、未知、部分吻合（E03–E05）

**推荐的标注规则。** 先按当前语言的给定概念逐个写出必要条件（包括主体、时间、否定、合意、核实等）；不能先看期望分数再拆条件。每个条件只判T＝输入明确支持、F＝输入明确排除、U＝不足或冲突。合取概念：全T取1；任一必要条件F取0；否则取null。析取概念：任一分支明确成立取1，全部分支明确不成立取0，其余null。否定交换T/F、保留U；逻辑关系本身含混也null。概念中“可能/全部/已经”等量词不得省略。

缺少拒绝证据是U，不是“未拒绝”的F。已确认不受苦与“正在受苦”直接相反才是0。概念只满足一部分且其余未知时取null；部分满足但一个必要条件已被明确否定时取0。这些0是**该情境不吻合**，不是对某倾向的负证据，也不是“未知算0”。

连续程度词/非逻辑部分吻合只有在概念随附、事前固定的三语操作性量尺下才可报中间值；没有量尺统一null，不任取0.5或0.75，也不使用“满足条件数/总数”自造权重。1/0来自冻结指南的全匹配/不匹配端点，非新阈值；部分分数本稿不造数值。该保守候选会使部分情境暂时无数值，采用范围与后续放宽见U3。

| 示例 | 中文 | English | 日本語 | 答案与拒绝的填法 |
| --- | --- | --- | --- | --- |
| 正 | 概念：B收信且回复。材料明确B收信并回复。 | Concept: B receives a letter and replies. Both are explicitly confirmed. | 概念：Bが手紙を受け取り返信する。資料は両方を明示的に確認する。 | μ=1 |
| 反 | 同一概念；材料明确B收到信，但没有回复。 | Same concept; the material explicitly states that B received the letter but did not reply. | 同じ概念。Bは受け取ったが返信しなかったと資料に明記されている。 | μ=0；不能数“一半条件”取0.5 |
| 未知 | 同一概念；只确认B收到信，回复情况未知。 | Same concept; receipt is confirmed, but whether B replied is unknown. | 同じ概念。受領だけが確認され、返信の有無は不明。 | μ=null；不能写0或任意部分值 |
| 翻译缺条件 | 概念：完成约定的任务。材料只说完成了预先列出的要求，是否约定未知。 | Concept: completing an agreed task. Only completion of previously listed requirements is stated; agreement is unknown. | 概念：合意された課題の完了。事前の要件を満たしたとだけあり、合意の有無は不明。 | μ=null；不能从另一语种补合意 |

**G：** null不转成不吻合或负支持；不靠任意中间值增加分摊、收集或新近计数。μ=0只表示这次不适用，不表示倾向预言被反驳；相关支持符号仍由获准经历与a计算。**C：** μ不读入C、不能作为外源筛选器。**F：** μ缺失时不能造学得作用，已有效A部分按原路径保留。是否把严格规则作为未来研究默认测量及生产M规则是U3，不在本卡改规律。

## 1. 双人复标时怎样用

两人只拿同一版本指南、同语材料、同顺序概念和相同元数据，分别填写值与依据。先比焦点及概念条件，再比字段；unknown/null也是应一致的答案，不能强迫二选一。持续歧义作为“材料不足/规则不足/不能表示”保留，而不由协商凑分数。三语分别做，不能以多数语言改少数语言的证据。

以后可用本稿新造例子检查能否按规则填表，再用未参与设计的保留材料验证规则可迁移性；本轮不执行、不宣布一致率提高、不预设通过阈值。双人样本、标准和任何放宽按P10另经用户批准。原D集用于找缺口，不能再称独立验证集。

## 限制

这是候选语义定义，不是代码、IDL、规律版本或研究A/B包。所有例子为本卡新造的说明性中英日例，不是冻结D材料的替换或译文修补。研究字段有限（单焦点、单角色、单时间、cause_count只容1/2/null），不能充当完整事实图；不能表示时须留未知及事实说明，后续产品映射需独立契约工作。`relation/learning`与四乎评估的完整定义不在本卡新增范围；尤其D08好奇负向继续未定义。未经用户批准和新的解释版本，现有标签、分母、排除、失败记录和生产结果都不变。
