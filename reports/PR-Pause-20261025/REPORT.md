# PR 暂停收尾 — 2026-10-25

## 当前状态

已逐个发布暂停说明、创建 paused 标签并添加到 10 个 PR。全部仍开放、为草稿，保留 needs-user；未关闭或合并任何 PR，未继续原任务或领取新卡。看板修改已完成，等待本独立 PR 的 Ubuntu CI 和用户决定是否合并；正式看板尚未切换至本分支。

## 暂停处理

| PR / 暂停说明 | 原因 | 核验状态 |
| --- | --- | --- |
| [#84](https://github.com/nanake-lab/nemu/pull/84#issuecomment-6397014000) | 纳入 Echo Core 定义工作 | open / draft / paused / needs-user |
| [#82](https://github.com/nanake-lab/nemu/pull/82#issuecomment-6397012730) | 纳入 Echo Core 定义工作 | open / draft / paused / needs-user |
| [#81](https://github.com/nanake-lab/nemu/pull/81#issuecomment-6397012279) | 纳入 Echo Core 定义工作 | open / draft / paused / needs-user |
| [#78](https://github.com/nanake-lab/nemu/pull/78#issuecomment-6397011824) | 纳入 Echo Core 定义工作 | open / draft / paused / needs-user |
| [#77](https://github.com/nanake-lab/nemu/pull/77#issuecomment-6397011393) | 纳入 Echo Core 定义工作 | open / draft / paused / needs-user |
| [#72](https://github.com/nanake-lab/nemu/pull/72#issuecomment-6397011016) | 纳入 Echo Core 定义工作 | open / draft / paused / needs-user |
| [#65](https://github.com/nanake-lab/nemu/pull/65#issuecomment-6397013151) | 纳入 Echo Core 定义工作 | open / draft / paused / needs-user |
| [#64](https://github.com/nanake-lab/nemu/pull/64#issuecomment-6397013582) | 纳入 Echo Core 定义工作 | open / draft / paused / needs-user |
| [#49](https://github.com/nanake-lab/nemu/pull/49#issuecomment-6397010561) | 纳入 Echo Core 定义工作 | open / draft / paused / needs-user |
| [#45](https://github.com/nanake-lab/nemu/pull/45#issuecomment-6397014450) | 等待声音模型选型 | open / draft / paused / needs-user |

#49 的说明明确写入用户决定：**V2-38–47 均不通过，PR 不放行，禁止继续执行原任务。** 技术验证不能替代该决定。原本 ready 的 #49、#84 转回草稿，其余保持草稿；源分支提交均未修改。完整 head、标签与评论链接见 prs.json。

## 看板变化

- open PR 的 paused 优先于 needs-user、draft 和 review。保留现有 catalog 显式覆盖规则。
- paused PR 不产生决定、合并、审查或 Windows runner 提醒，也不再查询其审查和 runner 状态。未暂停的其他 PR 提醒照常保留。
- 6 张关联卡 P4-02、P5-03、P5-04、P5-23、P5-25、P5-34 显示「暂停」，不进入进行中或可开工列表；其下游依赖不解锁。
- 新增「暂停的 PR」列表，完整显示 10 个 PR，包括 4 个没有现存任务卡关联的文档/研究 PR；全部任务卡增加「暂停」筛选。
- 同批标签、草稿状态下，旧逻辑生成 before.json，新逻辑生成 after.json：6 张 needs_you 卡变为 6 张 paused；10 个 PR 的提醒降为 0。已完成数保持 33；没有把暂停当完成。

## 验收

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| 远端 PR 状态与评论 | 10/10 open、draft、paused、needs-user；无 auto_merge | prs.json；逐项评论链接 |
| python -m unittest discover -s tools/status -p 'test_*.py' -v | 7 passed | tools/status/test_build_status.py；Ubuntu CI 新增同命令 |
| 当前 GitHub 数据新旧逻辑计算 | 6 张卡暂停；10 个暂停 PR 无提醒；依赖与完成数保持 | before.json、after.json |
| Prettier board.html / ci.yml；git diff --check | 通过 | 本机静态检查 |
| Ubuntu CI | 提交后核验，结果见本 PR Checks | 只运行 ci.yml 的 ubuntu-latest 作业 |
| Windows 自托管 Runner | 不触发 | windows.yml 显式排除 tools/status/**；本 PR 其他改动仅报告与 ci.yml，均不匹配其路径过滤 |

## 新增依赖

无。测试使用 Python 标准库；CI 仅在现有 Ubuntu 作业内追加看板回归步骤，没有修改 Windows 工作流或系统设置。

## 待用户确认

- 本独立 PR 是否合并；按本次要求不自动合并。
- 9 个 Echo Core PR 的定义工作、后续范围及何时恢复，由用户另行决定；本次不批准任何定义、ADR 或旧验收条目。
- #45 等待声音模型选型及恢复指示。

## 后续建议

本 PR 合并后，已有本地看板服务需重新启动才能载入新的 Python 逻辑。本轮未替换或停止用户现有看板进程。
