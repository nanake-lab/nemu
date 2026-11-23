### Q2　DOC-AGENTS-NOWAIT-01（6.1 Sol 中）· 条件：无，有空位就派

> **投入：** token 预算无上限，不要吝惜 token。该读的文件读全，该测的情况测全，证据给全。不要为了省 token 跳过步骤或简化验收。
>
> 分支 `task/DOC-AGENTS-NOWAIT-01`，从 `origin/main` 新建。只改 `AGENTS.md`：在 §3 黄金规则末尾追加一条：
>
> > **不等待、不轮询。** 执行者不为等别的 PR 合并、等 CI 或等用户回复而让会话一直跑。前提不满足时，在 PR 评论里写明在等什么，然后停下；条件满足后，由派发方另开会话接续。CI 也一样：推送后不守着看，下次接续时再读结果。
>
> 把 `tasks/requests/DOC-AGENTS-NOWAIT-01.md` 写成本卡原文。`git diff --check` 通过。开 PR，标题 `docs(DOC-AGENTS-NOWAIT-01): 执行者不等待`，加 `needs-user`，写「请审查」后停下。

所属路线：交付线。
