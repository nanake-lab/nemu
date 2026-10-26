# P5-04 当前阻塞状态

2026-10-26 更新：本机既有验收结果保留；Windows CI 仍有环境阻塞，另有两项未解决的交叉审查问题。不能合并。

**卡在哪：** head `fbcb7cc14ee492893729833237bf955cc57462b6` 的 [run 40323615588 / job 121574544222](https://github.com/nanake-lab/nemu/actions/runs/40323615588/job/121574544222)，第 19 步 `W02 NTFS storage checks` 在 `cargo clippy -p core-store --all-targets --locked -- -D warnings` 更新 crates.io 索引时失败，步骤 exit 1。`ppv-lite86` 源数据读取连续重试后仍为 30 秒 0 字节的低速超时；依赖链是 fact-view → fastembed → tokenizers → rand → rand_chacha → ppv-lite86。没有 Rust 编译诊断或测试断言失败，不能据此认定实现通过或失败。

**已做：** 读取完整 job / 汇总 job 日志、步骤与上传 ZIP。ZIP SHA-256 与 GitHub 一致；只有 toolchain.log、clippy.log，没有 tests.log，存储测试未开始。scope 为 desktop=false / storage=true，隔离、E2E、visual 都跳过；汇总检查仅因上游 failure 失败。本轮已 merge 最新 main@929f82d（合并 b4e6e73，仅文档/任务变化）；本卡源码、测试、锁文件及工作流相对 fbcb7cc 均未变。原始证据和详细定位见 [诊断](windows-failure-fbcb7cc-20261026/README.md)。

**边界：** 本次已成功在 RUNNER_TEMP 下写入并上传证据，没有旧 TEMP/TMP 覆盖，日志未出现拒绝访问。未修改目录权限、用户/系统/全局设置、runner、依赖版本、超时、阈值或测试；未切换镜像、清缓存、离线替代验收或主动重跑 CI。网络故障的更下层原因（出口、代理或远端等）尚无证据。

**后续选项（未执行）：**

1. 网络恢复后按原配置重跑 Windows 检查：不改变验收条件，代价是重跑和等待；仍可能超时。
2. 若持续失败，先只读检查该 job 环境下的 registry 连通性与 Cargo 日志：增加诊断时间；任何需要的系统/全局配置调整仍由用户决定并本人执行。

另见 [fbcb7cc 的交叉审查](https://github.com/nanake-lab/nemu/pull/65#issuecomment-6419219224)：空响应信封可超 user_view 字节预算、CPU clippy 缺少 Rust scope 条件，两项均未在此次 CI 环境诊断中修复。它们不是本次依赖下载超时的原因，复审通过前仍阻塞；ADR 0020 也仍待用户批准。保留 needs-user，不合并、不取下一卡。
