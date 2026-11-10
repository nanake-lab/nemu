重档：本卡包含预注册GPU及付费API实际运行；测量执行、评分可信度与方向/混合/身份专项分别报告。

# R-STEER-01 载体方向摸底

## 本阶段白话总览（2026-11-11）

当前状态：[03原样重采](C2-TEXT-RUN-03.md)2073/2073完成，原运行器与资源监督均退出0，00:12:45结束，早于同一授权截止00:28:18。15项数据核验及4项资源日志核对通过，220份快照的已知重负载类别均为空；主代理据记录和用户暂停重负载的确认进入评分准备。[新540项盲评](C2-JUDGE-TEXT-03.md)已由03输出重建，12源码静态检查、17项边界测试和18项独立机械核验通过，尚未发送，文字效应仍not_run。01资源冲突和02启动前编码失败原件保留、不回填、不覆盖；所有旧GPU窗口已结束并通知恢复runner/其它任务。

本阶段注入开发门已完成，结果为 **C2_injection_dev_insufficient_stop**：联结、成事在事前固定的10/16/22层中都没有可行层。注入留出与注入混合按规则停止并记not_run。文字首轮的 **C2_text_window_confounded_stop** 保留；本次03重采满足记录中的技术/资源门，仍须完整盲评才能判断文字效果和代价，不能先说文字路线更好。旧C0/C1失败不改，E2仍m=0。

联结有局部变化：英文22层的七点曲线单调、端点差D=1，也胜过两个随机对照；但两端事实忠实成功率均由2/2降到1/2，且中文/日文同层D=0，没能兼顾质量和三语一致性。成事8个可识别语言×层端点差均为0，另1个受invalid影响不可识别；部分成事注入改变了联结读数，未证明成事特异控制。[完整开发结果](C2-DEV-RESULTS-01.md)和[数值汇总](results/c2-development-summary-01.json)保留全部曲线、失败门与缺失，不换题、方向、阈值或层网格。

小能力基准未观察到超过原10个百分点容差的下降，但有74条截断，基线为中8/10、英9/10、日7/10；这不能证明通用能力无损，也不能抵销事实门失败。[54条提取](C2-EXTRACTION-01.md)和[1410条开发采集](C2-DEV-RUN-01.md)均完成、退出0。两根开发场景很少，三语和条件重复不能当独立样本。由于没有合格注入层，两种方法在同一留出场景的直接比较将缺少注入结果；后续只回答文字方法自身的效果与代价，明确这个限制。

共情、好奇的材料重设计与干预本阶段保持not_run。建议先完成当前文字状态检验，分清“状态表述能否产生可测效果”和“当前注入路线能否在事实约束下控制输出”，再由用户决定是否重设计另两个通道；当前不扩大通道或网格。这只是后续建议，不预先决定采用文字路线。

DeepSeek沿已认可的flash high及冻结rubric完成294项：291合规、3invalid，295次尝试、0最终技术失败。3invalid是两项引证门失败和一项完整length，原样入分母，不填0或重评。唯一网络中断收到0字节，30.005688秒后同字节重发，未知与重发均保留整额预留。本阶段费用上界 **US$0.999509166**，主动API时间1318.468秒；剩余硬预算US$4.000490834、时间约5881.532秒，后续必须承接，不能重置。旧US$1.483366224单独保留。密钥仅进程环境，调用进程已退出。

[开发盲评与分析源码](C2-JUDGE-DEV-BINDING.md)在首项API前冻结，分析首次退出0，未在看到评分后改代码或门槛。完整文件链/费用账本及18组核心指标经独立机械复算一致；选层裁定由主代理依原门作出，独立机械核验没有替代验收判断。[评分与分析证据索引](C2-DEV-SCORE-INDEX.json)保留完整原件。

第三块[文字状态留出采集](C2-TEXT-RUN-01.md)2073/2073完成、退出0、1239.906秒；15项独立机械核验通过，三语guard一致。但[23:15起CPU编译/测试与窗口重叠](C2-TEXT-WINDOW-01.md)，运行器只监测runner，未及时识别其它会话负载，归档时才发现；技术通过不能替代安静窗口要求。477自由输出中2条日语截断；1590能力输出严格正确1285，9条截断计错，无块基线中9/10、英10/10、日10/10；全部只作本轮受干扰记录，不签发文字效果或无损结论。原936注入/随机请求和12条注入能力诊断not_run，不补样。

[540项盲评准备](C2-JUDGE-TEXT-BINDING.md)的11源码静态检查、13项边界测试和17项机械核验通过，但因资源冲突永久保留为未发送准备；没有新增API费用。已准备[原样重采候选](C2-TEXT-RETRY-02.md)：同2073输入/ID/顺序、同原生探针、全新目录，另加只记类别的编译/测试资源监测；必须用户另批新40分钟窗口，不能自动重跑。原件不覆盖，不比较两轮后择优。三个旧GPU窗口均已结束并通知恢复runner。原始输入/输出/日志/向量/评分仅放私有证据，主仓只放报告、汇总和索引。PR保持draft/needs-user，整轮未完成。

### 运行前准备检查（保留边界）

- 新C++封装 `/W4 /WX /O2 /fp:precise` 编译通过；这不证明GPU回调已可用。
- 7个Python源码Ruff、strict mypy通过；3个离线边界测试通过（场景分区/条件计数、盲参照无状态块、等总预算/退化向量拒绝）。
- CPU分词核对8091条完整提示，最长120 token；75个能力诊断ID已固定。GGUF实际分词与GPU全部not_run。
- 最终包独立机械核验28项及补充31项无失败：输入、分区ID、哈希、公开协议字节、编译绑定和既有分词记录匹配；不据此宣称语义判据或GPU验证通过。私有原始文件的CRLF和提示模板末尾空行使常规diff空白检查报警，原输出已保存，原字节未为消除报警而改写。
- 运行前草案保留为私有c2-01/c2-02；当前c2-03补齐混合等范数随机控制，并将仅更换物件的留出材料改为不同事件结构。修改发生在任何模型输出之前；根ID不重合的机械核对不代替语义独立性判断。完整文件、哈希与固定证据提交见[C2证据索引](C2-EVIDENCE-INDEX.json)。
- 保留首次默认Python缺少ruff/mypy、首次构建记录路径错误，以及两轮类型检查失败。前者改用已存在的项目解释器，未安装工具；后两者只修路径和类型注解，未改评分口径。
- 主仓main核对基线1ae28013→2e69d10e包括#149其他任务证据脱敏和#147向P5-54追加重算简单性审查结论；P5-31及本卡规则、样本、引擎、模型均未变；无需为此重跑旧GPU/API。本次新组件另做自身检查，旧研究实际基线a7e8ae87不被冒改。

## 上一阶段已完成的分析与历史边界

2026-11-10按AGENTS第6节与#148完成证据迁移/隐私处理：[迁移说明](EVIDENCE-MIGRATION.md)、[逐文件证据索引](EVIDENCE-INDEX.json)、[公开背景聚合](results/background-summary.json)。原始证据及未脱敏版本位于私有`nanake-lab/nemu-evidence`固定提交`67bd1831d34cbc533ad354d46513b7ad56c1100d`的`R-STEER-01/`；原始字节与研究结果不变，不改写历史。

**部分完成：用户已核对FULL-02固定随机20及全部3条invalid并批准正式分析；冻结分析一次退出0，结果C1_semantic_insufficient_stop。** 泛情感、事实控制与反序重复一致性达到预注册门槛；三语目标候选综合门、自由输出门均失败。本轮完成了测量接口材料的独立评分与正式校准分析，没有证明四方向有效，也不能据此选层、定剂量、判断混合/文字路线或开启非零m。C0_insufficient_stop、C1_insufficient_stop原题/分/阈值不变；四方向、层强度、混合、文字干预、厂商人格/Base行为、身份投影均not_run，原三个研究问题仍无方向层面实测答案。

交付：[正式分析与门槛明细](FULL02-ANALYSIS.md)、[固定种子20条原输出/评分并列](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full02-review-01/REVIEW.md)、[全部3条invalid原文](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full02-review-01/INVALID.md)、[最终采集摘要](results/ds-full02-complete-summary-01.json)、[完整最终原件](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-full-02-complete-invalid-01/raw-filemap.json)。seed20261110、全504均匀无放回、原20 ID未换；三条invalid没有被抽中，另全部附出。两轮小样及FULL-01七次只作方法材料，不复用其分数；已获准保留的同一FULL-02评分逐件SHA携入。材料与已见小样有重合、72条是反序重复，不冒充新留出集或504独立场景。

累计预算上界**US$1.483366224**，包括全部旧费用、两份未知和成功重发整额保留；553次记录尝试/551完整HTTP响应/2份历史未知，不是账户账单，未查余额。FULL-02的504份完整响应usage峰值为US$1.362065544；reasoning已在completion内，不重复收费。最终全段主动时间2062.055271秒，低于90分钟；本轮无GPU工作、未沿用旧窗口。

