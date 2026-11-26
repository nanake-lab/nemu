重档。解释采纳、关系查询和来源分类决定哪些判断可交给外层事务。

# P5-06　语义核心（交付线 3/3）

这次补齐了核心记录 A、M 与辅助解释的入口。后来重新解释一件事时，核心从已提交前缀核对原来解释的是哪件事、处于哪个阶段、这次替换哪些目标。事件属性用锁定的来源规则生成，分别记录表达是否发生、内容是否属实、是否属于某人的主观体验。请安排独立交叉审查。

## 交付

| 部分 | 当前行为 |
| --- | --- |
| [1/3 首轮采纳](https://github.com/nanake-lab/nemu/pull/111) | 首次 IRL 仍须绑定本轮事件及事务。合法图、部分采纳、已记录失败经原入口读取。部分采纳的原因现在逐边记录原候选的零基序号，以及起点、终点或关系种类的问题。整轮拒绝仍记录阶段故障。 |
| [2/3 注册表](https://github.com/nanake-lab/nemu/pull/271) | 核心预留 ID，暂存、修订及有向别名按当时版本检查。其 squash 合并提交 `3d131a02943ea81f71e7100e99c628d7c73b3e9c` 是 3/3 的起点。 |
| [G04 解释与重解释](../../core/crates/semantic-core/src/interpretation.rs) | A 核对完整记录和四维值。M 核对正值记录，把实测零和有原因的未知按目标记入 coverage，未评目标缺席。失败阶段保留原降级产物的引用。候选倾向、教导、矛盾及同一事项标注按预留身份和实际规范对象核对；矛盾标注额外要求受信读集里的本人陈述。 |
| [G04 历史目标](../../core/crates/semantic-core/src/irl.rs) | 晚到的触发事件可指向旧事件。事务独立授权新端点、前任、替换目标与关系清单；在指定分支的基前缀读回前任采纳对象，再核对同事件、同阶段、覆盖、查询端点、用途、前沿、条数与清单摘要。 |
| [G05 来源属性](../../core/crates/semantic-core/src/attributes.rs) | `initial_attribute_rules` 从正式 IDL 取标签，产生版本 `p5-06-attribute-rules-v1` 及内容引用。按受治理的来源类别、事件种类、载荷标签三者精确选取表项。人的回复和定向接触带外源单位，账号按显式别名归人。桌面观察、自致执行结果、内部记录、维护分别保留其证据用途。每项核实保存范围、三值、依据；目标的前提随记录保留。 |

[规格、代码与测试对照](../../docs/traceability/P5-06.md) · [私有证据索引](EVIDENCE-INDEX.csv)

## 按原卡核对验收

| 编号 | 可在本卡检查的判据 | 整项状态及余项 |
| --- | --- | --- |
| SYS02 | 伪造来源、模型推断冒充自报、未知概念、越界跨度和非法边进入拒绝、隔离或退化路径；独立合法子图仍可读回。 | [not_run](SYS02/result.json)。缺从真实事务到下游状态的证据，尚不能核定非法部分没有产生核心作用。 |
| SYS03 结构 | 否定、条件／可能、带指令的引语、纠错、歧义和新词六条输入逐字核对来源、报告模态与未消解指代；均未产生行动或新概念。 | [not_run](SYS03/result.json)。完整结构场景还需核对各类含义的特定结构。语义准确率另属 P5-07。 |
| SYS04 | 注册表后继、环、冲突、悬空引用、旧版本身份、暂存可见性与对象替换有本包用例。 | [not_run](SYS04/result.json)。缺更换实际 embedding 后对旧身份、active 条件、F/G/J/θ 作用的联合核验。 |
| 16.15 | 图、类型、引用、认识论标签、容量和跨度的拒绝与部分采纳有用例。已提交前缀接口按记录读回图或失败状态。 | [not_run](16.15/result.json)。缺正式恶意输入组贯穿入口、真实重放及状态变化的结果。 |
| 16.16 | 已有概念从 provisional 升级或弃用均保留前任，旧 ConceptRef 按旧版本读取，有向别名不写回旧事件。 | [not_run](16.16/result.json)。缺真实保存的历史跨版本读取与实际 embedding 更换验证。 |

原卡恶意候选另含引语内的指令、别名环、把自述写成外源、伪造教导主体，以及越界 a/μ 和非单位方向。相应测试分别见 [图与引用](../../core/crates/semantic-core/src/tests.rs)、[注册表](../../core/crates/semantic-core/src/registry_tests.rs)、[解释记录](../../core/crates/semantic-core/src/record_tests.rs) 和 [属性](../../core/crates/semantic-core/src/attribute_tests.rs)。G05 的用例覆盖现有表项，枚举八种事件种类的命中或拒绝，并核对七种认识论标签与独立的内容真假记录。

## 验证与限制

Rust 1.99.0，实测提交 `0360106e`。合并基线从 #271 的 `3d131a02` 到 `ecee0b0e` 期间增加 #270，涉及生成类型、schema 和本包使用的黄金夹具。普通 merge 后重跑下表各项。源码与证据均可按提交核对。

| 检查（从仓库根目录，Cargo 使用 `--manifest-path core/Cargo.toml`） | 结果 |
| --- | --- |
| `cargo test -p semantic-core --locked --offline g04_` | [4/4 通过](https://github.com/nanake-lab/nemu-evidence/blob/841f7b18c3390ae3f6701806fc82a96ac8ce7b18/P5-06/part3-0360106e/G04/g04-0360106e.log)。覆盖逐边原因、A/M、辅助标注及历史重释；自签空集、错端点／用途／前沿、错事件／阶段／替换目标、缺前任均有反例。 |
| `cargo test -p semantic-core --locked --offline g05_` | [4/4 通过](https://github.com/nanake-lab/nemu-evidence/blob/841f7b18c3390ae3f6701806fc82a96ac8ce7b18/P5-06/part3-0360106e/G05/g05-0360106e.log)。覆盖规则选择、七类认识论标签、分项依据、账号归人及同片段单位、来源撤权和输入替换。 |
| `cargo test -p semantic-core --locked --offline` | [40 通过、0 失败、0 忽略](https://github.com/nanake-lab/nemu-evidence/blob/841f7b18c3390ae3f6701806fc82a96ac8ce7b18/P5-06/part3-0360106e/package-0360106e.log)，含首轮采纳、失败回读及六项注册表回归。 |
| `cargo clippy -p semantic-core --all-targets --locked --offline -- -D warnings`、`cargo fmt -p semantic-core -- --check`、`git diff --check origin/main...HEAD` | 通过。本机全工作区为 `not_run`；工作区验证交给 [GitHub 托管 CI](https://github.com/nanake-lab/nemu/actions/runs/45790545217)。 |

本包测试使用合成载荷、固定版本的读视图和事务实录夹具。真实存储原子发布、回滚、隔离调度、历史关系查全与生效提交祖先验证、聊天等待、Windows 全量检查均为 `not_run`。缺席的时间、阶段和单位须由后继提供正式事实，初版表未登记其载荷；桌面 IDLE 不生成缺席。属性记录生产到此为止，按目标装配证据和更新状态由后继负责。

首次托管构建在测试编译时发现同名 `IrlDispatch` 导入及测试视图借用期错误。改用本包明确类型并从视图借出目标，失败日志计入索引。[首次托管运行](https://github.com/nanake-lab/nemu/actions/runs/45788508155)

本 PR 只改 semantic-core 与本卡报告、追踪。没有新增依赖、修改 IDL 或增加差异编号。`attribute_tests.rs` 超过 400 行，在同一视图下同时检验正式载荷、规则选择、具体单位和三个核实轴；`irl_tests.rs` 和 `registry_tests.rs` 沿用前两部分的共同事务与前缀夹具。整卡未解除的验收缺口见 [BLOCKED.md](BLOCKED.md)。
