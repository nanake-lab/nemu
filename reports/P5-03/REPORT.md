# P5-03 入口与网关（W18）—报告

## 当前状态

2026-10-29 审查修复已完成本机验收，交回独立复审，PR #64 保持草稿，不自行合并。
先在原分支 merge `origin/main=787fd2bec82f17b247158a0a985c3ee2c214af03`，
首次合并提交 `3bd67896ad593be1b4d1e1ef11964559434b041b`；末次 fetch 后又 merge P5-23 的 main
`740c2289c2a86f400580b4e6d4aa07654c08ace6`（合并提交 `02b7ca5ee6dc1b1a62ddd583c77384855e8d4e60`）。均无冲突，未 rebase、未强推。

- R1：认证后等待下一请求首字节属于合法空闲，不再触发帧超时或拒绝审计；收到首字节后才启动
  不重置的整帧截止时间。认证握手仍受整体截止时间限制，半帧仍超时/断开/审计；会话寿命到期正常关闭，不写非法帧审计。
- R2：用户明确接受三类真实进程骤停＋新进程恢复作为本卡 SYS01 范围。**SYS01 在该范围内 passed，
  physical_power_loss 仍 not_run，不宣称等同。** 是否物理断电补测留到 E4 存储与备份阶段由用户另定。
  原文见 [用户决定](review-20261029/user-acceptance.txt)；[强制落盘点与缺失项清单](review-20261029/DURABILITY.md) 已逐项列出。

