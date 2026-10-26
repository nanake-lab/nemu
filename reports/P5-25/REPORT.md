# P5-25 动力学 C：θ 的学习与回应回路 — 报告

## 结果

**部分完成，PR #77 保持草稿。** 已按 2026-10-26 第4批定稿 V2-93 至 V2-98 更新纯 C、数值定义、测试和接口请求；新增 binary64 参考的500条原始流、72,822事件逐位差异为0。原 **2 对照及其失败原样保留，当前 core-kernel 测试36通过/1失败，不能宣称全套验收通过。I13 实际 L 事务和 AC2-09 全管线集成尚未完成，见 [BLOCKED.md](BLOCKED.md)。

原 head e294d4287fd81f1986d34ee6ab510722eb401621；fetch 后 origin/main 为 574eaa6（含 #90），以 merge 提交 700b040 合入，没有 rebase/强推。差异登记的追加冲突保留双方历史与 main 的新定稿内容。原仓库未提交工作没有搬入本分支。

## 交付物

- dynamics/time.rs、duration.rs：从 H 时间/停机记录的纯投影，运行时间对数平均、零时长独立计数、未完成段排除。
- dynamics/bounds.rs、interval.rs：按范围及单事件恒等式构造故障界，含先验衰减、旧商舍入残差、g 倒平方根与 clip、λ 对数时间范围和操作级浮点包络。
- dynamics/types.rs、mod.rs、update.rs：正常报警照常应用完整候选；计算故障 Err 保留事件前全部状态；保留现有 IDL ThetaChange 暂存。
- dynamics/tests/batch4.rs、binary64.rs：运行时间/对数统计/故障边界/原子性回归，新严格参考；原 tests/reference.rs 未改。
- [reference_binary64.py](reference_binary64.py)、[本轮证据](batch4-20261026/)：读取原输入，不换种子，不覆盖历史；记录全部失败尝试及最终检查。
- [METHODS.md](METHODS.md)：新的版本化次序和界的推导；[接口请求](../../tasks/requests/P5-25-dynamics.md)：H 时间认证、正式存储契约、I13 与 P5-26 需求。

本轮新改动只在用户允许的 dynamics/、reports/P5-25/、指定请求文件内。main 合并带入的其他文件不计作本卡新实现。没有改 core/contracts/、用户/系统设置或 status.json。

## 验收

证据基目录为 batch4-20261026；旧 evidence/、rounding-20261024/ 与各验收编号旧 result.json 不覆盖。

| 命令或检查 | 结果 | 证据和边界 |
| --- | --- | --- |
| 原 Python check_theta/check_feedback_loop | passed，0违例 | original-python.json；原代码/种子/阈值未改 |
| THETA-1 V2-93 新 binary64 参考 | passed，500流/72822事件/0位差 | checks/final-1/03-cargo-test-core-kernel；逐位 b/variance/σ/g + 原性质断言 |
| 历史 **2 逐位对照 | **failed，保留** | 同上：tests/reference.rs:56，4602808250010281358 vs 4602808250010281359；历史首处 σ 差1 ULP |
| 新参考 Python3.12.10 / 3.12.14 再生成 | 相同 SHA-256，输入差异0 | reference/reference.json、reference-python31214.stdout.txt |
| THETA-2 / AC2-25 性质部分 | passed | 固定ρ与自适应各7场景，原0.03/−0.7/−0.95阈值未改；不代表完整实验谱系校准 |
| AC2-14 / AC2-05 θ 部分 | passed（纯 C） | 各自致来源和无事件完整恒等；外源正例改变；128前缀重放，新增停机重放 |
| AC2-09 θ | passed（纯 C 边界）；全管线 not_run | 签名不接收 x/m/R，拒绝非m=0解释；外层真实改变x/m的集成尚未接通 |
| I13 | 暂存单元 passed；L事务 not_run | ThetaChange before/after/事件ID有效，正式快照/估计器/L原子提交在外层 |
| V2-95/96/97 定向测试 | passed | 9项batch4测试，含零时长、1ns、停机、变化ρ、先验衰减、严格大于、非法候选/非有限整状态拒绝；原报警测试改为继续应用 |
| cargo fmt --all -- --check | passed | checks/final-format；final-1 新增断言的格式失败日志保留，rustfmt 后通过 |
| cargo clippy -p core-kernel --all-targets --locked -- -D warnings | passed | attempt-2/02-cargo-clippy；attempt-1 参数过多失败原样保留，封装参数结构后修复 |
| cargo test -p core-kernel --locked --no-fail-fast -- --nocapture | **failed：36 passed / 1 failed** | final-1/03-cargo-test-core-kernel；唯一失败是原 **2 对照，0 ignored |
| cargo clippy --workspace --all-targets --locked -- -D warnings | passed | checks/final-1/02-cargo-clippy-workspace |
| cargo test --workspace --locked --no-fail-fast | **failed：103 passed / 1 failed（含doc-tests）** | checks/workspace-1/02-cargo-test-workspace；约9分钟完整结束，core-store 31单元+2文档测试均通过 |
| ruff / mypy --strict 新参考脚本 | passed | checks/python-1/；复用 core/contracts 锁定工具 |
| Ubuntu CI | 发布后核验 | 不能以本机Rust结果代替整个跨语言CI |
| Windows专项/GPU/用户谱系 | not_run | 无相应实现改动；未访问正式/排练存储，未启动GPU测量 |

