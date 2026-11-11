# P5-37 验收阻塞状态（2026-11-11 更新）

Windows runner 离线／未实测的问题已解除。按原可选后续 1 手动运行 `gh workflow run windows.yml --ref task/P5-37-cognition-idl`；[run 42963248080 / attempt 2](https://github.com/nanake-lab/nemu/actions/runs/42963248080/attempts/2) 完整工作流成功，actual checkout 为 `86acb718288bf48ddd50f65dddb5ff294fed60c7`。

- attempt 1：桌面四项、W02 与 W18 测试及 clippy 成功；W18 artifact 上传 `ECONNRESET`，self-hosted 与最终门禁失败。失败证据保留，不记为通过。
- 保存失败并定位上传故障后，重跑失败作业一次；attempt 2：桌面四项、W02 51 单元 / 3 文档测试、W18 42 测试及 clippy、三个 artifact 上传和最终门禁全部成功。
- 历史 draft 自动运行的 skipped／门禁失败仍为历史失败，不用它替代本次实测。报告收尾仅改 reports/P5-37/，被测代码、工作流和依赖未变。

完整两次结果、checkout、日志、artifact 与隐私脱敏说明见 [报告](REPORT.md) 和 [EVIDENCE-INDEX](EVIDENCE-INDEX.md)。原 artifact 的隐藏文件缺项已如实记录；原桌面 ZIP 本地保留，私有归档为附前后哈希的脱敏副本。

尚待：请[原审查方](https://github.com/nanake-lab/nemu/pull/155#issuecomment-6787476764)确认 Windows 一项，并完成独立交叉审查；ADR 0030 / V2-110–114（尤其 V2-113）仍待用户批准。PR 保持草稿、needs-user，不合并、不领取下一卡。持续认知真实时间、运行时 SYS17 等仍 not_run。
