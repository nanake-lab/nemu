# P5-25 动力学 C：θ 的学习与回应回路 — 报告

## 当前状态

按用户2026-10-26明确的纯函数范围完成代码调整，并按随后批准恢复固定libm依赖（0cf1605）。本机fmt、Clippy和工作区常规测试通过（90 passed / 0 failed / 1 ignored，含2个doc test）；旧 **2 显式诊断仍为failed。CI尚未全绿，保持草稿。修复后的Ubuntu run 40283963199因GitHub账户付款/支出限额问题未启动任何步骤，Windows runner也处于offline。外部条件恢复并确认最终head CI全绿后，才转可审查并留言「请审查」；不合并、不取下一卡。本轮不实现事务层写L或独立整段H重算，也不将它们伪装成本卡已通过的集成。

本次先fetch，将 origin/main@716f7f0 以merge a9b5e72合入；收到依赖恢复批准后再次fetch，并以merge e26bbcf合入origin/main@2bc04ba。未rebase/强推。保留上轮abebb85的V2-93至V2-98实现、原始输入与失败证据；main合入P5-27及后继契约卡P5-36后重新核验。除main合并外，新改动只在授权的dynamics/、reports/P5-25/和指定请求文件，以及用户本轮额外批准的kernel Cargo.toml唯一依赖行与Cargo生成的共享锁文件。未提交status.json、未改用户/系统设置。

## 用户确定的验收边界

- **THETA-1：** 数值验收使用新 x*x binary64 参考严格逐位比较。原 **2 对照标为带理由的 #[ignore]，属性/注释引用V2-93及证据路径，保留整个测试体、所有性质断言、原Python检查和失败记录。它是可显式运行的已知失败诊断，不把 ignored 写成 passed。
- **AC2-09：** C的函数签名不含x、m、R。调用方x初值和m取9组组合，在同一段128个外源事件的每个前缀比较θ位模式及完整估计器状态；所有组合一致。这里的x/m是纯函数调用边界夹具，不声称连接了真实运行时。
- **I13：** C只返回暂存ThetaChange，没有L或提交能力。测试检查before/after/事件ID、输入完整不变、丢弃候选不推进事件游标、重复计算结果相同、无事件不产生条目。真正同事务写L由P5-29验收，独立完整H重放由P5-26验收。

这些边界由本轮用户直接决定；不再把相应事务层未接通列为P5-25阻塞。包级依赖恢复也已获得明确批准并完成。当前阻塞是CI运行条件，详见BLOCKED.md。正式存储契约由main已登记的P5-36承接。

## 交付物

- dynamics/：纯C暂存、外源资格、四通道CUSUM、b/σ/g递推、运行时间对数时长、公式故障界、报警照常应用与计算故障原子拒绝。
- tests/binary64.rs：原500流/72,822事件的新参考逐位比较及原性质断言；tests/reference.rs保留原对照并明确diagnostic-only。
- tests/pure_boundary.rs：本轮新增AC2-09、I13纯函数边界检查；无真实L/存储mock。
- [METHODS.md](METHODS.md)：固定运算次序、ln/exp版本、时间投影来源与校验、范围及单事件界和实际浮点余量。
- [接口请求](../../tasks/requests/P5-25-dynamics.md)：P5-26/P5-29/P5-36的H时间认证、事务、重放、存储契约与长期时间域需求。
- closeout-20261026/：本轮命令结果与旧CI失败证据；各验收编号新增closeout记录，不改旧result.json或batch4记录。

## 验收

本轮证据目录为closeout-20261026/；各验收编号的closeout-20261026.json引用实际测试名与日志。checks/attempt-2/toolchain.txt记录启动时merge head e26bbcf，dependency.diff记录当时未提交的批准补丁；随后提交0cf1605与受测core目录完全一致（source-scope-final.json）。kernel为33 passed / 0 failed / 1 ignored。

| 命令或检查 | 结果 | 证据/范围 |
| --- | --- | --- |
| cargo fmt --all -- --check | passed | checks/attempt-2/results.json、fmt.log |
| cargo clippy --workspace --all-targets --locked -- -D warnings | passed | checks/attempt-2/clippy.log；此前CI的E0433失败记录保留 |
| cargo test --workspace --locked --no-fail-fast | passed：90通过、0失败、1 ignored | checks/attempt-2/workspace.log；旧 **2 ignored，新参考与性质测试正常运行 |
| THETA-1 新binary64参考 | passed，0位差 | 原500流/72822事件，严格b/variance/σ/g位比较，不加容差 |
| THETA-2 / AC2-25性质部分 | passed | 固定ρ和自适应各7场景，原阈值保持 |
| AC2-14 / AC2-05 θ | passed | 外源正例、自致/无事件完整恒等；128前缀纯函数重放 |
| AC2-09 θ纯函数边界 | passed | 9组x初值/m，每个前缀θ和完整估计器逐位一致 |
| I13纯函数暂存 | passed | 只返ThetaChange，不写L、不提交、不修改输入 |
| V2-95/96/97 | passed | 停机扣除、对数平均、零时长/开放段、先验衰减、严格大于、计算故障/报警区别 |
| 旧 **2 显式诊断 | failed，exit 101；原1 ULP差异复现 | checks/attempt-2/legacy-diagnostic.log；不属于常规验收，仍是失败 |
| Ubuntu CI @43da867（修复前） | failed，run 40272465565 | Rust缺libm，未进入测试；TS/Python步骤通过，历史记录保留 |
| Ubuntu CI @0cf1605（修复后） | failed，run 40283963199，未启动检查 | 账户计费门控，steps为空；ci-code/run-40283963199.json |
| Windows W02 | not_run | runner offline；草稿自动作业skipped，不算passed |
| 新Python参考ruff/mypy、两版Python同字节 | passed（上一轮，本轮脚本未改） | batch4-20261026/checks/python-1、reference-python31214.stdout.txt |

