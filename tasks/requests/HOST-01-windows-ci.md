# HOST-01 请求将宿主测试接入 Windows 检查

请在 Windows 工作流的变更路径和分组判断中登记 `core/crates/nemu-core/`，将 `cargo clippy -p nemu-core --all-targets --locked -- -D warnings`、`cargo test -p nemu-core --locked` 和桌面宿主心跳测试接入相关组。测试时使用独占的当前 SID 管道和临时 LOCALAPPDATA；源码里的终端会比对管道服务端 PID。

当前工作流的 W18 步骤显式选择 `-p gateway`，Ubuntu 的 workspace 测试只能执行宿主的非 Windows 分支。工作流在 HOST-01 核准目录外。

已决定：本 PR 将宿主的 Clippy 和测试接入自托管 W18 组，桌面心跳接入自托管 desktop 组。见 [Claude 交叉审查](https://github.com/nanake-lab/nemu/pull/258#issuecomment-7082856942)。
