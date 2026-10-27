# 按实际验证基线判断 main 同步 — 报告

## 当前状态

已落实用户 2026-10-27 的决定，仅修改流程文档。提交 PR 后停止，等待用户在主会话判断；不自行合并、不派发后续任务、不操作其他 PR。

## 交付物

- `AGENTS.md` 第 2、6 节：取消无条件同步；从最近一次实际验证使用的 main 基线判断增量影响，保留必要检查、审查、冲突处理及不确定性报告要求。
- `tasks/README.md`：删除「没有落后 main」这一合并前提。
- `.github/pull_request_template.md`：替换「latest main, tests rerun」的旧要求。
- 本报告：文档核对、GitHub 设置查询和 CI 范围核对证据。

## 验收

文档检查基于 `origin/main@3f8f98e5ac3cba2f2b4718ca242a5a0c226d0fbf`；此次只有 Markdown 改动，未运行构建、产品测试、CI 或 Windows 测试，也未手动触发工作流。

| 文档情景核对 | 结果与依据 |
| --- | --- |
| main 只改无关文档、报告或代理配置 | 第 6 节第 3 项：确认不影响构建、运行和测试后无需同步，不因此重跑 CI/Windows；仍须读取最新规则 |
| main 改相关源码、依赖或 CI | 第 6 节第 4 项：同步 main，运行受影响检查验证新组合；契约、生成器、工具链和测试同样适用 |
| 存在合并冲突，包括纯文档冲突 | 必须先同步解决，再按实际改动检查；不允许直接合并 |
| 最后一次 main 提交是文档，但此前存在未验证的相关源码改动 | 从实际基线 B 到 M 检查全部新增变化，不能只看最后一次提交 |
| 多项检查使用不同基线；上次仅核对无影响 | 分别核对各检查的实际基线；无影响核对不能将 B 推进到 M |
| 基线不明或无法确认相关性 | 明确报告不确定点，不作为无影响放行 |
| 文档一致性与 diff | 已检索现行流程说明并逐条核对；历史报告、追溯记录及 ADR 0016 中针对实际契约依赖的同步要求保持原样；`git diff --check` 通过 |

## GitHub 设置（2026-10-27 只读查询）

仓库 `nanake-lab/nemu`，使用已认证的 `gh api`：

| GET 路径 | 实际返回 |
| --- | --- |
| `repos/nanake-lab/nemu/branches/main/protection` | HTTP 404，`Branch not protected` |
| `repos/nanake-lab/nemu/rulesets?includes_parents=true` | HTTP 200，`[]` |
| `repos/nanake-lab/nemu/rules/branches/main` | HTTP 200，`[]` |

当前未发现强制分支最新的保护规则或 ruleset，无需用户修改 GitHub 设置。未更改任何仓库、用户或系统设置。首次查询因网络 EOF 失败，随后仅在查询进程内设置 `GODEBUG=http2client=0` 后成功，未持久化环境配置。

## CI 核对

- `ci.yml` 的 PR 与 main push 路径过滤已排除 Markdown、docs、tasks、reports 等；检查范围另按 diff 判定。
- `windows.yml` 按路径与事件触发；同步代码 PR 会改变 head 并产生 `synchronize` 事件，可能再次执行必要的 Windows 检查。免去无影响同步即可消除本次问题，无需修改工作流。
- `docs-guard.yml` 只检查冻结原文、任务目录 JSON 及自身工作流的改动；此次不涉及这些路径。
- 不把路径过滤当作语义无影响的证明；当前 head 的必要检查、所需审查、runner 安全限制和验收标准保持不变。

## 新增依赖

无。

## 待用户确认

无新增阻塞决定；用户已授权此流程文档修订。PR 由用户后续判断，本次不合并。

## 后续建议（不在本任务范围内）

非 Markdown 代理配置自身可能触发 CI；代码 PR 后续仅提交报告也可能产生新的检查事件。这些触发范围问题不在本次处理范围内，未修改工作流或缓存。
