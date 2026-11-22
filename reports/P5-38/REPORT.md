重档。本卡修改 Echo Core 权威事务、存储格式、契约校验与恢复。

# P5-38 念头的原子发布、接续与恢复

临时库可以在两条外部输入之间保存一次本地或云端产生的念头，登记工作状态和下一轮输入。重启后可以读回当时实际采纳的文字、来源和行动，迟返只进入审计。请求 JSON 按键名拒收传输头和凭据字段，正文中的凭据用语可按原字节保存。P5-38 验收存储、格式、发布与恢复。#240 已合入，Windows W18 的 47 项默认并行测试全部通过。[用户批准的拆分](https://github.com/nanake-lab/nemu/pull/238#issuecomment-7043361071) · [验收拆分与历史阻塞](BLOCKED.md)

## 当前状态

本地与云端各比较不中断、SQL 提交前失败、SQL 已提交后未收到成功通知三种运行。比较对象为实际发布的 V4 事务、Q 与重新取得的下一轮输入的 CanonicalV1 字节。捕获的引用包含 H、x、R、θ、L、U 与在途行动。复验为本机合成数据库，未接真实模型、API、用户排练库或正式库。物理断电与产品级 COG-A12 尚未验收。

## 交付与规格对照

| 位置 | 依据 | 新读法 |
| --- | --- | --- |
| core-runtime/src/cognition、core-store/src/activity.rs | ADR 0045 §4、0046 §2.1 | 先核对原 Q、请求、来源和前沿。采纳时把 RecordV2、CommitV2、x、Q、后继、已登记的本地行动随同一笔 V4 提交。重启从已登记的输入继续。 |
| core-store/src/activity_audit.rs | ADR 0045 D1、D8 | 空返回、拒收、迟返和无效采纳按原回执留独立审计。重试复用原审计，收到旧请求不能替换当前 Q。D8 校验拿到回执、留存的审核证据和精确的候选文字。 |
| core-store/src/raw.rs、objects.rs | ADR 0046 §2.1，PR #238 P1 | 实发 JSON 请求体的各层键均受检查，包括数组内对象及被重复成员遮盖的键。正文和返回原文按字节保存。独立字节资源按类型及摘要定位，读取时验 SHA-256、原长和预算。选区须为有效 UTF-8 标量边界，产品字节必须与采纳段一致。 |
| core-runtime/src/store.rs、core-store/src/transaction.rs、manifest.rs | ADR 0046 §4 | 新外部事件直写 V4.event_binding；历史 V2 按原标签解码。内部提交有独立位置且保留 D 游标。 |
| gateway/src/progress.rs:64 | 用户批准的网关回执读点 | V2 和 V4 事件沿用原有可见性、提交身份和事件绑定核验。V4 活动与未知标签均在事件读取边界被拒。 |
| fact-view/src/index.rs:106、retrieval.rs:125，core-runtime/src/cognition、task-layer/src/cognition | V2-88，用户确认 W19 范围 | W19 按原快照身份判断索引有效性。运行时发起查询，任务层接收已配对的原文、反应标记、显示文本、来源和两个时间。在本地材料预算下整项裁剪。ADR 0042 的远端组包还要决定哪些来源可外发，两者的资格和计量各有用途。 |
| core-types/src/semantic/cognition.rs、Python/TypeScript 模板及生成产物 | ADR 0046 §4，用户确认两个字段及「契约变更」标签 | latest_step 与 origin_record 接受新旧记录标签，错误的提交标签继续拒绝。IDL、既有标签与原有黄金文件在本 PR 中无差异。 |

两处超过 400 行。core-store/src/store.rs 保留原有的权威存储入口和能力边界，新增的活动、原始资源与审计各有模块。core-runtime/tests/store.rs 共用原来的临时库引导；本卡的回执、故障轨迹和 W19 组合测试放在 cognition 的测试文件中。

完整的卡外路径、各处差异行号和解释见[读端与卡外改动表](READERS.md)。[规格索引](../../docs/traceability/P5-38.md)

## 合入 #240 后的验收

