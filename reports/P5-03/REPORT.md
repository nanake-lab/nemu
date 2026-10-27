# P5-03 入口与网关（W18）—报告

## 当前状态

接线实现已完成，正在复验，PR #64 继续保持草稿。原分支已 merge
`origin/main=f24f1256259c998bc213a9dc93103fcfb9cfe89a`（#110 / P5-35），
合并提交 `62b544d6`；Windows 工作流冲突保留最新 main 的门禁与证据上传，追加 W18 范围，
Cargo.lock 通过离线重新生成解决。未修改契约、core-types、core-store 或 θ 消费者。
B3 的四项接口已可用且已接入，不再等待契约卡。最终检查/CI 尚在进行，独立审查尚未请求。

## 交付物

- `Gateway::serve`：实际当前 SID 管道服务；一个 Store actor，多个有界连接；只有生成的认证/上传/请求/响应类型。
- `authentication.rs`：安装令牌认证、版本协商、连接专属随机 session、服务端能力/预算绑定、过期与重连。
- `uploads.rs` / `build.rs`：完整对象配额预留、连续分片、声明绑定、CanonicalV1/域摘要校验、官方生成类型分派到 Store.prepare/prepare_r；完成前不确认，断线/过期释放缓冲与 pins，Store GC 回收孤儿。
- `dispatch.rs` / `hooks.rs`：事件、上传、查询、审批、取消、成长六路分派；T1 与来源/行动权威逐事件从可信处理器获取；控制默认拒绝，处理器成功后才 ACK。
- `progress.rs`：每次核心处理之前执行 Γ；接收位置/时间由 Store.ingest_terminal 分配；实际提交后观察公开 head，仅在有界待回执关联中保存发布摘要，核验事务与原请求后转发 CORE_COMMITTED，可逆序/延迟；无第二套持久回执或去重库。
- 独立持久拒绝审计复用 Store.audit_ingress_rejection；记录/字节/上传/连接/请求元数据/输出队列都有界。只有类型化容量错误能给安全拒绝；I/O/SQL/见证结果不确定时断开，不假装已拒绝或成功。
- `tests/service/`：真实认证、上传、分派、Store 提交、并发连接、撤权、审计/背压和子进程骤停恢复。
- `post-P5-35/check.ps1`：等待本机重负载空闲，再串行 fmt、工作区 clippy、完整 gateway 测试；D 盘缓存/临时目录，逐轮保留原始输出、输入 blob 清单与子进程 stdout/stderr。

## 认证、ACL、权限（必审）

- 当前用户 DPAPI 密文、CREATE_NEW、32 字节 CSPRNG、常量时间比较；认证对象与帧字节缓冲零化，无 token Debug/Serialize。持久审计仅有 enum/服务端位置/时间，不接受客户端字符串、原帧或 token。
- protected 当前 SID-only DACL、拒绝远程、first-instance；先创建同 ACL 等待句柄再交出已连接句柄。16 个内核实例上限，最多 14 个客户端任务，保留轮换余量；共享单写者串行访问 Store。
- B1：复用用户 Windows 10 nanake/nemu 两账户实测 `ERROR_ACCESS_DENIED (5)` / exit 0。原文 `second-account/user-message.txt` 不变；未提供完整窗口日志，不补造。当前修订改变并发/轮换数量而未改变 ACL 构造；新增测试检查四个并发句柄及每个等待句柄的实际 DACL/名称抢占拒绝，不声称重新代跑第二账户。
- session ID 绑定具体连接，不能在另一连接当令牌重放；失效后重新认证，新 session 不恢复未完成上传。身份、能力与预算来自可信安装配置，客户端不能指定 D 位置、接收时间、device/session。
- 标题/Codex 权限不在 session 缓存；同连接撤权立即拒绝下一事件。`CoreHandler` 必须解析历史引用、来源、行动结果和当下授权，不能信任上传声明。
- 审批处理器必须解析真实待审批请求/执行字节并调用 `terminal::validate_approval` 和一次性快照消费；它不是网关认证授予的执行凭证。P5-40/P5-41、core-runtime 的真实业务管理/编排仍归对应卡；本卡实现分派与拒绝边界，不伪造执行或成长权限。

## 验收

