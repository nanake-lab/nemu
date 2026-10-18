# P2-02 至 P2-06 与 P0-05 依赖状态

检查日期：2026-10-18（用户客户端日期）

| 卡 | 状态 | 证据与条件 |
| --- | --- | --- |
| P2-02 VRoid 塑形 | 部分交付，仍由用户负责 | 已有 `content/nemu/source/nemu-vroid-export.vrm`，VRM 1.0、VRoid Studio 2.14.0 导出。目录中没有 `.vroid` 工程或 `progress/` 截图，不能确认四次塑形记录或中期导出。当前文件只读，SHA-256 见 `REPORT.md`。 |
| P2-03 Blender 无头资产管线 | 阻塞 | 依赖 P2-04 和 P0-05 第二部分。P2-04 正在交 PR；P0-05 第二部分仍缺 P0-04 doctor 工具/输出、内容仓库子模块证据。项目要求 Blender LTS ≥5.1；当前任务提供的 4.5.14 不符合要求，且本机没有发现 `blender` 命令或 `NEMU_BLENDER` 路径。 |
| P2-04 资产契约校验器 | 已实现，等待本 PR CI | P0-02 已在 `origin/main`；测试、静态检查和原始模型 JSON 诊断见 `REPORT.md`。 |
| P2-05 尾巴、猫耳与弹簧骨调校 | 阻塞 | 等 P2-02 中期 VRM、P2-03 管线。P1-03 已合并。尾巴外形、四类弹簧手感和候选选择属于用户评审。 |
| P2-06 自定义表情与材质精修 | 阻塞 | 等 P2-02 中期 VRM、P2-03 管线。P1-03 已合并。表情组合和 MToon 候选需要用户逐项评审。 |

## P0-05 状态

- 第一部分：仓库和本机基础工具链大部分可见。已核对 Node 24.21.0、pnpm 12.11.2、Python 3.12.10、uv 0.12.15、Rust stable/MSVC target、.NET SDK 10.0.401、Git LFS 3.7.1、Codex CLI 0.162.1，且 `codex login status` 显示已登录。内容仓库子模块在当前检出中没有登记；Codex/Antigravity 三线程确认未验证。
- 第二部分：尚不能签收。P0-04 的 `tools/doctor.ps1` 与 `reports/P0-04/doctor.txt` 不存在；`git submodule status` 为空。Blender 安装位置与 VRM Add-on 状态未能从本机独立确认。
- 版本差异：用户给出的 Blender 4.5.14 LTS 与 `docs/ARCHITECTURE.md` 和 P0-04 的 ≥5.1 要求不一致。此项目规格没有授权代理降级版本门槛。
