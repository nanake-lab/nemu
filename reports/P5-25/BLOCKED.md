# P5-25 阻塞解除与验收归属（2026-10-26）

用户本轮已经作出决定，之前要求选择的两项已解除：

1. V2-93旧 **2 对照保留为 #[ignore] 诊断，属性/注释引用理由和证据，测试体、断言、输入及历史失败不变。本卡数值验收使用新x*x参考的严格位比较；ignored不等于passed，显式诊断仍应报告原失败。
2. I13、AC2-09本卡只验纯函数边界：C不接收x/m/R，改变调用方x初值和m不改变θ/完整估计器，C只返回暂存ThetaChange，不写L、不提交。实际事务与整段重放由P5-29、P5-26验收，不再列作本卡未完成项。

正式估计器契约已由main的P5-36承接；H时间认证与长期时间域列在请求文件，属于后继工作。CUSUM/n_tau/报警参数仍为待实验谱系校准的夹具，不宣称已正式定值。

原两项选择及libm依赖恢复的目录范围确认均已解除。本轮检查实测结果见REPORT.md和closeout-20261026；同head CI全绿后才转为可审查，独立交叉审查仍由用户安排。本会话不合并。

原失败不可抹去：旧σ首处相差1 ULP；evidence/、rounding-20261024/、batch4-20261026/及各编号原结果均原样保留。前一提交CI失败（run 40264985040）也另存ci-before。新诊断结果作为新证据追加，绝不覆盖旧失败。

## 已解除：main合并删掉libm声明

CI run 40272465565（head 43da867）在Rust编译阶段报7处E0433，未进入测试。原因已定位到P5-27从core-kernel/Cargo.toml删除原libm依赖，但dynamics仍按METHODS固定使用libm0.2.16。

用户2026-10-26明确批准后，提交0cf1605只恢复 core/crates/core-kernel/Cargo.toml 的 `libm = { version = "=0.2.16", default-features = false }`，由Cargo更新共享锁文件；不改公式或版本。本机fmt、workspace Clippy及kernel常规测试已经通过。PR #94也恢复此行，后合并者只保留一行。

## 当前外部阻塞：GitHub Actions计费与Windows runner

修复后的Ubuntu run [40283963199](https://github.com/nanake-lab/nemu/actions/runs/40283963199)（head 0cf1605）没有启动任何步骤，GitHub失败注释为：

> The job was not started because recent account payments have failed or your spending limit needs to be increased.

原注释与空steps记录见 closeout-20261026/ci-code/run-40283963199.json。它不是Rust编译失败，也不能记成测试通过。2026-10-26的GitHub runners API同时报告DESKTOP-8LBF84O为offline；锁文件改动需要Windows W02检查，草稿的自动Windows作业只显示skipped。

处理路径：用户处理GitHub Billing & plans中的账户状态，并恢复Windows runner后，重跑最终head的CI；在这之前继续保留草稿和needs-user。若暂不恢复这些外部条件，就保留当前代码和本机证据，不能省略CI或提前写「请审查」。不修改账户、用户或系统设置，不调整阈值，不以本机通过冒充CI全绿。
