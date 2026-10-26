# P5-25 数值方法：V2-93 至 V2-98

本版依据已定稿的 V2-93 至 V2-98（main 已合入 #90）、设计原则 8和任务卡2026-10-26追加：时长采用V2-95的运行时间对数平均；成长异常按V2-97只报警、不暂停，只有成长计算故障拒绝候选。临时方法标识 `P5-25-v2-93-98-libm-0.2.16` 标识本地实现与夹具，不是已发布的正式规律资源/存储契约。历史定义和失败保留在 Git 历史、`evidence/` 与 `rounding-20261024/`。

## 输入与纯函数边界

`stage(previous,event,law,fuses)` 没有 x、m、R、模型、时钟或存储句柄。外层从已提交 H 与存好的 m=0 解释构造事件，来源 enum/bool 只表示已认证投影，不负责认证。只有外界事件、他人回应进入 C；自身行动、触发程序结果、自述、欺骗引起回应及无事件保持完整状态不变。

外源序号严格增加，事件时间不倒退，紧邻 ID 不重复；全历史 ID 唯一性由 H/Γ 保证。正常返回完整状态和现有 IDL 的 ThetaChange，由外层写 L、提交；Err 不修改借用的旧状态。按用户2026-10-26决定，本卡I13只验收纯函数暂存与输入不变；真实同事务写L及整段重放交由P5-26、P5-29验收。

显式状态包含 law、θ、质量/反应和/平方和、原始 variance/σ、对数时长和/质量/均值/τ、零时长计数、开放段、检测器、h、外源计数、上个事件和运行时间投影、慢漂方向/计数。没有隐藏缓存；重放比较整个状态，包括浮点位和负零。

## V2-93：两个参考及 binary64 次序

每个有限 binary64 加减乘除独立舍入，平方用 x*x，不用 FMA、fast-math、pow 或容差。结果零规范化 +0。NaN/Inf、非零乘积/商下溢为零、零分母、不可表示的计数/保留率等返回故障。g/λ 的 clip 仅执行原公式，不裁剪 b。

新增 `reference_binary64.py`（参考标识 `P5-25-binary64-mul-v1`）读取原19分片的500条流、72,822个事件，不重新抽样。流头、a 和来源字节不变，独立生成 b、variance、g；Rust 严格比较位模式，并比较 σ 与参考 variance 的 sqrt。Python sqrt 和 Rust libm sqrt 在本批输入一致。

原 reports/echo-core-v2-draft/check_law.py、generate_fixtures.py、diagnose_theta1.py、旧 fixture/失败记录全部不改。按用户2026-10-26决定，tests/reference.rs的旧 **2 测试只新增带理由的 #[ignore] 与注释，引用V2-93及原始证据；测试体与所有断言不改，显式运行仍在首处差异失败。新测试额外完整执行原范围和来源排除性质断言。新参考通过不把旧比较改为通过，也不是 P5-26 的完整 H 重算第二实现。

先计算本事件 h，再 rho=sub(1,h)。公共质量每事件算一次；通道顺序 connection、empathy、curiosity、achievement：

1. residual=sub(a,b_before)。
2. S_new=add(mul(rho,S_old),1)。
3. Q_new=add(mul(rho,Q_old),mul(residual,residual))。
4. A_new=add(mul(rho,A_old),a)。
5. b_new=div(add(mul(n_b,b0),A_new),add(n_b,S_new))。
6. v0=mul(sigma0,sigma0)；v_new=div(add(mul(n_sigma,v0),Q_new),add(n_sigma,S_new))。
7. sigma_new=libm::sqrt(v_new)；g_new=clip(div(mul(g0,sigma0),sigma_new),g_min,g_max)。
8. 更新时长后，lambda_new=clip(div(1,tau_new),lambda_min,lambda_max)。

先验不衰减，残差使用事件前 b。故障界不替代这条递推。

## V2-94：CUSUM

保留四通道双侧高斯均值 CUSUM。首事件初始化；之后 r=a−mean_before、s=σ₀、d=shift_sigma*s，固定运算：

`linear=(d*r)/(s*s)`；`penalty=(d*d)/(2*(s*s))`；
`positive=max(0,old_positive+(linear-penalty))`；
`negative=max(0,old_negative+(-linear-penalty))`。

任一通道达到 threshold 算一次联合变点，全部累积量清零，均值设 a，段计数设1；否则 mean=old_mean+(a-old_mean)/new_segment_count。h=max(h_min,(n_h*h0+changes)/(n_h+external_events))；本事件所有旧统计乘本步实际 rho。

shift_sigma=1、threshold=8、n_h=10 仅为预先固定的合成夹具，待实验谱系校准，本轮未调参。仅方差改变、长期低幅变化、重尾/相关噪声不保证检出或误报率。不复用 R 的二元后验；替换方法须新规律版本并从 H 重算。

## V2-95：运行时间与对数平均

RunningTime::from_history(origin_ns,event_ns,stops) 只做纯整数投影：

