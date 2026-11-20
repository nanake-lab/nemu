# ADR 0046　说话与独处思考的契约缺口

交付线，提议。依据为 [ADR 0045 §4–5](0045-speak-think-wiring.md#4-念头写回-h-与说话人身份)。本卡定义可核验的记录，生产和授权仍归后继卡。

## 1. 现有字段能表达什么

| 缺口或来源 | 判定、证据与接线方 |
| --- | --- |
| A 云端内部文字 | 需要新增。旧结果要求本地 model_hash、tokenizer_hash、modulation，成功和截断还校验载体回执。P5-38 接内部提交，P5-39 接调度。[IDL](../../core/contracts/cognition.proto#L65-L79) · [校验](../../core/crates/core-types/src/semantic/cognition.rs#L59-L91) |
| B 内部产物 | 不需要。P5-38 以已提交的内部记录 ID、获采纳的产品及其不可变内容还原本人念头。对外说过的部分另查投递。[内部记录](../../core/contracts/cognition.proto#L83-L89) |
| B 终端文字 | 需要新增。文字无作者字段，配对会话绑定的是安装实例。P5-43/P5-53 必须取得受信主体证据。[UserText](../../core/contracts/terminal.proto#L36-L40) · [TerminalSession](../../core/contracts/terminal_ingress.proto#L33-L40) |
| B 用户语音 | 需要新增。转写、语言、置信度、时长和可选录音能定位语音，不能认证说话人。P5-43/P5-53 绑定受信身份和对应转写版本。[UserUtterance](../../core/contracts/terminal.proto#L26-L34) |
| B 转述的第三方 | 需要新增。通用来源和跨度未给出经治理的逐层发言主体。P5-43/P5-53 核对每层可见文字及原记录。[SourceMetadata](../../core/contracts/state.proto#L47-L55) |
| B 教导标注 | 不需要为教导者另加字段。P5-43 核对已接受的标注、event_id、binding 和 speaker，仅据此识别该次教导者。教导中另有引语时适用上一行。[TeachingAnnotation](../../core/contracts/records.proto#L69-L80) |
| C 关系及完整性 | 需要新增。parents/source_refs 缺方向用途的明确语义。更正链或空查询均不能证明前沿查全。P5-43/P5-53 对照同前沿的权威清单。[记录](../../core/contracts/state.proto#L9-L55) · [映射九行](0045-speak-think-wiring.md#3-关系类型映射-v1) |
| D 部分投递 | 需要新增。契约只给块数和字符数。音频完成取 JS UTF-16 长度，中断用已播样本比例乘长度再四舍五入。例如单个 🙂 长 2 个 UTF-16 单元，半程计 1，没有对应的 UTF-8 标量边界。即使是 ASCII，变速发音也不能由样本比例推出词界。P6-04 绑定通道、呈现对象和有证据的 UTF-8 字节半开区间。[回执](../../core/contracts/terminal.proto#L101-L109) · [块](../../core/contracts/terminal.proto#L136-L146) · [音频](../../packages/avatar-runtime/src/lipsync/audio-queue.ts#L282-L285) · [组包器](../../core/crates/task-layer/src/cause_projection/types.rs#L86-L93) |

## 2. 新消息及字段

全部消息的 `schema_version=1` 标识各自冻结标签下的字段布局。下表字段按 [IDL](../../core/contracts/speak_think.proto) 拼写。每个 ObjectRef 同时绑定类型、资源版本、摘要和字节长度。消费者从获准的存储取回并核验对象。规范编码允许独立验证结构；生产时的前沿、权限和出处由可信核心证明。

### 2.1 云端回执与内部提交

| 消息与字段 | 对应要求及读法 |
| --- | --- |
| RemoteCognitiveReceipt.binding | §5「哪次调用」与「哪些能力受限」。复用活动、步、请求、轮次、分支、基提交、Q 修订、输入前沿和快照，连同协议、权限和读集。environment 是核心快照中配置的本地资源；即使本地服务暂停，已绑定的本地资源也不能当作供应商权重。 |
| provider、requested_model、returned_model | §5「谁写」。分别记录服务提供者、实际请求标识、回包模型字符串。正常和截断要求实返模型；服务没有提供必要回包信息时保留真实故障信息。 |
| provider_revision、model_sha256、tokenizer_sha256 | §5「谁写」。三个独立可选值，缺失表示服务方相应信息不可得。哈希有值时必须为 32 字节 SHA-256，禁止全零占位。模型名可用和权重摘要可用是两件事。 |
| mode、local_pause_reason | §5「哪些能力受限」。唯一合法模式为非严格、thinking 关闭、仅内部文字。由本地可信调用者记录选择路径可用性，暂停时必须给出实际原因；缺省表示该路径可用，不证明执行过选择。 |
| parameters、format、mapping、meter | §5「按什么配置」。分别引用本次生效的生成参数、格式资源、关系映射和完整请求计量资源，均可按版本取回。 |
| capacity、output_reserve | §5、D5。计量资源约定同一单位；已发送时两项必填，预留不得超过容量。发送前终止可只保存已得知的值。能否发送仍要验证完整请求上界。 |
| input_manifest、sent_request、request_sha256 | §5「输入了什么」。清单保存有序材料身份、内容版本、区间、更正、读集和 D5 依据。实发对象是精确的应用层请求字节，request_sha256 单独对这些原始字节做 SHA-256。发送前终止仍绑定当时的材料快照，禁止填写实发对象或实发摘要。 |
| raw_response、returned_text、finish_reason、usage | §5「返回了什么」。原始响应（含可重建的流式片段）、实际抽出的 UTF-8 文本对象、服务端终止原因和可取得的用量分别保存。实际文字可以被拒。无回包时不构造原文、终止原因或用量。 |
| status、reason | §5 六种状态见下表。发送前终止必须给本地原因，取消、服务故障、拒绝必须给已知原因。服务方 finish_reason 原样保留。 |
| adoption、accepted_range、product、review | §5「采纳了什么」、D8。采纳时，区间位于 returned_text，product 是核心审核过的 CognitiveMaterial，绑定实际获准的文字及来路。完整采纳须覆盖整个非空文本；部分采纳的原文字节须与产品内容逐字节核对。拒收、失效或无文字时不允许产物和采纳区间。review 引用本次 D8 判定及依据，除无文字外必填。 |
| InternalActivityRecordV2.record_id、previous、started_at_ns、finished_at_ns | §4 内部记录，§5「哪次调用」。记录 ID、可选的旧版或新版前序记录、实际执行起止时刻；提交位置另由 H 固定。 |
| InternalActivityRecordV2.local、remote | §4 云端回执缺口。必选其一。本地沿用 CognitiveResultAndReceipt 全部字段与语义；云端只能存上述事实。只有已提交、经审查且有 product 的记录才可接续本人念头。 |
| InternalActivityCommitV2.binding、interpretations、activity | §4 原子写回、§5 采纳。binding 与选中分支的绑定必须一致，解释结果非空；activity 定位实际内部记录。保留未获采纳的尝试供审计，其返回内容不能变成已接受的念头。 |
| CoreTransactionV4.staged_h、staged_x、staged_r、staged_theta、staged_l、staged_q、staged_u、staged_concepts、staged_ledger | §4 原子发布，沿用 V3 中各暂存状态、概念与账本字段。H/Q 等待可信核心在同一事务校验并提交。 |
| CoreTransactionV4.processed_position、anchor_time_ns、accepted_results、audit_refs、event_binding、activity_commit | §4 原子发布与 §5 调用位置。分别保留已处理位置、锚时刻、获采纳结果、审计对象。事件来源与含本地或云端结果的内部提交互斥。 |

| status | 允许的 adoption |
| --- | --- |
| 正常 | 完整采纳、范围采纳、拒收、失效、无文字。完整采纳必须有非空产品。 |
| 截断 | 范围采纳、拒收、失效、无文字。即使已返回的部分全部可取，也保留截断事实。 |
| 取消、服务故障、内容被拒 | 拒收、失效或无文字。已收到的内容仍绑定实际返回对象。 |
| 发送前终止 | 无文字。保存本地原因。 |

### 2.2 身份、关系与投递

| 消息与字段 | 对应要求及读法 |
| --- | --- |
| Utf8Range.start、end | §4 精确范围，§5 采纳范围。零基 UTF-8 字节半开区间，非空。解析实际对象时还要核对两端均是标量边界。[Rust 切片检查](../../core/crates/core-types/src/semantic/speak_think.rs) |
| RecordVersion.branch_id、record_id、content | §3 两端、§4 身份及对象版本。分支中的原记录 ID 与有版本的不可变内容一起确定端点。原记录含多个产物时，用确切内容区分。 |
| QuoteLayer.source、visible、principal_id、identity_evidence | §4 作者。绑定原记录版本、它上面可见的字节区间、发言主体和可复核的受信身份证据。终端声明或模型给出的 ID 不产生该证明。 |
| SpeakerAttribution.layers、governance | §4 引语层级。按外层发言人至内层被引者排列，至少一层。每层绑定自己的原话及跨度；governance 记录认可这次逐层归属及对应关系的治理依据。引语无可核实来源时不构造该层的可信归属。 |
| RecordRelation.kind、origin、target | §3、§4。存事实种类和有版本的两个端点；两端属于同一分支。引述者指向被引者，更正者指向被更正者，撤权决定指向用途受限的原记录。 |
| RecordRelation.purpose、effective_commit、governance | §3 第 9 行、§4。用途是版本资源中的用途标识；治理对象证明该用途的授权范围和事实。以同分支的生效提交界定起点。查询前沿必须包含生效提交，并应用至该前沿所有后续更正或撤权；提交 ID 按历史祖先关系核对。 |
| RelationSet.endpoint、purpose、frontier、relations | §3 闭包完整性。查询键含确切端点、用途和同分支的已提交前沿；结果包含该范围内两个方向的全部关系，保留多种事实，禁止重复。按权威前缀中记录次序及记录内次序重建。 |
| RelationQueryReceipt.result、manifest、expected_count | §4「已查全」。在独立的可信前沿索引或从该 H 前缀重建的清单中取得 manifest 和计数。验证结果的计数、CanonicalV1 类型域摘要和字节长度；漏边后改计数仍过不了摘要。只有经核心固定或独立重建的清单才能证明完整性，调用者自行对残缺结果重新签清单没有证明力。空集同样要有证明。 |
| SpeechDeliveryV2.action_id、output_id、status | §4 实际交付。绑定动作、输出和该通道投递状态；保留已确认的范围，未知状态不提供任何可引用区间。 |
| SpeechDeliveryV2.channel、presentation、delivered | §4 原对象版本与精确范围。视觉显示与实际音频分别出回执，presentation 固定本通道实际呈现序列的 UTF-8 文本对象及原记录版本，delivered 只列证据确认的区间。区间按字节升序、相邻合并，允许非连续。完整须覆盖整个对象；部分至少一段且非全体；取消可保留此前已投递的部分。 |

### §3 映射表逐行来源

下列枚举名省略公共前缀 `REL_FACT_`。后继卡按绑定版本解释这些事实，未核实的候选必须补足独立证据才能使用。

| ADR 0045 §3 行 | 关系事实与证据边界 |
| --- | --- |
| 1 父记录与回复 | REPLY 需要显式回复证据。普通 parents 留作 UNRESOLVED；session 与 RPC reply_to 不构造回复边。 |
| 2 引用与对象 | QUOTATION、PARAPHRASE 需要原文依赖和跨度。SOURCE_LOCATOR 只说明定位；把通用 source_refs 推成正文互引仍未核实。 |
| 3 更正与撤回 | CORRECTION、RETRACTION 逐目标登记；反查包括更正的更正。 |
| 4 证伪与教导 | CONTRADICTION_ANNOTATION、TEACHING_ANNOTATION 保存标注事实；ACCEPTED_FALSIFICATION 需要已接受的治理及其目标，单凭标注未核实。 |
| 5 分组与合并 | SAME_EVENT_HINT、CONTACT、EVIDENCE_MEMBER、GOVERNED_MERGE。最后一项须有生效的治理，键相同不能推出已合并。 |
| 6 重解释与语义图 | REINTERPRETATION、SEMANTIC_LINK、PROPOSAL_SOURCE。只登记有确切记录端点的事实；概念节点不得冒充 H 原记录。 |
| 7 F、迁移与账本 | EFFECT_PROVENANCE、MIGRATION、TRANSACTION_MEMBER、LEDGER_REFERENCE 只记可定位端点的沿革。 |
| 8 认知、行动与投递 | ACTIVITY_PREVIOUS、MATERIAL_READ、PRODUCT_SOURCE、ACTION_DEPENDENCY、OUTPUT_DELIVERY。产物来源直接构成正文引用仍未核实，正文依赖另证。 |
| 9 权限 | PERMISSION_SNAPSHOT 表示绑定，USE_REVOKED 需具备目标、用途、治理和生效提交；从快照单独推断撤权仍未核实。 |

## 3. 备选方案

只把远端返回对象加进 audit_refs 最省类型，却仍要给内部记录必填的本地结果伪造模型摘要和调制。另一种写法是在新结果里包 local/remote，然后在新记录里再包新结果。现方案把分支直接放在活动记录上，省掉一层包装，完整复用获准的本地结果。部分投递如仅追加一个字符换算系数，无法恢复中断时究竟呈现了哪一段，尤其是语音停顿与变速发音。

## 4. 迁移与回退

本卡按用户更正使用 `@retains`。前驱保持可写、按原标签可读，旧字段号保留或 reserved，新号一律大于前驱最大号。检查器对这两条规则分别执行检查。[#164 的先例](0036-j-resources-and-effect-receipt.md#兼容迁移与回退) · [V3 先例](0030-cognition-record-contracts.md#兼容迁移与回退)

| 现有类型和生产者 | 后继及启用、退役条件 |
| --- | --- |
| CognitiveResultAndReceipt 是本地载体的返回合同 | P5-38/P5-39 仍以它承载本地结果。该类型没有后继，不退役；云端只写 RemoteCognitiveReceipt。 |
| InternalActivityRecord、InternalActivityCommit 及 CoreTransactionV3 是 P5-37 的活动、内部提交与事务合同；当前 core-runtime 构造、core-store 发布的事件事务仍为 V2 | P5-38 让 store/runtime 原子发布 RecordV2、CommitV2 和 TransactionV4，并以原标签读取旧版；P5-39 迁移调度、迟返、previous/latest_step 等引用和所有写方。本卡保留全部旧消息。旧组无剩余生产者、生产引用及活类型嵌入经过检查后，才由 P5-38/P5-39 协调在同一次契约更新中将三对 `@retains` 换为 `@supersedes` 并把前驱标为 `@superseded`。事件事务 V2 的原有迁移责任仍归 P5-38。 [实际生产点](../../core/crates/core-runtime/src/store.rs#L125) |
| SpeechDelivery 是终端投递回执，当前 gateway 和 fact-view 识别旧标签 | P6-04 迁移显示/音频生产、网关校验及事实读取。待全部写方切换后将 SpeechDeliveryV2 的 `@retains` 换为 `@supersedes` 并给 SpeechDelivery 标 `@superseded`。旧版仅按当时能证明的范围使用，不用新单位重解释旧数值。[入口](../../core/crates/gateway/src/validation.rs#L44) |
| 作者与关系没有上述语义的旧类型 | P5-43/P5-53 发布新的受信绑定与前沿清单。旧 parents、source_refs 保留原义，新增事实不从缺证记录倒推。 |

后继卡还须处理检查器对已经冻结的 retains 元数据的保护。届时仅允许「同一前驱、对应反向退役标记、活嵌入全部迁移」的单向转换，并提供反例，其他变更继续拒绝。本卡保留现有检查器。终端上传资格留给各接线卡逐类开放；一般终端不得签发治理、云端采纳或核心事务。部署前备份，按绑定的原标签恢复；缺少需要的对象或新版本写入验证失败时保留原存储。这里验证的是契约兼容性，存储迁移和原子发布由接线卡验收。[兼容检查](../../core/contracts/check_compat.py#L70-L71)

## 5. 后果

| 新增结构 | 能不能不加 | 一个人会怎样 | 加了以后，有什么从此改不掉 |
| --- | --- | --- | --- |
| 云端回执与内部容器 | 本地结果必须有真实载体，另一路文字必须记录实发与采纳。记录层一个分支足够。 | 想过、听到建议、采纳建议，可以分别说清。 | 已发给服务的字节及当时产生的记录成为历史；以后的模型可以更换。 |
| 作者及层级 | 只补无法由受信来源推出的作者。 | 能分清转述者和原话是谁说的。 | 已记录的归属如有错误，须追加更正并保留原记录。 |
| 关系与完整性 | 缺端点和前沿时无法核对更正或撤权；证明可从历史重建。 | 引用旧话时也会考虑对方后来的澄清。 | 历史前沿上哪些事实生效可以重查，后来的规则不得改写该前沿。 |
| 文本区间及投递 | 旧估算缺少可验证的精确边界。 | 话被打断时，只能说确定讲出来的部分。 | 已投递的文字无法收回；内容版本与区间随原回执保存。 |

遵循 ADR 0039 的来源、可重建性与核心写权。所有权威写入仍经外层事务，模型文字不增加证据组。
