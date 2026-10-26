# P5-04 当前阻塞状态

2026-10-27：两项 P2 已修复，新增三项回归、CI scope 11 场景及第一轮 fmt/clippy/完整 release 工作区测试通过。同步最新 main 后的复验等待本机资源；未完成前不推送本轮修复。独立复审、当前提交 CI 与 ADR 0020 批准仍是合并条件。

**已修复：** 完整计入 query_id 的最低响应信封，装不下则检索前 Error::Invalid，返回前再次核验总字节；CPU clippy 加回既有 Rust scope 条件。旧实现反向测试在预算断言处失败（exit 101），固定源码完整恢复后新增三项通过。见 [修复与原始证据](review-fix-20261026/README.md)。

**同步：** c4162dec 合并 main@65052c0（#87/#105/#106/#101）。Windows 工作流按 main 原样接收 #106 的 20→60 分钟 job 时限，blob a686643b9c6f7508e949157c36ddaf97e946e8fc；本卡未自行改时限或恢复 TEMP/TMP 覆盖。未修改权限、用户/系统/全局设置、runner、网络、依赖版本或验收阈值。

**历史环境失败保留：** fbcb7cc 的 [job 121574544222](https://github.com/nanake-lab/nemu/actions/runs/40323615588/job/121574544222) 第 19 步在 Cargo 读取 ppv-lite86 索引时低速超时，step exit 1；ZIP 哈希核对一致，只有 toolchain.log/clippy.log，存储测试未开始。不是这两项 P2 的运行失败证据，也未被本机通过改写。详见 [诊断](windows-failure-fbcb7cc-20261026/README.md)。

**余项与选择：**

1. 等当前 CI 作业释放本机资源后串行完成同步后复验，再推送并写「请审查」；等待会增加时间，不与 CI 编译/测试抢资源。
2. 推送后由当前 head 的 CI 验证；若依赖源超时持续，先只读定位 job 环境的 registry 连通性。任何系统/全局调整仍由用户决定并本人执行，不以改权限、换镜像、清缓存或放宽条件绕过。

不自行复审批准、合并或取下一卡；保留 needs-user。旧失败、旧 Windows 权限证据与全部本轮结果保留。