| 检查 | 当前结果 | 证据 |
| --- | --- | --- |
| 初轮 all-targets cargo check | passed | post-P5-35/check-initial.log |
| 初轮原生服务集成 | 11 passed，0 failed；随后加固发布证明/配额/上传重试，最终复验进行中 | post-P5-35/native-initial.log |
| 本轮 fmt / workspace clippy / 全部 gateway | 等待其他 runner 重负载结束后串行执行 | post-P5-35/runs/ |
| 当前 head CI / Windows W18 | 尚未完成；草稿不会将 Windows 跳过算作通过 | 待本轮 CI |
| SYS01 认证上传→D=(e1,e2,e2,conflict)，逆序提交回执、重连与并发 | 初轮 passed：H=2、processed=4、anchor=2、ledger=2；真实 Store 提交两次 | SYS01/README.md、native-initial.log |
| SYS01 未提交骤停恢复 | SQL 前 exit 86、SQL 后未见证 exit 86、持久接收后 TerminateProcess exit 1；全新子进程恢复后原键只提交一次 | post-P5-35/native-initial-*/、SYS01/README.md |
| 物理断电 | not_run；已向用户确认骤停注入是否满足本卡此项，未自行放宽 | BLOCKED.md |
| 第二账户拒绝 | passed，用户提供；当前句柄 ACL 自动复验另列 | second-account/ |
| P5-22 后继黄金/live 拒绝/查询审批上下文 | 纳入完整 gateway 重跑；原始冻结字节测试保留 | tests/terminal.rs、post-P5-22/ |

所有库均为 TemporaryRoot 或父进程新建的 tempfile 合成库。骤停子进程只在一次性许可验证后
使用其父进程临时根内的 LOCALAPPDATA；从未打开用户真正的排练/正式库，未改用户/系统环境变量。
不跑 GPU 或性能测量。历史失败日志保持原样，包括 Win32 233、黄金集合次序断言及 pnpm 参数错误；
本轮原始结果不覆盖旧证据。进程骤停不等于实机电源中断，不声称物理存储栈掉电测量。

## 规格与基线

详见 `docs/traceability/P5-03.md` 与 `SYS01/README.md`。沿用 ADR 0024、CONTRACTS 4.2–4.4、
I01/I04、I02/I03/I18/I21、I26/I29；无新协议、规律、阈值或差异登记。
本轮必要验证基线 B=M=f24f1256；2026-10-28 再 fetch 后 main 未前进。P5-22 的旧全矩阵不是
本轮新网关的通过证据；本轮重新执行 gateway 与相关静态检查，由当前 CI 验证组合。
B1 按 ACL 构造未变和新增实际句柄检查复用，来源/限制单列，不拿旧组件测试替代新服务验收。

## 依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| getrandom | 0.4.3（main 已锁定） | Apache-2.0 OR MIT | 新增直接引用，随机连接 session |
| zeroize | 1.9.1（对齐 main 已锁版本） | Apache-2.0 OR MIT | token/帧缓冲零化 |
| serde_json | 1.0.154（既有版本，新增 build 引用） | Apache-2.0 OR MIT | 官方 schema 的类型分派表生成 |
| tokio / subtle / sha2 / windows-sys | 1.54.1 / 2.6.1 / 0.11.1 / 0.61.2 | MIT / BSD-3-Clause / MIT OR Apache-2.0 / MIT OR Apache-2.0 | 既有 I/O、比较、审批摘要、Win32 |

既有传递依赖 bytes 1.12.1、mio 1.2.4、tokio-macros 2.7.2（MIT）、socket2 0.6.5（MIT OR Apache-2.0），
wasi 0.11.1+wasi-snapshot-preview1（Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT）；
测试 tempfile 3.27.0（MIT OR Apache-2.0）。锁文件重新生成，未修改其他 crate 的依赖声明。

## 待确认与后续

- [x] B1 第二账户证据；B2 #57 后继；B3 #110 的四项公开接口。
- [ ] 最终本机检查与当前 head CI。
- [ ] 用户明确 SYS01 骤停注入/物理断电的验收边界；有未完成验收时保持草稿。
- [ ] 独立审查认证、ACL、上传 pin/额度、持久回执与权限拒绝；完成必要检查后在 PR 写「请审查」，不自行合并或取下一卡。
