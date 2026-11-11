档位：重档，改动 Echo Core IDL、生成类型和跨语言语义检查。

# P5-42 J 资源与作用回执 — 报告

## 当前状态

部分完成：交叉审查 B1 已按用户指示修复，当前增量验收及原审查者复审见 [REVIEW-B1.md](REVIEW-B1.md)；下方 5e04340f 验收为初轮证据。PR #164 为 draft / needs-user；等待 ADR 0036 / V2-151 批准、独立交叉审查（见 BLOCKED.md）。按用户指示写「请审查」后停止，不领取下一卡。

## 交付物

- `core/contracts/j_resources.proto`：10 个新增消息、8 个枚举，方向集/验证记录/身体参数/作用回执；GenesisManifestV3 退出主动读出引用并保留旧 V2。
- 三语言 `j_resources` 语义模块：单对象、已解析资源、既有信封绑定检查；不签发资格、不发布生产信封。
- 生成类型、黄金向量、`baselines/p5-37/`、`verify_j.py`：接入正常 `test:core-contracts`，原 616 合法 / 860 非法向量不改字节；新增 30 合法 / 41 非法。
- `core-types/tests/j_numeric_roundtrip.rs`：调用 P5-12 纯数值实现，检查系数、保守总界、身体读出逐位往返与排除清单。
- ADR 0036、V2-151、`docs/traceability/P5-42.md`：字段来源、消费者义务、兼容/迁移/回退边界。

## 验收

本机最终源提交 `5e04340f407b904903133be70ed5ef22635aee19`，Windows / Python 3.12.10 / Node 24.21.0 / pnpm 12.11.2 / Rust 1.99.0。原始证据见 EVIDENCE.md，逐文件路径与 SHA-256 见 EVIDENCE-INDEX.csv。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | 通过，无生成漂移 | generate-check-final |
| `python core/contracts/check_compat.py` | 所有既有基线 0 违例；新增 p5-37 的 11 个 IDL 与原 Git blob 相同 | compat-final、baseline-source |
| `pnpm test:core-contracts`（D10.8 / SYS21） | 190 消息；每语言 16,773 用例通过；646 官方 protobuf 传输向量通过 | conformance-final |
| P5-42 跨资源与恢复 | 每语言 3 组合、15 资源拒绝、12 同版本恢复 / 12 跨版本拒绝通过；另含信封与保守舍入、β=0 身体正反例 | conformance-final / verify_j |
| `cargo test -p core-types --locked --offline` | 通过，含实际 P5-12 数值逐位往返 | core-types-final |
| `cargo fmt --all -- --check`；`cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 通过 | fmt-final、clippy-final |
| 契约 Ruff / `mypy --strict`（与 CI 范围一致） | 通过 | ruff-final、mypy-final |
| Node lint/typecheck/test/build；四个 Python 项目的 Ruff/mypy/pytest/build；Rust 工作区及独立 core-store | 通过：[Ubuntu CI 43044637186](https://github.com/nanake-lab/nemu/actions/runs/43044637186) | CI 原始记录与 PR 最终验收评论 |
| Windows 桌面/隔离/视觉、W02、W18 | 通过：[手动全量 43047851482](https://github.com/nanake-lab/nemu/actions/runs/43047851482) | CI 原始记录与 PR 最终验收评论 |

本机完整 `cargo test --workspace` 在主干 Windows CI 启动时中断，不计为通过；最终工作区通过结论依赖上述 CI，不用局部测试代替。

## 失败与未完成项

- 新冻结基线 growth.proto 保留源文件 EOF 空行，`git diff --check` 对该逐字节副本提示空行；已核对原 Git blob 一致，其余改动无空白错误，未为消除提示修改冻结文件。
- 初轮静态检查暴露 Python 类型标注/导入、Rust 单元素循环问题，已修正；首次冻结向量因 echo 导入路径失败，已修正显式路径。原始失败日志保留。
- 自查纠正了「注入系数全零 ⇒ 身体中性」的错误推断；普通模式身体值由外层依据快状态重算，只有无快状态排练强制中性，并加入三语言正反例。
- 早期草稿 Windows 汇总门禁因自托管步骤跳过而失败。随后 #157 隐私修复已合并并同步，当前源码提交的手动 Windows 全量验证通过。草稿汇总门禁的 skipped/failure 不伪称通过。
- main 自动启动 Windows CI，与本机完整工作区测试中途重叠；已停止本任务进程树并保留中断日志（exit 130），不算代码失败，也不宣称完整本机工作区通过。独立审查和用户批准仍未完成。
- 真实方向提取、三语能力无损、实际注入/缓存、已提交快照/原子发布、真实 genesis 迁移和 SYS17 均 not_run。所有夹具仅合成、未访问她的排练或正式存储；没有采用 R-STEER-01 研究结论或选定部署层位、hook、数值。

## 规格对照

见 [完整字段与验收对照](../../docs/traceability/P5-42.md) 与 [ADR 0036](../../docs/adr/0036-j-resources-and-effect-receipt.md)。外层需验证已提交快照、R⁺ 清单、数值重算、资格证据及已批准迁移链；接口通过不证明这些事实。

## 新增依赖

无新增运行时或外部依赖。仅给 core-types 添加仓库内 core-kernel dev-dependency（0.0.0，仓库内部代码，未新增第三方许可证），以调用真实 P5-12 数值实现。生成 schema/类型和黄金数据沿用机器生成的大文件，不手工拆分。

## main 增量核对

开工及早期本机测试基线 B=`6b9c9b0b136e6c3cd4483adf9e35a3298cf55093`。#155/P5-37 与 #101/P5-12 已合并且无 unverified；开工无其他开放契约 PR；ADR 最大 0035、差异登记最大 V2-150，故占用 0036/V2-151。

先核对 `f9c6ce74` 的 #162 状态登记，语义无新增影响。之后 M=`a6a0567f02efa5a183fa9fdb449c064afe11e661` 合入 #157，修改 Windows 证据脚本/工作流，已读操作性完整 diff 并 merge 为 `5e04340f407b904903133be70ed5ef22635aee19`；最终检查以这个组合运行。旧 B 的中断工作区结果不冒充新组合验证。再次 fetch 的 M=`88be8131d1240209da0a1c9428f2e0875c0e4380` 只增加独立研究/隐私审计报告和证据索引，无代码、契约、工作流或规范变化；相关运行/构建目录未引用这些报告，不影响本卡，未再 merge。

## 待用户确认

- [ ] （阻塞）ADR 0036 / V2-151 契约批准（用户合并）。
- [ ] （阻塞）独立高难度交叉审查。

## 后续建议（不在本卡范围内）

P5-08/19 接快照与发布；P5-13/31 接真实载体与验证；迁移接线在临时谱系通过 SYS17 后才可使用。字段约束不替代这些后继验收。

最终报告/证据索引提交只改 reports/P5-42/，相对被测源码提交无代码、生成物、依赖或工作流变化；按 AGENTS §6/9 核对文档收尾，不把不同提交号混称为同一次 checkout。Ubuntu 实际 checkout 为 e164c4af35541271c5e95e627008a49cd9b1a67e（父提交 a6a0567f、5e04340f），Windows 直接测试 5e04340f。

交叉审查补充的初轮证据边界：Windows 43047851482 的 e2e 为 3 passed / 2 skipped；click-through 50+50 与 voice-idle 60 秒没有实测，不将工作流成功写成这两项通过。B1 修复后的 Windows CI 因 runner 离线仍为 not_run，见 REVIEW-B1.md。