全工作区运行后，新增极小合法方差的范围界回退及1项回归；final-1 重跑全工作区Clippy和core-kernel全套。final-format只修正新增断言的排版，无语义变化。其余crate未改变，未反复运行慢测试。

原历史证据24项哈希全部一致，保留路径与原head逐文件diff为0，见 preservation.json。代码与说明文件的 git diff --check 通过；对全部暂存文件检查时只报告4个原始cargo stdout末尾空行，原始日志不为格式检查而改写。

## 浮点界的实测与局限

V2-96 的界不是调参项。采用操作级向外区间：每个基本算子向外一邻数，sqrt一邻数，libm0.2.16 exp按 <1ulp 误差说明向外两邻数；仅证明端点变化，候选值和严格oracle完全不动。旧商舍入残差 E 与本次递推包络一起计入，最后覆盖比较 subtraction。完整公式和理由见 METHODS 的 V2-96。

a=b 的回归实测 Δb=4.4507924433645485e−4，推导单步界4.450792443374784e−4；非零变化来自 n(1−ρ)(先验−当前)，不能删掉该项。三个边界先验各500次极端反应均未越界；等于界允许，next_up(limit) 拒绝。该检查只能发现越界违约，不能证明界内实现无错。极小的合法次正规方差可能使证明区间跨零或导数分母下溢，此时采用已证明的范围界，StepBounds.range_only显式标记；不能把证明工具的精度局限当作成长故障。新增回归通过，普通1500事件边界夹具的range_only全部为false。生产递推的非有限/下溢等仍拒绝候选。

完成时长的纳秒差仍要求 <2^53（约104.25天）；超出返回 TimePrecision，不能当成已覆盖长期运行。时间投影的来源真实性/完整性由外层认证，当前构造器只验证数值与次序一致性。CUSUM 的 shift_sigma=1、threshold=8、n_h=10，以及 n_tau/报警夹具，仍待实验谱系预登记校准。

## 规格对照

本轮目录约束不允许改 docs/traceability/P5-25.md，以下作为本版对照，旧对照中的候选语义以 V2-93–98 定稿取代。

| 规格条款 | 实现位置 | 测试/边界 |
| --- | --- | --- |
| ADR0019 §8 / V2-93 | numeric.rs、update.rs、新Python参考 | binary64.rs 严格位比较；原reference.rs失败不改 |
| v2.0 §8 / V2-94 | detector.rs（保持） | Gaussian似然/均值变点、h下限、THETA-2 |
| V2-78 / V2-95(a) | time.rs / ReactionEvent | 扣停机、记录重放、错误投影拒绝；H认证待外层 |
| V2-95(b)(c)(d) | duration.rs / types.rs | 几何均值、每步ρ、零计数、1ns、未完成段不加观测 |
| V2-96 | bounds.rs / interval.rs / finish | 先验衰减、g/λ映射界、严格大于、边界序列 |
| 原则8 / V2-97 | update.rs / StagedDynamics | 跳变慢漂照常应用，计算故障完整拒绝；P5-26复核另卡 |
| V2-98 / I18 | DynamicsState / 请求文件 | 全状态显式、无全局缓存；正式存储契约另开卡 |
| AC2-05/09/14/25、I13 | tests.rs / tests/reference.rs | 纯函数证据及未完成集成如上，未冒称全卡完成 |

## 环境与依赖

Windows，Rust1.99.0；原fixture Python3.12.14，新参考首次用3.12.10，再用3.12.14校验同字节。本轮按角色名启动 routine_reporter（gpt-6-luna/max）仅执行机械命令和证据整理；主代理解释规格、实施修改和判断验收。

Rust编译/测试串行，环境变量仅对子进程有效。CARGO_HOME=D:/Dev/cache/cargo，CARGO_TARGET_DIR=D:/Dev/cache/P5-25/target，TEMP/TMP=D:/Dev/cache/P5-25/tmp；Python工具环境在D:/Dev/cache/P5-25/contract-venv。测试只用固定合成输入与新分配临时存储。

没有新增依赖或修改锁文件。复用 libm0.2.16（MIT）及现有Python静态工具；新参考仅标准库。未下载模型、未使用用户数据。

## 待用户确认

- [ ] （阻塞）I13 与 AC2-09 的外层集成及正式估计器契约由后继授权卡完成；当前不能按全卡完成合并。
- [ ] （阻塞）高难度卡独立交叉审查尚未进行，测试全套仍红。
- [ ] （不阻塞当前纯C开发）实验谱系校准夹具；不得将当前数值作为正式生活参数。

V2-93–98 已定稿的选择不再重复列作待批准候选。

## 后续建议（不在本任务范围内）

按请求另开 V2-98 契约卡、接入 H 时间投影和 L 事务；P5-26 实现独立完整重算与报警记录；P5-30 校准检测/时长/报警参数。保留历史 **2 失败作为诊断证据，不通过改阈值、换种子、ignore 或删除断言清绿。

## 下一步

等待缺失集成与交叉审查；PR #77 保持草稿并评论列明完成和欠项，不合并、不取下一卡。旧云端执行/舍入诊断历史见 evidence/ 和 rounding-20261024/，本轮报告不改写其失败结果。
