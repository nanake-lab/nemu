# P5-03 审查 R2 — 强制落盘点清单

核对基线：已 merge main `787fd2bec82f17b247158a0a985c3ee2c214af03`，存储实现未改。
位置来自实际源码；blob/SQLite 缓存源码 SHA256 见 durability-source-manifest.json。
表内 core-store/src、gateway/src 分别指 core/crates/core-store/src、core/crates/gateway/src；行号按该清单固定。
本表是调用顺序与源码核对，不是逐次 WinAPI 跟踪，也不证明当前硬件在真实掉电下的持久性。

`File::sync_all()` 按用户要求的“等效调用”登记；它要求同步文件内容和元数据，不能用
`write_all`、关闭句柄或管道 `flush` 冒充。[Rust 官方说明](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all)。
SQLite 的 WAL/FULL 在每次事务提交增加 WAL 同步，走 VFS xSync；本地已锁定的 Windows VFS
`winSync` 调用 `osFlushFileBuffers(pFile->h)`（libsqlite3-sys 0.38.2 / sqlite3.c:51935）。
本机该缓存构建指纹的 LIBSQLITE3_FLAGS/CFLAGS 均为空，见 sqlite-build-environment.json；
未见禁用同步的 SQLITE_NO_SYNC 编译选项。API/配置保证仍以文件系统、设备正确履行为前提。
[SQLite 官方 WAL/FULL 说明](https://www.sqlite.org/pragma.html#pragma_synchronous)，
[Microsoft FlushFileBuffers 说明](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers)。

## 初始化及对象路径

| 路径/阶段 | 实际强制同步及位置 | 说明 |
| --- | --- | --- |
| 新建存储根与 objects 目录 | core-store/src/store.rs:70 `sync_dir(parent)`；:76 `sync_dir(root)` | 初始化命名空间目录项；不是每次业务提交重复创建 |
| 初始化 SQLite | schema.rs:13–14 设 WAL、synchronous=FULL；:23–27 复查 FULL；store.rs:86 `tx.commit()`，:88 `sync_dir(root)` | 初始 schema/header 提交后同步目录，再创建见证空间 |
| 初始化见证命名空间 | witness.rs:41 同步每个 d/commits/reject 子目录；:44 同步见证根；:46–49 同步其父目录 | 先完成目录可恢复性，后续记录不重新创建该目录 |
| 完成上传、新对象/依赖/事件/暂存快照/账本文件 | objects.rs:195 临时文件 `sync_all()`；:197 `persist_noclobber`；:199 `sync_dir(objects)` | 用于 gateway uploads.rs 的 Store.prepare/prepare_r，也用于 Store.ingest、stage 等正式对象写入；上传完成不等于进入 D/H |
| 对象原子发布 | platform.rs:72 `MoveFileExW(..., MOVEFILE_WRITE_THROUGH)` | 同目录、不覆盖，不使用跨卷复制/替代路径；文件内容已先同步，目录随后同步 |
| 复用已存在对象（含上次 rename 后崩溃的对象） | objects.rs:167–171 重新打开文件 `sync_all()`；:174 `sync_dir(objects)` | 不是仅检查文件存在后直接回成功，先重新建立同步边界 |
| Windows 目录同步公共函数 | platform.rs:29–40；GENERIC_READ/WRITE + FILE_FLAG_BACKUP_SEMANTICS 打开目录后 `sync_all()` | 错误传播，不静默忽略 |

## D、核心提交、过滤/冲突与拒绝审计

| 路径/回执 | SQLite 强制同步边界 | 独立持久见证与最终发送顺序 |
| --- | --- | --- |
| 终端事件接收 RECEIVED_DURABLE | terminal_ingress.rs:69 委托 ingest；ingress.rs:71 `tx.commit()`；同连接 WAL/FULL → Windows VFS xSync/FlushFileBuffers | ingress.rs:73–80 `witness::record(kind=d)` 全部完成后才产生持久接收回执，terminal_ingress.rs 返回 ReceiptV2；dispatch.rs 之后才入有界发送队列 |
| 实际核心事务 CORE_COMMITTED | transaction.rs:264 `tx.commit()`（包括 H/快照/账本/索引/head）；WAL/FULL → VFS 同步 | :266–273 `witness::record(kind=commits)` 完成后 Store.commit 才返回成功；gateway/progress.rs 还需核对实际发布 head、事务/事件/原请求关联后才转发 CORE_COMMITTED |
| Γ 重复过滤或稳定键冲突 | ingress.rs:143 `tx.commit()`；WAL/FULL → VFS 同步 | :145–152 `witness::record(kind=commits)`；不新增核心状态/账本/时间锚点。冲突还经 gateway 调用下行拒绝审计，失败不伪造成功 |
| 独立拒绝审计 | terminal_ingress.rs:118 `tx.commit()`；WAL/FULL → VFS 同步 | :120–127 `witness::record(kind=reject)` 完成后才返回已持久审计。满额/SQL/文件/同步失败不能声称成功审计或安全接收 |
| 各类见证文件公共写入函数 | 不替代上述 SQLite 同步 | witness.rs:72 临时文件 `sync_all()` → :75 写穿透发布 → :80–83 最终目标文件再次 `sync_all()` → :86 `sync_dir(kind)`，三类路径共用；既有目标也验证内容并再次同步文件和目录 |

## 缺失、未单独调用及未验证的部分

- **DPAPI 初始化令牌父目录：缺失显式目录同步。** gateway/windows.rs:244 对新建的密文文件调用
  `file.sync_all()`，但 create_secret_file 没有另外对令牌父目录执行 `sync_dir`。
  该文件在测试的三类崩溃点之前已经配置完成；本卡没有测“首次安装创建令牌时真实断电”。
- **对象发布后最终路径的第二次文件同步：新对象路径没有该额外调用。** 当前是临时文件同步 →
  WRITE_THROUGH rename → 目录同步；复用对象和见证文件则有对最终文件的显式同步。
  本条列出调用差异，不把缺少“第二次同步”自动判作已经证明的持久性漏洞。
- **每次 COMMIT 后没有网关另开的数据库/WAL 句柄 FlushFileBuffers、强制 checkpoint 或存储根目录同步。**
  事务持久性依赖受检查的 SQLite WAL/FULL + Windows VFS；WAL 在提交时同步，并不表示每次都
  checkpoint 到主数据库文件。数据库/WAL 辅助文件目录元数据的真实失电保证未另行实测。
- **内存状态不落盘是明确设计。** 未完成上传、连接 session、待回执关联/发布摘要、pin 所有权和输出队列
  没有自己的 fsync/第二日志；断线/重启丢弃，客户端重新认证、从 offset 0 上传、保留原事件 ID/稳定键重试。
- **传输 flush 不是落盘。** framing.rs 的 AsyncWrite::flush 仅处理管道发送，不算上述存储同步点；
  API 成功也不保证客户端已经收到回执，故断线仍按原 ID/稳定键处理未知结果。
- **未运行物理断电、驱动/控制器/磁盘缓存或电源保护测试，未做卷级 flush、系统设置调整或硬件能力认证。**
  用户 2026-10-29 明确接受现有三类进程骤停+新进程恢复作为本卡 SYS01 范围，物理断电继续 not_run；
  是否补测留到 E4 存储与备份阶段由用户决定。上述缺失/未验证点保留给该阶段核对，不悄悄修改存储实现。

本卡的认证、ACL、权限、持久回执审查仍由独立审查方完成；用户对验收边界的决定不代替代码审查。
