验证档位：轻档。仅新增研究报告与原始任务请求两份 Markdown；本卡不运行模型、不调用付费 API、不改代码或预注册。

# R-STEER-SURVEY-01：激活调控方法调研

## 结论在前

已有实证支持用激活方向改变表达，但本次检索未找到同时证明 **Qwen3.5-4B Q4_K_M、四个乎、中英日同一配置、七档单调、无状态外露且能力代价小** 的研究；这是一处证据缺口，不是“不可能做到”的结论。[P02][p02] [P07][p07] [P11][p11] [R19][r19]

强度增加可能先增强目标、再造成回落或失真；均值差、PCA、SAE、单神经元都没有跨模型通用的安全剂量或能力优势，不能照搬论文中的系数。[P02][p02] [P05][p05] [P08][p08] [P09][p09]

跨语言迁移有成功例子，但语言、维度和层配置的差异很大；词汇或概念被注入后直接进入输出也有实证，“说出概念”与“在场景中自然表达状态”必须分开。[P10][p10] [P11][p11] [P12][p12] [P13][p13]

对现行 C3 最有用的是这些解读边界：**2a 的七点非递减不等于七档可辨，2a 没有能力测试，当前 2a 是隔离 H0 callback 而非原生 cvec 路径，失败也不能直接归因为 4B 或量化。**这些判断沿用已冻结设计，不改变任何门、样本、剂量或后续顺序。[N02][n02] [N03][n03] [N04][n04]

Lyra 的 MIT 项目确有 Qwen3.5-4B 词级扫描和剂量实验，但“第 10–17 层主要控制语气与立场”比原证据更强；Neuronary 配方的 CC0、Neuronpedia 当前代码的 Apache-2.0 与各自模型、SAE、数据的许可也不能混为一谈。[R02][r02] [R03][r03] [R04][r04] [R09][r09] [R10][r10]

## 1. 范围、证据等级与项目基线

**状态：调研完成，待用户审查。**“可借鉴”在本文指已有方法及证据能怎样帮助理解结果；不提出新增实验或产品设计，不把外部论文中的门控、去词投影或训练方法加入 C3。后文只对照已有的 2a、2b、3、4 与留出检验。用户要求优先于研究卡的一般发散要求；没有新增设计需要提交黄金规则第 17 条的三问。[N00][n00] [N03][n03]

证据标签如下；“实证”均指原作者报告，不表示本卡复现过。

| 标签 | 本文含义 |
| --- | --- |
| 论文实证 | 原论文有明确模型、干预和测量；来源表保留所读 arXiv 版本或正式会议版 |
| 项目实证 | 作者仓库、研究日志或模型卡报告的实验；可信范围随样本和公开信息收窄 |
| 接口事实 | 源码、模型卡、许可证或文档可直接核验；不自动证明行为有效 |
| 推测／解读 | 本文基于上述证据作的推论，或原作者尚未测试的解释 |
| 未核实／未找到 | 本次可访问来源不能支持该结论；不等于全球不存在相关证据 |

先完整读取 `AGENTS.md`，再按 `echo-core-work/SKILL.md` §1 分级阅读。固定主仓库基线为 **`05e993eaeac1d0ef51e7df4b05e60478d0255612`**。只读任务相关原文及其明确引用的规格章节，没有把其他开放草案当作已采纳决定。[N00][n00] [N01][n01]

| 已读来源 | 固定位置与状态 | 本报告采用的范围 |
| --- | --- | --- |
| R-STEER-01 报告、C3 预注册 | PR #144，`4831d50161e3cb76484e2b92a942e63f3d22312a`，读取时 draft / needs-user | 报告最新状态优先于旧历史段；2a 用获批追加修订，不用原 8 层、9 点规模。[N02][n02] [N03][n03] [N04][n04] |
| C3 评分与解释文件 | 同一 PR #144 提交 | 非复述 T、S、meta 的区别；候选文件旧标题不覆盖报告中后续的采纳状态。[N05][n05] [N06][n06] |
| R-STEER-02 原报告及 V2 报告 | 上述 main 基线 | 原 C2 评分与按 C3 口径复评均保留，当前解释以 V2 为准。[N07][n07] [N08][n08] |
| ADR 0003 | 上述 main 基线 | 本地残差注入与文字投影的不同身份；后附修订一并读取。[N09][n09] |
| ADR 0036 | PR #164，`2200121f109db00e6f30d801adcdd19f465474df` | 仍为待审提案；其中资源／效果回执要求只用作明确的草案对照，不宣称已成为现行实现。[N10][n10] |
| 必要的被引规格 | 上述 main 基线 | v2.0 §§2.3、3、7–9、13、16、17.1–17.2；算子规格 §§12–14；差异登记 V2-116/117。四个乎不等于基本情绪类别，J 的固定位置、输入和预算合同仍有约束。[N11][n11] [N12][n12] [N13][n13] |

## 2. 方向怎样提取：方法与能力代价

### 2.1 方法不是一条从“粗”到“精”的质量阶梯

| 方法 | 实际提取／干预对象 | 已有证据与能力边界 |
| --- | --- | --- |
| 均值差／CAA | 匹配正负例同层激活之差取均值，再沿该方向加减；原始 CAA 取答案选项 token，生成时注入 prompt 之后的位置 | **论文实证：**能改变多类行为；原始 CAA 的能力检查是改造为二选一的 MMLU“正确项平均概率”，不能写成标准 MMLU 准确率无损。能力对比见 §2.2。[P01][p01] |
| PCA／主轴 | 选择变化最大的轴，但输入与中心化定义不统一 | AxBench 的 PCA 用中心化正例；LAT 在正负样本全体中随机配对，单位化差值后做 PCA；FaithSteer 又以对比差主成分称 PCA。**接口／定义事实：**同名不保证同一算法，不能直接移植排名。[P02][p02] [P03][p03] |
| 人设向量 | Anthropic 用特质正负提示生成回答、按评分筛选，平均回答 token 的残差激活后相减 | **论文实证：**Qwen2.5-7B-Instruct、Llama-3.1-8B-Instruct 上，每特质 20 道单轮评估题的自动 trait 分可调；微调后以大强度推理期抑制特质会损害 MMLU。它不是 Qwen3.5-4B 或任意人设的证明。[P07][p07] |
| SAE 特征 | 从稀疏字典选特征，沿 decoder 方向注入，或钳位 feature activation 后重建 | **论文实证：**解释性和稀疏性不保证小代价；特征筛选会明显改变结果。单特征钳位可严重降低知识和推理准确率。[P03][p03] [P04][p04] [P05][p05] |
| 单神经元 | 修改 MLP 中间通道；固定加法可映射为该通道的 down-projection 列方向 | **项目实证：**Qwen3.5-4B 上有词分布改变和剂量崩坏；没有相同强度效果下优于 CAA 的能力对照。设置值、乘法、消融与恒定加法是不同操作。[R01][r01] [R02][r02] |
| 线性探针／分类方向 | 学分类边界，再把法向量用于干预；识别任务和生成任务分开 | **论文实证：**FaithSteer 的 ITI／探针对照有能力损失；AxBench 也表明检测得更好不必然调控得更好。V 的可分性不能代替干预证据。[P02][p02] [P03][p03] |
| 条件调控 | CAST 在指定层检测条件投影，满足阈值才注入行为向量；另一 SAE 实验用外部安全分类器决定是否启用 | **论文实证：**可减少不该干预时的副作用，但引入条件判错、阈值和额外计算。CAST 的低误拒率不是一般能力无损；外部分类器也不是本地单个 4B 的免费组件。[P05][p05] [P06][p06] |
| 分层／位置调控 | 选层、选 token，或给各层不同向量 | **论文实证：**Anthropic 附录 J.3 比较逐层增量方向 `v_l-v_(l-1)` 与单层操作的 MMLU 权衡；这不同于 C3 第 3 阶段“2／3 层均分同一总预算”，不能当作后者的直接验证。[P07][p07] [N03][n03] |

