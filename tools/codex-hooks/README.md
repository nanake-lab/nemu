# Codex 会话感知（P6-11）

可选，默认关闭。钩子安装到**用户自己运行 Codex 的配置目录**，与 Nemu 助手的隔离 `CODEX_HOME` 分开。脚本要求显式给出两个绝对路径，不读取 `HOME` 或 `CODEX_HOME` 来选择目标。

本卡仅交付脚本与说明。开发和验收只使用临时假配置；没有代用户安装、修改真实配置、改变用户级或系统级设置。

## 安装与关闭

由用户检查路径并执行。保留运行此命令的代码目录及其 `node_modules`；钩子引用这里的发送脚本和已锁定的 Node/tsx。移动代码目录后，在新目录重新执行安装以更新路径。需要 Node 24 和项目锁定依赖，先按仓库说明运行 `pnpm install --frozen-lockfile`。

下面是**占位路径示例**，必须换成自己的 Codex 配置目录。常规用户配置目录为 `~/.codex`；不要把 Nemu 助手隔离账号的目录填在这里。Nemu 状态目录为当前桌面实例的 `userData/codex-session`（常规安装为 `%APPDATA%/Nemu/codex-session`；开发实例应使用其临时 userData）。

```powershell
$ownCodexConfig = 'D:\REPLACE-WITH-YOUR-OWN-CODEX-CONFIG'
$nemuSessionState = 'D:\REPLACE-WITH-NEMU-USERDATA\codex-session'
pnpm codex-hooks:install --config-dir $ownCodexConfig --state-dir $nemuSessionState
```

执行安装表示开启这一项 T1 会话元数据授权。安装只在指定 `hooks.json` 中追加三类钩子，并在指定 Nemu 状态目录保存开关和本机端点；原有钩子与其他键保留，`config.toml` 不改。重复安装不增加重复条目。Nemu 主进程检查这个开关，默认没有开关文件时不监听端点；修改开关后无需重启 Nemu。

按 [OpenAI 官方 Hooks 文档](https://learn.chatgpt.com/docs/hooks)，用户还需在 Codex 的 `/hooks` 中检查并信任新的钩子定义。脚本不替用户信任钩子、不改功能开关、不绕过管理员策略。若实际 Codex 版本或托管策略不支持这些本地钩子，保持关闭；本卡未操作真实 Codex 做验收。

关闭使用**同样两个路径**：

```powershell
pnpm codex-hooks:uninstall --config-dir $ownCodexConfig --state-dir $nemuSessionState
```

卸载先撤销 Nemu 开关，再删除与本次安装命令完全相同的三类条目。其他钩子保留；重复卸载无额外效果。状态文件保留为 `enabled: false`，使已载入旧配置的 Codex 钩子也立即停止发送。如果 `hooks.json` 后来损坏，卸载会报错并保留原文件，但授权仍已撤销；修复 JSON 后可再次卸载。安装器拒绝符号链接形式的 `hooks.json`，不自动覆盖损坏配置或猜测其他目标目录。操作锁残留时先确认没有安装/卸载正在执行，再由用户检查处理指定状态目录中的 `install.lock`。

## 传输与隐私

只发送一条 LF 结尾的 JSON，每次连接一条，示例：

```json
{"event":"PermissionRequest","sessionId":"synthetic-recording-001"}
```

事件名只取现有 `@nemu/contracts` 中的 `SessionStart`、`PermissionRequest`、`Stop`。会话标识限 1–128 个 ASCII 字母、数字、下划线或连字符；不发送任何可选上下文字段。发送端不会打开 transcript、工作目录或用户文件。输入中的提示词、助手答复、工具参数、命令、代码、文件内容、路径、权限模式和模型名都不进入发送对象或日志。

Windows 使用 `\\.\pipe\nemu-codex-session-<随机UUID>` 本机命名管道，不使用 TCP/HTTP。接收端不启用 `readableAll`/`writableAll`，使用操作系统默认管道权限，不修改系统 ACL 或账号。未对真实账号进行跨账号权限验收。Linux CI 使用临时 Unix socket，只验证相同字节与生命周期。

接收端使用契约中两个字段的严格子集重新校验，拒绝额外字段、未知事件、无效 UTF-8、超限和多条帧；单帧最多 1024 字节，最多 16 个并发连接，连接空闲 500 ms 即断开。关闭时拒绝新投递并销毁连接；每次发布前重读授权开关，阻止关闭后的在途数据进入总线。总线创建 `source: main` 的 `codex.session` 事件，由 P6-02 负责变成核心观察。本卡不打开权威存储，不控制审批，不决定她说什么或做什么。

发送端最多读取 1 MiB 钩子输入，管道连接/发送最多等待 300 ms；注册钩子的总超时为 3 秒。无接收端、输入无效或传输失败时静默退出，stdout 不产生批准、拒绝或模型上下文。Windows 命令通过带 `-NoProfile -NonInteractive -WindowStyle Hidden` 的 PowerShell 调用，参数按字面值引用，编码命令只包含本地执行路径，不包含事件内容。

官方 `Stop` 表示当前轮次结束，并不等同于关闭整个会话。这里原样报告该钩子名，不推断用户已离开。

## 测试与后续接入

```powershell
$env:NEMU_TEST_TEMP = 'D:\Dev\cache\P6-11\temporary-tests'
pnpm test:codex-hooks
```

测试创建随机临时假 Codex/Nemu 配置与本机管道，验证安装/卸载幂等、原配置保留、CLI 不回退到假 `CODEX_HOME`、真实生成命令的引用、隐私投影、非法帧及关闭时零事件。不会启动 Codex、Electron、浏览器或其他应用。

`fixtures/session-flow.jsonl` 是纯合成录制流，包含一次权限请求。P5-20 模拟器交付后，用它验证一次立即主动时机及权限、勿扰、预算、模式与最小间隔门控。当前 `origin/main` 没有该模拟器，这一验收项为 **not_run**；传输测试不能替代它。面板设置控件的接线需要其所属目录的授权，当前提供 CLI 开关与主进程服务。
