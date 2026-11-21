重档。交付线，修改 Echo Core IDL、生成类型和终端上传边界。

# P5-43 五项正式语义边界

## 当前状态

第一阶段交付供交叉审查。实现提交 `50f240fb` 基于 `origin/main@da40fd06`。P5-40 的 [#235](https://github.com/nanake-lab/nemu/pull/235) 仍开着，冻结基线和追加新黄金向量的前置条件尚未满足。[阻塞项](BLOCKED.md) · [草稿 PR #244](https://github.com/nanake-lab/nemu/pull/244)

## 交付物

- [ADR 0048](../../docs/adr/0048-formal-semantic-boundaries.md) 及 V2-190 提交五项字段与编码。开 PR 和交付前均核对 main 与开放 PR，P5-40 占用 0047 / V2-189。
- [IDL 草案](../../core/contracts/semantic_evidence.proto) 含 22 个新标签、9 个枚举及 6 个 `@retains` 后继。各端候选生成物用于编译和检查；三端上传入口拒收这 22 个标签。
- [Rust 语义检查](../../core/crates/core-types/src/semantic/evidence.rs) 包括按原载荷取字段和 UTF-8 字节、概念修订和有向别名、文字及语音转写适配、按 IRL/A/M 核对调度和采纳、逐范围核实及目标前提。受信前沿、身份、正式时间事实和实际 C 单位由调用端提供。P5-06/P5-53 的生产接线待后继卡执行。
- [规格与验收对照](../../docs/traceability/P5-43.md) 及[分阶段测试设计](TEST-DESIGN.md) 列出已有反例、冻结后需补的三端测试以及生产验收责任。新增运行时依赖为零；较大的变更来自生成器写出的描述符及语言类型。

## 验证

| 检查 | 草案结果 |
| --- | --- |
| `generate.py --check`、`check_compat.py --ref origin/main` | 0 个过期生成文件、0 项兼容违规。726 条原合法向量与 988 条原非法向量保持原样，基线目录没有改动。 |
| 官方 protoc 和 protobuf、契约 schema 单测 | IDL 编译通过，原 726 条传输向量通过；14 项 schema 测试通过。 |
| `cargo fmt --all -- --check`、`cargo clippy -p core-types --all-targets --locked -- -D warnings`、`cargo test -p core-types --locked` | 通过，含本卡 10 项语义边界、旧冻结向量冷/热/无缓存对照。 |
| 三端真实分块上传检查 | 各自接受一条旧合法完整分块；保持数据、偏移与预算相同，只替换标签，Rust、Python、TypeScript 各自拒绝 22/22 个候选标签。 |
| 根 lint、typecheck、`pnpm test`、`pnpm test:contracts` | 通过。根套件 54 个文件、620 项，契约包 8 个文件、354 项。 |
| Python 严格 mypy、Ruff | 生成的 Python 包与验证脚本通过严格类型检查；修改的 Python 源文件通过 Ruff。 |
| 新标签 SYS21 及完整跨端语义对照 | **未通过。** 已实跑 `pnpm test:core-contracts`，`verify.py` 在每个标签至少 3 条合法向量的断言返回 1。22 个候选标签目前均为 0 正、0 反，后续的三端新向量对照未执行。#235 合并前按派卡暂缓追加。 |
| CI 与整卡验收 | 实现提交的 Windows 草稿门卫返回失败，自托管 job 跳过。CI 以 [PR 检查](https://github.com/nanake-lab/nemu/pull/244/checks) 的具体提交为准。整卡工作区回归、三端新增向量和正式生产验收尚未完成。 |

原始日志、命令和摘要见[私有证据索引](EVIDENCE-INDEX.csv)。只使用合成输入及仓库测试夹具。

## 待确认及下一步

- [ ] 交叉审查和用户确认 ADR 0048、V2-190 及本次字段编码。
- [ ] #235 合并后 merge 最新 main，重跑生成器，从该版 main 冻结新基线，再追加黄金向量和跨端语义对照、重跑完整验收。生成物与基线遇冲突时重建。
