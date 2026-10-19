# P0-04 后续复核（2026-10-19）

## 本次追加范围

- 依用户授权，仅在 `docs/SETUP.md` 文末追加“首次开发环境搭建”；追加前 SHA-256 为 `F2179DE744C1051150C6A0BC2F0172F1BC7A9DF67CBC88021666467E214655FB`，原有字节前缀复核通过。
- 依用户授权创建 `tools/asset-pipeline/blender-version.txt`，值为 `5.2.2`。无头探针报告 `Blender 5.2.2 LTS (hash d13f752e3b9c built 2026-09-15 01:37:04)`，退出码 0。
- `tools/doctor.ps1` 改为支持 `-Check <单项>`；每项由独立 `pwsh` 子进程运行并受超时约束，超时会终止该进程树。Blender 文件发现不再递归遍历安装目录。

## Git 栈溢出调查

doctor 对 Git 的调用只有 `git --version` 与 `git lfs version`；没有 `git status`、`git submodule`、`git ls-files` 或任何 Git 仓库树遍历。没有证据表明这两个版本探针会递归遍历子模块。发现的非 Git 递归操作是旧版 Blender 查找器对 Blender Foundation 目录执行 `Get-ChildItem -Recurse -Depth 3`；已移除递归，只查登记路径和一层明确的 Blender 安装目录。此次电脑冻结/手动重启与栈溢出的因果关系仍未证实。

## CUDA Toolkit 判定

源代码检索（排除基准 trace 数据）未发现 `torch.utils.cpp_extension`、`CUDAExtension`、`nvcc` 编译调用或其他自定义 CUDA 构建步骤。P0-07 使用 PyTorch 预编译 CUDA 包执行模型探针，不需要本机 Toolkit 编译。doctor 将 Toolkit 改为 `OPTIONAL` 提示项；GPU 驱动与 model-adapter `torch.cuda.is_available()` 仍是必需检查。PyTorch 官方文档说明加载 CUDA 扩展时才需要另装 Toolkit/NVCC；见 https://docs.pytorch.org/docs/stable/cpp_extension.html。

## 当前 Office COM 证据

只读检查 64/32 位 `HKCR`：`Word.Application`、`Excel.Application`、`PowerPoint.Application` 在 64 位视图分别指向 `D:\WPS Office\12.1.0.29158\office6\wps.exe`、`et.exe`、`wpp.exe`；32 位视图指向 `C:\Program Files\Microsoft Office\Root\Office16\WINWORD.EXE`、`EXCEL.EXE`、`POWERPNT.EXE`。Click-to-Run 的 `ProductReleaseIds=O365HomePremRetail`。doctor 将 Office 桌面版保留为必需，并将 64 位 COM 服务器指向非 Microsoft Office 的三项标为 FAIL，给出先 Office 快速修复、再联机修复、仍被覆盖时修复/卸载 WPS 的建议。没有实例化任何 COM 对象。

## P2 状态和 P0-05 剩余依赖

| 卡 | 复核状态 |
| --- | --- |
| P2-01 | 完成；内容设定已提交 |
| P2-02 | 用户卡；目前只发现初始 `nemu-vroid-export.vrm`，没有 `.vroid` 工程、中期 VRM 或进度截图；原件不改 |
| P2-03 | 仍阻塞：P2-04 已合并，Blender 5.2.2 版本锁已由 P0-04 建立；P0-05 第二部分的内容子模块迁移待用户确认，VRM Add-on 和 Torch 还待用户安装/验证 |
| P2-04 | 完成并合并，PR #18 |
| P2-05 | 仍需 P2-02 中期 VRM、P2-03；P1-03 已合并；外形/手感要用户评审 |
| P2-06 | 仍需 P2-02 中期 VRM、P2-03；P1-03 已合并；表情/MToon 候选要用户评审 |

P0-05 第二部分还没有 `nemu-content` 子模块完成证据、完整环境验收或持久 Windows runner 验收。本次按用户要求只写迁移计划，不改 `.gitmodules`、Git 索引、`content/` 或其文件。

## Low IL 探针

