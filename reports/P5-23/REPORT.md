# P5-23 规律引擎 G — 报告

## 结果

纯 G 的 V3 状态接入与 V2-80/105 容量处理及本机验收已完成。PR #49 已恢复；等待独立交叉审查，不自行合并。当前基线为 `origin/main` 的 `f24f1256259c998bc213a9dc93103fcfb9cfe89a`，通过 merge `551928db` 合入，没有 rebase 或强推。

## 交付物

- `core/crates/core-kernel/src/law/state.rs`：只重新导出 P5-34 的四种 V3 消息；删除 `pending.rs` 和测试专用包络。实际 q 与独立 V 直接使用生成字段，递推次序不变。
- `law/fuse.rs`、`engine.rs`、`input.rs`：保护最新 K 个候选，移出其余证据最少的候选；人格不移出。容量耗尽时已有倾向继续成长，只暂缓新候选进入 R 并返回待提交的通知，不返回容量故障或暂停成长。K 和 H 导出的候选次序显式传入，不保存隐藏队列。
- `law/tests/capacity_v3.rs`、`capacity_replay.rs`、`v3_storage.rs`：K=1/K=2、容量调整、同义候选、拒收与重新提出、冻结历史 q、真实 V3 编码恢复及继续计算。
- [卡外接线请求](../../tasks/requests/P5-23-capacity-inputs.md)：K 的权威配置、H 候选次序视图、提交后通知和 P5-26 的迁移边界。未修改契约、生成代码、存储或运行时。
- [规格对照追加](../../docs/traceability/P5-23.md)、[差异登记状态追加](../../docs/traceability/diff-register.md)：落实 V2-80 至84、105；不新增待确认规格读法。V2-81 翻转和 V2-82 关系实现保留。

## 验收

环境：Windows，锁定 Rust 1.99.0，Python 3.12.10。仅使用合成数据和测试临时存储；重负载串行。原 Python 参考脚本、种子、规模、阈值和黄金向量未改。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `cargo test -p core-kernel --locked v3 -- --nocapture --test-threads=1` | 11/11 通过；300次 V3 编码/前缀逐位比较，51次容量通知、77次移出 | `verification/resume-20261029/v3-02.*` |
| `cargo test --workspace --locked -- --nocapture --test-threads=1` | 通过（退出0，约22分钟）；core-kernel 169通过/1个原有忽略 | `verification/resume-20261029/workspace-01.*` |
| `cargo fmt --all -- --check` | 通过（退出0） | `verification/resume-20261029/fmt-01.*` |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 通过（退出0） | `verification/resume-20261029/clippy-01.*` |
| `pwsh -File reports/P5-23/verify_ac2_01.ps1 -EvidenceDirectory reports/P5-23/AC2-01/resume-20261029` | passed（退出0） | `AC2-01/resume-20261029/` |
| `pwsh -File reports/P5-23/verify_law_6.ps1 -EvidenceDirectory reports/P5-23/LAW-6/resume-20261029` | failed（退出1）；3,000条序列、3,064个违例，按V2-60保留 | `LAW-6/resume-20261029/` |
| `git diff --check` | 通过；46个 law 源文件哈希与测试时一致 | `verification/resume-20261029/summary.json`、`source-sha256.json` |

工作区原有忽略项为 `dynamics::tests::reference::theta_1_original_500_streams_zero_violations`，由 main 按 V2-93 标记；本卡未增删忽略项。它不是 P5-23 验收通过证据。工作区 stderr 的编译错误来自预期失败的类型隔离文档测试，最终测试命令退出0。

同输入对照：2,539,894个标量状态、261,581组份额、7类显式故障；13,339个成功事件、5个原子故障，全部状态/分配/cf零位差。完整日志、命令、时间和源文件哈希在 `verification/resume-20261029/`。收尾再次fetch，main仍为上述f24f1256，无新增基线差异。

首轮容量测试为5通过/1失败：测试刺激恰好使旧候选达到 π=0.5，未达到测试要求的人格区间。修正合成输入的 sigma 后，保持 `π>0.5` 及全部原断言，新增同义候选、两种编码和随机前缀后11/11通过。原始失败日志 `capacity-01.stdout.log` / `capacity-01.stderr.log` 保留，不覆盖、不计作通过。

## 规格对照

| 条款/验收 | 实现与范围 | 验证入口 |
| --- | --- | --- |
| ADR 0019 §3–8、ADR 0021；AC2-01/06/07 | 显式 V、首次生成 q、普通修订冻结 q；LP1 三条、反事实、严格有限 binary64 与精度故障 | `recurrence` / `replay` / `audit`；原 scalar/event 逐位对照、`finite_contract` |
| AC2-02/03/05、I25 | 两侧份额各精确为1，重复不再计数，无证据不随时间改变 | 原 `engine` / `group_props` / `law_props` / `cases` |
| V2-80/105、AC2-21 | 最新 K 个保护、证据最弱移出、人格保留；拒收新候选时已有证据照常更新 | `capacity_v3`、原容量与单组上限检查 |
| I23 | 同一 H 与边界事件分别增量/整段重做；每步原生 CanonicalV1/Protobuf 恢复后继续 G；D5 存储分配重放 | 原125个随机 H 前缀与新增300个编码/容量前缀比较 |
| I10/I18/I24 | G 只读 R、b/σ、事件证据、规律与显式边界；不读 x/m/g/λ，不增加隐藏状态 | 类型签名/源边界检查；生成 V3 消息 |
| AC2-13、推论1–7 | 保留原 LAW-1…10 的规模；原 LAW-6 按 V2-60 继续报告失败 | `law_props` 与独立 `verify_law_6.ps1` |
| ADR 0014、V2-81/82/83/84 | 同义归并；跳过零 S 翻转；关系/分摊读法已确认 | 原别名、关系与回归；新增满容量别名用例 |
| I03/I29 | 晚阶段数值失败不发布部分状态；只用合成输入与临时目录 | `finite_contract`、`adr0019`、`capacity_v3`、测试夹具 |

原 LAW-6 的诊断测试通过不等于该原判据通过。该保证变化已由 ADR 0019 / V2-60 采纳，不增加下限或容差掩盖失败。

## 新增依赖

无。未修改运行时依赖、IDL、生成器或黄金向量；所有本轮改动的 Rust 文件均不超过400行。

## 待用户确认

没有新增的阻塞确认项。用户已取消暂停，V2-83/84 解决旧确认项，P5-34/#95 解除 q/V 消息依赖。仍须独立交叉审查本 PR；执行者不自行签发“审查结论：通过”。

## 后续建议（不在本任务范围内）

按卡外请求接通 K/H 视图与通知。完整有效 H、来源更正及新规律的整段重算、V2→V3 迁移/回退由 P5-26 等所属卡接续。本报告不宣称生产通知、生产迁移、AC2-19 或 SYS17 完成；V2-82 的诚实性信任缺口仍为既定后续。

## 下一步

交付至 PR #49，转为可审查并写“请审查”后等待独立交叉审查；不合并、不取下一张卡。
