# Echo Core 规格数学核验脚本

配合 [`docs/traceability/echo-core-math-audit.md`](../../docs/traceability/echo-core-math-audit.md) 使用。只读，不访问网络、数据库或她的任何存储；只用 Python 标准库，随机种子固定。

```powershell
python reports/echo-core-math-audit/run_all.py
```

全部性质成立、全部反例复现时退出码为 0。本次输出见 `results.txt`（Python 3.13.16，Linux x86_64）。

| 文件 | 内容 | 检查编号 |
| --- | --- | --- |
| `check_phi.py` | Φ 的恒等、半群、盒不变、收缩；非对角 Λ 与 λ=0 反例；分段一致；事件驱动下的非收缩反例；q 包络与双重投影 | PHI-0…PHI-9 |
| `check_bounds.py` | 驱动幅度界 η_x；‖W‖∞→2 的列二范数和上界；β 分配链与 ‖U‖op ≤ 1；非零参考点反例 | DRV-1、JW-1…JW-4 |
| `check_stats.py` | α_j、α_jn 的和；Hoeffding 半径；序贯查看的 Monte Carlo；样本量表；ε ≥ 1 的等价判据 | ST-1…ST-5 |
| `check_budget.py` | P/A 前缀索引对暴力区间检查；对齐窗口突发反例；有限根路径；τ_θ 下界 | PA-1、WIN-1、BUD-1、TAU-1 |
| `check_metric.py` | d_Θ 度量公理；w_c=0、哈希比较、秩编码 | DT-1…DT-4 |
| `check_partition.py` | ε 相容不可传递；极小细分不唯一；传递闭包漂移；merge 须整域验证 | G01-1…G01-4 |
| `check_schedule.py` | 非爆炸反例与充分条件；thinning 生存函数；最小间隔的门控与推迟 | NX-1、NX-2、TH-1、TH-2 |
| `check_structure.py` | 单事件顺序 DAG 与 X06；meta 自排除；谱系关系；Γ 与 Π_Γ | ORD-1、META-1、LIN-1、ING-1 |
| `check_ids.py` | 读取转换版规格，核对验收编号与不变量计数 | IDS-1 |

检查类型：`property` 表示脚本没有找到违反该性质的例子，不等于证明，证明在审计报告第 7 节；`counterexample` 表示报告依赖的反例按描述复现。