## 历史失败保留

旧 **2 对照与显式乘法在首处σ差1 ULP：4602808250010281358 / 4602808250010281359。原平方的37处差异、传播到σ的13处差异及精确整数舍入诊断见 rounding-20261024/；原失败见 evidence/tests/dynamics-attempt-2.log、THETA-1/result.json。上一轮常规kernel 36通过/1失败、workspace 103通过/1失败及fmt/Clippy早期失败日志均不改写。旧CI run 40264985040 也因该诊断失败，本轮保存其失败步骤摘要于 closeout-20261026/ci-before/。

本轮改变的是诊断的运行分类，由用户明确授权；没有删除断言、改变输入或阈值。新参考仍要求严格逐位相等。旧Python原性质检查独立保留。随后CI的E0433失败和修复后账户计费阻止作业启动的失败也分别保存，不因后续成功而抹去。

## 数值定义与边界

CUSUM的shift_sigma=1、threshold=8、n_h=10只是预先固定夹具，待实验谱系校准。n_tau与报警参数也未定为正式生活值。按V2-100，检测参数属规律版本，异常报警阈值属随规律版本登记的校准资源，u*不能调；Δθ_max不是调参项。

持续时间取H事件时间减去已记录停机区间，不测机器uptime。正完成段以ln T加入对数和/质量，旧统计每个外源事件乘本步ρ；零时长只递增审计计数；未完成段不计入、不按当前λ外推。ln/exp锁定libm0.2.16及METHODS的运算次序。

Δθ_max含范围界与单事件界、n(1−ρ)(先验−当前)、g倒平方根/clip、λ对数时长映射和操作级浮点包络。只严格大于触发，等于合法。极小合法方差使证明区间无法收紧时，range_only标记并使用已证明范围界，不误报生产故障；原始候选不移动。该界不保证发现所有界内错误。

时间投影构造器只校验数值/顺序，真实性、完整性与H前缀绑定归调用方。完成段纳秒差仍要求<2^53（约104.25天），超出TimePrecision，不悄悄截断；长期时间域交后继卡处理。这些限制被明示，不冒充正式谱系完整验收。

## 规格对照

| 条款 | 实现位置 | 测试/本卡边界 |
| --- | --- | --- |
| ADR0019 §8、V2-93 | numeric/update、新Python参考 | binary64严格位比较；旧reference诊断ignored，失败保留 |
| V2-94 | detector.rs | CUSUM、h下限、THETA-2 |
| V2-78、V2-95 | time/duration/types | 扣停机、几何均值、零计数、开放段排除、重放 |
| V2-96 | bounds/interval | 先验衰减、g/λ界、相等允许、越界拒绝、次正规正例 |
| 原则8、V2-97 | update/StagedDynamics | 报警仍应用；数值故障完整拒绝 |
| V2-98/103、I18 | DynamicsState、请求文件 | 全状态显式；P5-36负责正式编码 |
| AC2-09、I13（用户2026-10-26决定） | tests/pure_boundary.rs、stage签名 | x/m变化不影响完整结果；只暂存、不写L/提交 |
| AC2-05/14/25、THETA-2 | tests.rs、tests/reference.rs | 纯函数前缀重放、来源资格、反馈性质 |

## 环境与新增依赖

本机Windows/Rust1.99.0；编译缓存与测试临时目录在D:/Dev/cache/P5-25/，环境变量仅进程内有效。只用合成输入与临时存储，不访问用户谱系/系统设置。子代理本轮只做只读CI与证据核验，重负载由主代理串行执行。

用户批准恢复被main移除的原依赖libm0.2.16（MIT，default-features=false）。cargo update --offline -p core-kernel只向共享锁文件增加该包及core-kernel的依赖关系，没有改变其他包版本。未引入新数学实现；新参考仅Python标准库。PR #94也恢复同一行，后合并者解决冲突并只保留一行。

## 待用户确认

- [ ] （阻塞）恢复GitHub hosted Actions的计费运行条件和Windows runner；最终head CI通过后再转可审查。
- [ ] 高难度卡独立交叉审查；本会话不自审、不合并。

用户本轮已决定的诊断分类与纯函数边界不再列成待决项。实现验收问题如有失败在BLOCKED.md与最新PR评论明确记录。

## 后续（不属于P5-25验收）

- P5-26：独立完整H→(R,θ)重算、来源更正/复核、第二实现不可用的处理、H/L报警记录及整段重放验收。
- P5-29：真实事件管线与来源/停机记录认证，同一事务写ThetaSnapshot、完整估计器和L，原子提交/故障回退及运行时AC2-09集成。
- P5-36：V2-98/V2-103正式估计器契约、编码/校验、round trip和迁移；承接请求文件中的时间域需求。
- 实验谱系后继卡：预登记并校准CUSUM、n_tau、报警夹具。P5-25性质测试不能替代生活参数校准。

## 下一步

等待外部CI条件恢复；随后重跑最终head检查，全部通过才转为可审查并留言「请审查」。最新head及CI状态以PR检查页和本轮进度评论为准，不以旧提交的结果替代。保留草稿时不请求正式审查。
