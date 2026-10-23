# P5-03 入口与网关（W18）—报告

## 结果
部分完成，保持草稿。基线 origin/main 876cff7（P5-02 / #58 已合并）。
**当前交付是经过测试的边界组件，不是可运行的完整认证网关。** 阻塞见 BLOCKED.md。

## 交付物
- `core/crates/gateway/`：限长/限时帧、九类终端事件验证、安装令牌/认证能力、管道与 core-store 接收适配。
- `examples/acl_probe.rs`、SECOND-ACCOUNT.md：用户安排第二账户后的合成拒绝连接检查，无需新建账户或设置更改。
- `tasks/requests/P5-03-gateway-interfaces.md`：公开 D 尾位置、拒绝审计、终端认证/对象上传 IDL、稳定背压状态的最小请求。
- `.github/workflows/windows.yml`：只追加 W18 Windows 组件检查与证据上传；已有 core 路径触发范围覆盖本卡。
- `check.ps1`：本地检查时将 Cargo / UV / npm / pnpm / HF / Torch 缓存与临时目录统一指向 D 盘。

## 认证、ACL 与权限（必审）

- DPAPI **当前用户**范围，未使用 LOCAL_MACHINE；Windows CSPRNG 生成 32 字节安装令牌。
- 仅保存密文；CREATE_NEW 不覆盖现有/损坏令牌；受保护 DACL 只含当前 SID，句柄不继承。
- 令牌与认证能力无 Debug / Serialize；明文字节使用 zeroize；认证内容比较使用 subtle；错误输出脱敏。
- 管道拒绝远程，DACL 只含当前 SID；初次创建使用 first-instance。断开时先创建同 ACL 的后继实例，
  再关闭旧句柄，连续持有名称；最多两个内核实例轮换，借用规则保证逐会话处理。
  本轮结构集成发现复用旧句柄读到 Win32 233，已改为新句柄，20 次双向重连逐次复核实际 DACL/抢占拒绝。
- 实测当前用户连接/重连、名称抢占拒绝、管道与令牌文件实际 DACL 精确匹配。
  **第二账户测试由用户在 Windows 10 执行并报告通过**（nanake 服务端 / nemu 客户端，错误 5，exit 0），
  原样用户消息与证据来源说明见 `second-account/`；消息未附两个窗口的完整逐行输出，不补造日志。
  该用户结果来自此前探针；本轮轮换修复后没有代理代跑跨账户测试，ACL 构造不变，每个新句柄均有本机 DACL 断言。
- title / Codex 会话字段需要可信行动闸门的 T1 授权；每条事件重新传入当前判定，会话不缓存授权。
  同一会话撤权后的拒绝已有正反例。会话身份和授权不能取自未认证的客户端字段。
- 完整认证线协议、实例绑定、来源/父引用/结果权限解析、拒绝审计仍需接入请求中的接口。
  不把局部 token 验证当作完整终端认证验收。没有执行任何外部行动或开放正式服务。

## 验收

| 命令 / 检查 | 当前结果 | 证据 |
| --- | --- | --- |
| `cargo test -p gateway --locked -- --test-threads=1 --nocapture`（本轮源文件） | 16 passed，0 failed | continuation-1/test-final.log |
| `cargo clippy -p gateway --all-targets --locked -- -D warnings`（含手工探针） | 通过 | clippy-components.log |
| `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`（本轮源文件） | 通过 | continuation-1/fmt-final.log、clippy-final.log |
| `cargo test --workspace --locked -- --test-threads=1` | 98 passed，0 failed；core-store 31 项用时 967.91 秒 | checks.log |
| SYS01：端到端重投/冲突/乱序提交回执/未提交断电恢复 | not_run；组件持久接收/重连有证据，不能替代完整服务 | BLOCKED.md B3 |
| SYS01 结构集成子集：真实管道 + core-store | passed；重投/冲突不变状态、逆序延迟回执、未提交关闭重开；非真实断电 | continuation-1/REPORT.md、test-final.log |
| 其他账户拒绝连接 | passed（用户提供的实际跨账户结果，ERROR_ACCESS_DENIED (5)，exit 0） | second-account/user-message.txt、README.md |
| 实验谱系无终端管道/不能接收终端数据 | 组件测试通过；完整 runtime 以后复测 | test-components.log |
| #57 后继消息衔接 | not_run；#57 尚未合并 | BLOCKED.md B2 |