[审查原文](https://github.com/nanake-lab/nemu/pull/64#issuecomment-6486139131) 对应旧 head `80c4c23b`。
本轮仅改 gateway、本卡报告和规格对照；未修改契约、core-types、core-store、θ 或其他消费者。
原始失败与通过证据保留，修复不改审计额度、协议、阈值或生成类型。

## 修复与验证

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| 原生产实现＋新空闲回归 | 预期失败：零请求空闲 2.2 秒后被断开（帧 deadline=2 秒） | review-20261029/R1-before-fix.log、.json |
| 修复后 5 项原生定向回归 | 5 passed，0 failed；合法空闲/延迟回执/重复重连/正常会话到期不审计，认证及半帧超时保留 | review-20261029/R1-after-fix-focused.log |
| cargo fmt --all -- --check | passed | review-20261029/runs/20261029-143622-83651cd71d7347578b63c9a675d67c63/fmt.log、result.json |
| cargo clippy --workspace --all-targets --locked -- -D warnings | passed | 同轮 clippy.log |
| 末次 main 同步后的 fmt / 工作区 all-targets clippy | 再次 passed；P5-23 新增工作区 Rust 源码已编译检查 | review-20261029/final-main-fmt.log、final-main-clippy.log、final-main-checks.json |
| cargo test -p gateway --locked -- --test-threads=1 --nocapture | **42 passed，0 failed，0 ignored** | 同轮 gateway.log；包括全部既有 37 项与新增 5 项 |
| SYS01 三类真实骤停＋新进程恢复 | 重跑通过：SQL 前退出、SQL 后未见证退出、持久接收后未提交时 TerminateProcess；恢复后同一稳定键仅提交一次 | 同轮 SYS01/ 下 6 个子进程的原始 stdout/stderr 与 gateway.log |
| 物理断电 | **not_run**，用户明确不作为本卡的必需范围，是否补测交 E4 决定 | SYS01/result.json、用户决定原文 |

新正例：审计 max_records=2 不变，3 次正确认证后零请求空闲均超过帧 deadline，连接仍在；
第 4 次正确认证/查询成功，持久拒绝数=0、D 为空、H/anchor 不变。另验证等候延迟提交回执、
收到 CORE_COMMITTED 后再次合法空闲再查询仍可用，拒绝审计=0。
新负例：半帧头、半帧体各产生 INVALID_FRAME，仍能耗尽原 2 条预算并令后续认证 AUDIT_FULL；
无认证首字节的连接仍在 2 秒截止时间后关闭。没有用扩大容量隐藏错误。

`framing::read_after_idle` 仅供认证后的原生连接循环调用；认证及其他既有 `read` 语义保持整体截止时间。
网络循环保留同一个读 future 转发延迟回执，防止写回执时取消/重建半帧读取而失步。

## 认证、ACL、权限与持久性（必审）

DPAPI 安装令牌、当前 SID-only protected DACL、远程拒绝和名称连续占用未改。
T1 仍逐事件获取，控制默认拒绝，认证不授予行动/成长管理权限。跨账户 ERROR_ACCESS_DENIED(5)
沿用用户原文与同构 ACL 的实际句柄检查，未声称再次代跑第二账户。

已按用户补充要求检查上传对象、D、核心事务、Γ settlement、拒绝审计、见证命名空间及令牌创建的
强制同步顺序。详见 [DURABILITY.md](review-20261029/DURABILITY.md)：对象/见证 `sync_all`、
写穿透发布和目录同步；SQLite WAL/FULL 的 VFS 同步；回执在存储 API 成功之后才发送。
**缺失/未单独调用也已列明**：令牌父目录没有显式同步、新对象发布后没有额外的最终文件再同步、
没有网关额外的数据库/WAL 句柄同步或每次强制 checkpoint/目录同步、内存会话/关联没有第二日志。
管道 flush 不算落盘；源码/配置核对不等于设备级电源故障验证。没有借本轮修复改造存储。

## 基线与证据边界

此前验证基线 B=`f24f1256259c998bc213a9dc93103fcfb9cfe89a`；本轮 gateway 42 项实际基线 B₁=`787fd2be…`，
末次 M=`740c2289…`。首先同步至 B₁，随后又同步 M 并重跑工作区 fmt/clippy（该项新基线 B₂=M）。
新增为 R-CHOICE/R-KAPPA 研究记录与 P0-10 载体 ADR/补测卡，gateway 的规范/契约/存储/工作流输入未改。
P5-23 增量只新增 core-kernel/law 与其测试/登记；gateway 不依赖 core-kernel，合成事务处理器不调用 G。
B₁→M 的 gateway、core-types、core-store、契约、Cargo 配置/锁文件与工具链均无变化；42 项按相同依赖源码复用，
不声称在新 merge 提交重新执行了它们。末次同步后逐项确认最终受测输入 blob 未变；清单在同轮 source-manifest.json，
完整性核对以 Git 规范化 blob 为口径，不以 CRLF 工作树摘要冒充提交字节。

所有测试均为 D 盘缓存和合成临时库；未打开用户排练/正式库，未创建系统账户、改用户/系统设置。
未复跑无影响的模型/GPU/全量存储矩阵。冻结契约与九类事件、P5-22 后继、上传生命周期、分派、
持久回执、Γ、拒绝审计、背压、并发/重连与实验禁管道均由本卡完整 gateway 测试复验。

## 规格与依赖

规格对照：[docs/traceability/P5-03.md](../../docs/traceability/P5-03.md)。
无新 IDL、规律、阈值、存储格式或依赖；沿用 ADR 0024、I01/I04、I02/I03/I18/I21、I26/I29。
依赖仍为 core-types/core-store、tokio 1.54.1（MIT）、zeroize 1.9.1、getrandom 0.4.3、serde_json 1.0.154、
sha2 0.11.1、windows-sys 0.61.2（上述其余 MIT OR Apache-2.0）、subtle 2.6.1（BSD-3-Clause）；
测试 tempfile 3.27.0（MIT OR Apache-2.0）。未新增许可或供应链变更。

## 待复审

- [x] R1 修复与正/负回归、本机完整验收。
- [x] R2 用户范围决定、SYS01 状态闭合、强制落盘点/缺失项登记。
- [ ] 独立复审当前修复提交；实现方不代写通过结论、不合并、不领取下一卡。
