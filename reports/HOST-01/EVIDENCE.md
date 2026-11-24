# HOST-01 1/2 证据索引

[PR #258](https://github.com/nanake-lab/nemu/pull/258) · [授权与验收拆分](https://github.com/nanake-lab/nemu/pull/258#issuecomment-7080901147) · [握手回调限制](https://github.com/nanake-lab/nemu/pull/258#issuecomment-7080949946)

| 检查 | 观察结果 | 可复核证据 |
| --- | --- | --- |
| Windows 宿主端到端 | `cargo test -p nemu-core --locked -- --nocapture` 通过。20 条不同文字全部获 RECEIVED_DURABLE（位置 0–19）。读完第 20 个回执后强杀，重启逐项比对全部事件、位置及回执时间。另一条文字写完请求即强杀，终端未读回执；此次恢复发现原有 20 条，按相同键、ID 和内容重发得到位置 20。最终 21 条 D，processed=0。 | [脚本化终端](../../core/crates/nemu-core/tests/terminal.py)，日志 `host-r2.log` |
| 来源和握手 | 4 次真实握手得到不同会话。临时认证库中 4 条持久记录均指向同一 SID 派生主体和 desktop 安装，包括只认证未发事件的最后一个会话。错误令牌收到 AUTH_FAILED，关闭的成长能力和虚构父事件收到 CAPABILITY_DENIED；超长帧被断开。落盘的 4 个拒绝原因逐一核对。 | [宿主认证方](../../core/crates/nemu-core/src/authority.rs)、[文字谓词](../../core/crates/core-types/src/permission.rs)、[谓词测试](../../core/crates/core-types/tests/permission.rs) |
| 停止与重连 | 控制台停止到进程退出实测 0.375 秒，目标 5 秒。随后重新启动，同一受保护令牌完成新会话握手。测试的每个连接均核对管道服务端 PID。 | `host-r2.log` |
| 代码检查和构建 | `cargo fmt --check`、`cargo clippy -p nemu-core -p gateway -p core-types --all-targets --locked -- -D warnings` 通过，Windows release 构建成功。 | `clippy.log`、`release.log` |
| 桌面守护 | 真实 `SupervisedProcess` 从配置解析到 release 二进制，进程启动且 PID 保持一致。12 秒内收到 6 次排练心跳，重启 0 次。 | [桌面测试](../../core/crates/nemu-core/tests/desktop-smoke.mts)，日志 `desktop.log` |
| Windows 相关组 | `eeb65157` 通过。gateway 49 项含握手回调成功/拒绝/错误、SYS01 回执与崩溃恢复。W02 单元组 69 项通过。该提交至本地实测版本的宿主、gateway 和 core-types 产品源码相同。 | [Windows CI](https://github.com/nanake-lab/nemu/actions/runs/45036173568) |
| Ubuntu | `fc285730` 的静态检查与单测通过。非 Windows 平台启动时，测试确认进程失败退出并明确报“只支持 Windows”。 | [Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/45043322015)、[平台测试](../../core/crates/nemu-core/tests/host.rs) |

本机首轮宿主测试的正向夹具带有父项和推断来源，宿主拒绝了该样本。改为普通原文后通过。桌面首次调用本机已有的 tsx 启动脚本时缺少模块，宿主未启动；改用已核对的仓库锁定版本 tsx 4.21.0 后通过。各次测试均使用临时排练目录。

原始日志保存在[私有证据仓库的 HOST-01 目录](https://github.com/nanake-lab/nemu-evidence/tree/f929f7abe03cf43ac32dafac48200c6abbb18bb0/HOST-01)，提交 `f929f7abe03cf43ac32dafac48200c6abbb18bb0`。

| 文件 | SHA-256 |
| --- | --- |
| `HOST-01/clippy.log` | `09d040070855e289810aec3107eec6dc8d6fccbbcc331f283fb488944c63cf9d` |
| `HOST-01/release.log` | `bbf378ccdac107ffc34c82d9be81c66feebffeda510ff65cfdd95e8c8b691e0e` |
| `HOST-01/host-first.log` | `5a21bf44a7c85ddcfa38cc874f7dbbd9fbaa61f700aae49ce22968e92e1d5d01` |
| `HOST-01/host-r2.log` | `edd2f7cf7d2b2cd418f9047f2f4f6aff1b921104113209272c9be7abf64b2b9c` |
| `HOST-01/desktop-launcher-first.log` | `51a993952401f2273e420fe9c651d4b16585adeba459931a792059cffd33324b` |
| `HOST-01/desktop.log` | `fe965df75524bdc0e45269c38868881010f183f99cc9bd22d57aebf819ebe8bd` |

本阶段把同一 Windows 账户下的打字者视为 u*。本机账户借用者无法区分。绑定先写入排练专用的 `Nemu/rehearsal-auth.sqlite`，SQLite FULL 同步成功后才允许返回认证成功；持久记录保留主体摘要、安装、会话、记录号。1/2 的原键重发已实测，2/2 接入真实处理后核验 Γ 的重复终态。

宿主复用工作区包及锁文件中的依赖。Tokio 1.54.1（MIT）用于事件循环，tracing 0.1.44（MIT）用于日志，windows-sys 0.61.2（MIT 或 Apache-2.0）处理控制台信号，rusqlite 0.40.2（MIT）存储认证记录，sha2 0.11.1（MIT 或 Apache-2.0）派生主体号。
