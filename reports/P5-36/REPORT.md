# P5-36 θ 估计器状态的存储 — 报告

## 当前状态

部分完成：2026-10-27 从 `origin/main` 的 `3f8f98e5ac3cba2f2b4718ca242a5a0c226d0fbf` 建立 `task/P5-36-theta-estimator-idl`，先以草稿 PR 认领。已完整读取 AGENTS.md、ARCHITECTURE.md、CONTRACTS.md、Echo Core README 和本卡；继续完成规格原文及本卡必读、状态字段核对和基线检查。契约目录尚被未合并 PR 占用，见 [BLOCKED.md](BLOCKED.md)。本轮不取下一卡。

## 交付物

- 本目录：认领记录、阻塞和后续核验材料。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| git fetch origin | 通过；主线为 3f8f98e5 | 本报告基线 |
| generate.py --check / test:core-contracts / check_compat.py | 待运行 | 后续追加 |
| P5-25 内存状态与新消息逐位往返 | not_run；尚未实现新消息 | BLOCKED.md |

## 规格对照

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| AGENTS §2 目录互斥 | 只新增 reports/P5-36/；不修改占用目录 | PR #95、#65 状态及文件核对 |
| P5-36 / V2-98 | θ 估计器状态后继契约尚未实施 | not_run |

## 新增依赖

无。

## 待用户确认

- [ ] （阻塞）契约目录释放后才能实施；最终 ADR/契约仍需用户批准与独立交叉审查。

## 后续建议（不在本任务范围内）

无；继续本卡不受阻塞的阅读与证据整理。
