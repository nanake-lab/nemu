# P6-11：设置开关接线范围请求

P6-11 限定 `tools/codex-hooks/` 与 `apps/desktop/src/codex-session/`。目前已提供显式 `--config-dir` / `--state-dir` 安装与卸载，以及主进程服务；入口只追加两行模块登记。

要完成任务卡中「设置里开启时安装、关闭时卸载」的界面行为，需要设置界面与主进程设置适配器的所属任务接线，或用户授权扩大本卡范围。涉及现有 `apps/panel/src/` 设置界面、`apps/desktop/src/settings/`，以及必要的受验证进程边界。预期接口：用户明确选择自己的 Codex 配置目录并确认 T1 授权后执行 install；关闭先撤销授权再执行 uninstall；默认关闭，安装错误必须显示，不能自动选中隔离账号 CODEX_HOME。

当前未修改上述目录，也未改现有契约来添加设置 IPC。用户确认范围和接口前，不代装真实配置，不自行添加跨进程消息。