合并提交 `8d439f66` 的两个父提交为原 PR 头 `517aea0d` 与 main 的 `8ed77319`。三个入口及相关核验位置见[读端对照](READERS.md)。P1 已获复审通过。P2 的审计原因用词留给 P5-39 接入调度时处理。[复审原文](https://github.com/nanake-lab/nemu/pull/238#issuecomment-7058296504)

| Windows 本机检查 | 结果 |
| --- | --- |
| `cargo test --workspace --locked --no-fail-fast`，默认并行 | 559 通过，0 失败。1 个原有的 V2-93 旧 Python 舍入诊断标为 ignored。 |
| `cargo test --locked -p gateway`，默认并行 | 47/47，包括 SYS01 重连。 |
| W19 实际查询组合 | 1/1，内部提交后可读取历史反应及来源。 |
| `cargo fmt --all -- --check` 和全工作区全目标 Clippy（拒绝警告） | 均通过。 |
| 两种历史长度的思考轮回归 | 每个临时库预热后连续跑三轮。1 个历史事件时为 3.451、4.012、4.874 秒，24 个时为 3.191、3.630、4.000 秒。长历史与短历史的中位数比为 0.905，低于回归上限 3。 |

计时覆盖读取登记输入、构造合成回执与状态、发布念头以及核验新提交。样本比较的是两种外部事件历史下的热路径。冷启动仍从头核验，增量核验也保留 SQLite 完整性检查。上述数据只描述实测规模。[原始记录与退出码](EVIDENCE.md) · [Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/44886834915) · [手动 Windows 复验](https://github.com/nanake-lab/nemu/actions/runs/44889251746)

## 先前提交的验收

P1 复验使用合入 main `2954da2b` 后的代码提交 `49d45a90`，在 Windows 上使用锁定的 Rust 1.99.0 和临时库。下表保留以前提交的结果和已知失败。[证据索引](EVIDENCE.md)

- `cargo test --locked -p core-store -p core-runtime` 通过 101 项，包括存储 62 项、运行时 34 项和文档测试 5 项。P1 用例逐字节读回含 `password:`、`cookie:`、`Bearer xyz` 和 `sk-abcdefgh12` 的请求正文与返回原文。带受限键的请求在写入对象前被拒，原有摘要、长度、预算反例均通过。
- `python core/crates/core-runtime/src/cognition/tests/run_w19.py` 通过 1 项，覆盖显式组装的 W19 临时库调用。
- `cargo fmt --all -- --check` 和 `cargo clippy --workspace --all-targets --locked -- -D warnings` 均通过。

| 命令或检查 | 结果 | 覆盖范围 |
| --- | --- | --- |
| `pnpm test:core-contracts` | 通过 | Rust、Python、TypeScript 各 18,623 例。兼容性检查 0 项违规。两个引用字段另做新标签正反例。pnpm 预脚本报告共享依赖目录警告，契约脚本退出 0。 |
| 本地/云端故障和拒收回归 | 19845545，6/6 通过 | 两个 SQL 故障边界、幂等拒收、迟返重启、非法 UTF-8 与分割标量、云端控制字段、真实外部事件接在内部提交之后。 |
| SYS17 临时库 | fa5155ab 存储整包 61/61 通过；19845545 专项 2/2 通过 | 旧 V2 混合新 V4；格式 2→3、3→2 及提交前后故障。有 V4 或已确认审计时拒绝破坏性降级。 |
| W19 查询组合 | 19845545，1/1 通过 | 使用工作区真实 W19、core-runtime、task-layer；回归入口为 `python core/crates/core-runtime/src/cognition/tests/run_w19.py`。 |
| Rust fmt / Clippy | 通过 | 1.99.0；Clippy 使用工作区、全部目标、锁定依赖和拒绝警告。 |
| Windows 工作区默认并行 | 退出 101 | fa5155ab，545 通过、1 失败、1 个仓库原有忽略项。SYS01 重连回执的等待超过原有时限。 |
| Windows gateway 复验 | 串行 42/42；默认并行失败 | 19845545 默认并行两次为 40/42、41/42。未修改的 origin/main（469262e4）同命令两次为 41/42、40/42，同样复现了 service.rs:316 的回执超时和 adversarial.rs:214 的上传保留期断言。原有时限和断言均保留。 |
| Ubuntu 代码提交的 Node 及 Python 检查 | 通过 | 19845545 上 pnpm lint、typecheck、test、build 与四个 Python 项目检查成功。此项只计已经结束的步骤。 |
| COG-A12、D11-A、真实载体 | 转交后继卡，尚未验收 | 完整内部循环由 PLAN-LAW-SUCCESSION-01 的 37-1～37-5 后继卡和 P5-39 验收。COG-A10 仍由 P5-39 验收。 |

## 测试中发现并修复的问题

相同的远端拒收回执重试时，最初会额外写一条失效审计。现在先按不可变活动引用查原审计，测试检查重启后仍只有一条。原格式 2 的入口审计测试一度调用了要求格式 3 的新发布入口，现由测试专用历史写入器构造原 V2 提交，原有拒收、追加和回退断言继续执行。新增审计回执目录后，目录持久化测试覆盖原三个目录及新增第四个目录的故障边界。

此前的 Windows 自动检查针对草稿跳过 self-hosted 作业，汇总检查按仓库规则报失败。Ubuntu 的 Node 及 Python 步骤取自[本次代码提交的 CI](https://github.com/nanake-lab/nemu/actions/runs/44711048605)，Windows 的[草稿检查](https://github.com/nanake-lab/nemu/actions/runs/44711048804)保留原状态。最终 PR 提交的检查单独以 GitHub 显示为准。

## 依赖和锁文件

| 使用方 | 新增直接依赖 | 版本及许可证 |
| --- | --- | --- |
| core-runtime | task-layer | 仓库内部包 0.0.0，随项目使用 |
| core-store | sha2 | 已锁定 0.11.1，MIT OR Apache-2.0 |

`core/Cargo.lock` 的差异为两个依赖数组各新增一行，无删除、包版本或校验和变化。task-layer 的唯一运行时依赖为 core-types。运行时接收查询函数，实际产品尚无组合方。

## 限制与待确认

- P5-53 的可信解释接入、P5-73 的结果作用和 P5-75 的新规律运行时是未完成的上游接线。当前入口对持久谱系关闭内部发布，避免夹具给出的 x 进入权威历史。
- 已保存采纳区间、产品的 SourceMetadata、原请求及回执引用。后续组包需据真实外部来源决定 D2 自引资格。测试中的本地载体回执、云端文字、D8 判定均为合成材料。
- 卡面旧版的 κⁿ 和“排队跨过 τ 就不推动 x”已由 ADR 0032、0035 取代。这里没有给出“0 个超期结果推动 x”的测量值，行动执行凭证的到期判断仍独立适用。
- P5-39 承接 W19 产品调用方，并在持钥适配层验证发送前和入库前的字节不含密钥原值。本卡保留 W19 临时库组合回归。
- 合入 #240 后，Windows W18 已在本机通过。PR 保持草稿，等待交叉审查。
