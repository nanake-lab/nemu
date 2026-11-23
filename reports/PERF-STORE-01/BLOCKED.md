# PERF-STORE-01 基线测量阻塞

本卡要求在 Windows 和 Ubuntu 各自建立可重复的基线。2026-11-23，`wsl --list --quiet` 成功列出 Ubuntu，但 `wsl -d Ubuntu -- bash -lc …` 返回退出码 1 和 `Wsl/Service/CreateInstance/CreateVm/HCS/0x800705aa`。错误发生在虚拟机创建阶段，Ubuntu 内的检查命令尚未执行，工具链和 perf 均待核实。

基线修订为 `6734adc2bfe8b145e8062a2fc669dd2024dd2ba6`，包含 #238 的合并提交 `d34709197f24814e3a9be04d567897bf53f69224`。Windows 可用 Rust/Cargo 1.99.0，工作卷为 NTFS。现有 Ubuntu core-store 作业执行整包测试及写锁测试，缺少逐项计时、函数剖析及自定义测量入口。见[该修订的 CI 定义](https://github.com/nanake-lab/nemu/blob/6734adc2bfe8b145e8062a2fc669dd2024dd2ba6/.github/workflows/ci.yml#L97-L125)。

两个平台的测试枚举、逐项计时、慢项剖析、整组与 W02 计时、#240 两项复测全部为 `not_run`。产品优化、计时探针及其验证尚未开始。历史 PR 的测试数和计时仅作背景。

**解除方式**

1. 恢复本机现有 Ubuntu WSL 后重新派发，沿用现有环境；续做时先检查 Rust 1.99.0 和 perf 是否可用。
2. 指定并授权另一台具备 Rust 1.99.0 和可用 perf 的 Ubuntu 主机；须在该主机独立完成改前和改后整套测量。