**对 C3 的定义核对。**其主方向是语内均值后中英日等权、最后单位化；第 4 阶段 A 是逐对单位差的均值，B 是 **48 个差向量的未中心化二阶矩第一特征向量**。令差向量为 `x_i`，均值为 `μ`，则二阶矩 `M=C+μμᵀ`，其中 `C` 是中心化协方差。这是代数恒等式；取 `M` 的主轴与通常中心化 PCA 不是同一个操作。外部“PCA 失败”不等于 C3-B 已失败，外部“PCA 成功”也不能替它过门。[N03][n03] [P03][p03]

### 2.2 能力代价：保留量纲和比较条件

**论文实证一：FaithSteer-BENCH。**表 8 为 Llama-2-7B-Chat、八项 steering 任务平均的 `ACC(干预)-ACC(基线)`，单位是 **0–1 准确率差**。各方法使用各自校准后固定的操作点，并非达到同样目标强度后的公平代价排序。[P02][p02]

| 方法 | ΔRACE | ΔMMLU |
| --- | ---: | ---: |
| CAA | -0.00250 | +0.00125 |
| PCA | -0.20375 | -0.14375 |
| TopPC | -0.19875 | -0.14000 |
| ITI | -0.09625 | -0.05250 |

作者在 Qwen2.5-7B 的部分结果中，CAA 能通过能力门却没通过控制门；**“能力保持”可以同时伴随“控制不足”**。表内差异支持审慎比较，不能得出“CAA 永远最好”或为 C3 更换算法。[P02][p02]

**论文实证二：Phi-3 Mini 的 SAE 拒绝方向。**同一个 feature 22373、clamp=12，附录 A.7 表 15 如下；条件版用额外的 WildGuard Mistral 7B 判断输入，判安全时绕过 SAE。[P05][p05]

| 设置 | MMLU 准确率 | GSM8k 准确率 | WildGuard 不安全输入拒绝率 | WildGuard 安全输入误拒率 |
| --- | ---: | ---: | ---: | ---: |
| 不干预 | 68.80% | 82.50% | 58.33% | 6.03% |
| 持续干预 | 35.98% | 35.56% | 96.02% | 68.36% |
| 条件干预 | 66.19% | 84.69% | 85.90% | 9.10% |

**推测／解读：**这里更好的权衡部分来自“安全任务不施加该干预”，不能据此断言目标本身与能力已经解耦。分类器漏判也使部分目标输入绕过干预；该机制既不同于 C3 固定位置掩码，也超出当前单模型方案。[P05][p05] [N03][n03]

**论文实证三：SAE 结论依赖筛选。**AxBench 的 Gemma-2-2B／9B 四层平均综合分是 DiffMean 0.239、SAE 0.165、PCA 0.105；指标是概念、流畅度、指令相关性的调和平均，**不是一般能力准确率**。后续研究筛去低输出分特征，在 Gemma-2-9B L20 把自身复现分从 0.293 提高至 0.546，但评价集合变成保留特征子集，作者也承认原始复现分与 AxBench 有差异。二者不能压成“SAE 已被淘汰”或“SAE 已全面胜出”。[P03][p03] [P04][p04]

**对事实忠实的边界。**MMLU／GSM8k 测知识和推理，coherence 测可理解性，trait score 测目标表现；它们都不等于“忠于当前给定事实”。C3 已分别约束事实、连贯、未截断比例及独立能力任务；现有外部指标不能替代这些门。[N03][n03] [N05][n05]

## 3. 强度、倒 U 与细粒度分档

### 3.1 已观察到什么

| 来源与范围 | 原作者观察 | 能支持到哪里 |
| --- | --- | --- |
| 2601.21505，Llama-3-8B-Lexi-Uncensored，英文，190 人、19 条单轮提示、8 个 λ（含零） | 全层风格向量；0–7 人评中，喜悦在 λ=.30 为 4.63，.35 回落到 3.83；厌恶条件可理解度从零值 5.83 降到 .35 时 2.77 | **论文实证：**目标强度与质量可发生不同方向的变化；描述均值回落不自动说明每一相邻差都统计显著。[P08][p08] |
| 2511.18284v2，Llama-3.1-8B，第 15 层，50 类行为 | 作者汇总曲线报告目标表现随强度呈倒 U，连贯与相关性下降 | **论文实证：**该固定模型／层下的自动评分结果；不是对任意模型、七点或全部行为的定律。[P09][p09] |
| 2605.30443，Qwen3-8B，英文隐喻，L14 | α=.5/1/1.5/2 的目标比率为 37.8/65.0/51.2/45.5%；coherence 为 3.98/3.93/3.38/.94（0–4） | **论文实证：**该任务有可用窗口；目标判分有效数分别 74/100/86/77，coherence 为 100/100/91/77，不能当作 100 条全有效曲线。[P10][p10] |

Lyra 的初测为 **L22 层 N2742 附近的 21 个神经元和 8 个随机方向**：按其残差尺度，小于 .75 倍时 distinct-4-gram 比率≥.93，.75–1 倍时神经元组降到 .61、随机组 .80，超过 1.5 倍出现崩坏。**项目实证：**窄窗口的线索成立，但这是一项词汇多样性代理指标与有限样本实验；既不是全层安全界，也不是一般能力无损窗口。[R02][r02]

### 3.2 为什么剂量数字不能横比

| 来源 | 系数实际乘什么 | 与 C3 比较时缺少的等价条件 |
| --- | --- | --- |
| C3 当前 2a | 单位方向乘 `t*r_l`；`r_l` 是同 hook 的 192 个 E 末 prefill 激活范数中位数 | 当前只测 `t=[-.08,-.04,-.02,0,.02,.04,.08]`，H0、M0；不是原九点。[N03][n03] [N04][n04] |
| Lyra／Neuronary | 单元 add 剂量先按残差中位范数与 `W_down` 列范数缩放 | 校准文本、统计位置、方向和执行后端不同；dose=1 的残差变化约一份其校准残差范数。[R02][r02] [R04][r04] |
| 人类情绪研究 | 一个 λ 同时乘各层的原始情绪均值差 | 多层累积与向量长度不同，λ=.15 不是 C3 的 t=.15。[P08][p08] |
| 跨语修辞研究 | 单位向量乘所选层残差范数均值及 α，作用于 prompt token | 均值／中位数、校准集和 prefill／decode 覆盖不同；α=1 不能换算为 C3 的推荐值。[P10][p10] |

**关于“如何细分七档”的证据回答：**已有研究通常先扫系数，再按目标与质量挑可用点；本次没有找到将任意人设方向自动变成七个等感知间隔、三语都可靠的刻度方法。线性注入 `h'=h+a*v` 并不意味着生成行为关于 `a` 线性；语义目标和离散 token 结果还会随上下文改变。这是依据上述曲线和模型操作作的**解读**，不是为项目引入新校准器。[P08][p08] [P09][p09] [P10][p10] [N12][n12]

