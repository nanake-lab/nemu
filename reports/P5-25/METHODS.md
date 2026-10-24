# P5-25 参考方法与运算次序（编号：待定）

这是待审查的临时规律资源，不是已批准的 law version、IDL 或校准 profile。基线 main 为 `0b5412e95caab447c70148c473e544dbab28c57c`；已读 PR #73 head `25e8474958c7ccd9d0d37266a881c8df733f7058` 的 θ 提醒：必须使用自己的连续反应似然与反馈稳定性检验，不照搬 R 的二元后验，不绕过 h 下限。

## 输入、状态与事务边界

`stage(previous, event, law, fuses)` 是纯函数；没有 x、m、R、模型、系统时钟或存储句柄。调用者须从已提交 H 和存好的 m=0 解释构造 `ReactionEvent` 并验证来源。布尔标记不代替来源认证。只接受外界事件与他人回应；自身行动、触发程序结果、自述、欺骗引起回应原样返回整个状态。外源序号严格增加、事件时间不倒退；重复的紧邻 ID 拒绝，跨历史 ID 唯一性由 H/Γ 负责。

估计器状态显式包含规律参数、b/g/λ、权重/反应/平方和、σ、时长和/质量、τ、未闭合符号段、CUSUM 均值/累积量/计数、h、上次事件与慢漂计数。没有全局缓存。私有状态只能由 genesis 与 C 构造；重放从同一 genesis 按同序调用 C。测试比较完整结构并比较有限浮点的 round-trip Debug 表示（包括 −0），不只比较 ThetaValues。

正常结果暂存现有 IDL `ThetaChange { before, after, events }`；外层事务负责绑定资源、追加 L 并提交。此卡未实现 I13 的事务/持久化集成，不生成虚构 LedgerEntry，也没有把临时状态序列化为已冻结 IDL。`tasks/requests/P5-25-dynamics.md` 列出后继接口需求。

## 连续反应变点参考

选择双侧高斯均值变化 CUSUM 候选，四个通道逐个固定次序计算。参考均值为自最近检测变点起的样本均值。首事件仅初始化；以后残差 `r=a-mean_before`，`s=σ₀`，候选均值移位 `d=shift_sigma*s`。对连续反应的高斯似然比为：

`linear=(d*r)/(s*s)`，`penalty=(d*d)/(2*(s*s))`；
`positive=max(0, old_positive+(linear-penalty))`；
`negative=max(0, old_negative+(-linear-penalty))`。

任一通道达到 threshold 算一个联合变点，全部累积量清零，均值设为当前 a，段计数设 1；未触发则 `mean=old_mean+(a-old_mean)/new_segment_count`。h 为 `max(h_min, (n_h*h0+changes)/(n_h+external_events))`。先计算本事件 h，再用 `ρ=1-h` 衰减旧统计；自适应时旧权重为各后继保留率的乘积，常量ρ时恢复 `ρ^N`。

这不是变化概率的二元反应后验，也不是任意分布变化的完备检测器。固定 σ、经验均值、通道 OR 与重置策略都需要校准；对仅方差变化、长期低幅变化、重尾/相关噪声不作检出或误报率保证。测试中交替 ±0.05 噪声不记变点，持续移至 0.75 会触发；独立连续高斯 log-density 差验证似然式。参数 `shift_sigma=1`、`threshold=8`、`n_h=10` 仅为固定合成夹具，未因测试结果调参。

## 统计递推和时长

先用事件前 b 求残差。固定次序：`weight=ρ*old_weight+1`；`square_sum=ρ*old_square_sum+residual*residual`；`reaction_sum=ρ*old_sum+a`；之后按 §8 的带先验分式算 b 与 variance，再 `σ=libm::sqrt(variance)`。g 为 `clip((g0*σ0)/σ,gmin,gmax)`。先验永不衰减。

每个通道记录 `sign(a-b_before)` 的开始时间；正负号改变或变为 0 时关闭前一非零段，加入完整持续秒数，质量加 1。两个时长统计也按每个外源事件乘ρ。未关闭段视为右删失，不把当前长度算作完整观测；τ 为 `(n_tau*τ0+duration_sum)/(n_tau+duration_mass)`；λ 为 `clip(1/τ,λmin,λmax)`。`n_tau` 是待批准的临时参数。相同时间允许零时长；无事件不推进任何统计。纳秒差必须小于 2^53 才转 f64（约104天），否则显式 TimePrecision；此限制需要正式时间契约审查，不偷偷截断或调整时间。

## 熔断候选

逐个比较 b[4]、g[4]、λ[4] 的变化。任一 `abs(delta)>=max_step` 给跳变信号；每分量记录非零同向变化的连续次数，达到 drift_run 且离该方向边界不超过 drift_margin 时给慢漂信号，零变化重置计数。绝对距离单位和每分量共用阈值尚待确认。

最保守隔离：任何信号保留**整个**事件前状态，返回 proposed_values 和信号，L 暂存为空；不只裁剪 b/g/λ 后悄悄推进估计器。这样不发布超限状态，也不冒充正式的 P5-26 暂停/复核实现。外层必须记录信号、暂停并遵照已批准规律处理；不能继续消费后续事件假装这个事件已提交。该语义与潜在“限幅后提交”不同，列为阻塞差异。

## ADR 0019 §8 数值边界

所有权威运算使用有限 binary64、显式逐步次序；未使用 mul_add、fast-math、nextafter 或容差修补。每次存结果规范化 −0；输入规律的 −0 baseline 拒绝。NaN/Inf、非零乘积/商下溢成0、零分母、计数达到2^53、不能表示有效ρ均显式故障，完整候选不发布。g/λ 的 clip 仅执行规格允许的边界。

**已发现并保留的失败：** 原 Python 的 `residual ** 2` 与显式 `residual*residual` 有平台舍入差异。首处 σ 1 ULP、全数据诊断13处；没有把它归为可容忍误差。见 BLOCKED.md。检测器似然式单元测试中的 1e-14 仅核对两个代数表达式的诊断等价，不参与任何生产决策；THETA-2 的 0.03、−0.7、−0.95 均原样保留原检查阈值。

## 复现

环境按报告指定版本；在仓库根运行：

```sh
uv run --project core/contracts --locked python reports/P5-25/generate_fixtures.py
uv run --project core/contracts --locked python reports/P5-25/diagnose_theta1.py
cd core
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p core-kernel --locked dynamics -- --nocapture
cargo test --workspace --locked --no-fail-fast
```

首次 fixture 生成使用 Python3.12.14；生成器会覆盖派生 fixture，正式复现应在新的工作副本进行，以免覆盖已归档原始证据。预期当前 Rust THETA-1 严格 oracle 失败，不能当成已完成卡。原检查脚本未修改；二进制格式、种子、源哈希、fixture 哈希都在生成器及 `original-and-fixtures.json`。

## 2026-10-24 上传恢复（不改变数值定义）

大文件对象上传未完成；原2,421,126字节theta1.bin按131,072字节无损分成19个part入库，按000至018拼接恢复。`theta1-storage.json`记录每片和完整流哈希，与原oracle完全相同。Rust读取函数仅拼接这些片；生成器复现时同时生成原文件与分片。原断言和失败结论保留，原始evidence-manifest中的theta1.bin为重构逻辑文件。未重新运行失败验收；仅核验分片字节和编译。
