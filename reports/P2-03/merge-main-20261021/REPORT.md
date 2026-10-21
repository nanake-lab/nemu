# P2-03 合 main 与合并验收补充

- 用户授权：Claude 审查通过后，将 origin/main 合入任务分支；重跑 pytest、ruff、mypy、pnpm lint/typecheck/test，等最终 Ubuntu 与 Windows CI 通过后 squash 合并。合并后不取下一张卡。
- 审查：https://github.com/nanake-lab/nemu/pull/43#issuecomment-6310784213；结论「审查结论：通过」「没有阻塞问题」，审查对象 a78fe57。
- 2026-10-21 执行 git fetch origin，再 git merge --no-edit origin/main。合并提交：8ff7ea47e152ed024be52c6352b3a08bc3ad4744。
- main 新提交：9ec647a（#46），只在 docs/adr/0014-v2-amendments-from-research.md 和 tasks/P5-echo-core.md 追加共13行。AGENTS、P2-03 卡及必读文档未变化。
- 四条不阻塞意见不扩展为代码修复；合 main 是本次用户直接要求，其余许可字段、Python CI、模板配置保持原报告的后续建议。
- 原有未跟踪 logs/ 与 reports/Echo-Core-Research-2026-10-21/ 保持原状，不提交。

## 合并后的本地检查
结果与逐命令日志在本目录后续追加。

### 本地实测结果（2026-10-21，合 main 后）
- 正常主机权限下的六项检查全部 exit 0：pytest 21 passed；ruff 通过；mypy --strict 检查18个文件无问题；pnpm lint、typecheck 通过；Vitest 29个文件、443项通过。
- 完整命令、UTC 起止时间和退出码：`full-permissions/local-checks.json`；逐项原始日志在同目录。
- 首轮受限沙箱执行为12 passed、9个setup errors，均因无法访问既有 pytest 临时目录；保留原始 `pytest.log` 与 `local-checks.json`。以相同命令在正常权限环境重跑后全部通过，没有改测试、阈值或实现。
- 相对 Claude 已审提交 a78fe57，合并提交只带入 #46 的两处文档变化；实现与验收判据未改。新提交仅补充本次验收记录。

## 远端验收与最终操作
推送本补充报告后，等待同一最终提交上的 Ubuntu、Windows CI 成功，再核对 main 与审查结论并 squash 合并。最终 CI 和合并结果以 GitHub PR #43 的检查和合并事件为准；不取下一张卡。