旧日志只记录 `CreateProcessWithTokenW` 失败，未保留 `Win32Exception.NativeErrorCode`，因此旧失败的具体错误码不可从旧日志恢复。本次实现会在错误详情中保留 Win32 错误码；在 `ERROR_PRIVILEGE_NOT_HELD (1314)` 时按 Microsoft 文档回退尝试 `CreateProcessAsUserW`。单项重测结果和原始错误将在 `probes/LowIL.txt` 记录；无法启动探针仍为 FAIL，不会视为隔离通过。电脑冻结事件未证明与 Low IL 或 doctor 有因果关系。

## 测试和待执行单项检查

| 检查 | 结果 |
| --- | --- |
| PowerShell AST parse `tools/doctor.ps1` | 通过 |
| Pester `tools/doctor.Tests.ps1` | 17 passed, 0 failed |
| 其余 Windows 单项 doctor 探针 | 逐项执行后追加结果 |

尚未运行全量 doctor、Electron/Playwright、视觉测试或 Windows runner。runner 维持离线状态，等待用户完成 VRM Add-on 和 Torch 安装后再启动。

## 下一步

等待：用户审阅内容子模块迁移方案；用户安装 VRM Add-on 与 Torch；Office COM 冲突按建议修复；Low IL 单项结果确认。之后再通知用户开启 Windows runner，完成 PR #19 的 Windows 检查。
### 2026-10-19 单项复测结果

| 单项 | 结果 | 证据 |
| --- | --- | --- |
| Git | PASS | `probes/Git.txt`，版本 `2.54.0.windows.1` |
| Git LFS | PASS | `probes/GitLfs.txt`，版本 `3.7.1` |
| Blender | PASS | `probes/Blender.txt`，5.2.2 LTS 与精确锁 `5.2.2` 一致 |
| Microsoft Office 桌面版 | PASS | `probes/Office.txt`，Click-to-Run `O365HomePremRetail` |
| Office 64 位 COM | FAIL | `probes/OfficeCOM.txt`，三个 ProgID 的 64 位 LocalServer32 均指向 WPS；32 位视图均指向 Microsoft Office |
| NVIDIA 驱动 | PASS | `probes/NvidiaDriver.txt`，RTX 3080，驱动 610.88 |
| CUDA Toolkit | OPTIONAL | `probes/CudaToolkit.txt`，未安装不阻塞当前 PyTorch 运行路径 |
| Low IL 写入隔离 | PASS | `probes/LowIL.txt`；`CreateProcessWithTokenW` 返回 `ERROR_PRIVILEGE_NOT_HELD (1314)`，`CreateProcessAsUserW` 回退成功，低完整性子进程对 Medium IL 目录的写入被拒绝 |

旧日志没有保留 Win32 native error code，因此不能从旧日志确认当时也是 1314；本次单项复测直接观察到该错误码并验证了回退。Microsoft 文档说明 `CreateProcessWithTokenW` 需要 `SE_IMPERSONATE_NAME`，遇到 1314 时可改用 `CreateProcessAsUser`；本次回退成功，Low IL 的写入拒绝断言通过。

以上项目均是独立 `-Check` 运行；未启动 VRM Add-on、Torch、全量 doctor、Playwright/Electron 或 Windows runner。VRM Add-on 与 Torch 由用户安装后再单独检查；Office COM 注册需修复后复验。
## 最终单项复核（更正前文的执行时间点）

前文“尚未运行 Torch”记录的是 Torch 单项复测前的状态。后续已逐项运行 `-Check Torch`：当前环境缺少 `torch`，结果 FAIL，证据在 `probes/Torch.txt`；用户安装命令见 `docs/SETUP.md`。VRM Add-on 依用户安排留待安装后检查。

