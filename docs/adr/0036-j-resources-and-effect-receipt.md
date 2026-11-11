# ADR 0036：J 方向资源、身体参数与作用回执

状态：提议，P5-42 / PR #164，needs-user；用户合并即批准。2026-11-11。
编号核对：origin/main `6b9c9b0b` 与全部开放 PR，ADR 最大 0035、V2 最大 150。

## 背景

P5-12 已提供纯数值编译，NumericReceipt 不能作为权威回执。
v2.0 §§9.1–9.3、9.5–9.6、13、16，O §§13.1–13.3，ADR 0008 §3，
CONTRACTS §7 与 V2-116 要求资源、验证范围和作用的可核验绑定。
V2-117 已取消主动强度读出；本卡不恢复它，也不选定模型、层、hook、掩码或数值。
R-STEER-01 结果不作为输入；结构夹具没有正式资格。

## 决策

沿用 ADR 0007 CanonicalV1，全部新增消息使用独立标签；所有旧字段、字节和黄金向量保留。
字段以 IDL 为精确清单，下面逐组说明必要性，不在标识字符串或 bytes 中嵌私有协议。

| 类型 / 字段组 | 来源与必要性 |
| --- | --- |
| JHookBinding：carrier/model/tokenizer、layer_index、residual_stage、layer_kind、width、dtype、prefill_mask、decode_mask | O §13.1–13.2：绑定版本、唯一残差相加位置、相对归一化位置及张量/数值/位置规则；混合模型不把 mixer 强称 attention |
| JDirection：kind、tendency_id/concept（既有 ConceptRef，含 registry_version）、vector、beta | v2.0 §9.1–9.2、§7：四个「乎」或规范倾向/概念到明确方向资源，β 非负；标识不含概念原文 |
| JDirectionVector：version、values | v2.0 §9.1、O §13.1：可恢复的有限 binary64 一维方向；与 hook width 对照；注入 dtype 属载体绑定 |
| JDirectionSet：version、hook、directions | Ω 版本资源；四通道固定顺序，再按 CanonicalV1 tendency_id 排序；身份重复拒绝；不同概念可共用向量，验证项按集合位置分别绑定 |
| JValidatedDirection：vector、max_beta、measured_norm、norm_tolerance、zh/en/ja_evidence | §9.6/16：单位范数及单项允许范围，三语通路与能力验证证据；容差由独立校准记录，契约不选值 |
| JDirectionValidation：version、direction_set、fixture、directions、combination_max_beta、combination_evidence | §9.1/9.6：把单项和组合资格绑定整个方向集（因此绑定模型、hook、mask 和版本）；fixture 不得获正式资格 |
| BodyReadoutParameters：version、facing/tension/energy 的 NatureReal | §9.3/16、CONTRACTS §7：每行四维固定系数；偏置 (0.5,0,0.5) 与截断固定，不复制为可调参数 |
| JDirectionUse：kind、tendency_id、status、direction、validation、beta、coefficient | P5-12 数值轨迹、I06：纳入与排除及原因，区分未验证、缺失、验证不匹配；纳入必须有引用与有限数值 |
| JEffectReceipt：snapshot_id、anchor/projected 时间、x/R/θ 摘要、carrier/model/tokenizer、direction_set/validation/body_parameters/genesis、mode、directions、total_bound、body | O §13.2、ADR0008 §3、§9.3：一次投影、已提交输入及资源追溯、数值结果和保守总界；无 H/z/p/原文输入 |
| GenesisManifestV3 | V2-117：保留 V2 的字段号，14 reserved，去掉主动读出引用；其他语义不变 |

