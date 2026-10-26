# P5-04：fbcb7cc Windows 失败诊断

2026-10-26，Australia/Perth。结论：本次 self-hosted job 在依赖源读取阶段网络超时；没有发生本卡的编译诊断或测试断言失败。网络更下层根因未确定；本次不修复系统或目录权限。

## 对象与原始证据

| 对象 | 固定标识 |
| --- | --- |
| PR / head | #65 / fbcb7cc14ee492893729833237bf955cc57462b6 |
| 实际 checkout | ff4afbd110b5103c2e745d375c9089719b255e0c，合成合并到 main@317dde9 |
| Windows run / attempt | [40323615588](https://github.com/nanake-lab/nemu/actions/runs/40323615588) / 1 |
| self-hosted job | [121574544222](https://github.com/nanake-lab/nemu/actions/runs/40323615588/job/121574544222) |
| 汇总 job | [121607799867](https://github.com/nanake-lab/nemu/actions/runs/40323615588/job/121607799867) |
| artifact | [12654764805](https://github.com/nanake-lab/nemu/actions/runs/40323615588/artifacts/12654764805)，windows-P5-02-40323615588-1，894 字节 |
| 原始 ZIP SHA-256 | 20c648f230b27a177cf8182d3488e35cd7e07d93a4c6db065c9cc6b8f5de1dec |

`windows-P5-02-original.zip` 为连接器返回的原始 ZIP；下载后本机哈希与 GitHub artifact.digest、job 上传摘要三者一致。`artifact/` 为原样解包，仅含 `clippy.log`（2962 字节）和 `toolchain.log`（417 字节）。不补造缺失的 tests.log。`job-*.log` 是连接器返回的完整解码日志；UTF-8 文本保留原始 CRLF/ANSI。`connector-evidence.json` 同时保留解码文本和来源元数据，不含临时签名下载地址。

## 步骤与因果链

1. checkout 成功；`Decide scope` 输出 `full=False; base=HEAD^1; desktop=False; storage=True`。步骤 API 的 4–18 为 skipped，包含 transport、isolation、e2e、visual。
2. 第 19 步 `W02 NTFS storage checks` 创建 RUNNER_TEMP 下的证据目录，`rustup show` 成功，锁定 Rust 1.99.0 / x86_64-pc-windows-msvc。
3. 第一条 Cargo 检查是 `cargo clippy -p core-store --all-targets --locked -- -D warnings`。13:39:13 UTC 开始更新 crates.io index；13:42:24、13:42:55、13:43:29 UTC 分别报告剩余 3、2、1 次重试；13:44:05 UTC 报错。对应 Perth 为同日 21:39–21:44。
4. 错误链明确是 `fact-view → fastembed 7.3.0 → tokenizers 0.23.2 → rand 0.9.5 → rand_chacha 0.9.0 → ppv-lite86 0.2.21`，最终为 `unable to update registry crates-io`、`download of pp/v-/ppv-lite86 failed`、`transfer too slow: failed to transfer more than 10 bytes in 30s (transferred 0 bytes)`。这些版本及引用在当前 Cargo.lock/Cargo.toml 中存在；没有版本求解冲突或校验和错误。
5. 步骤报告 exit 1；没有独立记录 Cargo 原生退出码。失败发生在依赖源获取阶段，尚无编译或测试断言诊断；后续 `cargo test -p core-store --locked -- --test-threads=1 --nocapture` 未开始，因此没有 tests.log。
6. 第 20 步上传证据成功。汇总 job 的 `Require successful Windows checks` 读取 `WINDOWS_RESULT=failure` 后报 `Windows checks did not succeed (result: failure)`，是上游失败的传播，不能当作第二次隔离测试失败。

可定位原文分别见 `job-121574544222.log`、`artifact/clippy.log`、`job-121607799867.log`；步骤与结论见 `job-steps.json`、`run-jobs.json`。

## TEMP、权限与判断边界

- 失败 job 的证据路径是 `D:\actions-runner\_work\_temp\w02-40323615588-1`，写入和上传均已成功。工作流与 main 的 blob 同为 `65b29f2a8a73a7bb08c44492646f3952d8a17435`，证据路径使用 RUNNER_TEMP，没有本分支旧的 TEMP/TMP 覆盖。
- 此 job 没有执行隔离脚本；相关步骤是 scope=false 后的 skipped。日志中没有 Access denied / 拒绝访问。不能把旧的共享 TEMP ACL 诊断套在本次失败上，也不能据本次未执行宣称隔离验收通过。
- 当前证据足以定位 Cargo 依赖源网络读取超时；不足以区别出口、代理、远端服务或其他网络层原因，也没有证明只需清缓存、更换镜像或调整超时。
- 本轮未修改设置、ACL、runner、Cargo/npm/git 全局配置、镜像、依赖、超时、断言或阈值；未运行重负载或重跑 CI。保留失败，按用户指示记入 BLOCKED.md。

## 本轮提交与核验

先 fetch，再 merge 最新 main@929f82dbb267c47cda06a136b7d34adbfda0fdd3；合并提交 b4e6e73f11bfb44e7482c23ce01a20b0ae4ee8b4。main 新内容仅 #102 的任务/文档；相对被诊断的 fbcb7cc，本卡源码、测试、Cargo.lock 和工作流均未变化。之后只更新当前报告/BLOCKED 及本目录证据，不改卡/契约/存储。

原始 ZIP、解包文件、完整解码日志、JSON、工作流一致性和源码未变由 `verify-evidence.ps1` 核验；结果见 `verification.json`、`SHA256.json`。本轮只针对证据和报告的变更运行上述检查与 `git diff --check`，不把先前本机测试记为此失败 job 的复测。

最新 [交叉审查评论](https://github.com/nanake-lab/nemu/pull/65#issuecomment-6419219224) 的空响应字节预算、CPU clippy Rust scope 两项仍单独阻塞；本次网络诊断没有修改这两项。ADR 0020 仍待用户批准，needs-user 保留。

网络恢复后可按原配置重跑；若持续失败，可先收集 job 环境的只读 registry 连通性诊断。两者均未在本轮执行，不以绕过网络或放宽验收作为成功。后续选择和代价见上级 BLOCKED.md。
