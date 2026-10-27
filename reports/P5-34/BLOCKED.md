# P5-34 门控与已解决问题（2026-10-27，PR #95）

## B1 已解决：V2 新写入冲突与范围

用户批准的分阶段实现：V3 新写、V2 历史只读、旧 V2 head 明确拒写；本次不迁移、不填默认 q/V。将来需要时由 P5-26 按 v2.0 §2.2 从 H 运行 G，不另开迁移卡。受保护文件及必要性见 PROTECTED-SCOPE-20261026.md；完整 G/H、AC2-08/19/21、I23、SYS17 未由本卡签发。

## B2 已核验：supervisor 原失败保留

旧 Windows pnpm test 的 579/1 失败保留；固定 main b4eb7fd 三次与当时本分支均 580/580，不能声称 main 同样失败。十个相关 Git blob 一致，运行依赖不调用本卡 IDL/Rust 存储，根因未定位；未改 supervisor、阈值或设置。对照与原始失败路径见 REPORT.md。

## B3 CI 已通过；等待独立复审与用户合并

本轮已 merge main 3f8f98e5。修复源码 af200a67 的 [Ubuntu 40443460541](https://github.com/nanake-lab/nemu/actions/runs/40443460541) 与 [Windows 40443545530](https://github.com/nanake-lab/nemu/actions/runs/40443545530) 均实际成功；W02 35 项测试及 2 项文档测试通过，0 失败、0 忽略。归档见 verification/review-20261027/ci-af200a67.json 和两份原始 job 日志。PR 已按用户批准的 ready→W02 顺序转为可审查；报告补充提交的最终 head 仍单独核对[当前检查](https://github.com/nanake-lab/nemu/pull/95/checks)，具体 head/run 固定在 PR 描述及最新回复。旧成功不代替新 head，跳过/取消不算通过。

core-store 与契约、ADR 0021、V2-62/63 一并交叉复审；保留 needs-user，由用户合并，不自行合并或领取下一卡。

## B4 已解除：runner 离线及旧 20 分钟时限

2026-10-26 的 runner 离线、草稿守卫失败与排队快照保留在 verification/protected-20261026/。随后 acf73c3 的 W02 因 20 分钟作业时限取消，原日志/制品见 [失败记录](https://github.com/nanake-lab/nemu/pull/95#issuecomment-6418525763)。#106 将时限改为 60 分钟后，d047b417 的 [Ubuntu 40358933290](https://github.com/nanake-lab/nemu/actions/runs/40358933290) 与 [Windows 40358965023](https://github.com/nanake-lab/nemu/actions/runs/40358965023) 实际通过，W02 为 34 项测试及 2 项文档测试通过。本轮不再把 runner 离线列为当前阻塞，未改 runner 或系统设置。

## B5 已修复并验证，待复审：旧 V2 非空 WAL 拒写

[审查](https://github.com/nanake-lab/nemu/pull/95#issuecomment-6432647798) 发现拒绝前的读写连接关闭会检查点并清理 WAL。新增仓库回归在旧实现上失败：DB 4096→159744，WAL 284312→0。改为持有原 writer 锁的只读预检后，本地四项边界测试通过；完整 W02 对平面/分段 V2 分别确认 DB 4096→4096、WAL 284312→284312，原字节均相同。旧测试和阈值未改。原始失败在 attempt-04/storage.log，修复后在 attempt-05/storage.log（均位于 verification/protected-20261026/）；审查方原日志、输入哈希与实际 CI 日志在 verification/review-20261027/。独立复审仍按 B3 执行。
