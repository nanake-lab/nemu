# P0-04 阻塞项

## 1. 当前环境未满足 P0-05 第二部分要求

全量 doctor 输出保存在 `doctor.txt`，退出码为 1。已观测到：

- Blender 为 `4.5.14 LTS`，低于项目要求的 `>= 5.1`；`tools/asset-pipeline/blender-version.txt` 尚不存在。
- 当前 Blender 查询未发现启用的 `io_scene_vrm`。
- `nvcc` 未检测到，`workers/model-adapter` 的 uv 环境没有 `torch`。
- Microsoft Office 桌面版注册信息未检测到；开始菜单中发现的是 WPS Office。
- 命名管道 ACL 探针通过；Low IL 探针在 `CreateProcessWithTokenW` 处失败，未能证明或否定写入隔离。
- VRoid Studio 2.14.0 实际存在；doctor 原输出因开始菜单名称 `VRoidStudio 2.14.0` 与旧匹配式不一致而误报。脚本已修复，单测通过；完整诊断未重跑，详见 `doctor-corrections.md`。
- Node、pnpm、Python、uv、Rust/MSVC target、VS C++ workload、.NET 10、Git LFS、Codex 版本及登录、NTFS 本机 `%LOCALAPPDATA%` 和 119.3 GB 可用空间均通过。

按 P0-05 第二部分执行软件安装与内容仓库准备后，需重新运行 doctor 以验证更新状态。

## 2. 全量诊断后的异常重启

用户报告运行诊断后电脑崩溃并重启。重启后的 Windows 日志显示一次非正常关闭；应用程序日志也记录了 `git.exe` 与 `conhost.exe` 的栈溢出异常码 `0xc00000fd`。最近的 Kernel-Power 41 事件中 `BugcheckCode=0`。事件日志没有记录具体触发命令，也不足以确定是否由本次诊断引起；此前还存在更早的异常关闭记录。

为避免再次影响机器，已停止重复全量诊断。`doctor.txt` 是当时完整输出；11 个 Pester 单测在修复 VRoid 名称匹配后单独通过。Low IL 与 Blender/CUDA/PyTorch 探针尚未复验。

## 3. 任务规则冲突与依赖循环

### `docs/SETUP.md` 所有权

P0-04 卡把既有 `docs/SETUP.md` 列入目录，但根 `AGENTS.md` 规定 `docs/**` 只能追加报告、请求和 ADR，不能改已有内容。本卡没有修改该文件。请求见 `tasks/requests/P0-04-scope-and-version-bootstrap.md`。

### Blender 精确版本锁

P0-04 要求与 `tools/asset-pipeline/blender-version.txt` 完全一致；该文件由 P2-03 交付。P2-03 又依赖 P0-05 第二部分，而 P0-05 第二部分要求 P0-04 doctor 全部通过。当前没有得到批准的初始精确版本，也不能把 4.5.14 降格成合格版本或自行放宽验收。

## 4. 可选处理方式

1. 用户安装项目要求的 Blender 5.1 或更高版本，并指定首个精确版本锁的建立者；之后在稳定状态下重跑 doctor。
2. 用户授予 P0-04 对 `docs/SETUP.md` 的任务级修改权限，并指定是否由 P0-04/P0-05 建立初始 Blender 版本锁；或者把这两项转入有明确目录权限的后续卡。
3. 若系统仍发生栈溢出或意外重启，先处理 Windows/Git 主机问题，再运行 Low IL 和全量 doctor 探针；不能用未验证状态签收 P0-05。