C3 的现有七点门要求各语相邻 **非递减**，同时检查端点、根级方向、随机／非目标特异性和质量；它允许平台段。故通过只支持“在已测七点、当前材料与判分下满足冻结条件”，不能升级成七个相邻档都可辨、点间连续单调、强度等距或组合方向有效。遗漏的 ±.01 未验证；文学中的倒 U 也不能用来事后删掉失败端点。[N03][n03] [N04][n04]

## 4. 中英日迁移：已有成功不等于同一三语配置合格

**论文实证：**《Steering with Multilingual Lexicons》用英文情感词典方向控制 11 种语言的续写。表 2 的 SR 是 GPT-4.1 判目标情感、GlotLID 判语言正确的交集，英文词典系数 α=.5；每格从 all／top-50%／top-5 层选择自己的最佳配置。[P11][p11]

| Base 模型 | 英文 SR | 中文 SR | 日文 SR |
| --- | ---: | ---: | ---: |
| Qwen3-4B-Base | 44.5%（all） | 53.4%（all） | 23.9%（top-50%） |
| Gemma-3-4B-PT | 95.0%（all） | 86.8%（all） | 80.2%（all） |

这里的 Qwen 是 **Qwen3，而非 Qwen3.5**；任务是正负情感续写，不是四个乎或七档人设。中文、日文未报告对应人工复核，SR 的正负汇总细节也未在本次阅读中定位清楚；不能把表中数值解释为迁移增益或自然对话成功率。[P11][p11]

另一篇跨语修辞研究在四模型、六语言的 408 项比较中报告迁移效果，并考察跨语言共享方向；**未含日语**。跨语言平均方向有效不等于已经去掉了词汇成分，更不证明对本项目概念也有效。[P10][p10]

**对 C3 的解读：**同一方向在三语都能推动某种输出，是可能性证据；三语强度曲线相同、最好层相同、目标语保持与事实保真，是额外要求。C3 已使用语内均值后三语等权、每通道统一配置、逐语门和最差语排序；不能拿上表的“逐格最优”结果替代该合同，也不能把同根的三种语言当成三份独立故事。[N03][n03]

## 5. 状态词外露、概念复述与词汇成分

先分清三种现象：**提到目标词／同义词**、**自报或复述抽象状态**、**重复／乱码等生成失真**。它们可以共现，却不是同一指标；C3 的 meta 与非复述 T 正是为避免将第二种误计为自然表达。[N05][n05]

| 已核验来源 | 实际测到什么 | 减轻办法与证据边界 |
| --- | --- | --- |
| Lindsey，2025 概念注入 | 在无关的是／否问题中，注入强度增大时，回答提及所注概念的频率上升；晚层也可能先念出词再作自我解释 | **论文／研究报告实证：**直接词汇外露确实存在；不是 C3 的状态表外露率，也不据此判断模型有主观体验。[P13][p13] |
| Stolfo 等，ICLR 2025 | “禁止某词”的对比方向，有时反而在 unembedding 投影中提高该词 logits；减去“包含该词”的方向改善禁词表现 | **论文实证：**有禁词输出对照；Phi-3 的强度扫描同时显示禁词准确率与回答质量的权衡。操作是词 inclusion 方向相减，**不是从人设向量中正交去掉词汇成分并保持原人设效果**。[P12][p12] |
| Anthropic persona vectors 附录 B.2、M | 明确不同意用户、但语气热情鼓励的回答仍可能被评为高谄媚；相近 SAE 特征也可能代表普通应允措辞 | **论文实证：**表面语言能影响 trait judge；自动高分不能独立证明目标内部状态或行为因果特异性。[P07][p07] |

**去词成分是否已有成熟实证？**在核验的原始论文中，未找到同时验证“指定状态词及同义复述减少、剩余情绪／人设行为保留、常规能力保持”的正交去词联合实验。检索到的 unembedding 投影项目有待实现／预期结果文档，不能当作已完成效果；另有小样本作者笔记，但固定版本与时间未核实，未纳入结论。[X01][x01] [X02][x02] [X03][x03]

**推测／解读：**消去一个词相关方向，可能同时消去目标语义的一部分；反之，目标仍可通过同义词或自报句式表达。现有禁词实验没有解决这两个问题。因此报告只把它列为外部研究线索，不给 C3 增加投影、词表、过滤器或新提取法。[P12][p12] [N03][n03]

### 与已有 R-STEER-02 的对应

**项目已有实证：**V2 对同批 C2 **文字状态**输出按 C3 非复述口径重评，每模型 477 项、其中非零 432 项，但只有 3 个合成根。4B 的 meta 为 85.95–86.16%，非零档联合成功 1.62–1.85%；Sol 分别为 61.84–63.10% 与 29.40%，两者均只有 1/24 曲线通过形状门。上述区间是原分母下缺失的最坏界，不是总体置信区间。旧报告与 V2 同时换了评分口径和评审器，不能把差异全归因于某一个因素；这些也不是激活注入结果。[N07][n07] [N08][n08]

对 2a，若 S 上升而 T 不升，或 meta 上升，只能分别报告概念输出和外露，不能合并成有效表达。反过来，T 的变化也须同时看事实、连贯、截断与独立能力门，不能因为不再直说状态词就宣称已解决。[N03][n03] [N05][n05]

## 6. 哪些情绪／人设维度难调

**论文实证：**人类情绪研究中，厌恶、恐惧等有较明显的目标强度变化，惊讶最弱；惊讶的十人平均评分一致性 ICC 只有 .28，其他五维为 .71–.87。这意味着弱结果还可能包含测量难度，不能只解释为“模型没有该方向”。喜悦的高剂量回落已列于 §3；文中同时出现情绪串扰。[P08][p08]

50 类行为研究的自动评分显示人格倾向与知识负担较重的公众人物模仿不同。其主模型与层固定，judge 也可能偏好夸张表达；这是**特定设置中的难易差异**，不构成通用排名。[P09][p09]

**对四个乎的解读：**“惊讶”不能代替“好奇”，“谄媚”不能代替“联结”，“正面情感”不能代替“共情”，“某个人物的模仿”也不能代替“成事”。现行 C3 只主动调联结、成事，共情、好奇作为非目标串扰观察；本次没有找到足以排序这四个通道可调性的直接实证，2a 也不能据两条曲线推断另外两条已验证。[N03][n03] [N11][n11]

## 7. Qwen3.5-4B、量化和 llama.cpp 的证据

### 7.1 Lyra 线索核验

**项目实证，结论缩窄：**跨 32 层扫描每层抽取 384 个神经元、64 个随机方向；超过随机对照阈值的单元中，L10–17 推动的前列词更常见，作者给出 discourse／stance 词例。作者同时明确把“stance-specific 还是 generic”列为未测；后续改用全词指标，中层低谷仍存在。因此可写“词级结果与语气／立场线索一致”，不能写“已证明这些层的神经元主要负责语气和立场”。“中层特征更分布式”也只是作者解释。[R02][r02]

剂量窗口的直接证据见 §3，原测试后端是 HF／vLLM 等，不是 C3 的量化／引擎组合。随机方向也可能引起身份或风格改变；能产生引人注意的输出不等于该神经元具有独占的人设功能。[R01][r01] [R02][r02]

**接口事实与代数条件：**对无额外 FFN layer scale 的该结构，在 `down_proj` 输入对单元 n 加常数 c，会在其输出增加 `c*W_down[:,n]`。这是把 MLP 加法换成残差方向的依据；神经元编号不等于残差坐标，乘法、清零、钳位也不能如此替换。跨 hook、层尺度或精度变化仍需各自绑定，公式等价不是生成轨迹逐字节等价。[R02][r02] [R17][r17]

### 7.2 量化实证与“支持该格式”分开

