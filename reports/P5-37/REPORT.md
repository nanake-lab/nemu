重档：改动 Echo Core IDL、core-types 及跨语言生成产物，涉及持久契约与后继事务。

# P5-37 持续认知记录与工作状态 — 报告

## 当前状态

实现与缓存测试入口修复已提交 `a0e0d7ba92547c64caa6b9d013180aae58f1e38e`，PR [#155](https://github.com/nanake-lab/nemu/pull/155) 保持草稿、needs-user。契约层实现与本机验证完成；Ubuntu CI 已通过，Windows 手动工作流 attempt 2 已全绿（首轮上传故障及完整记录见下文）；等待原审查方确认 Windows 一项（见 BLOCKED.md）。等待独立交叉审查及用户批准 ADR 0030/契约变更；不自行合并、不领取下一卡。

## 交付物

- `core/contracts/cognition.proto`：12 条新消息、5 个枚举；独立内部记录、工作/在途状态、输入/回执、完整绑定、内部提交、事务 V3 与维护记录。由既有生成器输出 Rust/Python/TypeScript。
- 三语言 `cognition` 语义入口及三种探针：检查嵌套回执、完整请求绑定、合法完成状态、后继与行动绑定；解码后由消费者显式调用，README 写明职责。
- `fixture_cognition.py`、`verify_cognition.py` 和黄金向量：新增 51 合法、164 非法；原 565 合法、696 非法逐项保留。`baselines/p5-68/` 冻结开工 main 的 10 个 IDL 文件及全部黄金向量摘要。
- [ADR 0030](../../docs/adr/0030-cognition-record-contracts.md)、[规格对照](../../docs/traceability/P5-37.md) 与 V2-110 至 V2-114。开工 main 和开放 PR 均未占用预留编号，无需顺延。

## 验收

| 命令 / 检查 | 当前结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | passed，0 stale | contracts-02.log |
| `pnpm test:core-contracts` | passed；180 消息，每语言 15,973 案例（3,533 接受 / 12,440 拒绝）；官方 protoc 616 向量；新事务各语言 12 恢复 / 12 跨版本拒绝 | contracts-02.log |
| `python core/contracts/check_compat.py --ref 8839dcae73c98437f61222f49d08cafd9ba8f4e6` | passed，7 个已发布基线、current 与开工 main 均 0 违例 | compat-02.log |
| `uv run --project core/contracts --locked ruff check core/contracts workers/irl/generated/echo` | passed | ruff-02.log |
| `MYPYPATH=workers/irl/generated` 下锁定环境 `mypy --strict`（生成器、夹具、对抗/验证脚本、兼容/冻结脚本与 generated/echo，含导入的新模块） | passed，15 个入口文件 | mypy-03.log |
| `cargo fmt --all -- --check`；`cargo clippy --workspace --all-targets --locked -- -D warnings` | passed，缓存入口修复后复跑通过 | rust-fmt-02.log、clippy-02.log |
| `cargo test --workspace --locked`；修复后 `cargo test --workspace --locked --exclude core-kernel --exclude core-store` | 首轮在 core-types 缓存测试失败；修复后其余工作区全通过。未改动的 core-kernel 首轮 273 passed / 1 既有 ignored（V2-93 旧舍入诊断），core-store 51 passed；两包复用该有效结果 | cargo-test-01.log、cargo-test-02.log、rust-results-02.json |
| `pnpm lint` / `pnpm typecheck` / `pnpm test` | passed；54 文件、620 测试 | pnpm-lint.log、pnpm-typecheck.log、pnpm-test.log |
| Ubuntu CI | 修复后 CI passed；报告 head 的 CI #42881172237 也已成功 | [CI #42881172237](https://github.com/nanake-lab/nemu/actions/runs/42881172237) |
| Windows 完整工作流 | attempt 1 上传故障 failure；attempt 2 success，桌面四项、W02 51 单元 + 3 文档、W18 42 测试及两包 clippy 通过，三个 artifact 上传和最终门禁成功 | [Windows #42963248080 / attempt 2](https://github.com/nanake-lab/nemu/actions/runs/42963248080/attempts/2)、[证据索引](EVIDENCE-INDEX.md) |
| COG-A01–11/13、SYS17、持续认知真实时间/体验 | not_run，只有契约层合成验证；COG-A12 旧规则已被取代，不签发通过 | [规格对照及范围](../../docs/traceability/P5-37.md) |

原始数据与失败日志归档私有 `nanake-lab/nemu-evidence/P5-37/`，公开仅留 [EVIDENCE-INDEX](EVIDENCE-INDEX.md)。全部输入为合成数据，没有读取正式或排练存储。生成产物、黄金向量及契约基线属于代码和验收，保留在主仓库。

## 失败与修复

首次生成因新维护枚举与既有 `MaintenanceKind` 重名失败，改为 `CognitiveMaintenanceKind`。首次兼容检查发现 `@supersedes` 缺少前任停用标记；检查现有 store 写入门禁后选择已有 `@retains`（见下），未放宽门禁。首次 mypy 报夹具字典推断过窄，补显式类型后通过。首轮 Ubuntu CI 与本机全工作区测试发现缓存一致性测试的独立探针漏调新语义检查（非法向量 732 被接受）；在原测试探针追加相同入口，保留所有断言，`a0e0d7b` 修复后该测试通过。Windows 自动工作流因草稿跳过 self-hosted 作业，汇总门禁按规定失败，不能当作测试通过。失败日志保留在证据索引；未删除旧断言或降低阈值。

## 规格对照与边界

逐条对照见 [P5-37](../../docs/traceability/P5-37.md)，逐字段出处见 [ADR 0030 字段表](../../docs/adr/0030-cognition-record-contracts.md#决策与字段出处)。I14/I16/I18 的本卡证据为完整绑定、规范编码与显式状态字段；I02/I23、原子发布与重放须运行时继续验收。三语言检查不能证明引用存在、来源可信、实际 m 注入、当前 Q 未过期或行动已获授权。

`round_index` 仅为活动请求顺序；不实现已取代的 n/κ/心理期限。内部记录与自发行动类别用类型和来源关联表达；F 依据身份沿用 P5-68。D11 的发生/观察时间事实和消费键归 P5-72。保留 ActionPlan 既有执行期限并不赋予其心理作用期限含义。

**需要批准的分阶段安排（V2-113）：** V3 采用现有 `@retains CoreTransactionV2`，仍检查保留/reserved 字段号；暂不机器停用 V2。当前 store/runtime 仍生产 V2，立即加 `@superseded` 会触发 Objects::prepare 拒写。P5-38 接线后再停用旧写入；本卡不提前改 store。旧对象保持原标签，回退按旧应用与兼容备份恢复，不从旧草稿或 Q 猜造内部历史。

新手写实现文件均小于 400 行。原有大文件仅加探针/夹具登记；超过 400 行的新增基线与生成产物是已发布 IDL 的逐字节冻结或机械生成，无新增抽象层。无新增依赖。

## 基线与待确认

开工 main 为 `8839dcae73c98437f61222f49d08cafd9ba8f4e6`；契约 suite 与首次工作区检查的源码绑定 `ceea6722`；`a0e0d7b` 仅追加 cfg(test) 缓存探针的语义调用，已复跑受影响的工作区测试、fmt、clippy。清单与变更说明见私有 tested-source.json、toolchain-and-fix.json。本机 Rust/Cargo 1.99.0 与锁定工具链一致，Python 3.12.10、Node 24.21.0、pnpm 12.11.2；依赖按锁文件安装。开放 PR 完整文件列表核对未发现契约线占用。Ubuntu CI 实际 checkout 为 `eaa33c53f822faa5bcd4aeaacc7344c33825803a`，已核对两个父提交分别为上述 main 与 `a0e0d7b`；通过 Rust 工作区/存储、前端构建测试、Python 四项目与三语言契约门禁。收尾提交只更新报告、阻塞说明与证据索引，不改变被验证源码。

- [ ] （阻塞）用户批准 ADR 0030、V2-110 至 V2-114，尤其 V2-113 的分阶段停用安排。
- [ ] （阻塞）独立交叉审查通过；本执行会话不自审签发。
- [x] Windows runner 恢复在线并完成 `86acb718` 的完整工作流，attempt 2 success。
- [ ] （阻塞）请原审查方确认已补齐 Windows 一项；见 [BLOCKED](BLOCKED.md)。

## 后续建议

P5-38/39 完成真实持续认知、H/Q 原子提交与恢复；P5-72/75 处理新规律消费、时间事实与运行时接线。COG 的行为、资源、持续时长与真实体验由相应卡实测，本卡不以编码测试替代。


## 2026-11-11 Windows 补验记录

按用户指定命令 `gh workflow run windows.yml --ref task/P5-37-cognition-idl` 手动触发 [run 42963248080](https://github.com/nanake-lab/nemu/actions/runs/42963248080)，默认完整范围（desktop/storage/gateway）。被测分支 head、checkout 日志及桌面 run/manifest 的 SHA 均绑定 `86acb718288bf48ddd50f65dddb5ff294fed60c7`。本轮仅修改报告、BLOCKED 与证据索引，不改变源码、契约、测试、依赖或工作流。

首次 attempt 1：桌面 transport/isolation/e2e/visual、证据来源校验、W02 clippy 与 51 单元 / 3 文档测试、W18 clippy 与 42 测试均成功；W18 artifact 创建请求返回 `ECONNRESET`，self-hosted 作业及最终汇总门禁因此失败。保留完整失败日志、两个成功上传的 artifact 以及 runner 上未上传的 W18 日志；本地留存不替代成功上传或工作流通过。定位上传故障后仅重跑失败作业一次，attempt 2 全绿，2026-11-11 05:30:31–05:48:31 UTC；self-hosted 作业与最终汇总作业均 success，三个 artifact 均已上传。W02 51 单元测试耗时 743.73 秒、3 文档测试耗时 0.19 秒；W18 42 测试耗时 111.29 秒。完整 jobs/steps 结论及日志见私有证据索引。

本轮 fetch 的 main 为 `74f7bcaf41ec45bb31063fc079eeeae49927ac82`，分支实际 main 基线仍为 `8839dcae73c98437f61222f49d08cafd9ba8f4e6`。两者间只有 #156 的四份开工阅读规则文档；已按新版规则阅读，无产品、构建、依赖、测试、阈值或 CI 影响，按 main 影响判断规则不为本轮补验合并。Ubuntu CI #42881172237 也已完成成功，实际合并 checkout `538ac9ce5e5d63e1dffb0050632f590aa70176e5` 的父提交为 `8839dcae` 与 `86acb718`。

证据隐私与完整性：既有桌面采集器把全机进程清单写入 artifact；本轮不改采集代码。原 ZIP 本地保留，私有归档仅收入把 9 处进程清单替换为总数的脱敏副本，另存原 ZIP 摘要、逐文件前后摘要及转换脚本；原 manifest 更名为 source-manifest，不能用它校验脱敏后的字节。原 artifact 缺少 manifest 列出的隐藏文件 `test-results/electron/.last-run.json`，明确记为缺项；存在的 manifest 文件摘要均核验。测试日志、结果、提交绑定不改写。后续需单独修正采集范围和隐藏文件归档，本卡不扩展代码范围。
