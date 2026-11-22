# P6-02 1/2 尚未执行的桌面与核心联测

中档交付。命名管道、发件箱、桌面接线、文字面板和合成用例已提交草稿 PR #259。Windows 本机 lint、类型检查、构建和 625 项 Vitest 全部通过。命名管道用脚本核心走了真实 Windows I/O，React 渲染用例逐次核对了已收前缀，断线至重连重获持久回执实测 269.2 ms。

## Chromium 渲染器和 Electron

- `pnpm exec playwright test --config packages/terminal/test/playwright.config.ts` 为 `not_run`。每次启动前检查均遇到活跃的 `Runner.Worker`，按 AGENTS.md 第 9 节未启动浏览器。该用例从面板发送文字、观察逐块显示并点击 Stop。
- 草稿 Windows workflow 的自托管作业跳过。汇总门因为草稿状态显示失败，未执行桌面测试。见 [Windows 作业](https://github.com/nanake-lab/nemu/actions/runs/45035906731/job/137249495813)。
- runner 空闲时可单独验证 Chromium 用例。草稿转为可审查后，仓库的 Windows 工作流可提供 Electron 与隔离检查。两项各覆盖对应运行环境。

## 真实核心

- [HOST-01 #258](https://github.com/nanake-lab/nemu/pull/258) 仍开放。真实核心的 20 条回执与回执前后重启检查留给后续联测，本卡结果为 `not_run`。
- P5-53 仍需提供经认证的主动下行及可区分的回合失败信息。当前以生成类型注入脚本事件。真实回合是否开始、失败类型及实际结束由后继联测确认。接口需求见 [P6-02-core-push.md](../../tasks/requests/P6-02-core-push.md)。