| 来源 | 模型／精度／路径 | 证据等级与结论范围 |
| --- | --- | --- |
| llama.cpp cvector-generator 文档 | 示例 `llama-3.Q4_K_M.gguf`，支持 mean／PCA、导出及加载向量 | **接口事实：**说明工作流支持；示例没有对应行为和能力结果，不是量化无损实证。[R15][r15] |
| maxvision 的 Analgesia 模型卡 | 原始 Qwen3.5-4B 与改权重版本；stock llama.cpp，Q8_0 与 **Q4_K_M**，同一控制向量示例 | **项目实证：**作者报告原模型出现目标负面情感文字，并有三向量评分；说明该组合有可观察调控案例。未独立复现，未逐条审核 raw 输出；不是七剂量、三语、能力或量化因果对照。本卡不采纳其关于体验的解释，也不提出改权重方案。[R19][r19] |
| dahara1 的 EvilQwen 控制向量 | Lyra 单元 down-projection 列转 GGUF；Qwen3.5-4B **UD-Q4_K_XL**，日英短例及少数种子 | **项目实证：**有低比特和 llama.cpp 的示例，且大 scale 易失真；不是 Q4_K_M，更不是完整三语单调测试。[R20][r20] |
| 本项目当前 2a | 固定 Q4_K_M、b11669、全部五个入选层同一隔离 H0 callback | **项目执行记录：**已完成生成与工程检查，但在读取的固定提交仍在评分；不能提前宣称语义门或能力门通过。[N02][n02] [N04][n04] |

本次未找到 Qwen3.5-4B 在 BF16、Q8 与 Q4_K_M 上，保持模型版本、方向、实际注入范数、任务、采样和评分一致的完整能力／单调性对照。因此“量化是本次失败原因”与“量化对调控没有影响”均为**未识别的归因**。[R19][r19] [R20][r20] [N03][n03]

### 7.3 当前 2a 不应记成原生 cvec 验证

按 C3 固定的 llama.cpp 提交 `620d2225cc436ab0b001e3200e590e3b286ef9ab`，Qwen3.5 图在 FFN 残差相加后调用 `build_cvec(cur, il)`，随后标记 `l_out`；adapter 用 f32 方向张量相加，**第 0 层没有 tensor**。Q4_K_M 是权重格式，不能据此称注入方向也被压为 4 bit。[R16][r16] [R17][r17]

获批修订因此在所有入选层统一使用 `l_out-{layer}` 同步 eval callback 做 f32 加法，原生 cvec 关闭；数学位置仍为 H0。报告可描述“在 llama.cpp 引擎的 H0 实施了残差注入”，但不能据此证明原生 cvec 的层映射、载入和零路径已在本次 2a 验证。不同操作路径的工程证据不能互相借用。[N04][n04]

## 8. 可直接查阅的资源与许可证

许可证是读取时对应文件／模型卡的声明；代码、权重、配方、数据分开记录。这里没有新增依赖或引入资源，也不把未核实许可写成“无许可”。

| 资源与固定版本 | 对 Qwen3.5-4B 的用途 | 许可核验及限制 |
| --- | --- | --- |
| neuron-explorer，`7ec363d2d9686802a1b4aa7c6d8a2627b1759159` | 神经元浏览、调控、词分布扫描、研究日志与 lens 工具 | **代码 MIT**，已读原 LICENSE；不自动覆盖基模、外部 lens 或语料。[R01][r01] [R03][r03] |
| Neuronary，代码 `c23cb01bad1f37783247191c29d7d4bb3e898457`；线上 About 单独核验 | 配方、剂量单位、用户解释及输出例子，层／单元零起算 | **配方 CC0**；仓库根目录和已读 README 未取得代码许可原文，代码许可未核实。网站经验提示不是受控评估。[R04][r04] [R05][r05] [R06][r06] |
| Neuronpedia，`607628697b23908d2bbc633722b77a4e6fc9d8be` | 有 Qwen3.5-4B MLP 页面；文档列 32 层×9216 单元，Inference 为 **No** | **当前代码 Apache-2.0**；NOTICE 说明旧发行版为 MIT，不能沿用旧搜索缓存。代码许可不覆盖所有托管资产。[R07][r07] [R08][r08] [R09][r09] [R10][r10] |
| `decoderesearch/qwen-3.5-saes`，经 Neuronpedia 的发布页链接 | Qwen3.5-4B 及 Base 的 Matryoshka BatchTopK SAE；模型卡列 L3／15／27，65k features，2k／16k 前缀 | 直接相关的预训练字典资源；读取的模型卡未显示许可字段，文件列表访问失败，**SAE 权重许可未核实**。不继承 Neuronpedia／SAELens 的代码许可，也不视作四个乎方向。[R11][r11] [R12][r12] |
| `Qwen/Qwen3.5-4B` 官方模型卡／LICENSE | 4B、32 层、hidden 2560、FFN 9216 的模型身份 | **Apache-2.0**；另外发布的量化物、派生物仍需单独核对，不能代替 C3 固定 GGUF 哈希。[R13][r13] [R14][r14] |
| llama.cpp，C3 固定提交 `620d2225cc436ab0b001e3200e590e3b286ef9ab` | cvector 生成／加载、Qwen3.5 图与残差位置 | **MIT**；使用具体版本源码，不把 master 新参数或历史 PR 当作固定版本保证。[R15][r15] [R16][r16] [R17][r17] [R18][r18] |
| `maxvision/Qwen3.5-4B-Analgesia` | 作者的 Q4_K_M 原模型控制向量案例，附验证／证据目录 | 模型卡标 **Apache-2.0**；行为证据等级见 §7，不把作者声明代替独立复现。[R19][r19] |
| `dahara1/EvilQwen-control-vectors` | 单元列向量转 GGUF 以及 Q4_K_XL 日英示例 | 向量模型卡标 **Apache-2.0**；其所用量化权重是另一个资源，不能共用本条许可结论。[R20][r20] |
| Anthropic `persona_vectors`，`b8e0f044fe2410a6fad579f38324f03f13b4e917` | 提取、steering、验证与训练代码；原实验并非 Qwen3.5-4B | **Apache-2.0**；属于可读的方法实现，不是已有 Qwen3.5-4B 四通道包。[R21][r21] [R22][r22] |

补充资源：explorer README 链接的 [R／J++ lenses][r23] 被 README 标为 Apache-2.0，但其独立模型卡本次访问失败，**仅有二级声明**，未将其许可列为已核验。[R01][r01]

## 9. 对现行 C3 与 2a 结果解读的影响

以下是证据解释，不是新增设计、择优规则或执行授权。主读版本的 2a 为 **L/R/K=4/2/6**：联结层 `[0,15,21,24]`，成事层 `[7,15,21,24]`；3,924 项为 109 条件×12 根×3 语，共用随机控制去重后的数，5,220 个分析位置不是独立样本。四窗已生成；固定报告仍记录评分进行中、794 条达到 96-token 预算。该状态仅对所引提交成立。[N02][n02] [N04][n04]