三条invalid分别是顶层schema不符、完整length/空评分、引证门失败；均按原分母失败处理，不修分、不重发完整响应、不增加8192预算。预注册字面混合线索候选组0/17 invalid、其他3/487，未观察到向该代理组集中；但没有独立真实混合效价金标准，不能据此判断真实集中性或因果。[正式四门与完整结果](FULL02-ANALYSIS.md)已按原分母计算，未改题、评分或阈值；3条invalid使中共情、英共情和英好奇的自由效应不可识别，不插补或删分母。

用户指出的小样第9/10复述事实编码与效价不一致、第18失落语境缺联结负向保留，不提示评分模型纠正、不改口径。单一外部评判模型仍有构念偏差，技术合规不是语义正确。模型实际返回均deepseek-flash、指纹一致；官方文档版本DeepSeek-V4.1-Flash-1017，不是不可变权重快照。

[采集谱系/费用与复算入口](FULL02-PROVENANCE.md)保留全部失败与停止：旧high未知、FULL-01枚举缺口、弹窗与创建后隐藏失败、IncompleteRead及未保存partial的缺口、完整空评分误归段级停止、静态/编码/准备失败。原终态不倒改。分类修复只恢复用户已授权的完整非法单项继续规则，提示、high、8192、评分阈值及原分母均未调。KEY仅一次性不回显交接至进程环境，不入研究文件/日志/命令行/仓库；用户给出后未再索取。

[最终独立机械审计](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-length-final-audit-01/raw-filemap.json)各阶段一次退出0，核对源/请求/携入/全部结果/逐字证据/账/终态/样本与清单，没有计算语义门。全数据清单SHA256：8e331da96f76feba9a0f0c3773f7d1c3b251f6a3c4b5df74efed672870bcf648。[用户批准](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full02-formal-analysis-01/sample-review.json)与[正式分析](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full02-formal-analysis-01/formal-analysis.json)已保存；全部504原件摘要由分析器逐一核对，正式输入不变。报告后按用户要求停下等待，PR #144保持draft/needs-user，不合并、不领下一卡。

[此前采集提交前核对](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/results/ds-full02-final-check-01.json)：52源不变，6325份新增归档还原与源相同，195份新增JSON可完整解析，241个相对目标存在；新增原文/解压内容密钥格式检出0，范围仅本报告目录和请求入口。评分主管已退出。

[本次正式分析收尾核对](results/ds-full02-formal-final-check-01.json)：8份新JSON可解析、205个相对目标存在、23份执行/批准/独立复核原件gzip还原与缓存逐字节相同；范围、diff检查及含解压内容的密钥格式检查通过。未改研究程序，无新增API/GPU/费用，最新main仍a7e8ae87，无需额外merge或重复旧测量。

## 发散的路线与取舍

以下是执行者候选或既有约束，不写成用户已决定的答案。保留备选不授权临时换路线。

| 路线 | 值得考虑之处 | 取舍及理由 |
| --- | --- | --- |
| 原生整层残差、单层均值差方向 | 已有只读通路，最少新机制 | 首选研究路径；当前载体重新提取，旧向量不复用 |
| 中部层扫描 | 第15层从未选定 | 候选零起算8–23层，覆盖中间一半；是执行者探索范围，不声称最优或应正式采用 |
| 全32层扫描 | 可能找到中部之外的有效层 | 留作后续；先有界探索，失败不等于全模型无方向，扩围须新预注册 |
| attention残差后 | ADR0031备选，可能减少FFN变化影响 | 保留；原生通路验证失败后交用户决定，不同轮混扫两位置 |
| 所有位置 / 只末位置 | P0-10已测二者不同 | 主线固定原生全位置，不把掩码变成找通过的旋钮；提取末位置与注入全位置分开记录 |
| 多层注入 | 单层可能不能覆盖四方向 | 单层优先；逐层预算与总预算另冻，报告后由用户决定，不自动扩层 |
| 三语等权合并 / 分语提取 | 前者贴近§9.1，后者可诊断迁移 | 主候选等权均值差；各语言分量只诊断，不因某语好看换方向 |
| 泛正负情感、礼貌方向 | 四概念可能只捕捉共同情感或客服话术 | 后续必要控制；四概念×四指标串扰矩阵，不能以总体积极替代各概念 |
| 等范数随机 / 配对打乱 | 区分概念与一般扰动、句式 | 后续必要；多个固定随机种子全部报告，不挑有利对照 |
| PCA、白化、训练探针、移除共同情感 | 可能提高可分性，却增加研究者自由度 | 留后续；本轮简单均值差，失败不在同一题上挑变换，也不强行正交化 |
| 平衡代码的表达选择 | 实际行为可核对，仍可能有代码偏差/饱和 | C0交叉两字母表×映射×顺序×共同附语；保留真实未约束INVALID，不mask |
| 无标签候选续写似然 | 避开代码，但有长度/词频先验 | 后续独立接口；总/每token似然都报，二候选softmax非自然语义概率；C0失败不自动切换 |
| 自由回复+独立盲评 | 能核对自然输出中的概念与混合连贯 | 必须保存；独立评分未安排，不以词频签发语义结论 |
| 载体自评 | 本地无额外费用 | 不选主要评判；生成与判断同模型会循环偏差，若使用只能另报探索结果 |
| 外部模型 / 人工双语核对 | 独立语义判断，仍有评判偏差 | 外部调用先列模型、量、用途、费用上限并获批；人工按冻结rubric，不凭“像她” |
| 工具查询率、具体行动 | R-CHOICE有可核对先例 | 不作四个「乎」主指标；本卡测感受/表达，不把查询、服从等预设为方向真值 |
| 双/三/四通道与消融 | 线性注入不保证语义线性 | 后续六个二元对、四个三元组、四元组及逐项移除，事前全列，不能挑容易成功的组合 |
| 固定总预算 / 各自同剂量混合 | 分别隔离预算与体现组合增量 | 两者都需要；报告Σ绝对幅度与实际范数，抵消不能腾出预算 |
| 官方Base对照 | 可量化当前助手先验，但Base也可能含先验 | 保留；模板与能力公平对照后解释，不预断Base更适合 |
| 固定减身份向量 | 原生cvec可做，能扫剂量 | 只是加法候选，不是ADR0012逐位置投影，不能冒充第三问的验证 |
| 逐位置投影身份 | 贴近ADR0012操作定义 | 报告目录内最小回调探针可研究；先核对零操作/误差/清状态，读写API先例不等于已可用 |
| Base/Instruct权重插值 | P5-31列过的信息性候选 | 本轮不执行；生成新权重及量化顺序增加混杂，超出仅允许Base下载的准备，由用户另决 |
| 微调、LoRA、SAE、换引擎/模型 | 可能解决线性方向不足 | 本轮排除：改变学习机制/工具，不能回答当前冻结通路本身的可用性 |

收敛为：**C0 → 独立提取/开发/留出集 → 单方向层/剂量探索 → 留出单方向与混合 → 身份/Base及投影可行性**。Base准备与身份零值基线可独立进行，但每个实际运行块先冻结其字节输入、指标和停止条件，不一次铺开全部矩阵，也不因首轮失败偷换题目。

## 预注册

### FULL-02实际评分注册

同504请求/DS-RUBRIC-02的实际执行先后绑定144e6b1a（初70）、0c66e05d（无窗口41）、73c96dd2（有限重发156）、2319988a（分类修复238）。每段程序/请求/规则先推送核远端再跑，原停止/原件保持。

[FULL-02](DS-FULL-02.md)、[无窗口](DS-FULL-02-WINDOWLESS.md)、[用户批准的有限网络重发](FULL02-NETWORK-RETRY-01.md)、[完整invalid分类修复](FULL02-COMPLETE-INVALID-01.md)、[最终52源计划](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/inputs/ds-full02-complete-invalid-plan-01.json)。网络最多2次重发/至少30秒、完整响应不重发、三条最终技术失败停、总US$5、语义门阈值/原分母/seed20均冻结；无效结果不丢弃、不插补。两轮小样和FULL-01分数不进正式总体。完整采集后只发布20与描述诊断，用户确认全数据SHA后才准正式分析。

### 已可执行部分：C0

完整样本、主要比较、工程门、停止条件与命令见 [PREREG-C0.md](PREREG-C0.md)。执行文件：[calibration_probe.cpp](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/calibration_probe.cpp)、[fixtures.py](fixtures.py)、[calibrate.py](calibrate.py)；离线计分：[analyze_c0.py](analyze_c0.py)。462个请求全部零注入，区分“表达可不可测”与“方向有没有作用”；C0通过仍不证明构念有效或能力无损。

### 后继阶段框架（尚非可执行预注册）

下面是推荐，未给数字的量为待实测/待冻结。**没有该阶段的完整字节样本、分析入口和推送记录就不运行。** 混合判据按用户要求依据独立单通道结果制定，不能现在捏造保持比例。