所有 schema_version 固定为 1。方向/验证/身体资源的 version 与 ObjectRef.version 必须一致。
hook 的 shape 指 O §13.1 每个位置的残差向量 h_l,p，明确为一维 `[width]`；
batch/sequence 是适配器的运行时轴，不在资源里假定具体模型张量内存布局。
适配器须先核对实际 hook 的相加点、归一化相对位置、逐位置向量维度及 dtype，再应用固定掩码。
资源引用摘要按完整 CanonicalV1 对象计算，同时核对 type、byte_length、version；只核对字符串版本不足。
验证记录声明的是所列单项范围及组合总预算内的资格，不得把只测过一个系数组合的报告外推为整个范围。
zh/en/ja_evidence 和 combination_evidence 是可恢复的审计证据引用，不作为私有协议解码；
消费者必须独立确认它们包含 §9.6 所需通路、三语单调性和能力无损结论及预先登记的容差，才能信任该验证记录。
方向验证记录不引用自身；方向集不引用验证记录，避免循环。身体参数由 genesis 引用，参数不反向引用 genesis。

### 回执、资格和外层义务

`ModulationEnvelopeV2.effect_receipt_digest` 明确指 `JEffectReceipt` 的 ADR0007 对象摘要。
回执不含信封或其摘要，故没有循环。信封既有字段足以携带数值，不新增无必要的信封版本。
消费者逐项核对信封与回执的快照、投影时间、摘要、载体资源、系数和总界。

语义检查只验证字段关系及已提供资源；不证明对象确已提交或校准证据真实。
生产消费者必须解析全部引用，核对已提交快照/锚点及 x/R/θ 摘要，重算 J 数值，
完整对照 R⁺ 清单（不能遗漏待排除项），核对方向/概念、向量维度与单位范数、
单项和组合 β 验证范围、三语单调性及能力无损报告，拒绝 fixture 资格；
资源缺失、未验证和不匹配分别报告，不能默认为成功。四个「乎」方向不合格是故障，
学得方向不合格排除且不重缩放已有项。验证记录是声明，不是自签成功的授权。
语言注入系数为零不蕴含身体中性（例如 β=0、m 非零）；普通回执的身体值由外层依据快状态与身体参数重算，不能只凭注入系数判定。
数值预算按 P5-12 的逐项非负保守上舍入，不用 epsilon 放宽，不按带符号相消求预算。
genesis 记录初始载体/方向；当前载体可经 v2.0 §9.5 的已提交迁移改变。
资源接口仅把当前方向/验证绑定到当前回执，并将固定身体参数绑定 genesis；
外层必须验证初次启动与 genesis 一致，或核验从 genesis 到当前资源的已批准迁移链。
不能通过重写已签署 genesis 更新当前载体，也不能把本接口的通过当成迁移授权。

读取接口不能把 NumericReceipt 自动升级成权威：其缺失摘要/genesis/资格来源须由外层补齐并核验。
REHEARSAL_WITHOUT_FAST 只用于排练，系数为零、身体中性；不据其 fixture 数据签发 LIFE 资格。
真实载体、缓存失效、已提交快照获取和生产发布为 not_run，分别归 P5-31/13、P5-08/19 等接线卡。

### 兼容、迁移与回退

GenesisManifestV3 用 `@retains GenesisManifestV2`，不关闭尚在消费旧类型的代码。
旧 V2 字节按 V2 解码，禁止按 V3 重解释；旧主动资源不能填假对象或静默忽略后宣称已迁移。
后续谱系接线显式解析 V2、核对其其余资源，记录 V2→V3 迁移并保留原对象/摘要。
已签署 V2 的原签名仍针对原字节；V3 是显式迁移所得对象，不冒充同一份签名覆盖的新字节。
在临时谱系验证迁移、失败回退和 SYS17 后才切换生产；本卡不迁移用户存储、不提供发布入口。
失败保留旧系统，回退恢复旧清单引用；不逆改旧对象。冻结本卡之前的完整 IDL/黄金向量为 p5-37 基线。

### 身体与后续决定

CONTRACTS §7 的线性参数仅表达当前 P5-12 接口；不是对 ADR0032 §8 后继调控的实现或永久固定决定。
本卡不添强度/抑制参数、不选择审美参数。后继设计需要新消息而不重解释此处固定系数。

## 备选方案与后果

选用显式资源和回执；继续用不透明 bytes 会使三语核验无法落地，修改旧类型会破坏冻结字节，均不采用。
新增字段/标签需用户批准与独立交叉审查；编解码通过不等于正式资格、生产链路或真实行为通过。