| 发现／依据 | 对 2a 可以怎样解释 | 与已经登记的后续环节的关系 |
| --- | --- | --- |
| 可分方向不必然能控制生成。[P03][p03] | V 的正号比例／投影好，只支持表征诊断；2a 无效果不是矛盾 | V 已用于选层，不能当独立最终检验；原规则没有因 V 未过门就取消干预。[N03][n03] |
| 剂量效应可能回落、饱和或失真。[P08][p08] [P10][p10] | 七点全曲线、随机对照与质量共同决定冻结门；不能只挑好看的中间点 | 不增补 ±.01，不删端点；H 仍是候选锁定后一次检验。[N03][n03] [N04][n04] |
| 非递减与相邻可辨不同 | 通过允许平台段；不等于七个可感知等级，更不证明点间／范围外行为 | 沿用原门，不在报告中替换成更强或更弱的结论。[N03][n03] [N04][n04] |
| 单层／多层／token 位置的效应不同。[P07][p07] | 2a 只覆盖 H0M0，不足以否定所有位置方案 | 已注册 2b 比较 H0/H1、M0/M1/M2；M2 仅诊断，不能成为正式兼容候选。语义门控不在其中。[N03][n03] |
| 多层增量人设向量不是均分剂量。[P07][p07] | 不能借其能力收益预测 C3 第 3 阶段必然改善 | 第 3 阶段只做预定 2／3 层均分固定总预算，不换成差分向量或搜权重。[N03][n03] |
| “PCA”定义不一。[P02][p02] [P03][p03] | 外部 PCA 排名不能替代本项目比较 | 第 4 阶段仍只有逐对单位差均值与未中心化二阶矩主轴；SAE、探针和去词投影没有被加入。[N03][n03] |
| 词汇外露与 trait judge 的表面线索混淆已有实证。[P07][p07] [P12][p12] [P13][p13] | S、T、meta 的分歧本身就是结果；state word 增多不是自然表达成功 | 不改评分、不把 meta-only 算 T，不把独立双评用于覆盖主标签。[N04][n04] [N05][n05] |
| 跨语效果不保证共用配置，尤其不能拿逐语最优结果替代。[P11][p11] | 逐语报告并保留最差语；日语失败不能被中英平均掩盖 | 根携三语配对，12 根不是 36 个独立故事；H 的根数也按原定义解释。[N03][n03] [N04][n04] |
| 共情、好奇并未主动扫描 | 2a 只检验联结、成事，并观察非目标串扰 | 不能称四个乎已完成校准，也不能从情绪论文给四通道补排名。[N03][n03] [N11][n11] |
| 连贯、事实与一般能力是不同终点。[P02][p02] [P05][p05] | 2a 不运行能力，任何输出变好都不足以称“能力无损” | A-D／A-H 仍由已有阶段规则决定；本卡没有运行、增加或放宽这些门。[N03][n03] [N04][n04] |
| 外文论文常只汇总有效判分，曲线可能受缺失影响。[P10][p10] | 794 条达长度上限必须按现行规则保留；invalid／missing 不补零、不删分母 | 混合评分及 DS-only 敏感性、393 条独立双评照旧；不同评审器间一致不等于真值准确。[N02][n02] [N04][n04] |
| 2a 的 hook 与原生 cvec 不同。[R16][r16] [R17][r17] | 工程有效结论限定在当前 callback、固定模型与引擎 | 不借原生接口文档替代实际注入证据，不借外部 Q4_K_XL 例子证明本 Q4_K_M 路径。[N04][n04] |
| 现有量化证据不足以识别失败原因 | 未过门可表述为“冻结条件下不足”；不能表述“4B 根本不会”或“量化导致” | 现有更大模型预留仍为 needs-user / not_run，本卡目标是本地 4B，不发起更大模型或追加支出。[N03][n03] |
| 资源身份和行为证据必须绑定 | 可下载、可加载、许可证宽松，都不证明单调、事实忠实或组合可用 | ADR 0036 仍为提案；即使回执结构完整，也不能替代其要求的三语、单独／组合范围与能力证据。[N10][n10] |

**既有合同未改读。**ADR 0003 的本地表达身份、算子规格的 J 输入／固定位置／预算约束、C3 的选层／分母／门／停止线均保持原文；报告没有将“文献中可做”改成“项目已允许做”。[N09][n09] [N12][n12] [N03][n03]

## 10. 限制、交付与验证

### 10.1 已知限制及未完成的外部核验

- 这是一轮定向调研，不宣称穷尽全部文献。正文实证优先原论文、作者日志、模型卡与源码；没有运行模型、下载权重、复现分数、调用评分 API，也没有修改其他文件。
- 多数论文没有在本次已读方法／附录中找到明确量化精度，不能替作者补写 BF16／FP16。多个研究依赖自动 judge、单轮或短续写，不能据此保证长期角色表现。[P02][p02] [P07][p07] [P09][p09] [P10][p10]
- 2605.30443 的可访问 PDF 页眉为 **v3、2026-10-13**；摘要页和精确版本入口访问失败，提交历史未独立核验。报告仅按实际读到的 PDF 版本引用。[P10][p10]
- 2511.18284 的“每行为 200 例、50 行为、共 2,000 例”在原文内部不能直接相乘吻合；其 §4.2.2 对“gap”的文字与图轴定义也有歧义，不用来证明投影可分性是否预测干预。AxBench 正文与表 3 的部分 win rate 不一致。本文不使用这些有歧义的总数／胜率，也不擅自修正。[P09][p09] [P03][p03]
- Neuronary 代码、Qwen3.5 SAE 权重与托管数据、外部 R／J++ lens 的独立许可未完全核实，具体可用性见 §8；这些缺口不阻塞文献报告，但不能被读成可直接引入项目的许可结论。
- 2a 评分仍在其他任务中推进；本报告不读取其私有原始输出或替其签发结果。冻结来源之后的新提交，不自动包含在本文现状判断内。[N02][n02]

### 10.2 交付与检查

仅新增 [本报告](REPORT.md) 和 [原始任务请求](../../tasks/requests/R-STEER-SURVEY-01.md)。请求文件原样记录用户提示，不追加新设计或授权。

| 检查 | 结果 |
| --- | --- |
| `git diff --cached --check` | 通过，退出 0 |
| 新增 Markdown 相对链接逐个确认存在；Python 标准库路径检查 | 2 个相对链接均指向现有文件；引用式链接无未定义项 |
| `git diff --cached --name-status`；与用户允许文件集合比较 | 通过；恰好两条新增，路径即本报告与请求文件 |
| `git fetch origin main`；`git rev-parse origin/main`；`git log B..origin/main` | B=M=`05e993eaeac1d0ef51e7df4b05e60478d0255612`；无 main 增量，无需同步或扩大验证 |
| 云端文件与本地逐文件 Git blob 核对 | 结果与 SHA 记录在本 PR 的完成说明；不以本地文件存在代替已交付 |
| 模型／GPU／付费 API／全工作区测试 | not_run；本卡仅文献与源码只读核验 |

需要用户做什么：审查证据范围、许可证分层和对 C3 的解读是否准确。按用户要求保留 draft、`needs-user`，完成后在 PR 写“请审查”并停止；不把报告合并视为对新实验的批准。

## 附录 A. 已知线索核验结果

