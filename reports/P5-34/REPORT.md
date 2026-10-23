# P5-34 分配的实际保留率与倾向的证据量 — 报告

## 当前状态

部分完成。草稿 [PR #72](https://github.com/nanake-lab/nemu/pull/72) 保持 `needs-user`。已 merge 同步 origin/main `0b5412e`；本轮恢复、编译前、提交前及发布前再次 fetch，main 未变，ADR 0021、V2-62/63 没有撞号。用户已批准本 PR 扩围 core-store。验证的实现提交为 `8d9694ef44cfc2d6a75cc6eedf890150e5401183`，之后只补文档与证据；契约、生成、语义、存储及完整 Rust 工作区回归通过。main 尚无真实 G 重放器，生产迁移集成和 SYS17 未完成。等待交叉审查及用户批准，不能合并或取下一卡。

## 2026-10-24 空闲窗口验收（当前结果）

用户指示监测其他任务 Windows CI #39993887148，结束后停止 runner。该 CI 于01:37成功结束，01:37:35确认 Listener/Worker、编译及存储测试进程均无残留，本卡之后才启动重检查。见 `verification/idle-20261024/resource-window.jsonl`、`resource-release.json`。本轮重检查现已结束，runner保持停止，用户可恢复；缓存、target、虚拟环境和临时根均在D盘。

以下证据均在 `verification/idle-20261024/`，每个命令 JSON 含参数、开始/结束、基底HEAD/main、退出码和日志SHA-256。更早的「未运行」说明属于历史状态，以本节为准。

| 命令或检查 | 实际结果 | 证据 |
| --- | --- | --- |
| 生成检查、schema单测 | 0 stale；12 tests通过 | attempt-01/contracts.log/.json |
| `pnpm test:core-contracts`（施工合同10.8、本卡SYS21范围） | Rust/Python/TypeScript各11592用例通过（2659合法、8933非法）；官方protoc 464向量通过；Node中的浏览器安全Zod 106项通过 | attempt-01/contracts.log/.json |
| `check_compat.py --ref origin/main` | p5-01、p5-21、p5-32、current、origin/main均0违例 | attempt-01/compat.log/.json |
| Rust权威黄金冻结 | 445合法、457非法旧向量复核通过；追加19合法、35非法 | attempt-02/freeze.log/.json |
| 独立字节复核 `review_vectors.py --base origin/main` | 旧数组/原文前缀相同、Unicode未变；19个新增合法向量0差异 | attempt-01/bytes.log/.json |
| `cargo test -p core-types --locked -j 1 -- --test-threads=1` | 31项通过，含P5-34的7项，0失败/忽略 | attempt-01/types.log/.json |
| 定向core-store迁移边界测试 | 6项通过；其余31项由完整回归执行 | attempt-01/migration.log/.json |
| workspace Clippy（all-targets，`-D warnings`） | 通过 | attempt-01/clippy.log/.json |
| Rustfmt | 通过 | attempt-02/fmt.log/.json |
| 完整Rust工作区测试（locked，单编译作业/单测试线程） | 退出0；98项通过（含2项compile-fail文档测试），0失败；core-store 37项全部通过 | attempt-01/rust.log/.json |
| pnpm lint、typecheck、test | 通过；45文件535 tests通过 | attempt-02/lint、typecheck、node.log/.json |
| pnpm test:contracts | 7文件350 tests通过；语句/函数/行100%，分支97.46% | attempt-02/coverage.log/.json |
| 契约工具与三份生成Python的Ruff、CI范围mypy | 通过；mypy报告11源文件 | attempt-02/ruff、mypy.log/.json |
| irl、drive、model-adapter、voice-sidecar各自Ruff、strict mypy、pytest | 四项目全部通过，每项目1项脚手架测试 | attempt-01/python-*.log/.json |
| 真实G重放迁移、AC2-08/19/21、I23、SYS17 | not_run；存储替身不证明生产规律或迁移，crash测试不证明物理断电 | BLOCKED.md、ADR0021 §7 |

黄金现为464合法、492非法、18 Unicode；新增V3分布为分配6/10、倾向5/11、证据组4/7、快照4/7（合法/非法）。旧前缀逐项相同，Unicode无diff，p5-32基线未覆盖。

**本轮原始失败与修复：** 首次冻结退出1：V=0.9999999999999999、A=-1被Rust接受。复现确认默认JSON解析把V变成1，输入原位模式为`0x1.fffffffffffffp-1`。为既有serde_json 1.0.154启用float_roundtrip，并新增相邻值/次正规数JSON位模式测试后，旧黄金复核和完整三语言验收通过。原失败在attempt-01/freeze.log/.json及json-rounding-diagnosis.json，修复后冻结在attempt-02；没有重写旧期望、放宽阈值或增加字段。

首次fmt的证据包装器因无输出而未建空日志，后续Get-FileHash失败、退出码未落盘，该轮不作验收；原文摘录在attempt-01/fmt-harness-error.txt。包装器先建日志后，以attempt-02重跑通过。

工具链为Rust/cargo1.99.0、Node24.21.0、pnpm12.11.2、Python3.12.10、uv0.12.15；锁定项目环境为CPython3.12.14。机械核验和完整Rust命令按角色名调用routine_reporter（工具配置gpt-6-luna/max），主代理明确授予独占窗口并核对证据；不是交叉审查。

完整Rust命令于01:50:16开始、02:12:49结束；日志中的transaction.rs两处编译错误属于预期compile-fail文档测试，两项均标ok。`record_verification.py`核对所有命令日志SHA-256，生成command-index.json和49份实现文件的tested-source-sha256.json。按验收编号的结果在SYS21/、D10.8/（仅本卡契约范围），以及I23/、SYS17/、AC2-08/19/21/（not_run）；未签发全系统验收。

本轮证据目录的局部.gitattributes关闭文本换行转换，以保留日志原始字节与SHA-256；仅原始.log保留工具输出的表格填充空格和末尾空行，不按Git空白规则改写或告警。源代码的Rustfmt、ESLint与检查要求均未改变。实现清单同时保留本机文件SHA-256和实现提交的Git blob OID，便于跨平台核对。

## 交付物

- `docs/adr/0021-law-retention-fields.md`：两字段的来源、含义、计算次序、语义关系及四路径自查。
- `law.proto`：EvidenceAllocationV3.9 `actual_retention`；TendencyV3.18 `evidence_absolute`；EvidenceGroupV3.9、RSnapshotV3.6。旧 V2 定义保留并标取代；两级容器完整继承 reserved。
- 描述符规则 `positive`、`at_least_abs`：仅指向必填标量 double，使 q=0、V<|A| 在三语言结构层和浏览器安全 Zod 中一致拒绝，旧消息不加规则。Rust 语义层另核对 h、计数、M/U、B/π/r；带参数检查不从 B−V 反推基数。
- Rust/Python/TypeScript 生成文件；`core-types/tests/p5_34_contract.rs`（7个语义/字节/生命周期/JSON位值测试，已通过）；旧P5-32测试仅调整V2的取代状态断言，旧字段表检查保留。
- `fixture_retention.py`：7 合法、23 非法定向输入；含 q/V 非有限值的完整手工 Protobuf 与有效对照。`test_retention_schema.py`：新规则的目标合法性及冻结检查。`check_retention_browser.ts` 纳入 verify.py，执行浏览器安全 Zod，不打开浏览器。
- `baselines/p5-32/` 已由 P5-22 冻结：核对与 `91b31ed` 的7份 proto 完全相同，保留原文件和黄金摘要，不重复覆盖。
- `docs/traceability/P5-34.md`、V2-62/63；`tasks/requests/P5-34-core-store-v3.md`。
- 所有新建虚拟环境、缓存、编译目录在 D 盘；只用合成输入和临时目录。没有访问谱系数据。

## 2026-10-24 授权扩围后的存储变更

- `core-store/objects.rs`、`store.rs`：新 R 经 V3 专用分段入口，通用入口拒绝快照；保留 V2 的 NSI1 重组和摘要校验，不放开 superseded 新写入。
- `r_state.rs`、`r_validation.rs`、`transaction.rs`：旧 V2 分派保留原解码器；V3 加载 R 所指 LawVersion 核对 B/π/r，验证 q/V 和 0…N−1；阶段与提交再比对绑定规律。普通 stage 不接受旧 V2 head 的直接升级。
- `publication.rs`、`queries.rs`、`recovery.rs`：V3 证据索引、版本保留的历史查询、最新版本先于关联筛选；恢复时对照规范 R/事务核对倾向、组、分配索引。空分配后继不会恢复旧 V2 关联。
- `r_replay.rs::stage_r_replay`：校验全存储和回执，按完整已接受 H 加本次事件建立输入；保留 correction、解释结果和版本绑定，要求声明 genesis、LIFE 摘要匹配，事件/资源预算不足即报错。交由可信核心从有效 H 重建；输入资源视图不提供历代 R 统计对象。输出必须覆盖所有旧组（移除则为空后继），经原事务原子发布。无 q=1／V=B−c 默认转换，无生产数据访问、降格或自动回退。
- `tests/r_fixtures.rs` 明确构造 V3 合成数据，不依赖尚未冻结的新黄金；`r_legacy_fixture.rs` 仅在 cfg(test) 下写合成 V2 历史，公共 API 没有旧写入豁免；`r_migration.rs` 新增6项测试源码。既有对象、分段、单写、提交、隔离和崩溃用例通过共享夹具切到 V3，原断言和故障点保留。
- 新增模块均未超过400行，不新增运行时依赖、IDL字段或 SQLite 表/格式。

**实际重放的界限：** `stage_r_replay` 调用外部可信核心的重放实现；它本身不实现 G。当前 main 的 core-kernel 没有 P5-23/P5-26 的完整有效 H 重放器，因此没有可接入的生产实现。源码测试里的显式替身只检验存储输入、位值和发布边界，不证明 q/V 的规律计算正确。该集成、来源更正案例和 SYS17 仍是未完成项；不得用替身填补。

**并行 PR：** 先核对 #65 的 `cdc1b5e`，发布前再核对最新 head `51267299211ceee82c5bbf9366cf2cdece096921`，其 `queries.rs` 仍追加 `transactions_at`；本卡修改证据查询区段。没有取用它的实现或改它的分支。后合并者 fetch main、保留双方接口并解决冲突，再重跑受影响测试。

## 此前轻量检查（历史状态，当前结果以前节为准）

| 命令或检查 | 结果 | 证据（verification/run-01/） |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | 0 stale | generate-final.log |
| `python core/contracts/check_compat.py --ref origin/main` | p5-01、p5-21、p5-32、current、origin/main 均0违例；141消息、46取代、51枚举 | compat-final.log |
| `python -m unittest discover -s core/contracts -p test_*schema.py` | 12 tests通过 | schema-tests-final.log |
| 定向输入与冻结基线独立审计 | Python 7合法、23拒绝；6个手工非有限消息的有效对照通过；p5-32 7份proto字节相同 | retention-inputs.log、check_retention_inputs.py |
| `pnpm lint`、`pnpm typecheck`；新 browser 脚本独立类型检查 | 通过 | pnpm-lint-after-install.log、pnpm-typecheck.log、retention-browser-typecheck.log |
| `uv run --project core/contracts --locked ruff check core/contracts workers/irl/generated/echo` | 通过 | ruff-final.log |
| CI范围 `mypy --strict`（工具与生成Python） | 13文件通过 | mypy-ci-scope.log |
| `cargo fmt --all -- --check` | 通过；只解析和格式化，不编译 | rustfmt.log |
| Rust语义测试、clippy、完整契约验收与黄金冻结 | 尚未运行，不作通过结论 | 等待串行资源窗口；见 BLOCKED.md |
| 全工作区回归 | 尚未运行；授权后已补消费者代码，仍待编译验证 | 请求 P5-34-core-store-v3.md、BLOCKED.md |

扩围后的轻量检查单独位于 `verification/core-store-v3/`：

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| `cargo fmt -p core-store -- --check` | 退出0；只解析/格式化，未编译 | light-checks.md |
| `git diff --check` | 退出0，无输出 | light-checks.md |
| `check_store_sql.py` | 从源码提取查询，在内存 SQLite 执行7项探针通过；不运行 Rust Store | check_store_sql.py、light-checks.md |
| 新存储测试、既有存储回归、clippy、真实 G 迁移和 SYS17 | 全部 not_run | 等用户资源窗口；真实重放器还待集成 |

保留了两项命令/环境问题的原输出：直接把整个 core/contracts 交给 mypy，模板目录没有包上下文，返回2（mypy.log）；按既有 CI 指定文件及生成包运行则通过（mypy-ci-scope.log）。首次 pnpm lint 的自动依赖同步拒绝复用的 node_modules junction，脚本最终退出0但日志有安装警告（pnpm-lint.log），不把该记录用作验收。随后仅移除本工作区的两处 junction，以现有 D盘 store 离线安装锁定依赖，再正式运行 lint/typecheck 通过（pnpm-install.log、pnpm-lint-after-install.log）。安装未新增依赖，忽略依赖安装脚本；没有跳过测试或断言。

同步 main `0b5412e` 后重跑生成、兼容、12个schema测试、lint、typecheck，退出码全0，证据在 `verification/after-main-0b5412e/`。工具链：Node24.21.0、pnpm12.11.2、Python3.12.10、uv0.12.15；契约锁定环境为CPython3.12.14。Rust工具链尚未实际用于本卡编译，不报告其验收结论。

2026-10-24 授权扩围前的交接只复核已有日志、提交、范围和编号；随后按新授权完成上述存储代码和轻量检查，黄金文件仍未改。机械性日志/路径/编号及轻量命令核验按角色名调用 `routine_reporter`（配置为 gpt-6-luna/max），主代理核对返回证据；不将其当成交叉审查。Windows 工作流对草稿 PR 跳过自托管作业，本轮不手动触发 CI。cargo fmt 命令仅在进程环境指定 `CARGO_TARGET_DIR=D:\Dev\cache\cargo-target\P5-34`；未配置用户级环境变量。

## 规格对照

见 `docs/traceability/P5-34.md`。q 追溯 ADR 0019 §5/§6/§8.5、ADR 0021 §2；V 追溯 ADR 0019 §1/§8.2/§8.5、ADR 0021 §1–3；共同追溯 v2.0 §5.2。存储扩围追溯 ADR 0021 §7；完整 G、来源更正与容量仍需其归属卡实现，生产迁移集成尚未完成，不签发 AC2-08/19/21、I23 或 SYS17。

`semantic.rs` 为403行：保留旧 V1/V2 API和语义行为，在同一纯函数接口文件追加V3检查，便于对照三代与复用既有计数检查；没有把旧行为改成V3语义。生成文件和既有大文件 fixture_inputs.py 的长度由生成/已有向量组织决定；新定向向量独立放在 fixture_retention.py。

## 新增依赖

无新增依赖或版本变化，Cargo.lock未变。现有serde_json 1.0.154（MIT OR Apache-2.0）启用float_roundtrip特性以保留JSON binary64输入，该特性自身无新增依赖。

## 待用户确认

- [ ] （阻塞）ADR 0021、契约变更与 V2-62 起的登记。
- [ ] （阻塞）实现方之外的交叉审查。
- [x] 用户已授权在本 PR 完成 core-store 消费者迁移，无须重复授权。
- [ ] （阻塞）接入真实有效 H 重放器，并完成迁移集成验证；当前仅实现存储边界，不默认补字段。
- [x] 本机完整Rust工作区、三语言契约及静态检查已通过；真实G集成仍单列阻塞。

## 后续建议（不在本任务范围内）

- P5-23 合并本卡后改用生成的 V3 类型，证明完整 G 的重放、分摊、容量与来源更正。
- ADR 0021 自查中，潜在变化检测器后验/裁剪状态及资源版本只列出，交用户结合 P5-33 决定，没有自行加入。

## 下一步

本机验收及证据已补齐，更新草稿PR并写「请审查」后停止，保持draft/needs-user，不合并、不取下一卡。等待交叉审查、用户批准及真实有效H重放器集成；runner已停止，用户可恢复。
