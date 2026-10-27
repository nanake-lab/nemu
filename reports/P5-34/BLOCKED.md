# P5-34 门控与已解决问题（2026-10-27，PR #95）

## B1 已解决：V2 新写入冲突与范围

用户批准的分阶段实现：V3 新写、V2 历史只读、旧 V2 head 明确拒写；本次不迁移、不填默认 q/V。将来需要时由 P5-26 按 v2.0 §2.2 从 H 运行 G，不另开迁移卡。受保护文件及必要性见 PROTECTED-SCOPE-20261026.md；完整 G/H、AC2-08/19/21、I23、SYS17 未由本卡签发。

## B2 已核验：supervisor 原失败保留

旧 Windows pnpm test 的 579/1 失败保留；固定 main b4eb7fd 三次与当时本分支均 580/580，不能声称 main 同样失败。十个相关 Git blob 一致，运行依赖不调用本卡 IDL/Rust 存储，根因未定位；未改 supervisor、阈值或设置。对照与原始失败路径见 REPORT.md。

## B3 源码 CI 已通过；最终门控未完成，保持草稿

本轮已 merge main 3f8f98e5。修复源码 af200a67 的 Ubuntu 40443460541 与 Windows 40443545530 均实际成功，W02 35+2；这份归档不替代后续提交的检查。报告提交 c6525e65 的 Ubuntu 40456116455 成功，Windows 40456116456 的 W02 35+2 通过，但随后制品上传失败，整个 Windows 工作流为 failure，详见 B6。PR #95 已按用户要求恢复草稿，保留 needs-user，尚未发布本轮“请审查”。最新 head 在 PR 描述中固定。

core-store 与契约、ADR 0021、V2-62/63 仍需一并交叉复审；最终 CI 全绿后由用户合并，不自行合并或领取下一卡。

## B4 已解除：runner 离线及旧 20 分钟时限

2026-10-26 的 runner 离线、草稿守卫失败与排队快照保留在 verification/protected-20261026/。随后 acf73c3 的 W02 因 20 分钟作业时限取消，原日志/制品见 [失败记录](https://github.com/nanake-lab/nemu/pull/95#issuecomment-6418525763)。#106 将时限改为 60 分钟后，d047b417 的 [Ubuntu 40358933290](https://github.com/nanake-lab/nemu/actions/runs/40358933290) 与 [Windows 40358965023](https://github.com/nanake-lab/nemu/actions/runs/40358965023) 实际通过，W02 为 34 项测试及 2 项文档测试通过。本轮不再把 runner 离线列为当前阻塞，未改 runner 或系统设置。

## B5 已修复并验证，待复审：旧 V2 非空 WAL 拒写

[审查](https://github.com/nanake-lab/nemu/pull/95#issuecomment-6432647798) 发现拒绝前的读写连接关闭会检查点并清理 WAL。新增仓库回归在旧实现上失败：DB 4096→159744，WAL 284312→0。改为持有原 writer 锁的只读预检后，本地四项边界测试通过；完整 W02 对平面/分段 V2 分别确认 DB 4096→4096、WAL 284312→284312，原字节均相同。旧测试和阈值未改。原始失败在 attempt-04/storage.log，修复后在 attempt-05/storage.log（均位于 verification/protected-20261026/）；审查方原日志、输入哈希与实际 CI 日志在 verification/review-20261027/。独立复审仍按 B3 执行。

## B6 当前阻塞：最终报告提交的 W02 制品上传失败

- 核验提交：c6525e650a7d0be7a1278e9586ae5c3b696b2592；源码与 af200a67 相同，之后仅 reports/P5-34/ 有改动。
- 实测：Ubuntu run 40456116455 成功；Windows run 40456116456 中 W02 35 项测试及 2 项文档测试全部通过，0 失败、0 忽略。两种 V2 布局的 DB 4096→4096、WAL 284312→284312，完整字节均保留。
- 失败：Windows job 122029713840 的第 20 步 actions/upload-artifact 在 06:09:39 UTC 开始上传，06:14:39 报 `Upload progress stalled`；随后输出 Uploaded bytes 4725 也没有改变 failure 结论。汇总 job 122043122361 因依赖失败而失败。不是 60 分钟超时或测试断言失败，具体网络根因未定位。
- 已做：恢复草稿；保存 runner 原字节日志与 SHA-256、GitHub 连接器取得的完整日志文本、逐步 CI 结论。gh 下载返回 EOF 的空文件和错误元数据保留。证据：verification/review-20261027/final-ci-c6525e65/。未改源码、测试、阈值、工作流或 runner 设置，未自动反复重跑完整测试。
- 选项 1：确认 runner 到 GitHub 制品服务的连接稳定后，按已批准的 ready→W02 顺序重新验证当前分支最终 head，全部成功后再回复“请审查”。
- 选项 2：维持草稿，由 CI 维护工作单独定位上传连接问题；本卡不放宽上传门禁或改验收设置。其他修复、报告和证据已完成。