初始失败日志全部保留：九类正例里用了空撤回夹具；重连最初错误地 drop/rebind 后立即开客户端，
随后改为保留同一个 first-instance 句柄并先进入 accept；clippy 发现测试借用对象的无效 drop，已改用
作用域结束借用。没有删断言、跳过测试或调整门槛。
上述同句柄方案是历史中间态；本轮更强的集成测试暴露其重连读取失败，现已替换为连续持名的双实例轮换。
编译夹具失败和重连 Win32 233 原始日志均保存在 continuation-1/，修复过程与验收边界见其 REPORT.md。

全量测试完成后，最后收紧了每事件权限参数、增加撤权用例和 DPAPI 异常输出检查；对改变的 gateway
重跑最终 14 项，并重跑全工作区静态检查，未重复无改动的 16 分钟存储矩阵。手工第二账户探针经过
all-targets 编译检查，未伪造第二账户运行结果。首次全量矩阵期间有一次短暂的 gateway clippy 编译重叠，
因此不声称该次全量运行具有严格独占测量条件；本卡没有做性能/GPU测量，测试无失败。
本轮未改 core-store/契约；新增两项 Windows 结构集成测试并修复管道，重跑 gateway 16 项和全工作区
fmt/clippy；没有重跑无改动的全量存储矩阵。先前提交 9367149 的 Ubuntu CI 已确认通过
（run 39897522947，continuation-1/ci-9367149.json），不将其当作本轮新提交的 CI 结果。

## 规格对照

见 `docs/traceability/P5-03.md`。本卡未修改 IDL、已发布编码、黄金向量、core-store 格式、规律或阈值，
未新建第二套存储/接收计数。卡外接口使用任务请求登记，不代为补写协议。
测试使用 D 盘 TEMP 下的新建 TemporaryRoot / tempfile 和合成字段，不打开任何已有排练库或正式库。
Rust 1.99.0 / Cargo 1.99.0，Windows 本机验证。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| tokio | 1.54.1 | MIT | 管道、异步 IO、时限；架构指定技术 |
| zeroize | 1.8.2 | Apache-2.0 OR MIT | 令牌缓冲擦除 |
| subtle | 2.6.1 | BSD-3-Clause | 认证内容比较 |
| windows-sys（已有锁定版本，新增 gateway 使用） | 0.61.2 | MIT OR Apache-2.0 | SID、DACL、DPAPI、随机源 |
| core-store / core-types / serde_json（已有） | 工作区 / 1.0.154 | 仓库 / MIT OR Apache-2.0 | 唯一存储、生成契约、契约哈希 |

新传递依赖：bytes 1.12.1、mio 1.2.4、tokio-macros 2.7.2（MIT）；socket2 0.6.5
（MIT OR Apache-2.0）；wasi 0.11.1+wasi-snapshot-preview1
（Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT）。许可证从下载的 Cargo.toml 逐项核对。
测试另用已锁定的 tempfile 3.27.0。依赖/编译缓存均在 D 盘。

## 待用户确认 / 安排
- [x] 第二 Windows 账户拒绝连接实测通过（用户提供证据）；完整窗口日志未附，非代理执行记录。
- [ ] （阻塞）#57 合并后合并主线，补后继消息校验，再验收。
- [ ] （阻塞）解决接口请求中的跨卡接口缺口，之后完成服务和端到端验收。
  用户最新指示：等 #57 合并后由用户另行安排；此前不自行修改契约或扩展到其他卡目录。

## 后续建议（不在本次实现范围内）
当前用户 ACL / 安装令牌不是对同账户恶意进程或取得所有权管理员的隔离边界；worker 的低完整性
启动和完整 action-gate 权限接入仍由其拥有卡负责。具体未完成项以接口请求和 BLOCKED 为准。

## 下一步
保持草稿 / needs-user，等待上述阻塞消解；未申请最终审查，不取下一张卡、不合并。
本轮结束前再次 fetch / merge origin/main，仍为 876cff7，分支已包含最新主线；#57 仍为 OPEN。
本轮查询的 #57 head 为 a57c8b13e774a4470f0e1529c200d9fce6567323，未取其未合并契约。
全部验收完成后再次同步并重跑检查，才转为可审查、提交审查请求并停下。
