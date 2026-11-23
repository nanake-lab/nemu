风险档位：重档；改动 core-kernel 的测试构建入口，需要独立交叉审查。

# P5-65 条件性模块纳入构建与 CI

## 当前状态

实现与 Windows 全工作区复核完成，代码提交 CI 通过，CI 时长已记录；最终文档提交的检查状态见 PR 最新验证评论，独立交叉审查待用户安排。PR #134 保持 draft / needs-user，由用户合并，合并即批准开卡。未取下一张卡。

## 交付物与边界

- lib.rs 末尾仅追加条件性交付注释及两个 `#[cfg(test)]` 模块登记。两个模块新增进入 crate 回归的测试为 evidence 40 项、recompute 28 项。
- 在 tasks/cards.json 与 tasks/P5-echo-core.md 登记 P5-65；最新 main 与开放 PR 检查时编号未占用。两份 requests 追加“测试构建登记已完成”，生产登记、真实 H 接线、迁移与正式验收仍由 P5-54 等后继处理。
- 两个目录共 32 个文件（含源码、测试、verify.ps1）与基线逐文件 Git blob 一致：[证据](verification/module-blobs.json)。工作区测试现已覆盖，两个 verify.ps1 原样保留，仅作历史证据入口。
- 条件性交付仍禁止生产启用；旧 EV7/I27/EV9 测试不签发 ADR 0032/0033 后继语义验收，P5-24/P5-26 整卡未因此完成。未新增依赖、IDL、黄金向量、存储格式或差异登记。

## Windows 验证

基线 B：`da8c18c2400d74ef5c9ec6b10563fdfdebe86f24`；被测代码提交：`f7d83db599d19d34e6a8ac6f45a52196d5d8f66f`。本机为 Windows / x86_64-pc-windows-msvc / Rust 1.99.0，见 [工具链](verification/toolchain.txt)。命令在 core/ 执行，重负载串行；仅进程内设置 CARGO_HOME=`D:/Dev/cache/cargo`、CARGO_TARGET_DIR=`D:/Dev/cache/P5-65/target`、CARGO_BUILD_JOBS=4、TEMP/TMP=`D:/Dev/cache/P5-65/tmp`。无用户谱系参与。

| 命令 | 结果 | 证据 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 通过 | [fmt](verification/fmt.log) |
| `cargo clippy -p core-kernel --all-targets -- -D warnings` | 通过 | [kernel-clippy](verification/kernel-clippy.log) |
| `cargo test -p core-kernel` | 241 passed、0 failed、1 ignored；命令 344.30 秒，测试 320.17 秒 | [kernel-test](verification/kernel-test.log) |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 | [workspace-clippy](verification/workspace-clippy.log) |
| `cargo build -p core-kernel --lib --locked` | 通过；普通库构建 | [production-build](verification/production-build.log) |
| `cargo test --workspace --exclude core-kernel --locked` | 通过，776.89 秒 | [其他工作区测试](verification/workspace-other-tests.log) |

Windows 工作区测试分为 core-kernel 与其余全部成员两段，避免重复运行刚通过的 320 秒测试；不减少测试范围。唯一 kernel ignored 是既有 `dynamics::tests::reference::theta_1_original_500_streams_zero_violations`（V2-93），未新增忽略或放宽断言。耗时原始值见 [timing](verification/timing.txt)。

## CI 覆盖与时长

workflow 不改。ci.yml 的 `^core/` 范围判断启用 Rust checks：`cargo clippy --workspace --all-targets -- -D warnings` 与 `cargo test --workspace --locked --exclude core-store` 均包含 core-kernel 测试目标；工作区成员为 `crates/*`。这里排除的是 core-store，不是两个模块。W19 既有可选 CPU 特性检查照常运行；windows.yml 路径规则不触发本次纯 kernel 改动，Windows 复核由以上本机运行提供。

代码提交 CI [41682430112](https://github.com/nanake-lab/nemu/actions/runs/41682430112) 通过：workflow 总计 **977 秒（16分17秒）**，hosted job 972 秒，Rust checks **796 秒（13分16秒）**，W19 27 秒；core-kernel **241 passed、1 ignored**，测试执行 363.20 秒。原始 [run 元数据](verification/ci-code-run.json)、[jobs/steps](verification/ci-code-jobs.json) 与 [日志](verification/ci-code.log) 保留。

比较基线见 [CI 历史数据](verification/ci-baselines.json)：同为 Rust+W19 范围的 PR 运行 41384300050 总计 405 秒、Rust 361 秒；push 运行 41387711019 总计 697 秒、Rust 634 秒。最近祖先全范围运行 41415788259 总计 1269 秒，但包含存储、Node、Python 等额外工作，不能直接比较。相较同范围 PR 基线，总时长增加 **572 秒（9分32秒）**、Rust 增加 435 秒；相较同范围 push 基线，总时长增加 **280 秒（4分40秒）**、Rust 增加 162 秒。checkout 本次约 140 秒，各次编译/测试速度有明显波动；这些是观察差值，不是仅由新增 68 项测试造成的受控增量。

首个代码提交的 Docs guard [41682429942](https://github.com/nanake-lab/nemu/actions/runs/41682429942) 在 checkout 阶段约 5 分钟后取消，未运行目录校验，记录保留于 [原始日志](verification/docs-guard-cancelled.log)。本地 136 张卡的 JSON、唯一编号、依赖与里程碑校验通过；这不替代最终提交的远端 Docs guard。

## 验证基线与文档检查

代码 CI 实际 checkout `6fe0eaf19dd9685fedab1d632c00aacfe34b2e98`，已核对两个父提交分别是 main B=`da8c18c2400d74ef5c9ec6b10563fdfdebe86f24` 与 PR head=`f7d83db599d19d34e6a8ac6f45a52196d5d8f66f`。2026-11-04 再 fetch 后 main M 仍等于 B；无增量需同步。最终追加只含报告、规格对照与两份请求状态，代码/依赖/workflow 不变。

本地 `git diff --check`、新增相对链接、136 张任务卡的 JSON 解析/唯一编号/依赖/里程碑检查通过。历史 CI 元数据由子代理按 `.codex/agents/routine-reporter.toml` 的 gpt-6.1-terra / max **显式参数试用**只读采集；不是按角色名自动加载验证，不承担审查或验收裁决。

## 规格对照

见 [P5-65 对照表](../../docs/traceability/P5-65.md)，保留 I02/I03/I18/I23/I24/I25/I29 的权限、回归与条件性边界。无正式新规律、行为或生产验收。

## 待完成与用户事项

- [x] Windows 全工作区检查及代码提交 CI 通过，CI 时长变化已记录。
- [ ] 最终 head 必要检查：以 PR 最新验证评论及 check rollup 为准；不得拿旧 head 绿灯代替。
- [ ] 独立交叉审查（请审查）；本实现者不代签。
- [ ] 用户合并批准开卡及任务目录登记。