| 阶段 | 运行前必须冻结 | 主要比较与结论边界 |
| --- | --- | --- |
| E提取 | 三语等权正负对；事实、句式、否定、长度、位置平衡；独立领域根；统一末个prefill提取 | 各语均值差再等权合并/单位化，零范数即失败；原激活保留。符号/算法不依行为题更改；语言分量、余弦、一般情感与礼貌另报 |
| L开发扫描 | 每方向同一候选层集；典型‖h‖只由提取/校准样本算；β/‖h‖网格、Σβ上界、数据与预算的数值先定 | 正负双侧及0；随机、打乱、错概念方向；层×方向×语言全曲线与能力损失。开发题只能选层，不能发布资格 |
| T留出检验 | 全新领域/实体/表述根；采样种子；最小效应、各语单调容差、能力非劣容差及多重比较方法在看结果前冻结 | 配对零值及每个等范数随机；按情境根聚类区间，三语分别、全剂量相邻差、实际输出与串扰。探索最佳层不是确认性证据 |
| M混合 | 从独立单通道开发结果制定保持/抵消/压倒/不足的差值与区间规则，推送后再看新混合题 | 双/三/四通道全组合；每通道同剂量及相同Σβ两种控制；单通道、完整混合、逐项移除；同时测连贯/能力。单效应近0不取不稳定比值，无法识别不能叫无干扰 |
| I厂商人格 | 三语自然任务、直接身份问题、需/无需道歉、需/无需披露、合成多轮分别冻结；事实p固定 | 当前m=0、Base同格式、当前投影干预；另有随机投影、假操作、单层/全层消融。身份减少还须保住事实身份、诚实披露、能力、连贯，不回应不是成功 |

候选选层规则：开发集排除三语无方向性、非单调或能力超容差的层；有共同可行层时，先比较最小三语效果下界，再按能力下降最小、Σβ最小、较早层作预定机械平局规则；无共同层则列各方向可行集合，报告多层备选交用户决定。效果尺度、界、网格、容差及是否采用此顺序须后继运行前冻结；**当前没有选出层**。

概念/文风区分：同概念用不同表面实现，不同概念匹配表面形式；共同礼貌附语只是一个有限控制，另需长度、词汇、一般正负情感、错概念和随机方向。只有目标效应跨三语/表面版本重复、强于一般扰动且未被共同情感解释才有支持。文字自述、词频或激活可分都不足，独立语义和连贯评分缺失时只能判不足。

后继盲评的 [语义口径候选](SEMANTIC-RUBRIC.md) 进一步分开概念证据、一般情感、连贯、事实能力与厂商人格；缺失/中性/无法判定分别保留，不造“越不像助手越好”的口味总分。它是设计交付，具体样本、标注者与数值判据仍须后继运行前冻结，当前没有人工评分。

能力：C0的48次oracle只是4个简单根的接口检查。后继能力集独立覆盖计算、规则、事实忠实、约束格式、三语理解，INVALID/拒答/到上限全部保留；报下降区间，不以“不显著”宣称无损。当前无外部评判授权，不调用，也不让载体自评签发通过。

### Base与身份操作

