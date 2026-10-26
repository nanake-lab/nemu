# P5-04 两项 P2 修复与验证

针对 [fbcb7cc 审查评论](https://github.com/nanake-lab/nemu/pull/65#issuecomment-6419219224)。修复始于 2026-10-26，验证于 2026-10-27；基于 092cb2fe，第一轮本机检查全部通过后，再 merge main@65052c0（合并 c4162dec）并复验。不修改原始失败或旧核验证据。

1. `paging.rs`：先编码最小 QueryResultV2，完整计入 query_id；最小信封超预算则在检索前返回既有 Error::Invalid，绝不截断 ID。实际返回前重新核验总编码大小；裁剪到零条也不能绕过预算，投影/记忆/审计返回项/续页同步。
2. `.github/workflows/ci.yml`：给本 PR 新增的 W19 CPU clippy 步骤追加与相邻 Rust 检查相同的 `if: steps.scope.outputs.rust == 'true'`；没有改 scope 判定本身。同步 main 时原样接收 #106 已在 main 批准的 Windows job 时限 20→60 分钟，工作流 blob 为 a686643b9c6f7508e949157c36ddaf97e946e8fc；未自行调整超时或恢复 TEMP/TMP 覆盖。

`src/tests/panel_budget.rs` 新增三项回归：空 NO_MATCH / DENIED / INDEX_LAGGING 的 45 个预算/ID 组合（含 70000 ASCII、UTF-8、零、低一字节、精确边界），正向结果全部裁空，以及大 ID 导致逐页裁剪后的完整原文/顺序/续页。

`check-ci-scope.ps1` 提取并执行工作流的实际 scope 脚本，只模拟 git diff 输出；11 个场景检查 stage/desktop 单独变更不启动 Rust，fact-view、共享 Cargo/工具链、工作流、根构建文件、手动全量及 diff 失败仍覆盖。它另核验 CPU 步骤使用同一 Rust 条件；不是一次真实 CI 运行。

运行命令、起止时间、退出码和原始输出按同名 JSON/日志保存。测试/编译启动前检查本机没有 Runner.Worker、cargo 或测试重负载；CI 后续自行启动不作为性能验收窗口。本轮没有做性能测量。缓存/构建在既有 D 盘位置，临时库均为新的 TemporaryRoot EXPERIMENT。没有改变权限、系统/用户/全局设置、runner、网络、依赖版本或验收阈值；job 时限只接收上述 main 已合并的变更。

fbcb7cc 的 Cargo registry 超时另见 `../windows-failure-fbcb7cc-20261026/`；新修复与旧网络失败分别记账，原失败不改成通过。当前验收结果以本目录完成后的日志和上级 REPORT/BLOCKED 为准。
