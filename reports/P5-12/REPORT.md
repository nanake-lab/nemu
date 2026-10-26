# P5-12 J 编译与读出 — 报告

## 当前状态

**部分完成，不能合并。** J 数值编译、绝对预算检查、身体读出及局部结构测试已实现；正式信封/作用回执、genesis 资源解码和主动强度读出受 V2-116 阻塞，见 [BLOCKED.md](BLOCKED.md)。[PR #101](https://github.com/nanake-lab/nemu/pull/101) 保持 draft + needs-user，尚未请求最终交叉审查。没有接取下一张卡。

用户于 2026-10-26 从 Claude 改派给 Codex。工作树 `D:\Dev\nemu-wt\P5-12`，分支 `task/P5-12-j-compiler`，基线 `origin/main@b4eb7fd52f848eb042b2446c3f4ebf2416108c6c`。P5-10 #94 与 P5-27 #92 均已合并；开始实现前已推送认领提交并开草稿 PR。

## 交付物

- `core/crates/core-kernel/src/modulation/`：纯数值入口 `compile_numeric`，只借用 θ/x/R/t/空 validity。直接调用既有 `effects` 与 `fast`，从提交锚点投影一次；四天性固定顺序、学得项按规范倾向顺序。未验证或缺失方向被排除并列出，不缩放旧项。
- 同目录 `config.rs`、`numeric.rs`：只读数值配置、引用格式与绑定检查、各项及组合 β 范围校验；非负绝对和向上保守舍入，拒绝 NaN/Inf 与溢出，无 epsilon 宽限。此检查不验证真实方向或证书内容。
- 同目录 `readout.rs`：三轴天性线性组合，中性 `(0.5, 0, 0.5)` 与范围按 CONTRACTS §7；先检查有限乘加再截断。参数由调用方提供，没有审美默认值；本轮测试参数仅为结构夹具。
- `rehearsal_without_fast`：只接受 REHEARSAL，返回零系数与中性身体读出，无 x 输入；运行标记与停止自主行动仍须外层接入。
- 20 项新增测试、308 对独立精确有理数预算向量及生成脚本；[J06 说明与结果](J06/README.md)、[result.json](J06/result.json)、`verification/` 原始日志和输入 SHA-256。
- [规格对照](../../docs/traceability/P5-12.md)、[接口请求](../../tasks/requests/P5-12-resources-and-receipt.md)、差异登记新增 V2-116。

生产代码全部在卡内目录。`lib.rs` 只追加一行模块登记（AGENTS 共享例外），其余卡外改动仅新增报告/请求/对照记录以及追加差异登记。未改 fast、effects、IDL、Cargo 清单/锁文件、配置、任务卡或正式资产。Rust 文件均少于 400 行；1542 行 JSON 是 308 个预算向量的格式化原始数据，保留完整便于独立复算。

`NumericReceipt` 是进程内轨迹，不是可发布回执：没有 Serialize/WireMessage、没有自造摘要。`NumericCompilation` 也不是生产 `ModulationEnvelopeV2`。它不能证明输入是同一已提交快照、ObjectRef 的内容真实或 genesis 已核验；这些是未来权威解码与发布边界的责任。

## 验收

本地 Windows、仓库锁定 Rust 工具链，`CARGO_HOME=D:\Dev\cache\cargo`，编译与 TEMP/TMP 在 `D:\Dev\cache\P5-12`。进程内配置，未修改系统/用户设置。主代理串行执行，构建 jobs=2、测试线程=1；开始前检查其他重负载与 Runner.Worker。按角色名调用 `routine_reporter`，仅只读核对完成的日志、测试数、路径及哈希；未授予重负载窗口或验收裁决职责。未使用 GPU。

测试在提交实现前执行，因此日志中的 `revision` 为认领提交；`inputs` 逐文件记录实际受测源码的 SHA-256。只读复核已确认 kernel 与 Clippy 两份记录各 26/26 个输入与当前源码一致（2026-10-26 18:49–18:50 +08:00），不能仅按该 `revision` 把测试误读为只测了认领报告。

[tested-git-blobs.json](tested-git-blobs.json) 同时记录这 26 个文件的受测 Windows 字节哈希和待提交 Git blob 身份；两者经核对仅可能存在仓库规则造成的 CRLF/LF 差异。克隆后的核对使用 Git blob 哈希，原始日志中的本机哈希保留不改。

| 命令或检查（含验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `cargo test -p core-kernel --locked --offline` | passed：65 项，其中新增 20 项 | `verification/kernel-20261026-183537-213.log` / `.json` |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | passed | `verification/clippy-20261026-183849-036.log` / `.json` |
| `cargo test --workspace --locked --offline` | passed：120 项单元/集成测试 + 2 项 compile-fail 文档测试，0 失败/忽略；总计 1193.17 秒 | `verification/workspace-20261026-183938-846.log` / `.json`；26/26 个输入哈希与当前源码一致 |
| `cargo fmt --all -- --check` | passed | `verification/fmt-20261026-190029-027.json`（命令无输出） |
| `python -B reports/P5-12/generate_bound_vectors.py --check`；该文件 `mypy --strict` 与 Ruff E4/E7/E9/F/I | passed：308 对独立参考向量复算一致；静态检查通过 | `verification/{oracle,mypy,ruff}-20261026-190106-829.log` / `.json` |
| J06 数值结构：正向两层调制、未授权零作用、绝对预算、相消拒绝、旧项不缩放 | passed（局部） | `J06/result.json`；308 对 Fraction 算术向量，各测两个顺序 |
| 确定性与记录 t 重放、零态、静息点、身体读出、F 未接通排练路径 | passed（局部） | 核测试：64 次逐位重复、1024 组固定样本、输入不变、规范排序、身体溢出拒绝 |
| 完整信封、输入摘要、规范作用回执、genesis/方向验证资源装载 | not_run | V2-116，当前无发布入口 |
| 两个读出的整体零态/排练验收 | not_run | 身体部分通过，主动函数缺定义 |
| J01/J02/J03/J04 行为、J05 KV、J07 迁移 | not_run | 真实载体、模型与行为验证不属于本卡局部数值结果 |
| 独立交叉审查 | not_run | 本卡仍有阻塞；完成后在 PR 写「请审查」 |

测试使用合成内存状态；工作区存储测试使用新临时目录，不打开她的排练或正式存储。部署 profile、真实方向提取/验证、模型注入与行为有效性均未宣称通过。没有因本卡修改 TypeScript/Python worker 或 UI，不运行其应用验收；预算向量生成脚本另外检查。

资源等待记录：第一次 fmt 检查发现其他 cargo 进程，未启动命令（`verification/fmt-20261026-190000-978.json` 为 `not_run`）；确认这些进程结束后执行 fmt 通过。未跳过断言或测试，未改阈值。以上已执行检查没有失败结果。

CI 以 PR 当前 head 的检查为准，本地通过不替代 CI；推送后在 PR 记录结果。本轮纯 core-kernel 路径触发 Ubuntu Rust 检查，不在 `windows.yml` 的触发路径内。未要求或触发 Windows/GPU 作业。

## 规格对照

完整条款、实现和测试映射见 [docs/traceability/P5-12.md](../../docs/traceability/P5-12.md)。

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| v2.0 §9.1–9.2；ADR 0008 §3 | `modulation/compile.rs`，复用 `effects`、`fast` | 正向两层系数、候选/未验证排除、记录 t 逐位复现 |
| O §13.1；核验 §7.4；S §6；J06 | `config.rs`、`numeric.rs` | 绝对预算、符号相消、零态超额、浮点向上界 |
| v2.0 §9.3；CONTRACTS §7 | `readout.rs` | 固定三轴、中性、截断、有限性 |
| O §13.2；v2.0 §9.4、§16 | 待契约/函数定义 | not_run；V2-116 |

I03/I05：收窄 DTO，无 H/z/o/p/c_task/叙事接口；I06：复用 R⁺ 编译且排除未验证方向；I18/I21：无持久状态或 IO、只借用输入并验证不变；I23：固定顺序、有限 binary64、正零和逐位重放；I29/AT12：纯合成夹具不签发生产资格。跨 CPU/编译器逐位兼容尚未签发。未新增 IDL 或改变协议黄金向量；本模块算术向量不冒充协议向量。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| 无 | — | — | 复用现有 Rust 依赖；独立向量生成仅 Python 标准库 |

## 待用户确认

- [ ] （阻塞，V2-116）方向集/验证记录/固定读出/J 作用回执的字段及规范编码；契约接口请求已提交。本卡目录不含契约，当前 P5-34 #95 占用相关目录。
- [ ] （阻塞，V2-116）主动读出的具体函数、单位、中性值、界及版本资源；未选线性、指数或其他默认。
- [ ] （阻塞）补齐以上后完成全部卡内验收，再安排高难度卡独立交叉审查。

## 后续建议（不在本任务范围内）

- 契约所属卡提供资源与回执后，将局部数值输出封装为正式 envelope，并核验摘要、genesis、方向验证证据、hook/shape/dtype/mask。
- 外层运行时接入已提交快照、记录 t、结构故障与排练状态/自主行动门控；P5-13/P5-31 完成真实载体与方向资格。不得从本报告推导生产启用许可。

## 下一步

等待缺失定义和目录归属决定；其余不受影响的数值实现与测试已推进。整卡不标完成，PR 不转可审查、不合并、不领取下一卡。
