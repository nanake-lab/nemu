# P5-36 首次审查修复证据

审查基线 `59d7a04f1f19843a4ab1b6c6ae834cc1f49d213a`；[审查结论](https://github.com/nanake-lab/nemu/pull/108#issuecomment-6461866895)。main 基线仍为 `f24f1256259c998bc213a9dc93103fcfb9cfe89a`。本目录只记录修复增量，不重建旧黄金数据。

## Limits 不一致

Rust θ 语义边界、参数摘要、kernel 导出/恢复改为显式接收调用方 `Limits`；对象准备/读取传入原 `StoreConfig.encoding`。不改变 IDL、字段限制、数值算法、冻结向量或 Python/TS 语义边界。

- `limits-before-rebuild.log`：源码修改后、探针重新构建前，对旧探针的预期失败。1 MiB 通过，1 MiB+1 的 Rust ThetaLastEvent encode 报 string limit，复现审查反例。
- `contracts.log`：三语言各 13,224 个既有向量，加 150 个共同资源预算回归；覆盖 LastEvent、完整快照和参数消息的 1 MiB/1 MiB+1 encode、CanonicalV1 decode、Protobuf decode，以及窄字节/项数/深度预算和错误摘要的拒绝。宽配置共同为 4 MiB/4096/64。
- `kernel-limits.log`：实际 C 接收大事件 ID/大规律 ID，导出/参数摘要/双编码/恢复和后续 stage 全部位值相同；窄配置拒绝。
- `store-limits.log`：两种大小经过 prepare/stage/commit/关闭重开/read；窄配置 prepare 无对象或发布记录，read 拒绝。
- `python-mypy.log` 是新增回归脚本的初次类型检查失败（循环导入常量的类型推导及 Path 参数），`python-mypy-02.log` 为修正后的通过。原始失败保留。

## 写锁生命周期

`platform::WriterLock` 在 Store 的 DB/对象字段销毁后显式 unlock；失败记录原始 I/O 错误，之后仍关闭句柄。获取失败保留 `TryLockError::WouldBlock` 或内部 I/O errno，不再全部改写成另一 writer。没有获取重试、等待或放宽单写者门禁；未改 SQL/回执/故障断言。

Linux 定向测试用 `try_clone` 确定性地保留同一 open file description：先验证原 close-only 行为仍阻塞新 writer，再验证显式 unlock 后即使旧副本活着也可取得新锁；关闭旧副本不能释放新 owner 的锁。记录 `/proc/locks` 和 `/proc/self/fdinfo`。另做固定 128 次并发 spawn/drop/reopen，获取失败直接记录并失败，不重试。CI 在现有完整存储套件之外仅追加一次这些短测试的 `--nocapture`，保留系统锁证据。

依据：[Rust File 锁/解锁文档](https://doc.rust-lang.org/std/fs/struct.File.html#method.unlock)、[TryLockError](https://doc.rust-lang.org/std/fs/enum.TryLockError.html)。同一打开文件描述被 dup/fork 共享的生命周期与历史 Ubuntu 失败相容，但没有历史失败当时的持锁进程追踪，不能断言那一次的唯一根因已经确认。新回归定位并消除的是可确定复现的 close-only 生命周期缺陷，最终 Linux 实测以修复提交的 CI 原始日志为准。

旧数据迁移、H 重放、成长语义和完整运行时编排未扩围。修复提交的完整测试/CI 结论见主报告和最终 PR 复审请求；历史成功不能替代本次结果。