| 线索 | 核验身份／版本 | 结果 |
| --- | --- | --- |
| arXiv 2601.21505 | *The Effectiveness of Style Vectors for Steering Large Language Models: A Human Evaluation*；v1，2026-01-29 | 已读原文及表 IV；英文人评情绪实验，非 Qwen。[P08][p08] |
| arXiv 2511.18284 | *What Can We Actually Steer? A Multi-Behavior Study of Activation Control*；v2，2026-01-11 | 已读原文，倒 U 与多行为差异有作者报告；样本总数有内部疑点。[P09][p09] |
| arXiv 2605.30443 | *Cross-Lingual Steering for Figurative Language Generation*；可访问 PDF 页眉 v3，2026-10-13 | 已读原 PDF；不混用搜索索引的旧摘要；版本登记访问限制见 §10。[P10][p10] |
| arXiv 2603.18329 | *FaithSteer-BENCH: A Deployment-Aligned Stress-Testing Benchmark for Inference-Time Steering*；v1，2026-03-18 | 已读能力表和固定操作点；不是 Qwen3.5/Q4 研究。[P02][p02] |
| Anthropic 2025 persona vectors | arXiv 2507.21509，v3，2025-09-05；官方研究页 2025-08-01 | 已读论文、方法／限制及代码许可；实验为 Qwen2.5-7B、Llama-3.1-8B。[P07][p07] [R21][r21] [R22][r22] |
| Lyra neuron-explorer MIT、L10–17、窄窗口 | 固定提交见 §8 | MIT 属实；词级扫描与有限剂量实验属实；层功能与普适窗口须缩窄，不能照搬。[R01][r01] [R02][r02] [R03][r03] |

## 附录 B. 来源链接与定位

检索与正文来源核验日期：**2026-11-12（UTC）**。原始机械摘录另记录 Australia/Perth 日期；本轮检索时两者均为 11 月 12 日。GitHub 项目原文采用固定提交；网页和模型卡未列 revision 的，表示按访问日期读取，不能假定以后仍相同。未引用搜索摘要代替可取得的原文。

### B.1 原始论文与研究报告

