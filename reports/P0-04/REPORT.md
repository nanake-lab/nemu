# P0-04 环境检查脚本 — 报告

## 结果

部分完成（见 `BLOCKED.md`）。Doctor 脚本和 11 个 Pester 单测已交付，当前环境诊断也已保存；P0-05 第二部分仍有软件、仓库和 Windows 隔离验证阻塞。电脑发生异常重启后，没有重跑全量诊断。

## 交付物

- `tools/doctor.ps1`：Windows 开发环境检查表，包含依赖版本、GPU、VRM、存储位置、命名管道和 Low IL 检查。
- `tools/doctor.Tests.ps1`：版本判断与 VRoid 开始菜单名称识别的 Pester 单测。
- `reports/P0-04/doctor.txt`：全量诊断原始输出。该输出记录在 VRoid 名称匹配修正之前；修正后的单项证据见 `doctor-corrections.md`。
- `reports/P0-04/DEPENDENCY-ASSESSMENT.md`：P2-02 至 P2-06 与 P0-05 当前依赖状态。
- `reports/P0-04/BLOCKED.md`：尚需用户处理或澄清的事项与机器稳定性记录。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `Invoke-Pester -Script tools/doctor.Tests.ps1` | 通过，11/11 | `pester.log`、`pester-result.json` |
| `pwsh -NoLogo -NoProfile -File tools/doctor.ps1` | 脚本完整执行；退出码 1，因环境缺项而非 0，符合缺少必需项时返回非 0 的要求 | `doctor.txt`；VRoid 名称误报更正在 `doctor-corrections.md` |
| VRM 原件 SHA-256 | 未变化：`5484C94F54F6C000767FEBACBC99FD7EA23AC074D9CA8EECEBAF65B967DE39AB` | 当前原件哈希；见依赖评估 |
| Windows 异常重启后的稳定性检查 | 事件日志记录异常重启与 `git.exe`/`conhost.exe` 栈溢出；起因未能确定，因此未重跑全量诊断 | `BLOCKED.md` |

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| 无 | — | — | 使用 Windows PowerShell、Pester 和系统自带检查接口 |

## 待用户确认

- [ ] （阻塞）安装并启用项目要求的 Blender LTS 5.1 或更高版本及兼容 VRM Add-on；明确最初的 Blender 版本锁由哪张卡建立。
- [ ] （阻塞）处理 `docs/SETUP.md` 修改权限与 Blender 版本锁依赖循环，见 `tasks/requests/P0-04-scope-and-version-bootstrap.md`。
- [ ] （阻塞）完成 P0-05 第二部分的内容仓库子模块、缺失环境项和长期 Windows runner。
- [ ] （不阻塞）用户提供 P2-02 中期 VRM 导出后，继续 P2-05/P2-06 的资产工作和审美评审。

## 后续建议（不在本任务范围内）

- 先处理 `BLOCKED.md` 中的环境与所有权事项，再按 `DEPENDENCY-ASSESSMENT.md` 解锁 P2-03。
- Low IL 验证在本机稳定性问题澄清后重试；失败时保留原始 Win32 错误码，不能把“无法启动探针”当成隔离通过。

## 下一步

等待用户处理 P0-05 第二部分的本机软件与内容子模块，并澄清初始 Blender 版本锁和 `docs/SETUP.md` 的任务范围。P2-03 解锁后继续资产管线；P2-05/P2-06 仍等 P2-02 中期导出。