| 后续检查 | 结果 | 证据 |
| --- | --- | --- |
| Node、pnpm、Python、uv、Rust/MSVC、VS C++ workload、.NET 10 | PASS | 各自 `probes/<Check>.txt` |
| Codex CLI 版本与登录状态 | PASS | `probes/Codex.txt`、脱敏后的 `probes/CodexLogin.txt` |
| VRoid Studio 2.14.0 | PASS | `probes/VRoid.txt` |
| 本机 NTFS/LocalAppData、命名管道、Windows 版本、磁盘容量 | PASS | 各自 `probes/<Check>.txt` |
| Blender LTS 标记和锁版本 | PASS | `probes/Blender-LTS.txt`，5.2.2 LTS；锁值 5.2.2 |
| PowerShell AST 与 doctor Pester | PASS | AST 无解析错误；`pester-2026-10-19-v2.log`：18 passed, 0 failed |

`NEMU_BLENDER` 当前未设置。doctor 通过安装登记找到 Blender 并通过版本检查；P2-03 的管线直接读取 `NEMU_BLENDER`，因此 SETUP 已追加用户设置该变量的命令。原始 VRM 哈希仍为 `5484C94F54F6C000767FEBACBC99FD7EA23AC074D9CA8EECEBAF65B967DE39AB`，本轮未改动源文件。

当前 P0-05 第二部分仍等待：用户确认内容子模块迁移方案、安装 VRM Add-on 和 Torch、修正 64 位 Office COM 指向 WPS 的问题，以及之后的 Windows runner 验收。全量 doctor 与 Windows runner 均未运行。
## 2026-10-19 用户澄清后的单项复核和修正

用户澄清 VRM Add-on 已安装。单独检查 Blender Preferences 后确认模块 `bl_ext.blender_org.vrm` 已启用。原 doctor 只检查旧模块名 `io_scene_vrm`，因此产生了假 FAIL；现已修正识别逻辑，兼容旧模块和 Blender 扩展模块，并将模块名写入检查输出。

| 单项 | 结果 | 证据 |
| --- | --- | --- |
| Blender 5.2.2 LTS 与用户级 `NEMU_BLENDER` 路径 | PASS | `probes/Blender-after-user-path.txt`；用户环境值为 `D:\blender\blender.exe` |
| VRM Add-on | PASS | `probes/VRMAddon-after-checker-fix.txt`；启用模块 `bl_ext.blender_org.vrm` |
| Word、Excel、PowerPoint 64/32 位 COM | PASS | `probes/OfficeCOM-after-repair.txt` |
| PyTorch CUDA | PASS | `probes/Torch-after-user-update.txt`；`torch=2.11.0+cu128; cuda=True` |

Office 检查确认机器级 64 位 COM 注册原本已指向 Microsoft Office；当前用户 HKCU 中三个 CLSID 的 `LocalServer32` 默认值被 WPS 覆盖。已只将这三个用户级默认值改为各自机器级 Office 服务器字符串，保留 WPS 安装和其他注册值。原值、新值与键路径记录在 `probes/OfficeCOM-repair-2026-10-19.txt`。修复后 doctor 对两个注册表视图均判为 PASS。

Torch 探针输出 `cuda=True`，同时提示环境缺少 NumPy、Torch 的 NumPy 互操作初始化不可用。该探针只判定 CUDA 可用性；没有安装或改动其他包。P0-04 未据此扩大 model-adapter 依赖范围。

本次 doctor 变更的 Pester 3.4.0 单元测试为 21 passed、0 failed，完整输出见 `pester-2026-10-19-v5.log`。`v3`、`v4` 保留了两次使用 Pester 5 参数调用旧版 Pester 3 的命令错误记录；这些调用未执行测试，随后按本机 Pester 3.4.0 语法运行成功。

用户已明确批准执行内容子模块迁移方案。只读查询确认目标私有仓库 `https://github.com/nanake-lab/nemu-content` 存在且当前凭据可查看。仍按任务依赖与一张卡一条分支规则，待 P0-04 Windows 检查完成后，在 P0-05 第二部分分支实施；当前没有修改 `.gitmodules`、`content/`、Bible 文件或其历史。

截至本节，Blender、VRM Add-on、Office COM 和 Torch 已各自独立复验通过。全量 doctor 与 Windows runner 均未运行。PR #19 的 Windows desktop/isolation 检查现在需要自托管 runner；运行任务为 PR #19 当前最新提交的该 Windows job。等待主代理更新 PR 后，先通知用户启动 runner。
