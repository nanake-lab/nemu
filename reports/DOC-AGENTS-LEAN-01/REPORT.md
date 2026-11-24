中档：改动了 `tools/status/` 看板脚本与测试，看板部分按中档运行测试与静态检查；规则文档部分按轻档完成链接与差异核对。

做了什么：按用户决定更新 `AGENTS.md` 和报告模板（PR 拆小、普通代码卡用 PR 描述代替 `REPORT.md`、派卡预授权、固定工作区、投入不设上限与分工说明），并更新看板脚本支持 `<任务号> n/m` 拆分 PR 判定、标题编号异常标注及无 `REPORT.md` 时的回退。
对她或对项目意味着什么：开发与审查更简洁，执行者可在不碰契约/第三方依赖/验收/她是谁的红线内自主推进，且拆成多个小 PR 后看板不会提前误判完成，也不会漏报写错的编号。
需要用户做什么：审查并决定是否合并本草稿 PR（已加 `needs-user`）；不改旧卡报告。

# DOC-AGENTS-LEAN-01 — 报告

## 当前状态

全部文档修改、看板实现与本机检查已完成，未发现阻塞项；PR 保持草稿状态并标 `needs-user`，等待用户审查。基线 `origin/main`：`8ed77319`（2026-11-21）。

## 交付物与逐处改动

- [AGENTS.md](../../AGENTS.md)：规则更新（6 处）。
- [report-template.md](../../docs/agents/report-template.md)：明确 `REPORT.md` 模板适用范围，新增「代码卡 PR 描述」短模板。
- [DOC-AGENTS-LEAN-01.md](../../tasks/requests/DOC-AGENTS-LEAN-01.md)：逐字保存本卡原文与用户决定原话。
- [build_status.py](../../tools/status/build_status.py)：解析 `<任务号> n/m`（含合法分片、末片、旧式标题及非法编号异常标注）。该文件改动前已为 530 行，本次仅在其既有状态计算流程上小幅扩展至 625 行，未另拆文件。
- [delivery.py](../../tools/status/delivery.py)：无 `REPORT.md` 时允许 `None`/缺失输入，仅按 `BLOCKED.md` 判定部分交付且不报错。
- [test_build_status.py](../../tools/status/test_build_status.py)、[test_delivery.py](../../tools/status/test_delivery.py)：覆盖旧式标题、`1/4` 已合并、`4/4` 已合并、异常编号（`5/4`、`0/3`、`1/0`、`-1/3`、`1/a`）、开放态异常提醒及无 `REPORT.md` 场景。
- 未改动 [design-research-cards.md](../../docs/agents/design-research-cards.md)（无冲突内容，不需要修改）。

