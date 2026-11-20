# CONTRACT-WIRE-01　Windows CI

在准备提交本卡时，GitHub Actions 报告仓库唯一的 Windows self-hosted runner 为 offline、busy=false。本机契约验收已经完成，草稿 PR 的自动 Windows 作业会按仓库配置跳过。

按任务分支手动触发 Windows 工作流，完整覆盖 desktop、storage 和 gateway。请用 [Windows 作业列表](https://github.com/nanake-lab/nemu/actions/workflows/windows.yml?query=branch%3Atask%2FCONTRACT-WIRE-01) 对照 [PR #234](https://github.com/nanake-lab/nemu/pull/234) 的 head SHA。该提交的手动作业成功即解除本项；等待分配或跳过均保留为待验。

恢复方式是启动仓库 Windows runner 领取已经排队的作业。若本次排队作业过期，runner 恢复后对同一分支当前提交重新触发，保留先前运行记录。更换提交时须在新 head 上重新验证。
