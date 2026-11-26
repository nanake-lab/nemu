# P5-53 请求 P5-49 修复测试导入歧义

#270 合入后，`semantic-core` 的 [IRL 测试](../../core/crates/semantic-core/src/irl_tests.rs)同时通配导入 `irl::*` 和 `generated::*`。两者都导出 `IrlDispatch`，第 181 行构造本机调度输入时，Rust 报 E0659。main `ecee0b0e` 和 P5-53 实现提交 `53d3d2fb` 的日志均指向这一行。这两个类型和测试文件在两次运行间字节相同。[main 检查](https://github.com/nanake-lab/nemu/actions/runs/45787663834/job/139798696656) · [分支检查](https://github.com/nanake-lab/nemu/actions/runs/45792273151/job/139814397078)

请由接口所有者显式选用该测试原来使用的本机类型，保留已冻结的线类型，并以 `cargo clippy --workspace --all-targets -- -D warnings` 和工作区测试核验。P5-53 的处理器和终端下行不需要变更此接口。
