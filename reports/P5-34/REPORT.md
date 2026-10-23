# P5-34 分配的实际保留率与倾向的证据量 — 报告

## 当前状态

部分完成。草稿 [PR #72](https://github.com/nanake-lab/nemu/pull/72) 保持 `needs-user`。基于 origin/main `541be9e`，之后已 merge 同步 `0b5412e`（无冲突）；2026-10-24 本轮再次 fetch，main 仍为 `0b5412e`，没有缺少的 main 提交，ADR 0021、V2-62/63 没有撞号。IDL、生成类型、语义检查、定向输入与测试源码已有实现，现有静态检查日志通过；Rust 编译、黄金向量冻结及完整三语言验收尚未启动。**用户2026-10-24本轮再次明确「继续等待空闲通知」：不启动编译，不自动恢复。** 另有 core-store V3 消费者迁移的卡外范围请求。本轮将现有实现和阻塞说明提交到草稿 PR，写「请审查」供初步审查；全部验收尚未完成，不能合并。不取下一张卡。

## 交付物

- `docs/adr/0021-law-retention-fields.md`：两字段的来源、含义、计算次序、语义关系及四路径自查。
- `law.proto`：EvidenceAllocationV3.9 `actual_retention`；TendencyV3.18 `evidence_absolute`；EvidenceGroupV3.9、RSnapshotV3.6。旧 V2 定义保留并标取代；两级容器完整继承 reserved。
- 描述符规则 `positive`、`at_least_abs`：仅指向必填标量 double，使 q=0、V<|A| 在三语言结构层和浏览器安全 Zod 中一致拒绝，旧消息不加规则。Rust 语义层另核对 h、计数、M/U、B/π/r；带参数检查不从 B−V 反推基数。
- Rust/Python/TypeScript 生成文件；`core-types/tests/p5_34_contract.rs`（6 个语义/字节/生命周期测试，待运行）；旧 P5-32 测试仅调整 V2 的取代状态断言，旧字段表检查保留。
- `fixture_retention.py`：7 合法、23 非法定向输入；含 q/V 非有限值的完整手工 Protobuf 与有效对照。`test_retention_schema.py`：新规则的目标合法性及冻结检查。`check_retention_browser.ts` 纳入 verify.py，执行浏览器安全 Zod，不打开浏览器。
- `baselines/p5-32/` 已由 P5-22 冻结：核对与 `91b31ed` 的7份 proto 完全相同，保留原文件和黄金摘要，不重复覆盖。
- `docs/traceability/P5-34.md`、V2-62/63；`tasks/requests/P5-34-core-store-v3.md`。
- 所有新建虚拟环境、缓存、编译目录在 D 盘；只用合成输入和临时目录。没有访问谱系数据。

## 验收

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
| 全工作区回归 | 尚未运行；core-store V2生产路径已发现静态阻塞 | 请求 P5-34-core-store-v3.md |

保留了两项命令/环境问题的原输出：直接把整个 core/contracts 交给 mypy，模板目录没有包上下文，返回2（mypy.log）；按既有 CI 指定文件及生成包运行则通过（mypy-ci-scope.log）。首次 pnpm lint 的自动依赖同步拒绝复用的 node_modules junction，脚本最终退出0但日志有安装警告（pnpm-lint.log），不把该记录用作验收。随后仅移除本工作区的两处 junction，以现有 D盘 store 离线安装锁定依赖，再正式运行 lint/typecheck 通过（pnpm-install.log、pnpm-lint-after-install.log）。安装未新增依赖，忽略依赖安装脚本；没有跳过测试或断言。

同步 main `0b5412e` 后重跑生成、兼容、12个schema测试、lint、typecheck，退出码全0，证据在 `verification/after-main-0b5412e/`。工具链：Node24.21.0、pnpm12.11.2、Python3.12.10、uv0.12.15；契约锁定环境为CPython3.12.14。Rust工具链尚未实际用于本卡编译，不报告其验收结论。

2026-10-24 本轮交接只复核已有日志、提交、范围和编号，`git diff --check origin/main...HEAD` 无输出；没有重跑测试，也没有改实现或黄金文件。机械性日志/路径/编号核验按角色名调用 `routine_reporter`（配置为 gpt-6-luna/max），主代理核对返回证据；不将其当成交叉审查。Windows 工作流对草稿 PR 跳过自托管作业，本轮不手动触发 CI。Cargo target 没有作用户级配置；未来获准编译时仍只在启动命令的进程环境中指向 D 盘。

## 规格对照

见 `docs/traceability/P5-34.md`。q 追溯 ADR 0019 §5/§6/§8.5、ADR 0021 §2；V 追溯 ADR 0019 §1/§8.2/§8.5、ADR 0021 §1–3；共同追溯 v2.0 §5.2。完整 G、来源更正、容量、生产迁移仍由其归属卡验证，不签发 AC2-08/19/21 或 SYS17。

`semantic.rs` 为403行：保留旧 V1/V2 API和语义行为，在同一纯函数接口文件追加V3检查，便于对照三代与复用既有计数检查；没有把旧行为改成V3语义。生成文件和既有大文件 fixture_inputs.py 的长度由生成/已有向量组织决定；新定向向量独立放在 fixture_retention.py。

## 新增依赖

无。

## 待用户确认

- [ ] （阻塞）ADR 0021、契约变更与 V2-62 起的登记。
- [ ] （阻塞）实现方之外的交叉审查。
- [ ] （阻塞）core-store 的 V3 消费者迁移安排：扩大范围或另开配套卡。未获授权不改 core-store。
- [ ] （阻塞）补齐尚未运行的 Rust、黄金冻结与完整三语言验收后再评定完成。

## 后续建议（不在本任务范围内）

- P5-23 合并本卡后改用生成的 V3 类型，证明完整 G 的重放、分摊、容量与来源更正。
- ADR 0021 自查中，潜在变化检测器后验/裁剪状态及资源版本只列出，交用户结合 P5-33 决定，没有自行加入。
