# PR #4 更新到最新 main

## 结果

按用户要求，把 main d523e0b382d3283e0cb1965c17b4e28c209d1676 合入 task/P5-01-idl，合并提交为 85c44789d46f2ac9504c7165697fcd898edd3a03；自动合并成功，无冲突。此记录与复验日志另行提交，推送后的最终头提交需重新通过双平台 CI。

## 已确认的历史 CI

- PR #7 run 38733202545：Ubuntu / Windows 均成功；已 squash 合并为 d523e0b。
- main run 38735198220：成功。
- PR #4 原 1654ee8，run 38584387442 attempt 2：Windows 成功，沿用未重跑的已成功 Ubuntu 作业。相同旧提交复跑通过，说明失败未稳定复现，但仍不确定历史根因；不把旧提交绿灯用于更新后的提交。

## 更新范围

本次仅合入已在 main 的 P1-01 和 PR #7 诊断/证据隔离改动。相对原 PR 头 1654ee8，core/contracts、core/crates、workers、冻结原始规格、架构及契约均无变化；不重开已裁决事项，不新增运行时依赖，不改变验收阈值。P5-01 原有 Ubuntu 三语言一致性检查保留，Windows 使用 main 的诊断工作流。

## 合并后本地验证

执行 frozen-lockfile 安装，再运行 verify-main-20261015.ps1（原 verify-full.ps1 的等价副本，仅证据目录改为 acceptance/main-d523e0b）。27/27 检查通过，旧证据未覆盖。

- Rust/Python/TypeScript 各 5228 用例通过；78 种消息、1306 合法与 3922 拒绝用例；官方 protoc 241 条互操作向量通过。
- Rust fmt/clippy/workspace tests、pnpm lint/typecheck/test/build/test:contracts、契约工具 ruff/mypy、四个 Python 项目的检查和测试、三个生成目录 mypy 均通过。
- frozen-specs 检查通过，无冻结规格改动。
- 证据：acceptance/main-d523e0b/checks.json 及逐命令日志。本轮不在本地运行 Electron，Windows 桌面及隔离检查由推送后的 CI 执行。

## 下一步与 runner

旧三项任务已完成，runner 检查时在线且空闲。推送本次 PR #4 更新会触发新的 Ubuntu / Windows CI；Windows runner 仍需在线完成该作业。当前不合并 PR #4，不启动下一张卡。任何后续合并均需最终头提交双平台全绿及 AGENTS.md 其他条件满足。