| 位置 | 改前要点 | 改后要点 |
| --- | --- | --- |
| `AGENTS.md` §2 认领后 | 仅说明单 PR 认领格式 `<类型>(<任务号>): <标题>` | 追加「PR 拆小」：支持拆为多个可单独验证的小 PR，标题写 `<类型>(<任务号> <n>/<m>): <标题>`，首个草稿 PR 为认领，末个 PR（`n = m`）合并才算完成 |
| `AGENTS.md` §2 工作区 | 每张卡用 `git worktree add` 新建 `D:\Dev\nemu-wt\<任务号>` | 整段替换为固定槽位 `D:\Dev\nemu-wt\<槽名>`（如 `codex-a`、`gemini-a`），切卡前确认无未推送/未提交改动，从 `origin/main` 切新分支 |
| `AGENTS.md` §2 分工 | 默认 Codex；卡上写 Claude 或 Antigravity 均读作 Codex | 默认 Codex，也可由用户派给 Gemini（Antigravity）；写 Claude 读作 Codex，写 Antigravity 由用户决定派给 Gemini 或 Codex |
| `AGENTS.md` §3 第 13 条后 | 遇到卡外目录/依赖/并行等事项一律按第 13 条停下问 | 追加「派卡预授权」：满足四条红线（不改 `.proto`、不加新第三方包、不放宽阈值/验收、不涉及她是谁）的实现层事项可自主决定并在 PR 描述写「决定 / 理由」 |
| `AGENTS.md` §3 第 18 条 | 共 17 条黄金规则 | 追加第 18 条「投入不设上限」：不省 token 与时间，不为省 token 跳过步骤、缩小语料或简化验收 |
| `AGENTS.md` §6 完成标准、证据与报告 | 所有卡统一要求写 `reports/<任务号>/REPORT.md` | 仅设计/研究/演示与实验卡（`DESIGN-`、`R-`、`DEMO-`、`MATH-`、`PLAN-`）及含正式验收/GPU 测量的卡写 `REPORT.md`；普通代码卡不写 REPORT，用 PR 描述（白话三行 + 不超过十行正文）并可内附证据索引 |
| `docs/agents/report-template.md` | 仅含通用 `REPORT.md` 模板 | 开头注明 `REPORT.md` 模板仅用于 AGENTS.md 第 6 节列出的写报告卡，并补充「代码卡 PR 描述」短模板 |
| `tools/status/build_status.py` | 仅识别 `(<任务号>)`，任意已合并 PR 均按完成/报告部分交付处理 | 解析 `(<任务号> n/m)` 与旧式标题：有 `n = m` 或旧式标题可算 `done`（保留 `unverified` 与部分交付规则）；仅合并 `n < m` 算 `partial` 且等待栏写「剩余 PR」；`5/4`、`0/3` 等写错标题不算完成，在看板上标为 `needs_you` / 状态栏「标题异常」/ 等待栏「标题异常」并加入 `attention` |
| `tools/status/delivery.py` | 假定 `report` 为 `str`，未显式注明无 `REPORT.md` 场景 | 接受 `report: str \| None` 与 `blocked: str \| None`，无 `REPORT.md` 时仅依据 `BLOCKED.md` 判定且不报错 |

## 验收与检查

环境：Windows，Python 3.12；`pytest` / `ruff` / `mypy` 复用 `D:\Dev\cache\DOC-AGENTS-LEAN-01\` 虚拟环境与缓存，未在 C 盘建新缓存。

| 命令或检查 | 结果 |
| --- | --- |
| `D:\Dev\cache\DOC-AGENTS-LEAN-01\venv\Scripts\python.exe -m pytest -o cache_dir=D:\Dev\cache\DOC-AGENTS-LEAN-01\pytest tools/status -v` | 25 passed, 29 subtests passed |
| `python -m unittest discover -s tools/status -p 'test_*.py' -v` | 25 tests OK |
| `uv run --project workers/irl ruff check tools/status`（`UV_PROJECT_ENVIRONMENT` 与 `RUFF_CACHE_DIR` 均指向 `D:\Dev\cache\DOC-AGENTS-LEAN-01\`） | All checks passed |
| `uv run --project workers/irl mypy --strict tools/status`（`MYPY_CACHE_DIR` 指向 `D:\Dev\cache\DOC-AGENTS-LEAN-01\mypy`） | Success: no issues found in 6 source files |
| `git diff --check` | 通过（无空白/冲突标记错误） |
| 新增与改动 Markdown 相对链接核对（`AGENTS.md`、`docs/agents/report-template.md`、`tasks/requests/DOC-AGENTS-LEAN-01.md`、`reports/DOC-AGENTS-LEAN-01/REPORT.md`） | 全部目标文件存在 |

## 失败、阻塞与未完成项

- 联机试跑 `python tools/status/build_status.py` 查询 GitHub API 时遇到远端网络 `EOF`，未纳入本机验收；看板核心逻辑与回退路径已由 25 项离线单元测试覆盖。
- 无阻塞项，无新增依赖。

## 待用户确认

- [ ] （阻塞 / `needs-user`）确认 `AGENTS.md`、`docs/agents/report-template.md` 规则更新及看板异常标题标注方式（状态 `needs_you`、状态栏与等待栏显示「标题异常」并在「等你处理」区提示异常 PR），审查后由用户合并。
