档位：重档；本卡修改 Echo Core IDL 与三语言契约边界。

# P5-68 契约变更：D07 与 D09 新规律的状态

## 当前状态

已按 [针对 1a4b751f 的首审](https://github.com/nanake-lab/nemu/pull/146#issuecomment-6733725890) 修复内部依据重访的阻塞项，等待原审查方复审。PR [#146](https://github.com/nanake-lab/nemu/pull/146) 保持 draft / needs-user，未合并；当前提交的 CI 与复审通过前不称整卡完成。

首审 head `1a4b751faa7725c2fd9ca8c503e496207923621e`；本轮只改三语言 law_state 语义层、生成物、夹具/黄金向量及角色注释/报告。main 基线 B=M=`a7e8ae8790d297ac697956f6f1256d2e2b9403d5`，2026-11-09 再 fetch 后没有增量，无需 merge。

## 首审修复与本轮验证

- 先在未修改语义层的首审 head 上运行 [失败回归](SYS21/review-role-red.log)：三语言 × 编码/CanonicalV1/Protobuf 共 9 路均错误接受内部依据＋重访。随后各语言增加一条 §4.2–4.3 组合检查；[修复后原回归](SYS21/review-role-green.log) 全部拒绝，内部依据＋目标更新的 9 路仍通过。[复现脚本](SYS21/review-role-regression.py) 使用与正常契约 CI 相同的夹具输入。
- `FastEffectRole` 注释与规格映射明确：role 是本次作用的角色，不改原记录的固定角色。§5.2 的首次有效补解释可作目标更新；首次有效、现行目标归属等历史条件交后继治理/消费边界验证。没有添加消费键或身份规则。
- [冻结记录](SYS21/review-freeze.log)：原 564 合法/693 非法逐项不变，只追加 1 合法/3 非法，总计 565/696。新增非法输入分别覆盖编码与两种解码路径，合法向量进入既有三语言往返套件。
- `pnpm test:core-contracts` passed：[本轮完整日志](SYS21/review-contracts.log)，三语言各 14,755 例，生成物 0 stale，全部基线 0 违例，官方 protoc 565 向量、旧 θ 恢复及 SYS17 契约前提均通过。
- Rust：`cargo test -p core-types --locked`、该包 all-targets clippy、全工作区 fmt passed：[测试](evidence/review-core-types.log)、[clippy](evidence/review-clippy.log)、[fmt](evidence/review-fmt-02.log)。初次 fmt 要求新增调用换行，[失败输出](evidence/review-fmt.log) 保留，修正后复验通过。
- Python：契约及三份生成包 ruff、严格 mypy，以及三个 worker 的严格 mypy/pytest passed：[ruff](evidence/review-ruff.log)、[mypy](evidence/review-mypy.log)，worker 日志为 evidence/review-*-mypy.log 与 review-*-pytest.log。
- 全仓 TS lint/typecheck/test passed（54 文件、620 项），见 [lint](evidence/review-lint.log)、[类型](evidence/review-typecheck.log)、[测试](evidence/review-node-tests.log)。完整远端 CI 在推送修复 head 后重跑，结果与链接记录在 PR 的逐条回复；不把上一 head 的状态当本轮结果。
- 修复 head `9e043fe2` 的 [CI #408](https://github.com/nanake-lab/nemu/actions/runs/42577591962) 已通过 Node/Python/Rust、存储和完整三语言验收，随后独立兼容工具 mypy 因找不到 `echo.*` 失败（[原日志](evidence/review-ci-408-failure.log)、[本机同命令失败](evidence/review-compat-mypy-red.log)）。此前本机的 MYPYPATH 掩盖了该 CI 配置遗漏。尝试全限定导入后，全量检查报重复模块名（[输出](evidence/review-full-mypy-02.log)），已撤回该方案。
- 最终按 AGENTS §2 的共享 CI 只追加规则，在 `.github/workflows/ci.yml` 的 P5-21 步骤追加 MYPYPATH，指向现有 `workers/irl/generated`，与夹具的运行时路径一致；原命令、strict 要求及断言不变。两条原 mypy 命令均通过：[兼容工具](evidence/review-compat-mypy-final.log)、[全量契约](evidence/review-full-mypy-final.log)。契约源码、字节和向量仍是 `9e043fe2` 的版本，后续只追加 CI 配置及证据；新 head 再跑 CI，结果记录在 PR 回复。

## 交付物

- [law_state.proto](../../core/contracts/law_state.proto)：九种新消息、两个枚举；完整 θ 状态加 P、后继参数 P₀、依据身份/槽/快照、F 作用记录和迁移记录。旧 IDL 不变。
- 三语言显式语义边界：Rust `semantic/law_state.rs`、Python/TS 模板及生成物；θ 校验沿现有函数区分消息版本。消费前调用语义检查，可信 H/资源解析归调用方。无规律实现、无 core-kernel 修改、无消费键。
- [fixture_law_state.py](../../core/contracts/fixture_law_state.py)、[verify_law_state.py](../../core/contracts/verify_law_state.py)：接入正常契约 CI；累计新增 38 合法、73 非法黄金向量，每个新消息至少 3/3。R4 修订与合并、h 大于目标等是合法正例。
- 冻结 main 的前代 IDL/黄金摘要基线，追加 [契约说明](../../docs/CONTRACTS.md) 与 [逐字段规格对照](SPEC-MAPPING.md)。没有新增差异登记或参数默认值。

## 首审 head 的初版验收（本轮增量验证见上）

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | passed | [日志](SYS21/generate-check-01.log) |
| `python core/contracts/check_compat.py` | passed：全部基线 0 违例 | [日志](SYS21/compat-01.log) |
| `pnpm test:core-contracts` / SYS21 | passed：每语言 14,718 例；双编码、摘要、官方 protoc 一致 | [完整日志](SYS21/contracts-02.log)、[环境与向量说明](SYS21/README.md) |
| SYS17 契约前提 | passed：3 个空槽迁移；每语言 36 项新旧互解拒绝、18 项旧字节回读 | [范围及证据](SYS17/README.md) |
| `pnpm lint`、`pnpm typecheck`、`pnpm test` | passed：54 文件、620 tests | [lint](evidence/lint-01.log)、[类型](evidence/typecheck-01.log)、[测试](evidence/node-test-01.log) |
| Python 契约 ruff / mypy --strict | passed：14 文件；包含新夹具、迁移检查与生成代码 | [ruff](evidence/ruff-01.log)、[mypy](evidence/mypy-02.log) |
| 三个 worker 的 ruff / mypy --strict src tests | passed | evidence/ 下 irl、drive、model-adapter 的对应日志 |
| `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings` | passed：修复后复验 | [fmt](evidence/cargo-fmt-02.log)、[clippy](evidence/cargo-clippy-02.log) |
| Rust 工作区测试（分批完整覆盖） | passed：首轮 core-kernel/core-store 通过；探针修复后其余包及全部 doc-tests 通过。首轮整命令失败保留，不伪称一次全绿 | [首轮](evidence/cargo-test-01.log)、[其余工作区](evidence/cargo-test-remaining-02.log)、[kernel/store doc-tests](evidence/cargo-doc-02.log) |
| 三个 worker 的 `uv run --locked pytest` | passed：28 / 1 / 1 项 | [irl](evidence/irl-pytest-01.log)、[drive](evidence/drive-pytest-01.log)、[model-adapter](evidence/model-adapter-pytest-01.log) |
| 真实时间端到端/生产迁移 | not_run：本卡只编码时间锚点，不实现等待、推进或接线；后继 P5-75 验收 | [SYS17 边界](SYS17/README.md) |

## 失败、阻塞与未完成项

- 首轮 `cargo test --workspace --locked` failed：缓存测试探针只调用 θ 语义边界，遗漏新增快状态边界，接受了非法向量 false/660（槽坐标不一致）。补上与正式三语言探针一致的调用后，原缓存测试通过；未改断言或黄金向量。[原失败](evidence/cargo-test-01.log)、[定向回归 9/9](evidence/core-types-test-02.log)。首轮已通过的 core-kernel/core-store 产品及测试代码均未改，保留该证据；剩余工作区和这两包 doc-tests 分批补完。
- 首轮契约验收的 TypeScript 启动找不到 zod；补 D 盘缓存输出的依赖链接后原命令通过，原失败日志保留。首次独立 mypy 调用未设生成包搜索路径，3 个 import-not-found；以完整生成包输入并设置 MYPYPATH 后严格检查通过。均未放宽检查。
- 实现提交的 `git diff --check` 仅指出新冻结基线 growth.proto 原有末尾空行。为保留发布字节没有修剪；9 份基线 IDL 与 main Git blob 对照一致，除此冻结副本外代码 diff 检查通过。[原输出](evidence/diff-check-01.log)、[代码检查](evidence/diff-check-code-01.log)、[基线核对](evidence/baseline-copy-01.log)。原始日志也不做空白修整。
- Ubuntu [CI #406](https://github.com/nanake-lab/nemu/actions/runs/42568888129) 首轮 failed：Electron 下载 HTTP 504，桌面设置测试加载失败，[原日志](evidence/ci-406-attempt1.log) 保留。同提交重跑后 Node lint/typecheck/test/build、契约 coverage、transport 及四个 Python 项目检查通过，但 Rust 复现同一缓存探针 false/660 失败，[第二轮日志](evidence/ci-406-attempt2.log) 保留。最终提交包含探针修复，需要新的 CI；本机结果不替代远端结论。
- Windows [#267](https://github.com/nanake-lab/nemu/actions/runs/42568888098) 的 self-hosted 作业因 draft skipped，汇总检查 failed；没有 Windows CI 通过结论。保持用户要求的草稿状态。
- 首审指出的内部依据＋重访漏检已按失败回归修复；首审 head 的 Ubuntu [运行](https://github.com/nanake-lab/nemu/actions/runs/42570795466) 因 Electron 下载 HTTP 504 失败，需在修复 head 重跑。等待原审查方复审；合并必须由用户在审查和必要检查完成后执行。

## 规格对照

[SPEC-MAPPING.md](SPEC-MAPPING.md) 覆盖所有字段、ADR 0035 §9 五条以及 I18/I23/I16/I03/I21/I24/I29 的本卡检查方式。保存状态不等于证明规律或运行时接线。旧 θ 从 H 重算、真实存储迁移与故障回退、前态引用真实性均留给后继卡。

Rust 分批补验命令：`cargo test --workspace --locked --exclude core-kernel --exclude core-store` 与 `cargo test -p core-kernel -p core-store --locked --doc`。只复用首轮中两个未受测试探针修复影响且已通过的包；没有删减断言、忽略失败测试或修改阈值。

## 新增依赖

无。大文件均为现有生成 schema、冻结黄金向量、发布基线或证据；手写新模块不超过 400 行。

## 待用户确认

- [ ] （阻塞）请原审查方复审本轮修复，通过后由用户决定合并。
- [ ] （阻塞）合并前确认当前 head 的必要 CI/Windows 检查通过；本报告不豁免失败或 skipped。

## 后续建议（不在本任务范围内）

P5-69/P5-70 实现新规律；P5-72 定义消费键；P5-75 接入语义边界、版本切换与存储迁移。没有开展这些卡。
