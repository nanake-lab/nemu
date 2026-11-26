# P6-01 验收证据

## 1/2 复验

实现提交 `4abad0f8ac13b4b08db956ab7c374618e8cfb858`。Windows 本机 `cargo test -p task-layer --locked --offline -- --nocapture` 通过 67 项，`cargo clippy -p task-layer --locked --offline --all-targets -- -D warnings` 通过。复验包括设置缺失时的完整回复、逐字段覆盖、配置错误、ADR 0047 权限绑定的正反例及既有 SYS14 回归。私有证据提交为 [a0fa6bffc71864fca9c4e4457ace89e22539e54c](https://github.com/nanake-lab/nemu-evidence/tree/a0fa6bffc71864fca9c4e4457ace89e22539e54c/P6-01)。

| 原始文件 | SHA-256（Git 文件字节） |
| --- | --- |
| P6-01/windows-tests-4abad0f8.log | `aecab169868a645db4fdc237621f12dd80684d900a5d160904e597cf2e1c51ae` |
| P6-01/windows-clippy-4abad0f8.log | `3f67917d0b8db3376f0b8a5a32c9584aed1403ca8f08afe2e5cecc8d950170e7` |

## 首轮验收

代码提交 `7f9e9a3eb1820be05132de810913d50ca95bf662`。Windows 在临时测试存储运行 `cargo test -p task-layer --locked --offline -- --nocapture`，65 项通过，0 项失败，0 项忽略。包括 50 项现有单测、15 项集成测试。[Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/44888568455) 通过工作区 fmt、Clippy、除 core-store 外的工作区测试及独立的 core-store 测试。整次运行结果为 success。

| 检查 | 实测结果 |
| --- | --- |
| [X01](X01/result.json)、[SYS06](SYS06/result.json) | 任务输入分别变化，固定输入的 W03 结构夹具结果一致。 |
| [SYS14](SYS14/result.json) | 模式 B 解释失败保留原因，生成和登记块数均为零，迟到成功无法重开。合法无作用在等待 150.455 ms 后正常生成，在 100 ms 生成预算内完成生成与替身登记，用时 0.600 ms，两项计数各为 1。单独允许的无调制路径保留脚本生成器的实际用量与故障依据。 |
| [SYS08](SYS08/result.json)、[SYS13](SYS13/result.json) | 真正的输出流、行动请求、持久化和跨进程重放均为 `not_run`。 |
| I28 | zh/en/ja 候选及清单的 JSON 静态检查通过。协议加载、草稿状态和用户设置测试通过。N2 用户签字待定。 |
| 查询、主动选择、云端组包及说话核对 | 进程内断言通过，覆盖三种查询、五类非成功状态、空计划及越界拒绝、旧计划失效、要点来源与一次组包。实际终端、生成器、权限门的接线见[阻塞记录](BLOCKED.md)。 |

原始日志在[私有证据仓库 P6-01](https://github.com/nanake-lab/nemu-evidence/tree/4a35d1c605b96df75ac63b958dd5eaa7b8ab9bbc/P6-01)。以下 SHA-256 对应该提交中的 Git 文件字节。

| 文件（相对于证据仓库） | SHA-256 |
| --- | --- |
| P6-01/windows-task-layer-7f9e9a3e.log | `6f9ad0a73446ea60537f69677b0bc61319c9f4c427ca5d879d4ad2d94171e79a` |
| P6-01/ci-bef69a7c-failure.log | `e62efc028b5af3c31f22206154ddd16410461c9bfe9cfc7f81cd34be39a5133f` |
| P6-01/ci-11903209-failure.log | `cd0d000e00951a580afa94496ce5cb70e5a7268793be9356af2549719be44021` |
| P6-01/ci-ec4a8163-failure.log | `364bffb9dbe8d86da37c84e8a5f01771d3f612ef27ec3aaf9a2767cbe40ea8ac` |

三份较早的失败日志分别记录 Clippy 对复合类型、立即可构造错误值、借用 W19 夹具的可选 feature 条件的诊断。代码提交已修正，Ubuntu 的 Rust checks 已复验。Windows 草稿 PR 的自动状态门会跳过 runner 并返回失败；上表 Windows 数据来自本机直接运行。
