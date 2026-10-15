# Windows E2E 诊断与当次证据隔离 — 报告

## 结果

诊断实现及本地验证完成；等待独立 PR 的 Ubuntu / Windows CI，不合并 PR #4，不启动下一张任务卡。用户于 2026-10-15 授权本独立分支，仅补诊断，不改变 5 秒窗口等待或任何断言。

分支基于最新 main d908bf8（已包含 P1-01）。PR #4 保持 1654ee85a00f36396658bba85f3d68f8adecbee0。原故障根因仍未确定，本 PR 不声称修复了窗口偶发缺失。

## 交付物

- apps/desktop/src/main/diagnostics.ts：仅在 NEMU_EVIDENCE_DIR 开启时记录进程、窗口创建、加载、显示、渲染失败、退出及会话锁定/解锁事件。UTC 与单调时间同时记录；app ready 后记录 Electron 原生 idle/locked/unknown 状态。
- apps/desktop/tests/smoke.spec.ts：增加启动及 Playwright 观察窗口事件；启动解析后异步采集 Windows 快照，不等待快照后再开始原断言。诊断子进程最长 10 秒，仅限制诊断采集；测试总时限仍为 30 秒，窗口等待仍为默认 5 秒。
- tools/scaffold/windows-evidence.ps1：测试前、启动后、测试后采集进程 ID/名称/会话、TCP 监听端口及所属进程、会话查询和会话事件。查询不可用或无权限明确记录，不将 interactive 或 Active 误报为未锁屏。不采集命令行、环境变量全集、窗口标题或文档内容。
- Electron 标准输出和错误输出由 Playwright 的 pw:browser 从 spawn 起捕获，完整保存在 e2e.log，并提取 electron-out.log / electron-err.log。空输出保留空文件；日志保留 PID 和时间差。
- .github/workflows/ci.yml：Windows 证据写入 RUNNER_TEMP/nemu-evidence/<run_id>/<attempt>/windows，目录存在即拒绝复用；artifact 名同样带 run 与 attempt。上传只在身份及文件新鲜度校验成功后执行，失败测试仍可上传其有效诊断。
- 旧 reports/P0-01 不再整体上传。P1-01 的视觉生产器归 Antigravity 所有，保持源码与基准不变；wrapper 仅收集本次命令起始之后写出的 comparison.json / diff 图片，再执行统一校验。
- manifest.json：记录 run、attempt、checkout SHA、开始/校验时间、文件创建/修改时间、长度与 SHA-256；拒绝旧时间戳、错误身份、链接文件/目录。不把 manifest 自身列入其哈希清单。

## 验收

本地测试开始前仓库 runners API 返回 0，进程列表无 Runner.Worker / Runner.Listener / Electron；未启动 runner，全部测试使用原有临时 scratch 存储。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| pnpm lint | 通过 | lint.log |
| pnpm typecheck | 通过 | typecheck.log |
| pnpm test | 332 项通过 | unit.log |
| 原 expect 调用 AST 比较 | 26 个调用文本（忽略空白）一致；原 poll.toBe(2) 未改 | assertion-check.json |
| pnpm test:transport | 通过 | local-all-checks/transport.log |
| Windows Low IL 隔离 | 拒绝中完整性目录写入，正向写入控制通过 | local-all-checks/isolation.log |
| pnpm test:e2e（包含完整构建） | 通过；最终诊断代码也重跑通过 | local-final/e2e.log、e2e-results.json |
| pnpm test:visual | 18/18 通过，全部 0 差异像素，无基准更新 | local-all-checks/visual.log、stage-visual/comparison.json |
| pwsh -NoProfile -File tools/scaffold/windows-evidence.tests.ps1 | 新文件通过；重复目录、旧时间戳、错误 attempt 均拒绝 | provenance-tests.log |
| 上传前 Verify | 最终 E2E 14 个文件校验通过；完整检查批次 26 个文件通过 | 两个 local 目录的 manifest.json |

local-all-checks 是完整 Windows 命令批次；local-final 是随后补齐 Playwright 观察事件和 idle-state 后的最终 E2E。两者 run.json 的 SHA 是验证时基线 HEAD，代码当时尚未提交，不能冒充已提交版本的 CI。CI 必须另行验证最终提交。没有修改冻结规格或新增依赖。

## 限制

诊断本身增加少量文件写入和一个有界 OS 查询子进程，可能影响时序，不能据绿灯倒推原故障原因。若系统 API 返回 unknown，或启动早于 app-ready 即失败，初始锁定状态仍可能未知；原始错误保留。旧 run 的复跑使用旧工作流，不会继承本次新增诊断与证据隔离。

## 待用户确认

- 无新的设计决定。等待用户启动 Windows runner；运行顺序见 RUNNER-PLAN.md。

## 下一步

独立诊断 PR 全绿且分支包含最新 main 后按规则 squash 合并。之后仅对 run 38584387442 的失败 Windows 作业复跑一次（头 1654ee8；历史合并测试提交 82c65bb）。PR #4 在 Ubuntu 和 Windows 均通过及其余合并条件满足前仍不合并；不启动下一张卡。