| 编号 | 原始来源 | 本文所用位置 |
| --- | --- | --- |
| P01 | [Rimsky 等，Steering Llama 2 via Contrastive Activation Addition，ACL 2024 正式 PDF][p01]；[论文页](https://aclanthology.org/2024.acl-long.828/)；[arXiv 2312.06681](https://arxiv.org/abs/2312.06681) | §3、§7／表 5、§10；正式版与 arXiv 首作者姓名展示有差异，按对应版本保留 |
| P02 | [Ding 等，FaithSteer-BENCH，2603.18329v1][p02]；[登记页](https://arxiv.org/abs/2603.18329) | §3–4、附录 B–F、表 4／6／8 |
| P03 | [Wu 等，AxBench，2501.17148v3][p03]；[作者代码](https://github.com/stanfordnlp/axbench) | §3.3、§4、§5.2／表 2–3／图 4 |
| P04 | [Arad 等，SAEs Are Good for Steering — If You Select the Right Features，2505.20063v2][p04] | §4、表 1、Limitations |
| P05 | [O'Brien 等，Steering Language Model Refusal with Sparse Autoencoders，2411.11296v2][p05] | §3–4、附录 A.7／表 15 |
| P06 | [Lee 等，Programming Refusal with Conditional Activation Steering，2409.05907v3][p06] | §3–4、表 2、附录条件阈值；拒绝任务，不冒充通用能力实验 |
| P07 | [Chen 等，Persona Vectors，2507.21509 PDF][p07]；[登记页](https://arxiv.org/abs/2507.21509)；[Anthropic 官方研究页](https://www.anthropic.com/research/persona-vectors) | 读取版本 v3；§2、§5.1／图 7、§8、附录 B.2／J.3／M |
| P08 | [Diallo 等，The Effectiveness of Style Vectors…，2601.21505v1][p08]；[登记页](https://arxiv.org/abs/2601.21505) | §III–VII、表 III／附录 C 表 IV；不把 η² 的情绪主效应误记为强度效应 |
| P09 | [Bas、Novak，What Can We Actually Steer?，2511.18284v2][p09]；[登记页](https://arxiv.org/abs/2511.18284) | §4.2、§6、§8；数值冲突见 §10 |
| P10 | [Liu 等，Cross-Lingual Steering for Figurative Language Generation，2605.30443 PDF][p10] | PDF 页眉 v3；§3–5、附录 B、G／表 24–26 |
| P11 | [Laurent 等，Steering with Multilingual Lexicons…，AACL 2026][p11]；[论文页](https://aclanthology.org/2026.aacl-long.9/)；[作者代码](https://github.com/lucindalaurent/SteeringWithMultilingualLexicon) | 表 1–2、§5、附录 A.4–A.5；会议日期 2026-11-06 至 11-10 |
| P12 | [Stolfo 等，Improving Instruction-Following…，ICLR 2025 正式 PDF][p12]；[论文页](https://proceedings.iclr.cc/paper_files/paper/2025/hash/8c3262a4c965ba9888f120d4f9e13478-Abstract-Conference.html)；[作者代码](https://github.com/microsoft/llm-steer-instruct) | §5、附录 F.4／图 12、表 13／图 19 |
| P13 | [Lindsey，Emergent Introspective Awareness in Large Language Models，2025-10-29][p13] | 注入概念、跨层、无关是／否问题控制；仅使用其可观察输出结果 |

### B.2 项目与许可证原文

| 编号 | 链接与定位 |
| --- | --- |
| R01–R03 | neuron-explorer：[README][r01]、[research_log][r02]、[MIT LICENSE][r03]；均固定 `7ec363d2…` |
| R04–R06 | Neuronary：[About／dose／CC0][r04]、[README][r05]、[固定仓库目录][r06]；代码固定 `c23cb01b…`，网站单独读取 |
| R07–R10 | Neuronpedia：[Qwen3.5-4B 模型页][r07]、[neurons 文档][r08]、[当前 LICENSE][r09]、[NOTICE][r10]；后两者固定 `60762869…` |
| R11–R12 | [Neuronpedia Qwen3.5 SAE 发布页][r11]、其链接的 [Hugging Face SAE 模型卡][r12] |
| R13–R14 | Qwen3.5-4B：[官方模型卡][r13]、[LICENSE][r14] |
| R15–R18 | llama.cpp：[cvector-generator README][r15]、[llama-adapter.cpp][r16]、[qwen35.cpp][r17]、[LICENSE][r18]；均固定 C3 引擎提交 |
| R19 | [maxvision/Qwen3.5-4B-Analgesia README][r19]，Q4_K_M 表与作者提供的验证／证据说明 |
| R20 | [dahara1/EvilQwen-control-vectors 模型卡][r20]；带硬件查询参数的可访问入口，模型卡本身仍为同一资源 |
| R21–R22 | Anthropic persona_vectors：[固定仓库][r21]、[Apache-2.0 LICENSE][r22] |
| R23 | [bubskee Qwen3.5-4B R／J++ lenses][r23]；独立页面访问失败，仅 explorer README 声明可读 |

### B.3 本项目固定来源

| 编号 | 链接 |
| --- | --- |
| N00–N01 | [AGENTS.md][n00]；[echo-core-work/SKILL.md][n01] |
| N02–N06 | [R-STEER-01 REPORT][n02]；[C3-PREREG-01][n03]；[C3-2A-AMENDMENT-01][n04]；[C3-RUBRIC-01][n05]；[C3-INTERPRETATION-CANDIDATES][n06] |
| N07–N08 | [R-STEER-02 原 REPORT][n07]；[V2-REPORT][n08] |
| N09–N10 | [ADR 0003][n09]；[ADR 0036 提案][n10] |
| N11–N13 | [v2.0 第一性规格][n11]；[算子规格][n12]；[差异登记][n13] |

### B.4 保留为线索、未用于效果结论的来源

- [X01：unembedding-steering-benchmark][x01]：README 的方案／预期结果，不作已完成去词实证。
- [X02：steering-rebels][x02]：参考实现、校准和完整结果仍列待完成，不作效果依据。
- [X03：logit-steering 的 orthogonal-complement 笔记][x03]：小样本作者报告；提交日期、固定版本与完整联合指标未核实，不作成熟证据。
- [日语情绪方向的 IEICE 官方题录](https://ken.ieice.org/ken/paper/20260228acUG/eng/)：全文未取得，未用来给维度排序。
- [Cross-lingual Functional Vectors for Emotion Detection](https://aclanthology.org/2026.findings-emnlp.2030.pdf)：情绪分类而非角色表达，未混入跨语言表达成功率。
- [Extracting and Steering Emotion Representations in Small Language Models](https://arxiv.org/pdf/2604.04064)、[Psychological Steering in LLMs](https://aclanthology.org/2026.acl-long.79.pdf)、[AutoLexSteer](https://aclanthology.org/2026.findings-emnlp.2202.pdf)：已用于扩大检索覆盖；正文以更直接回答本卡问题的原始证据为主，不将这些不同指标混算。

## 附录 C. 实际检索式

以下逐条保留本轮实际输入；检索日期均为 **2026-11-12**。system2 用于初查，system1 用于补覆盖与原始来源核验。直接打开论文、表图、仓库与许可证不另伪造为检索式；其链接列于附录 B。

### C.1 指定论文与方法比较

1. **system1**：`"2601.21505" arxiv`
2. **system1**：`"2511.18284" arxiv`
3. **system1**：`"2605.30443" arxiv`
4. **system1**：`"2603.18329" arxiv`
5. **system1**：`site:arxiv.org "SAE" "steering" AxBench prompting perplexity 2025 steering`
6. **system1**：`site:arxiv.org "Programming Refusal with Conditional Activation Steering" conditional activation steering capabilities`
7. **system1**：`site:arxiv.org "activation steering" "sparse autoencoders" "general capabilities" benchmark`
8. **system1**：`site:arxiv.org activation steering "layer" "intervention" "conditional" benchmark degradation`
9. **system2**：`"Steering Language Model Refusal with Sparse Autoencoders" arxiv conditional`
10. **system2**：`site:arxiv.org "Steering Language Model Refusal" SAE capability conditional`
11. **system1**：`site:arxiv.org/abs/2605.30443 "Linfeng" 2026 arxiv v3`
12. **system1**：`"2605.30443v3" October 13 2026`
13. **system1**：`"2603.18329" layer coefficient capability Phi Qwen FOS`
14. **system1**：`"SAE" "activation steering" quantization 4-bit Qwen empirical`
15. **system1**：`site:aclanthology.org "Steering Llama 2 via Contrastive Activation Addition" MMLU truthfulness coherence layer coefficients`
16. **system1**：`site:arxiv.org "Steering Llama 2 via Contrastive Activation Addition" Panickssery 2023 residual stream layers 13 20`
17. **system1**：`site:arxiv.org "representation engineering" principal component centered difference first PCA equation`
18. **system1**：`site:arxiv.org "activation steering" PCA centered "difference" singular values first principal component`

### C.2 跨语言、人设与外露

1. **system2**：`site:anthropic.com/research persona vectors monitoring controlling character traits language models 2025 code steering`
2. **system2**：`persona vectors paper 2025 arxiv github vectors emotional sycophancy evil hallucination steering strengths`
3. **system2**：`cross-lingual activation steering emotion personality Chinese Japanese English multilingual steering vectors paper`
4. **system2**：`activation steering lexical leakage orthogonal projection word vectors removing concept word output paper`
5. **system2**：`"activation steering" "lexical" "leakage" concept words steering vector output`
6. **system2**：`"steering" "lexical leakage" activation vectors large language models`
7. **system2**：`"steering vectors" "lexical" "leakage" word repetition concept extraction`
8. **system2**：`"activation steering" "word" "orthogonal" "lexical" steering vector`
9. **system2**：`"activation steering" "concept words" "explicitly" outputs leakage model mentions vector`
10. **system2**：`"steering vector" "explicitly" "word" "steering" output leakage emotion personality`
11. **system2**：`"steering vectors" "word repetition" emotional "steering" activation`
12. **system2**：`"activation steering" "leakage" "emotion" output steering vector`
13. **system1**：`"Cross-Lingual Activation Steering for Multilingual Language Models" 2601.16390 steering results Chinese Japanese`
14. **system1**：`"Persona Vectors" Chen Arditi Sleight 2507.21509 github code coefficient trait results MMLU`
15. **system1**：`"activation steering" "lexical leakage" steering vectors linguistic tokens orthogonal projection`
16. **system1**：`"emotion" "steering vectors" cross lingual Chinese Japanese language models persona`
17. **system1**：`"activation steering" "semantic" "lexical" "orthogonal" vectors unembedding removal steering concept`
18. **system1**：`"activation steering" "concept" "word" "leakage" frequency steering output`
19. **system1**：`"steering" "topic" "leakage" "steering vector" persona language model`
20. **system1**：`"activation steering" "steering awareness" concept vector knows steered paper`
21. **system1**：`Jeong emotion steering 0.03 Qwen2.5 Chinese leakage emotion vectors 2026 arxiv`
22. **system1**：`"emotion" "steering" "Chinese" "leakage" Qwen2.5 Jeong`
23. **system1**：`"emotion steering" "lexical" "leakage" concept word activation LLM`
24. **system1**：`"emotion" "steering" "lexical" "repetition" activation vector model`
25. **system1**：`"steering vectors" "lexical" "projection" steering remove tokens unembedding language model`
26. **system1**：`"activation steering" "remove" "lexical" component "projection" vector`
27. **system1**：`"activation steering" "remove" "word" "embedding" projection lexical steering semantic`
28. **system1**：`"steering vectors" "output words" "projection" "orthogonal" semantic concept lexical`
29. **system1**：`Anthropic emergent introspective awareness language models injected thoughts concept verbalizes "without" 2025`
30. **system1**：`"injected thoughts" "concept" "word" language model introspection steering rates anthropic 2025`
31. **system1**：`"activation steering" "repetition" "emotion" generated output emotion vector words steering human evaluation`
32. **system1**：`"activation steering" "lexical leakage" "remove" projection steering vectors emotion`
33. **system1**：`"Psychological Steering in LLMs: An Evaluation of Effectiveness and Trustworthiness" ACL 2026 PsySET models emotion personality benchmark table`
34. **system1**：`"PsySET" "emotion" "personality" "steering" model 2026 effectiveness`

### C.3 Qwen、量化与开源资源

1. **system2**：`Lyra neuron-explorer qwen3.5 4b neuronary.org github`
2. **system2**：`neuronary.org Qwen3.5-4B neuron explorer steering layers 10 17 Lyra`
3. **system2**：`llama.cpp control vector Q4_K_M small model experiment activation steering`
4. **system2**：`site:github.com/ggml-org/llama.cpp "controlvector" gguf direction layer llama_apply_adapter_cvec`
5. **system2**：`site:github.com/ggml-org/llama.cpp "cvector-generator" PCA activations difference directions F32`
6. **system2**：`activation steering control vectors quantized 4bit model experiments Qwen 4B GGUF github`
7. **system2**：`"control vector" "Q4_K_M" "4B" gguf steering`
8. **system2**：`site:huggingface.co/dahara1/EvilQwen-control-vectors Q4_K_M llama`
9. **system2**：`"EvilQwen-control-vectors" "Q4_K_M"`
10. **system2**：`"Qwen3.5-4B-Analgesia" control vectors raw evidence Q4_K_M GitHub`
11. **system1**：`github neuron-explorer neuronary Lyra Qwen3.5 4B license neuronpedia`
12. **system1**：`site:neuronpedia.org qwen3.5-4b mlp neuron lyra`
13. **system1**：`github neuronary.org source Lyra neuronary repository license`
14. **system1**：`llama.cpp cvector-generator quantization Q4_K_M steer vector experimental gguf author`
15. **system1**：`"Qwen3.5-4B" "control vector" "Q4_K_M" llama.cpp`
16. **system1**：`"EvilQwen-control-vectors" Q4_K_M 4B`
17. **system1**：`"Qwen3.5-4B" "steering" GGUF Q4_K_M llama.cpp control vector`
18. **system1**：`Neuronpedia qwen3.5-4b neuron dataset license neuronary MLP Nov 2026`
19. **system1**：`github Neuronpedia source license code MIT AGPL license huggingface datasets qwen3.5-4b`
20. **system1**：`site:neuronpedia.org/qwen3.5-4b/ "Neuronary"`

### C.4 补充核验

1. **system1**：`"activation steering" "lexical" "remove"`
2. **system1**：`"activation steering" quantization Q4_K_M llama.cpp`
3. **system1**：`"CAA" "PCA" "capabilities" activation steering benchmark`
4. **system2**：`"activation steering" "lexical components" "removal"`
5. **system2**：`"activation steering" "lexical leakage"`
6. **system2**：`"steering" "unembedding" "semantic" "projection"`
7. **system1**：`"Improving Instruction-Following" "Activation Steering" Stolfo`
8. **system1**：`site.anthropic.com "introspection" "2025" Lindsey`
9. **system1**：`site.arxiv.org "AxBench" "2501.17148"`

[p01]: https://aclanthology.org/2024.acl-long.828.pdf
[p02]: https://arxiv.org/html/2603.18329v1
[p03]: https://arxiv.org/html/2501.17148v3
[p04]: https://arxiv.org/html/2505.20063v2
[p05]: https://arxiv.org/html/2411.11296v2
[p06]: https://arxiv.org/html/2409.05907v3
[p07]: https://arxiv.org/pdf/2507.21509v3
[p08]: https://arxiv.org/html/2601.21505v1
[p09]: https://arxiv.org/html/2511.18284v2
[p10]: https://arxiv.org/pdf/2605.30443
[p11]: https://aclanthology.org/2026.aacl-long.9.pdf
[p12]: https://proceedings.iclr.cc/paper_files/paper/2025/file/8c3262a4c965ba9888f120d4f9e13478-Paper-Conference.pdf
[p13]: https://transformer-circuits.pub/2025/introspection/index.html
[r01]: https://github.com/lyramakesmusic/neuron-explorer/blob/7ec363d2d9686802a1b4aa7c6d8a2627b1759159/README.md
[r02]: https://github.com/lyramakesmusic/neuron-explorer/blob/7ec363d2d9686802a1b4aa7c6d8a2627b1759159/research_log.md
[r03]: https://github.com/lyramakesmusic/neuron-explorer/blob/7ec363d2d9686802a1b4aa7c6d8a2627b1759159/LICENSE
[r04]: https://neuronary.org/about/
[r05]: https://github.com/lyramakesmusic/neuronary/blob/c23cb01bad1f37783247191c29d7d4bb3e898457/README.md
[r06]: https://github.com/lyramakesmusic/neuronary/tree/c23cb01bad1f37783247191c29d7d4bb3e898457
[r07]: https://www.neuronpedia.org/qwen3.5-4b
[r08]: https://docs.neuronpedia.org/neurons
[r09]: https://github.com/hijohnnylin/neuronpedia/blob/607628697b23908d2bbc633722b77a4e6fc9d8be/LICENSE
[r10]: https://github.com/hijohnnylin/neuronpedia/blob/607628697b23908d2bbc633722b77a4e6fc9d8be/NOTICE
[r11]: https://www.neuronpedia.org/qwen3.5-saes
[r12]: https://huggingface.co/decoderesearch/qwen-3.5-saes
[r13]: https://huggingface.co/Qwen/Qwen3.5-4B
[r14]: https://huggingface.co/Qwen/Qwen3.5-4B/blob/main/LICENSE
[r15]: https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/tools/cvector-generator/README.md
[r16]: https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/src/llama-adapter.cpp
[r17]: https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/src/models/qwen35.cpp
[r18]: https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/LICENSE
[r19]: https://huggingface.co/maxvision/Qwen3.5-4B-Analgesia/blob/main/README.md
[r20]: https://huggingface.co/dahara1/EvilQwen-control-vectors?hardware=rtx-2080-super
[r21]: https://github.com/safety-research/persona_vectors/tree/b8e0f044fe2410a6fad579f38324f03f13b4e917
[r22]: https://github.com/safety-research/persona_vectors/blob/b8e0f044fe2410a6fad579f38324f03f13b4e917/LICENSE
[r23]: https://huggingface.co/bubskee/qwen3.5-4b-r-jpp-lenses
[n00]: https://github.com/nanake-lab/nemu/blob/05e993eaeac1d0ef51e7df4b05e60478d0255612/AGENTS.md
[n01]: https://github.com/nanake-lab/nemu/blob/05e993eaeac1d0ef51e7df4b05e60478d0255612/.agents/skills/echo-core-work/SKILL.md
[n02]: https://github.com/nanake-lab/nemu/blob/4831d50161e3cb76484e2b92a942e63f3d22312a/reports/R-STEER-01/REPORT.md
[n03]: https://github.com/nanake-lab/nemu/blob/4831d50161e3cb76484e2b92a942e63f3d22312a/reports/R-STEER-01/C3-PREREG-01.md
[n04]: https://github.com/nanake-lab/nemu/blob/4831d50161e3cb76484e2b92a942e63f3d22312a/reports/R-STEER-01/C3-2A-AMENDMENT-01.md
[n05]: https://github.com/nanake-lab/nemu/blob/4831d50161e3cb76484e2b92a942e63f3d22312a/reports/R-STEER-01/C3-RUBRIC-01.md
[n06]: https://github.com/nanake-lab/nemu/blob/4831d50161e3cb76484e2b92a942e63f3d22312a/reports/R-STEER-01/C3-INTERPRETATION-CANDIDATES.md
[n07]: https://github.com/nanake-lab/nemu/blob/05e993eaeac1d0ef51e7df4b05e60478d0255612/reports/R-STEER-02/REPORT.md
[n08]: https://github.com/nanake-lab/nemu/blob/05e993eaeac1d0ef51e7df4b05e60478d0255612/reports/R-STEER-02/V2-REPORT.md
[n09]: https://github.com/nanake-lab/nemu/blob/05e993eaeac1d0ef51e7df4b05e60478d0255612/docs/adr/0003-speech-modes.md
[n10]: https://github.com/nanake-lab/nemu/blob/2200121f109db00e6f30d801adcdd19f465474df/docs/adr/0036-j-resources-and-effect-receipt.md
[n11]: https://github.com/nanake-lab/nemu/blob/05e993eaeac1d0ef51e7df4b05e60478d0255612/docs/echo-core/echo-core-v2.0-first-principles.md
[n12]: https://github.com/nanake-lab/nemu/blob/05e993eaeac1d0ef51e7df4b05e60478d0255612/docs/echo-core/ada-core-operators-spec-v1.0.md
[n13]: https://github.com/nanake-lab/nemu/blob/05e993eaeac1d0ef51e7df4b05e60478d0255612/docs/traceability/diff-register.md
[x01]: https://github.com/hughvd/unembedding-steering-benchmark
[x02]: https://github.com/aryan-cs/steering-rebels
[x03]: https://github.com/ntrillard/logit-steering/blob/master/writeup-orthogonal-complement.md