`running_ns=(event_ns-origin_ns)-sum(stop.end_ns-stop.start_ns)`。

调用方从同一已提交 H 前缀提供稳定原点、事件时间、V2-78 全部已闭合停机区间。构造器检查排序、非空、无交叠、范围在 origin..event 内及整数溢出。每步检查 event_ns 对应当前事件、原点不变、累计停机不倒退、停机增量≤墙钟增量、运行时间不倒退。字段只能经构造器生成。

这些校验不能证明来源真实、完整或前缀内区间身份一致（等长替换仍可能通过）。外层必须绑定 H 前缀、认证全部停机记录及事件不在停机内；不读机器 uptime/当前时钟，不按重算耗时补值。超出本卡的接口见请求文件。x 经 Φ 按实际经过时间回落的 V2-78 不变。

每通道保存 sign(a−b_before) 及非零段起点的累计 running_ns；符号改变或变零时关闭前一非零段，T_ns=running_ns−start_ns：

- T_ns=0：只递增 zero_duration_count，不向对数和/完成质量加项，不补 ε。
- T_ns>0：要求 T_ns<2^53，精确整数转 binary64 后一次除以 1,000,000,000 得秒数。更长段返回既有 TimePrecision，保留旧状态。约104.25天的单段精度限制待后继时间契约处理，不能偷偷截断。
- 未结束段作为右删失排除；不加当前长度、密度、生存项或当前 λ 的尾部外推。
- 无外源事件不更新。每个外源事件仍衰减以前的完成段，即使本步无新完成段。

固定数值次序，log=libm::log，exp=libm::exp，锁定现有 libm 0.2.16：

1. U_new=mul(rho,U_old)，M_new=mul(rho,M_old)。
2. 仅完成且 T>0 时：q=finite(log(T))；U_new=add(U_new,q)；M_new=add(M_new,1)。
3. p=mul(n_tau,finite(log(tau0)))。
4. z_new=div(add(p,U_new),add(n_tau,M_new))。
5. tau_new=finite(exp(z_new))，若为零返回故障；然后算 λ。

即 τ̂=exp((n_tau·ln τ0+Σω·ln T)/(n_tau+Σω))。log 返回值属于版本化 binary64 数据，不能换系统库、代数重排或以 exp(−z) 替换规范的 1/exp(z)。genesis 同样经过 div(mul(n_tau,log(tau0)),n_tau) 再 exp，避免无完成段时因初始化路径不同出现变化；因此 τ̂ 初值不承诺等于 τ0 位模式（夹具10秒得到10.000000000000002秒）。本次数值定义必须随规律版本保存。n_tau=10 也只是夹具，待第16节校准。

## V2-96：推导的计算故障界

Δθ_max 不是配置项。每步输出 StepBounds，次序 b[4]、g[4]、λ[4]，有效界取范围界和单事件界较小者。仅 abs(after−before)>limit 返回 ComputationBound；等于界合法。越界必是违约，界内程序错误不保证被检出。

### 恒等式与先验衰减

通用矩 y=(n*p+U)/(n+M)，本步追加 d 个观测（b/v 的 d=1，时长 d∈{0,1}），观测 o：

D=n+rho*M+d；
`y'−y=[n*(1−rho)*(p−y)+d*(o−y)]/D`。

b 取 p=b0、o=a、M=S；v 取 p=σ0²、o=(a−b_before)²、M=S。即使 a=b，先验项仍可引起非零变化，不能只用 |a−b|/D 作界。

旧 y 已舍入，不一定等于存储统计量的实数商；实现显式带入旧商残差 E=(n*p+U)−y*(n+M)，恒等式分子另加 rho*E。这覆盖 genesis、先验平方与乘法的舍入差异。

### g 与 λ

实数范围界 |Δb|≤2、|Δg|≤g_max−g_min、|Δλ|≤λ_max−λ_min。K=g0*σ0，clip 为1-Lipschitz：

`|Δg|≤min(g_max−g_min,K*|v'−v|/[sqrt(v')*sqrt(v)*(sqrt(v')+sqrt(v))])`。

实数 S 从0开始且 h≥h_min>0 时，S≤1/h_min，v≥n_sigma*σ0²/(n_sigma+1/h_min)>0。实现不拿实数界硬比较浮点 S，而用本步存储统计量的区间给出更紧的 v/v' 正下界；若证明区间不能给出正性或有限的更紧界，则保留始终成立的范围界并标记 range_only；不能把证明工具的精度局限误报为生产计算故障。

时长 z 用同一个矩恒等式，p=log(τ0)、U=对数和、M=质量、d=完成指示、o=log(T)。T∈[10^-9,(2^53−1)/10^9]秒，实数 z 位于先验和这些 log(T) 的凸包。倒数映射满足：

`|exp(−z')−exp(−z)|≤exp(−min(z,z'))*|z'−z|`。

实现用 z/z' 区间较小下端点求斜率，与 λ 范围界取小值。实际 λ 仍用1/exp(z)。没有外推未结束段，也没有可调 λ 故障阈值。

