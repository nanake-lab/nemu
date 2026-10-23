# P5-03 续作：第二账户证据与 SYS01 结构集成

## 结论与范围

第二账户验收 B1 按用户提供的实际 Windows 10 结果记为 passed，完整原文见
`../second-account/user-message.txt`。用户消息没有附完整窗口逐行输出，未重建窗口日志。
认证、ACL、逐事件权限仍列为 PR 必审内容。既有探针通过发生在此次轮换修复之前；
当前修复保持相同 ACL 构造，自动测试逐次检查新句柄，不声称用户已复跑当前版本。

本轮 Windows gateway 16 项测试和工作区 fmt/clippy 通过。只在 D 盘临时目录新建合成库。
使用真实 Windows 命名管道、IDL 生成的 EventEnvelope/Receipt 和 core-store 公开 API。
测试协调器预置 payload/provenance，使用本地 token 能力，没有自造认证/上传线协议。
固定测试 D 位置不能替代 runtime 的权威分配；不打开 SQL，不修改 core-store/IDL。

## SYS01 子集断言

- 同一连接收到 e1/e2 的 RECEIVED_DURABLE，两者实际 store.commit 后故意不发送提交回执并断开。
- 新连接重投 e2；延迟的 CORE_COMMITTED 按 e2、e1 逆序传送，逐一核对完整回执的 event ID/key，
  不能把上一次输入当作当前回执的所属事件。
- D=(e1,e2,e2,conflict)，H=(e1,e2)；重复返回 FILTERED，冲突返回 REJECTED/STABLE_KEY_CONFLICT。
- 重复和冲突都不改变快照或锚点；最终 processed=4、anchor=2、ledger=2，无待处理输入，store.verify 通过。
- RECEIVED_DURABLE 后未提交就关闭/重开临时库，事件仍待处理，提交后才发 CORE_COMMITTED。
  这是有序关闭重开，不是进程强杀或真实断电。

测试夹具仅产生合成 NO_GROWTH/GROWTH_OFF_REHEARSAL 事务，用于验证实际持久提交的回执边界。
不证明真实 runtime、成长、行为、终端认证或上传分派有效。完整 SYS01 仍 not_run / B3 阻塞。

## 发现并修复的问题

初始测试夹具使用了 main 的 RSnapshotV2 不存在的字段（sys01-initial.log），已按生成类型修正。
之后在同句柄 DisconnectNamedPipe/ConnectNamedPipe 重连后的首次读取出现 Win32 233
ERROR_PIPE_NOT_CONNECTED（sys01-after-fixture-fix.log、sys01-reconnect-diagnostic.log）。
原先仅连接成功的测试漏检了这一问题。

修复为：初次 first-instance；断开时先创建相同显式 SID DACL、拒绝远程的新实例，
然后 disconnect/关闭旧实例。最多两个内核实例重叠，连续占有名称；Listener 的借用仍只允许逐会话处理。
新实例创建或旧实例断开失败时保留旧实例并返回错误，不降级到默认 ACL。
参考锁定 Tokio 1.54.1 本地 named_pipe.rs 的实例轮换模式，不复用旧会话 overlapped I/O。

回归测试执行 20 次双向读写重连，每次遗留未读帧，再断开；每个新句柄的 DACL 精确等于当前 SID，
每次都验证其他 first-instance 绑定失败。未删断言或放宽期限来隐藏故障。

## 证据与待办

- test-reconnect-fix.log：修复重连后的 16 项通过；test-final.log 是补强提交回执逆序断言后的最终 16 项。
- fmt-final.log、clippy-final.log：最终工作区检查通过（先前 fmt.log/clippy.log 也保留）。
  fmt 无输出即通过，退出码已检查。
- source-manifest.json：本轮测试代码与依赖/CI 的 LF 归一 SHA256；上级同名文件保留此前版本证据。
- ci-9367149.json：此前提交的 Ubuntu CI 成功，不代表当前提交。
- 本轮重新 fetch；origin/main 仍 876cff7211389f7058668dfb2b9592634d517abc，分支已包含。
- #57 仍 OPEN，最新查询 head a57c8b13e774a4470f0e1529c200d9fce6567323，mergedAt=null。
- 等 #57 合并后合并 main 并接入后继消息；接口请求依用户安排，此前不修改契约。
- 保持 draft / needs-user；完整验收前不申请最终审查、不合并。