2026-11-08只读官方接口确认 [Qwen3.5-4B-Base固定修订](https://huggingface.co/Qwen/Qwen3.5-4B-Base/tree/1001bb4d826a52d1f399e183466143f4da7b741b) `1001bb4d826a52d1f399e183466143f4da7b741b`，Apache-2.0；两个权重片共9,319,828,056字节。不是当前模型851bf6e8修订。实际下载/派生哈希归档后才能运行；同P0-12 CPU转换器 `--outtype bf16 --no-mtp`、CPU量化器 `Q4_K_M 4`，视觉产物如准备则同 `--mmproj --outtype f16`，文字测量不加载mmproj。

Base对照候选用同一合成事实p/任务，在共同聊天格式和共同纯文本续写格式下组成模型×格式2×2。不存在适合Base的官方chat模板时明确不适用，不加性格提示、不挑有利few-shot；报告模板、长度与失配。Base不响应只支持该配置结果，不能因少说“助手”就称人格更干净。[Assistant Axis研究](https://www.anthropic.com/research/assistant-axis) 在所测Base也观察到相关方向，并发现远离助手方向可影响身份稳定；不能外推为当前4B已证实。

官方固定修订的 [Base说明](https://huggingface.co/Qwen/Qwen3.5-4B-Base/blob/1001bb4d826a52d1f399e183466143f4da7b741b/README.md) 明确它是仅预训练权重，面向研究等用途而非直接交互；同时说控制token在训练中见过。因此“含聊天控制token”与“已经会稳定按聊天指令回答”必须分开核对。已下载config的text_config与当前载体逐内容相同，含32层、2560维、每第四层full_attention；[配置对照](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/base-config-comparison.json)只证明配置声明相符，不证明权重、模板行为或引擎运行相符。后续层扫描还应分列full/linear attention层型，但不能把层间差异直接归因为层型因果作用。

ADR0012的操作是 `h' = h − u(uᵀh)`，系数随每层每位置h变化；固定 `h' = h − αu` 不能替代。[native_probe.hpp](../../spikes/carrier-engine/native_probe.hpp)只安装固定加法；[activation_trace.hpp](../../spikes/carrier-engine/activation_trace.hpp)提供张量读取/写回先例。更好的办法是在报告目录做最小投影探针、先核对零操作和数值通路，再测行为；若位置/布局/回调不支持，保留失败交用户决定，不改引擎。身份提取只用第一人称事实陈述材料，不写扮演人格的提示，不把拒绝方向当身份方向。

人格按分母分层：无诱导任务中的品牌/助手自称；身份问答与事实p冲突的品牌声明；无需道歉时无根据道歉；模板化服务套话（冻结rubric独立评分，普通礼貌不自动算）；多轮编造身份/经历或事实矛盾。真诚问AI身份时正确披露、恰当道歉不罚，不追求一律0次AI。多轮逐回合带全上下文重新prefill，跨条件不复用旧状态；只用合成对话，不读她的H。

## 结果

### FULL-02正式四门（2026-11-10用户核对后）

[完整正式分析、阈值和证据](FULL02-ANALYSIS.md)。中/英/日目标符号分别61/64、55/64、59/64；目标事实且连贯57/64、58/64、59/64；各概念还须≥14/16，中目标质量差1、英符号/共情/成事不足、日共情12/16，故目标综合门三语均失败。泛情感16/16、15/16、16/16与事实16/16三语通过；反序重复的概念/事实/连贯总体66/72、71/72、68/72且各语达到门槛。

自由输出质量24/48、29/48、40/48均低于44/48。联结/成事的事件条件差在三语达到≥0.5，但共情中英不可识别、日为0；好奇中为0、英不可识别、日0.5。所有自由门失败。这是零注入下事件条件/输出与评判的校准，不是m或文字状态的因果效应，不能挑通过的格子宣告接口可用。

本次随机20已知错误照原分使用：第10“沮丧”漏成事负向；第20从“之后没再见面”事实编码联结负向且效价中性；第3“陶瓷碗”误标新增关系/亲历；第2同事焦虑判faithful略宽。用户认可评分可信不等于每项正确；单评判器、仅8根、既见材料/重复与事实混杂均限制外推。不修分、不重评，C0/C1原结果不变。

### FULL-02：正式语义分析完成，按注册停止

501条rating、3条invalid、0条最终技术失败；3条invalid均来自自由续写条目（含重复总体），中文1/英文2/日文0。技术校验不等于语义正确。最终20的完整字段、原文证据、对象、事实参照和理由在[并列页](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full02-review-01/REVIEW.md)，三条invalid未抽中而在[附页](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full02-review-01/INVALID.md)完整展示，没有换样。

| invalid ID | 原因 | 原响应处理 |
| --- | --- | --- |
| 26867ceb738bf2198db469d3 | 顶层schema字段不符 | finish_reason=stop，原JSON保留，不修复/重发 |
| 3bffc4de3ca549e250b0327c | 完整length、8192输出全为思考、评分正文空 | HTTP/进程正常；旧fatal保留，新目录计invalid，不增额度/重发 |
| 90ff09af0af4204419938de8 | 逐字引证门失败 | finish_reason=stop，原JSON/引证保留，不修复/重发 |

三条按冻结规则在原分母记失败；重复对任一失败计一致性失败，自由符号单元含失败则不可识别/不通过。[正式分析](FULL02-ANALYSIS.md)已执行四门，结果C1_semantic_insufficient_stop。固定混合字面线索候选组0/17、其他3/487；同评判器返回mixed的13项均非invalid，但这属于内生标签，不能作为独立真实效价真值。小样9/10、18已知偏差不因技术校验而解除。

| 预算账组成 | 峰值上界US$ |
| --- | ---: |
| 正式504完整响应的usage | 1.362065544 |
| 既有方法材料48次（含旧high未知） | 0.100000992 |
| FULL-02旧IncompleteRead整额预留 | 0.011862300 |
| 成功重发按整额保留超出其usage部分 | 0.009437388 |
| 合计（非账单，总上限5） | 1.483366224 |

504响应prompt645286、completion1104912（其中reasoning909389已包含）、total1750198；全部返回deepseek-flash/指纹bc19ba97cbe016479903222bb1578db6。按UTC与公开峰值费率记账，没有账户查询。FULL-02物理505次尝试/504完整响应/1历史未知；加方法材料后553/551/2未知，原未知未销账。成功重发只有原IncompleteRead那条一次，已有完整响应没有重发；本轮238没有新断流，30秒规则仅离线真实测量触发，未冒称外部实测。

[原始最终6290文件](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-full-02-complete-invalid-01/raw-filemap.json)、[发布命令/退出码](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-length-review-publication-01/raw-filemap.json)、[终态/样本独立审计](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-length-final-audit-01/raw-filemap.json)。全504恰一最终结果，全部原件/副本与清单SHA一致。数据清单未改。用户于2026-11-10核对新20及全部invalid后批准正式分析；[运行记录](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full02-formal-analysis-01/execution.json)退出0，无新API/GPU或费用。

### FULL-01方法材料：7响应/6合法/1非法，原停止保留

[摘要](results/ds-full-stop-summary-01.json)、[全部原始记录](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-full-01/raw-filemap.json)、[独立机械核对](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full-audit-01/raw-filemap.json)、[第3项原输出与非法枚举](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full-failure-01/FAILED.md)。实际UTC2026-11-09 16:31:39–16:32:11（Perth2026-11-10 00:31–00:32），API请求ordinal3的匿名ID为b0734b8fd8be8df908569bf9，原输出“相手を思うと、隔たりを感じる。ありがとう。”，原效价label=both。原DS-RUBRIC-01没有mixed字符串，校验器第121行只接受mixed等五值。冻结的逐字证据/格式门照常拒绝，不修JSON、不映射、不复评；坏例没有rating.json，原content另明确标为unvalidated展示。

失败观察UTC16:31:59.480850之后新增reserve为0；此前已在途的项依注册收集到终态。7份HTTP/子进程正常，6份合法评分完整保存；停止标签item_failure，原主程序result不改。497未发；全量随机20、正式数据清单、用户核对门及C1正式分析都未运行，不计算6份的“通过率”替代语义门，也不将技术格式失败叫C1语义失败。

本轮prompt8655（hit5760/miss2895）、completion16792（内含reasoning14018），7次用量均结算，新增US$0.021053460；加原US$0.078947532为US$0.100000992峰值上界，非账单。实际model全deepseek-flash、fingerprint均bc19ba97cbe016479903222bb1578db6；文档版本DeepSeek-V4.1-Flash-1017，不冒称日期快照。未查询余额或账号。原未知调用费用继续完整预留，两轮小样不作正式统计。

[本轮准备/源快照](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full-preparation-01/raw-filemap.json)、[静态/6单测/PS AST](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full-checks-01/raw-filemap.json)、[离线fixture](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full-offline-fixtures-01/raw-filemap.json)均保留；6测试0.374秒，覆盖4并发、预留、首次失败停止、抽样和核对门保护，但没有发现提示缺mixed枚举这一覆盖缺口。执行器的停止机制工作正常，不能据此称评分接口完整。新[候选一句](VALENCE-SCHEMA-PROPOSAL.md)仅补既有valence枚举说明，不给失败文本或点评；它会改变实际提示字节，需用户批准和新阶段冻结，不能改本轮原始状态。


### 方法材料：high剩余4项完成，原20对照

[续测原始记录](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-resume-01/raw-filemap.json)、[独立审计](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-resume-audit-01/raw-filemap.json)、[全部20项与逐项来源](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-review-02/REVIEW.md)、[派生汇总](results/ds-flash-high-complete-summary-01.json)。本轮实际执行UTC2026-11-09 15:34:59–15:36:00（Perth23:34–23:36），主管有真实终态，4次HTTP完整响应/评分、4子进程exit0且未触发墙钟上限；原16份评分及旧非思考20份评分没有重评或覆盖。API全为deepseek-flash，fingerprint均bc19ba97cbe016479903222bb1578db6，官方文档版本DeepSeek-V4.1-Flash-1017；没有另一个不可变日期字段。

新4项prompt4949（hit3712/miss1237），completion8473，其中reasoning6934；峰值上界US$0.010560972。已知high20合计prompt24689（hit17280/miss7409）、completion44482（含reasoning36851）；非思考20的completion7851。额外生成量是实际代价，不保证更可信，也不把这两段外部服务时段当严格延迟基准。

| 总账组成 | 峰值费用上界US$ |
| --- | ---: |
| 非思考20已知usage | 0.011447052 |
| 原high16已知usage | 0.045143808 |
| 续测high4已知usage | 0.010560972 |
| 原第17未知尝试整额预留 | 0.011795700 |
| 总计（41次记录尝试，40完整响应，1未知） | 0.078947532 |

各次先预留后结算，reasoning已在completion内，未双重收费；没有查询账户余额/账单。第17再次尝试是用户明确允许继续后的单独计次，不把它伪装为原调用恢复。后3个原not_run本轮才执行；原中断阶段仍如实保持未完成。504全量未运行，20小样不并入正式门。

[工程诊断](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-engineering-01/raw-filemap.json)没有找到历史触发者：有界系统日志无匹配崩溃/重启，五组330秒无KEY/无网络寿命探针正常完成，五分钟清理假设未复现；全部in_job仍为true，不声称脱离全部嵌套job。用户明确允许在历史原因未知时继续。修复是独立主管接过进程内KEY、逐HTTP子进程按父monotonic墙钟结束、记录PID/心跳/退出，主代理同时查主管存活。回环保活0.6秒截止与异常exit7的真实时间测试通过，原假token测试及失败日志保留。

[首轮检查](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-resume-checks-01/raw-filemap.json)3项单测/PS AST通过；测试源码未用json导入、HERE间接导入各造成一条Ruff/mypy失败，原样保留。只修导入后的[窄复核](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-resume-checks-02/raw-filemap.json)通过；准备清单补入复用的排版源后[该文件静态检查](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-resume-checks-03/raw-filemap.json)通过。源/4请求在调用前冻结推送，合并排版逻辑也事前冻结。只用标准库与现有同一Python3.12.10基础解释器，不安装依赖、不改全局设置、模型/引擎或原探针。

代开入口的首条组合命令曾被自动审批拒绝（blocked by policy，无更具体原因），未执行也未调用API；用户随后明确要求“直接打开我输入”，仅打开冻结入口的较窄命令成功。原拒绝与新接受分开保留；输入窗口交接后退出、主管继续运行，结束后所有本轮PID均不在，没有KEY持有进程或密钥文件。启动阻塞已解除，不再要求用户操作旧手动入口。

**待用户抽查的边界：** 第3/5/8/10旧疑点标签均变化；第18联结由negative转absent且新增动作被另标亲历，第20“成事满足感”仍被判unsupported，是否把感受与事实混淆需按冻结口径核对。全部20的22字段、对象、方法猜测、原文证据和理由都展示，不只展示变化有利的项。标签一致数/转换是描述统计，不能替用户确认可信度或签发C1语义门。

独立机械审计的冻结、账目/进程、合并页三阶段各运行一次、全部退出0；15源/4请求、4响应/评分的逐字证据、逐笔费用及终态一致，旧未知预留独立重算仍保留。旧非思考60份、旧high48份项目文件及账本与原归档一致；新页68份拷贝、20对440个标签行和全部转换匹配，原high终态仍缺失。缓存中的独立审计脚本415行，是为将冻结、三轮账本、4请求/进程和20组归档/排版核验放在一个可复算入口；正式执行模块均400行内。写该审计脚本前一次工具JavaScript包装语法错误未启动shell或审计，原报错和UTC缺项留档，没有用重跑审计掩盖失败。以上只是机械记录一致性，不作语义可信度判决。

### 原high阶段的中断（原样保留）

完整[派生汇总](results/ds-flash-high-summary-01.json)、[逐项审查/标签转换](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-review-01/REVIEW.md)、[原始82文件归档](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-01/raw-filemap.json)、[独立机械对账](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-audit-01/raw-filemap.json)。第17条`8be4a42ea150b56ddf567799`只有请求与预留，无response/rating/结算；后3条ID列在汇总，not_run。预留在HTTP调用之前写入，不能单凭预留证明请求已送达。原`run/result.json`与`entry-result.json`没有补写；派生recovery-summary明确标注非原始终态，结束时间/退出码为null。

已确认响应窗口UTC2026-11-09 14:47:25–14:51:40（Perth22:47–22:51）；首次发现进程缺失UTC15:00:24。此前只观察账本更新、没有连续核实进程存活，所以不能把文件停更时长当作持续等待或精确退出时间。socket180秒不是总墙钟截止；[官方文档](https://api-docs.deepseek.com/quick_start/rate_limit/)允许非流式空行保活，但没有另采本条流，不能认定它是本次原因。原注册中在途最多额外180秒的预期未得到可靠保证。没有网络/模型失败结论，也不归因用户关闭窗口；后续应先独立验证总时限与进程终态监控，再考虑新的实际调用。

16响应均`deepseek-flash`、指纹`bc19ba97cbe016479903222bb1578db6`；文档映射版本DeepSeek-V4.1-Flash-1017，实际API无独立日期快照。prompt19740（hit13568/miss6172），completion36009，其中reasoning29917；按总completion计费，不再加一次reasoning。新16结算US$0.045143808；未知第17保留US$0.011795700，加旧20 US$0.011447052为US$0.068386560上界。没有查询账户/余额，实际总扣费未知。相同16条的旧非思考completion6259，新的总completion36009，额外生成量是实际代价，不证明正确性。

| 已见疑点 | 旧非思考 | high原标签（不代表执行者裁定正确） |
| --- | --- | --- |
| 第3条“修好了，但成否未知” | 事实faithful，无矛盾 | 事实contradicted，矛盾yes |
| 第5/8条纯主观高兴 | 事实unsupported | 事实faithful |
| 第10条仅叙述研究行为 | 好奇positive | 好奇absent |
| 第9条截断比喻 / 第13条主观难过 / 第15条宽慰 | unsupported / unsupported / contradicted | 均faithful，需结合全部原文和参照抽查 |

所有22字段标签转换及原文证据都保留；无准确率、通过阈值或“更像她”分数。16份结构/引证核验合规不等于语义可信。独立机械审计退出0仅说明现存记录对账一致，明确判运行终态缺失，不能写成研究执行通过。原消息、11源、20冻结请求与49份展示拷贝字节一致；全部旧结果保留。

冻结展示程序假定正常终态，本次保留原文件，另用[故障后展示适配](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds_high_interrupted_review.py)读取明确标注的派生汇总；只改终态读取位置/标题/解释，比较字段、计数与原评分未变，差异记录在原始归档。该新文件[静态检查](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-high-checks-03/raw-filemap.json)Ruff/strict mypy退出0；终端临时查看曾因GBK不可编码退出1，之后改UTF8展示，失败观察保留，未重新评分。无产品实现/正式配置/方向改动，未跑全工作区CI/Windows或GPU。

| 部分 | 当前状态 | 结论边界 |
| --- | --- | --- |
| 原文、探针只读核对 | 已完成 | 15层是占位；固定加法≠投影；旧R-CHOICE失败不代表当前四方向失败 |
| C0零注入接口 | 实测完成，测量门失败，按预注册停止 | 当前接口不足以进入方向试验，不判方向失败 |
| 四方向、层/剂量、三语、能力 | not_run | 不能推荐具体层、β或正式可用方向 |
| 多通道混合及连贯 | not_run | 不能称无抵消/压倒或能力无损 |
| Base准备 | 首次900秒超时（curl28）；已批续传约24分56秒后连接提前关闭（curl56），新partial1,179,954,622字节；转换not_run | [首次失败](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/base-download-01/raw-filemap.json)、[续传原始记录](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/base-continue-02/raw-filemap.json)，不是模型不兼容结果 |
| 人格与投影 | not_run | 没有Base行为优势或身份移除成功结论 |
| 独立语义评分 | not_run | 自动槽指标不足以完整区分概念与文风 |

### C0：完整执行与未通过的门

实测head为`98d716346ab34daf983567f2d6db49256b6b6622`，main基线仍为`cb95af4d`，与预注册/输入/可执行绑定一致。[完整结果](results/c0-summary-01.json)、[外层命令/时刻](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/results/c0-launch-01.json)、[授权记录](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/results/authorization.json)、[原始数据映射](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/c0-run-01/raw-filemap.json)。外层2026-11-08 Perth 22:02:27.343–22:05:14.641，共167.298秒；其中原生流程墙钟158.640秒，包含模型启动、逐请求检查与写盘，**不是GPU忙碌时间或性能基准**。462请求按固定顺序完成，无重试、换题、换门或剔除；每请求清attention/recurrent并完整prefill。退出后未见残留探针。

下列分母包含重复布局，只描述本批固定材料；表达场景根只有8个、oracle规则根只有4个，不能把表中请求当作独立情境或广泛语言能力样本。

| 语种 | 表达首token合法 | 表达mass≥0.95 | oracle真实正确 | oracle mass≥0.95 | 动态/布局格满足数 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 中 | 128/128 | 128/128 | 14/16（87.50%） | 13/16（81.25%） | 1/8 |
| 英 | 128/128 | 128/128 | 14/16（87.50%） | 14/16（87.50%） | 3/8 |
| 日 | 128/128 | 123/128 | 11/16（68.75%） | 5/16（31.25%） | 3/8 |

三个语种的oracle正确率均低于事前0.90门，oracle代码质量也未过门。48条oracle共39条槽内正确，9条错误中2条为真实argmax不在合法集合（均是日语阈值题），没有用二代码条件概率把它们修成成功。这两条实际输出都为数字`1`，也是规则所对应的数值答案，但接口要求的是A/B代码，故仍计INVALID；不能把表中68.75%称作日语理解准确率。表达格式合法不等于客观答案接口可用。

24个概念×语言×附语格要求全部满足，实测仅7格。48个根×语言×附语组中43组有中间概率中位数，但只有11组的8布局范围≤0.20；本次主要问题包含布局不稳，不能只概括为天花板。最大范围为中文empathy-1无附语的0.922960，即同一场景只换代码/顺序/字母表，q跨约92.30个百分点；中位数0.527035并不能使其成为可靠的“居中题”。全部逐布局值保留在结果文件。

客观题也显示这一混杂：7+8题在三语的两种代码映射中，正确选项列在后面时六条全错，列在前面时六条全对。这只是当前槽/提示/模型组合的观测，不足以归因为模型不会加法、某个字母偏好或某一种单独机制。没有用结果把题改成另一个接口后重跑。

三语首尾各一对248320词表完整float32 logits字节相同。24条自由回复全部由`llama_vocab_is_eog`判定生成结束、无64-token截断，UTF-8均有效、均无生成的think标记；这里核对EOG，不把它冒称为已逐项验证某个指定EOS token ID。它们没有得到独立语义/连贯评分。一般请求按预注册只存候选logits、logZ、argmax及top12，只有六个guard存完整词表，因此不能声称其余请求的argmax/logZ已从全词表独立复算。

### 自由回复的探索性阅读（非独立盲评、非人格专项结论）

未提供好坏的“朋友发来近况”场景，中文生成“最近过得挺充实”，英文生成“I'm so sorry to hear that you're going through this”。二者都补进了未给出的处境方向。这里的sorry是对他人处境的慰问表述，不能仅按词命中自动计为无故认错；它也不能作为共情方向已被调制的证据，所有请求都是零注入。

英文星图输入已说newly unfolded，回复却说waiting to be unfolded；另有叙事续写、重复输入与补充未给事实的例子。说明“下一句回复”的题型仍有任务/语义歧义，后续必须同时核对事实与自然表达，不能只让选项槽独自承担概念测量。执行者在这24条中未见品牌或AI助手自称，但这不是身份专项样本，没有Base比较，不用于估计总体流露率。

原始2322文件共28,665,151字节，按P0-12方式去重为1421个gzip对象、3,595,391字节；缓存原文件不删改。[独立机械复核](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/audit-c0-01/raw-filemap.json)逐文件解压后与原件字节、长度、SHA256全部相同，压缩对象摘要也一致。432个选择项的q/mass、24格布尔与主汇总均无差异；六份完整词表的argmax/候选logits/logZ可独立复算，其余只核对保存的摘要与top12。原生日志记录`offloaded 33/33 layers to GPU`，摘录及未匹配到error/fail关键词的记录在复核证据中；关键词无错误不等于功能验收通过。

证据边界：运行前冻结入口会对55个引擎文件逐项重哈希并与P0-12核对，入口已完成；但没有把该时点的55项摘要另存到binding，只能指出源码门控与成功执行，不能称为独立的运行时逐项快照。PREREG-C0及续传候选里“尚未收到/未批准”是冻结时的状态，保留运行前字节；本轮实际授权与执行以results和本报告为准。

### Base续传终态

严格执行用户批准的同URL单次续传候选，先复制到新目录，C0退出后才启动。Perth 2026-11-08 22:06:43.194–22:31:39.156，外层1495.962秒，curl退出56：`schannel: server closed abruptly (missing close_notify)`；未用满30分钟上限，不自动使用剩余预算重试。原始partial仍为869,961,948字节、SHA256 `f146443d…57fdd`；新partial为1,179,954,622字节、SHA256 `9ddad636…e04d62`，已核对新文件的旧前缀与原partial相同，两份均保留在D盘。

[完整摘要与路径](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/results/base-continue-binding-02.json)、[实际命令/时刻](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/results/base-continue-launch-02.json)、[日志/脚本原字节归档](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/base-continue-02/raw-filemap.json)。新文件未达到官方完整首分片5,329,398,712字节，不能通过其完整SHA256，未当作可加载模型；另一分片、完整Base转换/量化/运行均未完成。本卡没有改下载来源、模型、全局网络设置或工具链，也没有将连接错误解释成模型兼容性失败。

### C1续研阶段（2026-11-09，Perth）

用户批准原卡内重设计接口，随后增加“把状态写成文字”对照。新阶段的 [C1预注册](PREREG-C1.md) 先收敛无代码续写LL与独立语义盲评，冻结全新8个合成根、三语1686条零注入请求；[文字对照](TEXT-STATE-CONTROL.md)冻结两措辞×两位置、弱中强/正反向映射，以及状态外露、表达升级、厂商腔的可观察分类。文字强度与注入没有共同单位，只比较方法内单调与固定序位曲线/能力代价，不事后匹配成“同强度”。两组后续共用题、采样、能力集与独立评分。没有性格、风格、语气样例或扮演提示，正式是否采纳留待用户另立ADR。

模式B的J_text作为备选保留并排除本轮：它依赖模式A按m生成的样例库，不能用人写示例代替。评分者只见事实/任务和输出，不见状态块/方法/剂量；输出自己复述状态造成的盲法泄漏要计数，不能删去。比较尚未运行，不能回答文字和注入谁更好。

接口先测内容动态范围、事实顺序与形式偏移、长度/礼貌/文本先验、事实复制控制、三语首尾logits；独立表单同时核对四概念、泛情感、事实与连贯。锚事实改变导致续写LL变化不是方向生效，连续增益也不是概念概率。C1自动门和独立语义门都过后，才准备C2全新提取/开发/留出、层/剂量/随机控制与两方法比较的完整预注册；本次不先提取方向或用C1挑层。

C1运行前准备：完整1686条实际样本SHA256 `8d9b8face032dbfead47df5cde8d73c7cba3fb580bf5257000ef2b9e5e1ee6b3`；[执行源/输入绑定](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/inputs/c1-preparation.json)，实际运行缓存`D:/Dev/cache/R-STEER-01/c1-run-02`（01保留未运行）。Ruff、strict mypy（5文件）、4项数值/配对单测通过；CPU tokenizer预检1686条，最长prompt97、continuation20 tokens。原生`/W4 /WX /O2 /fp:precise`编译通过，exe SHA `8deda88442ff33bd07deb98c88c71b215f980dd3dae4ab3b17b74c28b1c65fd3`；准备检查没有加载模型。首轮mypy四条类型错误原样保留，修复TypedDict/显式无类型包接口，未改题或阈值；删除未使用草稿端点前后实际jobs规范JSON逐字节相同。源码/构建绑定位于[最终检查](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/c1-checks-02/raw-filemap.json)、[原始失败](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/c1-checks-01/raw-filemap.json)、[构建](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/c1-build-01/raw-filemap.json)。根语言重复仍仅8个根，不以1686充样本量。

[独立提交前核对](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/c1-preflight-check-01/raw-filemap.json)：12份冻结源码、3372个输入与jobs哈希一致；五份新增归档277/51/29/22/5文件逐项与原件、解压字节、摘要一致；11份Markdown的89个相对链接存在、29份变更JSON可解析、27份手写文本≤400行，C0八份冻结文件相对旧交付`ea28ab67`无差异。检查时未跟踪C1文件单列，不能把git diff当作已覆盖未跟踪内容；提交前还检查暂存diff。机械代理有一次只读PowerShell概览ParserError和一次Windows通配路径rg错误，正式独立脚本14条命令均退出0。最初概览未保存独立原始stdout/stderr/精确UTC/退出码，只保留命令与观察转录，报告不声称已恢复其原始字节。

暂存检查发现新构建脚本末尾多一空行，已只删除该空行、PS AST仍通过，编译命令和C++未变所以不重复编译。失败原输出、前后脚本与摘要、更新后暂存检查退出0、02缓存重新冻结的12源码/3372输入核对见 [末次绑定](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/c1-final-check-02/raw-filemap.json)。题目、门、采样和exe均未改变；旧01绑定不覆盖。

旧[OpenAI评分提案](C1-BLIND-RUBRIC.md)曾申请最多504次/US$20，用户当时不批准；该方案始终调用0/费用0，冻结文件与原始`judge-decision.json`保留当时状态。之后用户新授权DeepSeek并指定Flash，已完成20小样，当前状态见总览，不再使用旧OpenAI提案。C1自动门未过，不存在仅差API就能进入下一阶段的结论。

运行后按冻结取项规则制作了[未评分盲包](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/results/c1-blind-items-02.json)：432条目+72反转概念表顺序的重复，固定SHA排序/匿名ID；[映射与预期](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/results/c1-blind-key-02.json)供离线核对，不能交给评分者。生成器[prepare_c1_blind.py](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/prepare_c1_blind.py)无API/评分功能，Ruff/strict mypy通过，[材料化记录](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/results/c1-blind-materialization-02.json)标明这是运行后整理而非新评分或替代试验。公开仓库中的映射不能保证评分者不偷看，未来盲评需隔离发包；本次语义标签仍全部not_run。

### C1实测结果

历史窗口背景现仅公开允许的聚合信息，见[背景汇总](results/background-summary.json)。原桌面/GPU进程表与未脱敏背景段已原样迁入私有证据仓库；项目探针与runner的记录仍可核对。缺失的内存/显存分项不补造，不能从程序身份推断重负载。

外层UTC `2026-11-09T05:39:00.7849733+00:00` 至 `05:55:46.6920327+00:00`（Perth13:39–13:55），约1005.907秒；原生流程998.312秒，含加载/进程检查/IO，不当作GPU忙时或纯生成耗时。1686/1686协议OK，原生/外层退出0，无自动重试。结束确认探针退出并通知恢复runner。

| 冻结的门/计数 | 中 | 英 | 日 | 判定边界 |
| --- | ---: | ---: | ---: | --- |
| 概念×语言锚差（每语4格） | 4/4 | 3/4 | 4/4 | 英文好奇失败；要求12格全过 |
| 未定条件处于锚内部10%–90% | 50/64 | 33/64 | 44/64 | ≥48；英/日失败 |
| 事实顺序偏移在冻结范围内 | 66/96 | 50/96 | 60/96 | ≥80%；三语均失败 |
| 同义形式稳定 | 23/48 | 26/48 | 28/48 | ≥36；三语均失败 |
| 加长形式稳定 | 25/48 | 21/48 | 25/48 | ≥36；三语均失败 |
| 感谢形式稳定 | 35/48 | 20/48 | 31/48 | ≥36；三语均失败 |
| 事实轮换复制，总/均LL都排对 | 16/16 | 16/16 | 16/16 | 复制控制通过，不代表通用能力无损 |
| 自由输出技术有效 | 48/48 | 48/48 | 47/48 | 均≥95%；语义未评分 |
| 两端完整首logits逐字节一致 | 是 | 是 | 是 | 每份248320个f32 |

主指标为同串先验扣除后的每token增益对比g。以下是事件正锚−负锚的根等权平均，单位nats/token；**不是方向效应**：

| 概念 | 中 | 英 | 日 |
| --- | ---: | ---: | ---: |
| 联结 | 0.974005 | 0.443656 | 0.749035 |
| 共情 | 1.240717 | 0.407879 | 0.943502 |
| 好奇 | 0.453403 | −0.207292 | 0.382535 |
| 成事 | 0.857632 | 0.634280 | 0.972829 |

英文好奇两根分别0.015746和−0.430329。按冻结公式，同一候选的prior在正负事件锚相减时消去，不能把这一项简单归因于prior扣除；构念、措辞与事实语境仍纠缠。三语事实复制都排对，而表达形式和顺序门大量失败，表明这次取消代码并未提供足够稳定的概念测量。不得挑一个形式、语言或原始LL替主指标签发通过。

决定`C1_insufficient_stop`。未改题、阈值、方向或评分；四方向提取/剂量/选层、混合、文字方法干预、厂商人格专项/Base行为及身份投影全部not_run。原C1运行当时独立语义门not_run；现FULL-02经用户批准正式分析，语义门亦不足，详见[正式结果](FULL02-ANALYSIS.md)，原自动结果不倒改。原始11824文件96,225,407字节无损去重为5308个gzip对象8,396,180字节，[完整归档](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/c1-run-02/raw-filemap.json)；[冻结分析器的全量汇总](results/c1-summary-02.json)与[分析命令](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/results/c1-analysis-02-command.json)。

[独立机械复算](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/c1-audit-02/raw-filemap.json)不导入主分析器：1542个评分请求、16890个逐token项、192 prior/1152 target配对、576个g/原始总分/均值/Gsum、12格及三语全部分母/计数与汇总一致；7条正式命令均退出0。六份完整首logits独立重算argmax/logZ/target/top12一致；其他请求只核保存的target/logZ及摘要关系，不能声称其全词表归一化或argmax已独立复算。11,824个归档原件/解压字节/长度/SHA及5308压缩对象全部一致。运行head、源码、输入、模板、55项引擎实际摘要与记录匹配；未再读大模型权重。

唯一自由技术无效是日语`c1-1234`：到64token上限、无EOG，UTF-8有效、非空、无think/special标签，末尾为`まるで静かな湖に`。原输入、输出、逐token记录在审计归档中，没有补采或删除；47/48仍满足冻结的95%技术门，不改变其他失败。GPU CSV没有嵌入采样时刻，不能拿文件时间冒称准确采样时间；窗口起止以launch/binding/exit记录为准。stderr关键词未命中错误只作日志事实，不当作行为验收。

审计脚本610行，集中保留逐token、配对/分母、全词表guard、绑定/背景及全量归档核对的单一复现入口，是缓存中的独立证据代码，非生产实现；超过400行原因在此说明。正式核验后一次只读PowerShell结果格式化ParserError仅保留观察转录，未恢复原始分流/UTC/退出码；正式数值核验没有失败或重跑。

另将12份实际冻结源码/协议/语料按运行绑定重新验证后保存[原始字节快照](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/c1-source-snapshot-02/raw-filemap.json)，13文件含快照记录；这是运行后采集且全部匹配实际准备SHA，不伪称运行时复制。Git文本可能规范化换行，离线精确核对可从这个gzip快照取原字节。构建exe原字节已在准备构建归档保留，模型/引擎沿固定哈希复用。

对失败的解释边界：同义/加长门失败不等于模型语言或长文本能力差；本指标的每token分母也会随中性附句改变，属于测量本身的长度依赖。好奇材料同时涉及学习进展和继续求知的意愿，两者不能未经独立语义核对就视为同一构念；目前没有证据把英文反向归结为载体不关心学习。保留这些排除不了的解释，不在本轮换分母或改词寻求通过。

[收尾核对](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/results/c1-closeout-03.json)：12冻结源仍匹配，新增源码/审计归档13/39文件再次逐字节核对，504份盲呈现输出与实际候选/原生成对应、语义评分0；39份JSON可解析、当时103个相对链接存在，diff/范围与C0冻结检查通过。[校验程序与日志](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/c1-closeout-03/raw-filemap.json)保留UTF-8修正后的退出0记录；此前收尾辅助脚本误用默认GBK读取UTF-8盲包，失败观察转录保留，未保留首次独立原始分流。该修正只涉及收尾检查，不改变任何C1实验程序、指标或结果。全工作区CI/Windows未运行，不能称其通过。

| 原问题 | 当前答案边界 |
| --- | --- |
| 四方向是否单调、跨语一致、在哪些层、损多少能力、是否概念本身 | 尚无方向实测。C0和C1均为测量接口不足；C1已执行并失败，不解释为四方向失败 |
| 两个以上方向的保持/抵消/压倒与连贯/能力 | not_run，单通道门未通过前不启动混合；文字混合用同一组合/评分另作比较 |
| m=0厂商人格、Base对照、身份移除代价 | Base下载及同链转换完成，行为专项和投影not_run；固定减法仍不等于逐位置投影 |
| 新加文字与注入比较 | 方法/映射/口径已设计，干预not_run；两者没有共同剂量，不能承诺文字传状态正式可用 |

## 对P5-31的建议（由用户决定）

1. 分开接口可用性、方向探索与留出确认；先测动态范围及标签×位置完整交叉，保存自然输出。C0的大幅代码布局变化、C1的事实顺序/表面形式不稳定，都不支持直接带入P5-31扫描。[后继路线比较](MEASUREMENT-FOLLOWUP.md)给出一手研究依据；C1实测说明无代码LL加同串先验/均值归一化仍不足，不能把取消代码等同于问题解决。
2. 中部层与‖h‖比例双侧剂量可作为候选，范围和机械选层规则事前定；单层优先，无共同层列可行集合，不自动多层。8–23未实测，不是应采纳值。
3. 提取、开发、最终单通道、混合与身份使用不同场景根；三语等权且分别报告。一般情感、礼貌、错概念与等范数随机共同检验特异性，不按口味挑方向。
4. 混合先定组合与Σ绝对预算，用单通道结果预定判据，再测新题；相消不腾预算，单效应太小判不可识别。
5. 人格按有分母类别测；区分品牌、礼貌、无故道歉、必要披露。报告Base格式/能力失配；投影与加法分别命名验证，少自称却不回应/编造/损能力不是成功。
6. 正式资源绑定、P5-13适配器、迁移集成、学得方向及其与「乎」混合仍由P5-31完成。本研究不签发AC2-16/17、I17或非零m启用资格。
7. 用户新增的文字状态对照应报告全部措辞/位置与状态外露、表达升级、厂商腔/能力代价，不能与更大的模型或人写角色效果混比；模式B样例路线是第三种机制。是否允许正式文字路径须另立ADR，本研究只提供证据与建议。
8. C1实测后，无代码LL加同串先验/均值归一化仍不能直接承担扫描接口。更好的下一步是先做冻结构念材料的独立三语语义核对，再另行设计全新校准。用户已确认新20及全部invalid可信，正式语义门仍失败：目标候选中英文的符号/事实边界及英日共情识别、自由输出质量与部分概念差值不足。建议将感受、事实复述、自身/他者对象分开做新的独立校准；不在旧题修分或扩大层网格替代测量识别。

9. 评分器也须冻结型号、思考模式、输出额度和有效采样设置。本次同20材料在两个设置下事实标签8/20变化，先核对构念与边界、再安排新的独立校准，不从这些已见结果改口径找通过。是否采用该建议由用户决定。

10. 把评分器的schema、引证和空评分失败与语义正确性分开登记。本次3/504是技术invalid计数，不是方向失败率；完整空评分不应触发挑选式复评，8192额度也不能出结果后增大找通过。后续注册需同时冻结取证/重发资格和失败分母，防止执行保护与单项invalid规则冲突。

11. 若要判断失败是否集中在真实混合效价上，需要另冻独立盲人工标签或获批的不同评判来源，覆盖同一总体并给出用量/预算；当前同评判器标签和字面线索只作代理。上述均为未来预注册/P5-31建议，不改变本批题、口径或阈值，是否采用由用户决定。

## 失败、阻塞与未完成

- **当前停止：** 新固定20和全部3个invalid已获用户确认并批准正式分析；C1_semantic_insufficient_stop，三语目标综合门和自由门均未过。按注册和用户要求停下等决定，不启动替代试验。
- **原始接口失败：** C0_insufficient_stop与C1_insufficient_stop原题/分/阈值保持；自动接口门未过。四方向、选层/强度、能力、混合、文字干预、厂商人格/Base行为及投影仍not_run，不能靠外部评分完成签发方向有效/开启非零m。
- **正式评分invalid：** 3条原分母失败，分别为schema、完整length空评分、引证失败；完整响应均未重发/修分。完整length曾被封装误归段级停止，旧fatal与费用保留，后继只修分类继续，不增加8192预算。
- **网络/进程历史失败：** 原high主管丢失终态原因未知，FULL-02旧IncompleteRead没有保存partial/读前HTTP状态，均不能回补或确定责任。两份未知整额预留保持；用户批准后仅原IncompleteRead成功重发一次，成功重发另保留整额，不把它再称未知。
- **执行与准备失败：** 弹窗及创建后隐藏仍闪、Base curl28/curl56与gguf信息查询错误、展示程序GBK读取UTF-8、静态导入/类型/格式错误及审计工具错误均保留。只修相应工程边界/编码，不通过新题、换方向或调阈值找通过；未运行准备修订标not_run。
- **Base/身份与GPU边界：** 官方Base已完整核验/同P0-12链CPU转换，行为未运行；固定向量减法不替代逐层逐位置身份投影。旧GPU窗口已释放，采集阶段仅API，本次正式分析仅本地CPU，无新GPU工作，也不借旧窗口运行新试验。
- **局限：** 单一外部评判模型可能把复述事实混作概念/感受，技术合规不消除此偏差；72重复不是独立单位，字面线索不是真实混合效价标签。当前结果不能称为她的情绪、关系、成长或决定。

## 规格对照与检查

| 位置 / 依据 | 本研究读法 |
| --- | --- |
| v2.0 §§2.3/3/9.1/9.6，ADR0032 §§2/10 | 合成事实、格式、语言规则；候选端点不是人格指令；提取与行为核对分开，不用“像她”评分 |
| O §§12–14，ADR0031/0034 | 先清attention/recurrent，再撤除/安装向量、完整prefill；C0只零值；组合按Σ绝对预算 |
| P5-31两条追加，ADR0012 §4 | 新载体重提取/选层；混合和身份/Base分别测；旧17层/占位15层不是选择，加法不冒充投影 |
| I28/I29，ADR0005 | 全合成、只读旧探针和官方模型；不读正式/排练存储，不新增权威状态 |
| I17、AC2-16/17 | 不签发验收；formal resource、迁移、启用归所属卡与用户 |

本卡写入限制优先，规格对照放本报告，不在docs/traceability新增文件或差异登记；未改规格读法、IDL或黄金向量。无新增产品运行时依赖；研究代码仅Python标准库、既有MSVC、P0-12固定引擎/模型，许可沿其登记。手写文件保持400行内。

续研只读CPU分词预检复用P0-12 venv中既有tokenizers/transformers/torch，不安装依赖；旧OpenAI提案未调用，当前Flash小样只用标准库官方HTTPS。文字状态仅因用户11月8日的本卡隔离对照授权而列入，不扩张正式§2.3或§9.7。

### 准备核验（不含GPU）

| 命令/核对 | 结果 | 证据 |
| --- | --- | --- |
| 冻结`ds_length_collect.py analysis` | 用户新20批准后一次退出0；C1_semantic_insufficient_stop，API/GPU新增0，3invalid按原分母 | [完整正式分析](FULL02-ANALYSIS.md)、[实际命令/终态](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full02-formal-analysis-01/execution.json) |
| 正式分析独立机械复核 | 完整性/数值两阶段各一次退出0；52源、5724清单artifact相符，432基项/72重复/12概念语言效应与null传播独立重算一致；不委派语义判定 | [摘要](results/ds-full02-formal-audit-summary-01.json)、[脚本/原始结果](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-full02-formal-audit-01/raw-filemap.json) |
| `pwsh -NoProfile -File reports/R-STEER-01/build.ps1 -OutputDirectory D:/Dev/cache/R-STEER-01/build-01` | `/W4 /WX /O2 /fp:precise`编译退出0 | [构建原记录与二进制归档](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/build-01/raw-filemap.json) |
| 4个Python的Ruff、`mypy --strict` | 最终均退出0；首轮3条类型错误原样保留，修复循环变量复用/因子表类型，未改样本或门 | [首轮](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/checks-01/raw-filemap.json)、[最终](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/checks-02/raw-filemap.json)，含精确argv/UTC/退出码/stdout/stderr |
| `python -m unittest discover -s reports/R-STEER-01 -p test_c0.py -v` | 3项通过；保护代码×位置全交叉、饱和不可被oracle掩盖、布局反转不可被均值掩盖 | 同上；不是概念效果或能力验收 |
| 当前载体SHA256、55个引擎文件 | 全匹配P0-12 | 首轮机械记录；未加载模型/GPU |
| 冻结输入、类型修复前后比较、CPU tokenizer检查 | 462请求；非guard456项内容/顺序完全相同；最长84个prompt tokens，全部在460/512限制内 | [完整实际提示](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/inputs/c0-jobs.json)、[token数](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/inputs/c0-token-counts.json)、[执行绑定](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/inputs/c0-preparation.json)、最终机械记录 |
| 最后一次准备入口核对 | 调整冻结文件清单后，calibrate.py Ruff/strict mypy与prepare均退出0；绑定PREREG、代码与完整实际提示，允许REPORT独立记录Base准备进度 | [检查记录](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/checks-03/raw-filemap.json) |
| 两个PowerShell AST、Markdown相对目标、`git diff --check` | 准备时通过；提交前再核对新增文件；独立解压46个归档文件全匹配，执行绑定与实际源码匹配 | [首批链接](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/markdown-links-01.json)、[提交后核对](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/preparation/final-check-01.json) |
| 全工作区CI/Windows | not_run；研究目录按现有路径规则不触发CI，跳过不能写成通过 | 本卡无产品实现变更；C0实际测量已按重档留原始证据 |

机械核验子代理按当前项目配置显式传入`gpt-6.1-terra/max`；未验证按角色名自动加载。主代理判断范围/门/结论，子代理只跑命令与报证据，未委派规格解读。原始检查、失败与构建无损gzip归档并逐文件回查；模型权重只留D盘缓存。生成的完整语料、token计数及映射超过400行是为保留证据，不属于手写实现。

## 依据

Flash准备检查：5份Python的Ruff/strict mypy、4项离线边界单测、密钥入口PS AST通过；首轮7条多语句行/1未用import和1条Any返回类型问题的原始失败保留，[首轮](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-checks-01/raw-filemap.json)、[修正后](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-checks-02/raw-filemap.json)。未调整评分口径/样本/门或API模式；等待进程源码在修复前后字节未变。504份完整payload及20条选择在[冻结准备](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-preparation-01/raw-filemap.json)、[计划](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/inputs/ds-flash-plan-01.json)、[小样索引](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/inputs/ds-flash-pilot-index-01.json)。KEY从未由检查代理读取，所有检查未调用API/模型/GPU。main仍为a7e8ae87，与上次实际基线相同，无新增变化，不需额外merge或重复GPU测量。

Flash实际调用：2026-11-09 UTC09:30–09:31（Perth17:30–17:31），响应model均`deepseek-flash`，system_fingerprint均`bc19ba97cbe016479903222bb1578db6`；官方价格页当时映射`DeepSeek-V4.1-Flash-1017`，API未返回独立日期版本字段，不冒称不可变权重。prompt 24,189 token（hit17,792/miss6,397），completion7,851；整数nano-USD累计11,447,052，20次全部结算、无重试/全量调用，未查询余额/账户。官方价格在调用前再次核对，证据与实际绑定/原始响应都在小样归档。可读审查页是运行后机械排版，输出/评分字节与原件相同，排版程序ds_review.py的Ruff/strict mypy通过；长审查页为生成证据，完整保留20条，不是手写实现。

Flash独立核对：[首次审计](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-audit-01/raw-filemap.json)冻结源/504请求检查退出0；逐笔的请求字节、评分等于响应content、引证、20 reserve/settle及费用均一致，但目录计数条件因审计程序变量names被usage字段覆盖而退出1，原失败保留。主代理仅将目录变量改名后在[新目录复核](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-audit-02/raw-filemap.json)，全部条件退出0，没有API重试、改标签或调整语义规则。独立计算`17792×6+6397×300+7851×1200=11447052`，每笔均低于事前预留；语种7/7/6、类型8/6/3/3、全量0。该审计只核数字/字节/结构，不判断语义可信。第二份审计stdout原字节为GBK，JSON证据为UTF-8，原始评分/日语输出没有改码。

Flash[12源原字节快照](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-flash-source-snapshot-01/raw-filemap.json)在运行后核对冻结SHA再保存，解决Git换行规范化的复算问题；请求/响应73文件完整归档。审查页末尾空行仅作格式修整，[失败与修整证据](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/ds-review-format-01/raw-filemap.json)保留，原评分JSON/响应不变。实际源与C0/C1已冻结程序均未因评分结果改动。

原文：AGENTS.md §§2/3/5/6/9、[ARCHITECTURE](../../docs/ARCHITECTURE.md)、[CONTRACTS](../../docs/CONTRACTS.md)、[Echo Core README](../../docs/echo-core/README.md)、[v2.0 §§2.3/3/9](../../docs/echo-core/echo-core-v2.0-first-principles.md)、[O §§12–14](../../docs/echo-core/ada-core-operators-spec-v1.0.md)、ADR [0012](../../docs/adr/0012-continuity-carrier-mobile.md)/[0031](../../docs/adr/0031-carrier-engine-and-injection.md)/[0032](../../docs/adr/0032-presets-as-starting-points.md)/[0034](../../docs/adr/0034-qwen35-4b-conditional-carrier.md)、[P5-31全文与追加](../../tasks/P5-echo-core.md#p5-31-方向提取与验证)。

历史：[P0-10报告](../P0-10/REPORT.md)、[注入](../P0-10/INJECTION-REPORT.md)、[速度](../P0-10/BENCHMARK-REPORT.md)、[P0-12报告](../P0-12/REPORT.md)、[R-CHOICE-01 DEVLOG](../R-CHOICE-01/DEVLOG.md)、[R-CHOICE-02 DEVLOG](../R-CHOICE-02/DEVLOG.md)、[carrier-engine README](../../spikes/carrier-engine/README.md)。本卡未重算历史结果或扩大结论。

方法先例：[ActAdd](https://arxiv.org/abs/2308.10248) 展示对比激活加法；[单方向移除研究](https://arxiv.org/abs/2406.11717) 展示残差投影先例；都不证明当前四个「乎」或身份方向有效，不提供可直接继承的层/剂量/阈值。

## 下一步

开发两通道均无合格注入层，注入留出/混合not_run。03原样重采已完整核验，新540项评分包已准备；推送核远端后沿已有授权调用，承接当前费用/时间，不重评首轮。文字混合待单通道结果按原公式填定，如运行须另申请GPU窗口。完整研究尚未完成，不合并、不领下一卡。