### 按实际运算计入浮点余量

interval.rs 只构造证明界，不写回估计器。点区间来自实际 binary64 旧状态/输入；每次 +、−、×、÷ 端点向外扩一个相邻 binary64，覆盖各自舍入及次正规数。乘除检查四种端点组合；除数跨零、非有限端点拒绝。平方残差先用减法区间包住；先验 σ0² 和 K 也按原乘法包住。

通用矩同时算原递推区间像 I、含 E 的恒等式区间 J、R=abs(I−(point(y)+J))。单事件矩界为向外求和的 abs(J)+R，再向上一个相邻数覆盖最终 Δ 舍入。R 是本次运算的前向误差包络，包含独立区间造成的保守量；没有经验 epsilon、拟合系数或事后调宽。

sqrt 端点向外一邻数，libm 0.2.16 sqrt 是正确舍入实现。exp 端点向外两邻数：锁定源码 src/math/exp.rs 的误差说明为 <1 ulp，两邻数覆盖幂次边界两侧间隔不同。log 使用数值定义返回的 binary64 点值，不另加对实数 log 的近似误差。

g/λ 另加旧 θ 与旧统计量映射之间的舍入差、新映射区间宽度，最后向上一邻数包住比较时的 subtraction。g 的 K/sqrt/分母/clip 和 λ 的 exp/倒数/clip 均按同一操作级区间传播。g、λ 范围宽度向外舍入；b 的2精确。仅证明过程中溢出、方差下端点非正或无法形成有限的单步界时，该分量使用 V2-96 固定范围界，StepBounds.range_only 显式标记。这不是可调阈值，也不修改候选；保证仍是超界即违约、界内错误不保证发现。生产递推的非有限/下溢/时间违约已在此前返回故障，不能由此回退掩盖。

next_up/next_down 只移动证明端点，绝不移动候选值、oracle 或验收阈值；Rust 与新参考仍严格 to_bits() 相等。

## V2-97：报警与成长计算故障

按V2-97，成长异常只报警、不暂停。DynamicsFuses只保留报警夹具jump_notice、drift_run、drift_margin，待实验谱系校准。jump_notice与V2-96的Δθ_max分开：前者只触发通知，后者严格大于才构成计算故障；等于计算界允许通过。

跳变/慢漂：候选 θ、全部统计、事件进度与慢漂计数照常返回，同时输出 Jump/SlowDrift 和 ThetaChange；通知/复核由外层/P5-26 实现，不暂停成长、对话或行动。

计算故障：非有限、超出推导界、无法满足版本化数值前提等返回 Err，没有半个候选可提交；旧 θ、估计器和事件游标完整保留。不裁剪后继续、不吞故障事件。外层记录故障，修复后从 H 补算。来源/时间校验失败另返回输入故障，不当作正常经验提交。

第二套完整 H 重算、缺失/超时处理、实现不一致裁决、八个中性探测、绑定 law 与 H 前缀的 H/L 系统记录属于 P5-26/外层。本卡不伪造其验收；结构故障仍按 §12 门控。

## 复现与证据保全

PowerShell，仓库根，参考输出目录必须全新：

```powershell
python -B reports/P5-25/reference_binary64.py --out-dir D:/Dev/cache/P5-25/reproduce-reference
Set-Location core
cargo fmt --all -- --check
cargo clippy -p core-kernel --all-targets --locked -- -D warnings
cargo test -p core-kernel --locked --no-fail-fast -- --nocapture
cargo test --workspace --locked --no-fail-fast
```

常规验收仅使用新参考的严格逐位对照；旧 **2 对照按用户明确决定标为诊断性 ignored，失败不改写为通过。可显式复现：cargo test -p core-kernel --locked dynamics::tests::reference::theta_1_original_500_streams_zero_violations -- --exact --ignored --nocapture；该诊断预期仍失败，不属于常规验收。原 fixture 是 Python3.12.14 产物，新参考环境与哈希见 batch4-20261026/reference/reference.json。每条命令结果以 REPORT 和独立 attempt 日志为准。


## 2026-10-26 纯函数验收边界

用户明确本卡 I13、AC2-09 只验收纯函数能证明的部分。tests/pure_boundary.rs 以编译期函数类型检查 C 不含 x、m、R、L 或提交能力参数；调用方保有的 x 初值与 m 取 9 组组合，同一段128个外源事件逐前缀核对 θ 的 to_bits() 及完整估计器状态。夹具不模拟真实运行时、不声称已测试全管线。暂存测试证明借用输入不变、丢弃结果不推进游标、重复计算相同；C返回 ThetaChange，真实写L与提交仍由P5-29负责，独立整段重放由P5-26验收。

最新main已登记P5-36负责V2-98/V2-103正式估计器契约；V2-100要求检测与异常阈值随规律版本/校准资源登记，u*不能调。现有DynamicsFuses只是本地校准资源视图，不能当作u*的可调边界。
