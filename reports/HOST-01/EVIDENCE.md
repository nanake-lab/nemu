# HOST-01 1/2 证据索引

基线 `d3470919`。草稿 [PR #258](https://github.com/nanake-lab/nemu/pull/258) 的 Checks 记录远端结果。

| 检查 | 本次结果 | 入口 |
| --- | --- | --- |
| Rust 全工作区格式 | 通过 | 在 `core/` 执行 `cargo fmt --check` |
| 新二进制进入正常工作区 | 通过 | `cargo metadata --offline --locked --format-version 1` 列出 workspace member 及 `nemu-core`、`host` 两个 target；现有 `crates/*` 已覆盖新包 |
| 测试终端源码与输入 | Python 语法检查通过，TerminalEvent、SourceMetadata、GrowthControl 三个合成夹具通过现有 Canonical 编解码往返 | [测试终端](../../core/crates/nemu-core/tests/terminal.py) |
| Windows 编译、Clippy、进程与真实终端 | not_run | `cargo clippy -p nemu-core --all-targets -- -D warnings`、`cargo test -p nemu-core -- --nocapture`。脚本覆盖临时库、私有令牌、认证、关闭的成长能力、文字拒绝、超大帧的持久审计、控制台停止和重连 |
| 桌面启动与心跳 | not_run | [桌面守护测试](../../core/crates/nemu-core/tests/desktop-smoke.mts)，使用仓库 tsx 运行脚本，参数为构建出的 nemu-core.exe。它在临时目录复现发布布局，并检查真实守护进程的 PID、心跳和重启计数 |
| 20 条文字的持久回执、回执前后强杀和重发去重 | 阻塞，未取得运行证据 | [阻塞记录](BLOCKED.md) |
| Ubuntu CI、Windows CI | 见 PR 当前提交的 Checks | Windows 工作流目前没有运行宿主测试的步骤，见 [工作流请求](../../tasks/requests/HOST-01-windows-ci.md) |

运行依赖来自现有锁文件及本地工作区：Tokio 1.54.1（MIT，网关事件循环），tracing 0.1.44（MIT，日志），windows-sys 0.61.2（MIT 或 Apache-2.0，Windows 控制台停止信号），以及工作区内的 core-store、core-types、gateway。本 PR 没有引入新的第三方包。
