轻档（当前只改任务卡和诊断报告）。

# DIAG-SYS01-01 · Windows 上的 SYS01 并发超时

## 当前证据

诊断基线是 `c5636361`。同一提交的主分支 Windows CI 以 `cargo test -p gateway --locked -- --test-threads=1` 运行，42 项中 41 项通过。唯一失败的是 `sys01_authenticated_upload_dispatch_reconnect_gamma_and_reverse_receipts`，在测试客户端读取回执的 `service.rs:316` 返回 `gateway deadline exceeded`。整个测试二进制耗时 131.92 秒。单线程运行也可能遇到该症状。失败行是多个请求共用的读取函数，单凭该日志无法识别哪一条回执超时。[Windows CI 日志](https://github.com/nanake-lab/nemu/actions/runs/44697435088)

本机计划在独立工作区对未修改的 gateway 各做十轮默认并发和单线程测试，原始 stdout/stderr、失败位置、耗时逐轮保存。当前等待同机重负载释放，CI 记录单列，不计入本机轮次。

## 已核对的夹具边界

| 排查项 | 源码证据 |
| --- | --- |
| 临时目录、固定名和 DPAPI | 每个用例重新分配随机临时根。数据库、DPAPI 令牌均位于各自的根下。`"sys01"` 是根下的局部名字。见 [TemporaryRoot](../../core/crates/core-store/src/lineage.rs) 与 [服务夹具](../../core/crates/gateway/src/tests/service.rs)。 |
| 管道、端口与实例 | 服务夹具用 SID、进程号和用例标签命名管道。`sys01_pipe` 使用另一前缀和 `dedup`、`restart` 两个标签。两条路径都走本地命名管道。每个管道名允许 16 个实例，服务夹具最多 4 个并发连接。见 [结构测试](../../core/crates/gateway/src/tests/sys01_pipe.rs) 与 [Windows 管道创建](../../core/crates/gateway/src/windows.rs)。 |
| 共享状态 | 服务开关、提交计数、等待反转的回执分别属于各自的 `Core` 实例。包内共用的黄金向量在 `OnceLock` 初始化后只读。见 [服务夹具](../../core/crates/gateway/src/tests/service.rs) 与 [测试入口](../../core/crates/gateway/src/tests/mod.rs)。 |
| 等待和时序 | 反序回执用例先停住合成核，再显式放行；第一次提交的结果由夹具持有，第二次真实提交完成后才按相反顺序返回。客户端读帧从等待第一个字节起计时，原有期限为 10 秒。见 [验收用例](../../core/crates/gateway/src/tests/service/acceptance.rs)、[提交夹具](../../core/crates/gateway/src/tests/runtime_fixture.rs) 和 [读帧](../../core/crates/gateway/src/framing.rs)。 |
