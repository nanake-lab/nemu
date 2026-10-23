# P5-22 阻塞已解除

## 当前状态

**已解除（2026-10-23）。** 原 Windows runner 离线仅是当时的验收阻塞；最终 [Windows CI 39928691322](https://github.com/nanake-lab/nemu/actions/runs/39928691322) 已 success，[Ubuntu CI 39910674806](https://github.com/nanake-lab/nemu/actions/runs/39910674806) 亦 success，二者受测 head 均为 `a57c8b13e774a4470f0e1529c200d9fce6567323`。不能将此解除解释为 runner 当前在线状态的保证。

A1 已经用户另开的 Codex 新会话[复审通过](https://github.com/nanake-lab/nemu/pull/57#issuecomment-6360980179)，按 2026-10-23 规则代替 Claude 审查要求。用户于 2026-10-23 合并 [PR #57](https://github.com/nanake-lab/nemu/pull/57)，合并提交 `b6eb8721b420b4aa168e0c27789f63307959f84c`；合并即批准契约、主动边界候选和 V2-51–54。本卡无待处理的合并阻塞。

## 原阻塞及证据

修复提交为 `fc5f6da`，其后的 `a57c8b1` 仅补报告。当时 runner API 返回 offline，早期 Windows 作业排队，用户暂时无法启动，因此未将排队记作通过。最终通过的 Windows run 是上述 39928691322，而不是早期排队/取消作业。

原始验收、失败日志及修复记录均保留，见 REPORT.md、REVISION-2.md、revision-2-final-checks.json。本次只更新报告，不改代码、不删除或覆盖原始证据。
