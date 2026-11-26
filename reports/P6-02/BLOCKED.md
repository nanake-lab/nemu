# P6-02 1/2 尚未执行的桌面与核心联测

中档交付，涉及终端发件箱、桌面接线、聊天面板和合成测试。永久拒绝清出发件箱，对应消息以灰字显示「没送到」和原因。Windows 本机 `pnpm lint`、`pnpm typecheck`、`pnpm test`（627 项）及 `pnpm build` 通过。首次类型检查发现新增测试的事件类型过宽，修正后通过。复跑的脚本核心命名管道用例从断线到重获持久回执实测 272.4 ms。

## Chromium 渲染器和 Electron

- Chromium 阻塞已解除。Windows 本机在 `Runner.Worker` 空闲时执行 `pnpm exec playwright test --config packages/terminal/test/playwright.config.ts`。首次运行因输入框选择器也命中消息列表而失败，限定输入框后复跑通过 1 项，用例耗时 773 ms。用例覆盖拒绝原因、后续正常发送、逐块显示及 Stop 保留已显示文字。
- Electron 与隔离检查仍为 `not_run`，待草稿转为可审查后的 Windows 工作流。上一提交的草稿工作流跳过了自托管作业，汇总门显示失败。[上一轮 Windows 作业](https://github.com/nanake-lab/nemu/actions/runs/45035906731/job/137249495813)

## 真实核心

- 真实核心的 20 条回执与回执前后重启检查为 `not_run`，按审查安排交给 HOST-01 2/2（Q9）。[PR #259 审查](https://github.com/nanake-lab/nemu/pull/259#issuecomment-7082887346)
- P5-53 仍需提供经认证的主动下行及可区分的回合失败信息。当前以生成类型注入脚本事件。真实回合是否开始、失败类型及实际结束由后继联测确认。接口需求见 [P6-02-core-push.md](../../tasks/requests/P6-02-core-push.md)。
