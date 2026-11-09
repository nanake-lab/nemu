# P5-68 规格对照

唯一新增 IDL：[law_state.proto](../../core/contracts/law_state.proto)。依据：[ADR 0035](../../docs/adr/0035-law-definitions-d07-d09-d11.md)、[ADR 0029](../../docs/adr/0029-theta-estimator-storage.md)、[ADR 0007](../../docs/adr/0007-initial-idl-canonical-encoding.md)。各行列出全部字段；没有运行参数默认值。

| 消息 / 字段 | ADR 0035 具体条款 | 实现与检查 |
| --- | --- | --- |
| 全部消息的 schema_version | §9 P5-68(5)，编码沿 ADR 0007 决定 2–8 | 新标签的 schema_version=1；三语言结构编码、缺版本/未知版本反例 |
| ThetaEstimatorParametersV2.law_id、numerics_id | §2、§3.2、§7、§9 P5-68(1) | 新数值 ID；快照 law_version.resource_version 对齐 law_id；旧数值 ID 拒绝 |
| baseline_weight | §3.2、§7：P₀ 有限且 >0 | 新字段 23，保留不用旧 8；+0/负数拒绝，次正规和大有限数合法 |
| baseline_prior | §3.2：b₀ 沿用 | NatureVector 的 [-1,1]⁴ |
| gain_prior、sigma_prior、tau_prior_s、variance_weight、duration_weight | §3.3：σ、g、λ 及固定先验不变 | 复用 ADR 0029 的正值与范围检查 |
| change_prior、change_prior_weight、change_min、detector_shift_sigma、detector_threshold | §3.1–3.3、§7：检测器不变 | 复用现行参数边界；不增可靠度打分或次数门槛 |
| gain_min、gain_max、recovery_min、recovery_max | §3.3、§7：现行上下界 | 复用有序范围与快照当前值检查 |
| jump_notice、drift_run、drift_margin | §3.3、§7：报警参数沿用、需重新校准 | 无默认值；复用现行结构校验，不实现报警 |
| ThetaSnapshotV3.revision、law_version、values | §2、§3.2、§3.6、§9 P5-68(1) | 保留旧字段 2–4；版本资源绑定与当前 g/λ 范围 |
| estimator | §3.2–3.3、§9 P5-68(1)：ADR 0029 完整状态 | 原 ThetaEstimatorState 原样内嵌，含 S、U 以及全部中间值、计数和时间；不重算或修补 |
| parameters、parameters_digest | §2、§3.2、§9 P5-68(1) | 后继参数与 CanonicalV1 类型域分离摘要；旧 10、11 reserved，新 13、14 |
| prior_mass | §3.2、§9 P5-68(1) | 有限 ≥0，允许次正规/+0；genesis P=P₀；与四通道共享 S 对应 |
| FastBasisIdentity.kind、record_id | §4.1、§4.2、§7、§9 P5-68(2,3) | 记录分支非空 ID；内部支路无 ID，规范键唯一即至多一个内部槽；不推断身份 |
| LawLearnedCoordinate.tendency_id、value | §4.1、§4.3、§4.7 | 学得坐标身份非空、值有限；不套 x 的盒范围 |
| LawStateVector.nature、learned | §4.1、§4.3、§4.6 B6′ | 四坐标 NatureReal 与学得坐标集合，排序绑定身份和值，不分别排序裸数组 |
| FastBasisSlot.basis、current、target | §4.1、§4.6、§9 P5-68(2) | 坐标身份一致；无 h/目标幅度比较，无 y 或历史区间状态 |
| FastStateSnapshotV3.revision、representation_version、anchor_time_ns、nature、learned | §4.1、§4.4、§4.8 | 原字段号、类型与限制保留；4、5 继续 reserved；共同锚点，无另设时钟 |
| FastStateSnapshotV3.slots、law_version | §2、§4.1、§9 P5-68(2,4) | 新字段 9、10；依据规范键排序且唯一，槽与 x 同坐标；必填规律引用也令空槽 Protobuf 与旧消息分开 |
| FastEffectRecord.basis、role、target | §4.1–4.3、§9 P5-68(3) | 四角色枚举；共同依据引用；每条作用记录一个依据，多个重访作用按 H 次序由 F/事务记录 |
| theta_before、effects_before | §4.1、§4.3、§4.7 | 前态 θ 快照与 R⁺ 作用视图 ObjectRef；保存最近目标更新所用前态的引用由调用方负责 |
| interpretation_refs、governance_refs、law_version | §4.1–4.3、§4.7、§9 P5-68(3) | 非空引用集合与规律引用；解释/治理格式由其拥有卡定义，不私造语义治理消息 |
| FastStateMigrationRecord.previous_snapshot、migrated_snapshot、prior_decomposition_unknown | §4.8、§9 P5-68(4) | 前后标签检查与 unknown=true；H 系统记录、L 引用和原子发布交 P5-75 |

## 验收映射与边界

| 条款 / 不变量 | 本卡证据入口 | 范围 |
| --- | --- | --- |
| §9 P5-68(1–3,5)、SYS21 | fixture_law_state.py、golden/legal.json、golden/illegal.json；verify.py | 每新消息 ≥3 合法/≥3 非法，结构与语义边界三语言一致，双编码与摘要往返 |
| §4.6 R4、B6′ | legal_law_state 的修订 h₂=-1/v₂=0、合并 h=2/v=1、0.1+0.2 舍入正例 | 接受已给定状态，不执行或证明 F 规律 |
| §3.6、§4.8、SYS17 | verify_law_state.py | 三个空槽迁移夹具保留 x/锚点，并建立前后摘要引用的迁移记录；拒绝新旧互解；旧字节回读。存储迁移/回退、故障注入仍属 P5-75 |
| I18 | 完整 θ 状态内嵌、槽与目标明确编码；无派生 y | 类型与边界检查，不在核心之外增加状态 |
| I23、I16 | 明确版本/引用、CanonicalV1 与旧向量保留 | 本卡不签发整段 H 重算或结构重放结论 |
| I03、I21、I24 | 纯契约检查，不实现 F/C，不提供写存储接口 | 运行时与学习隔离验收交后继卡 |
| I29 | 夹具均为合成数据；verify 不打开权威存储 | 既有 θ 恢复验收仍用临时测试目录 |

没有新增差异登记或待定参数。V2-144–150 的实现范围仅取 ADR §9 为 P5-68 指定的部分，D05、消费键、规律、存储接线均不在本卡。
