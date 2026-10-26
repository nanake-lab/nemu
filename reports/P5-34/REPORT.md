# P5-34 分配的实际保留率与倾向的证据量 — 报告

## 结果

已完成用户批准的分阶段实现与本地验收；最终提交的 CI 门控见 BLOCKED.md。[PR #95](https://github.com/nanake-lab/nemu/pull/95) 保持 draft/needs-user，最终 head 的 CI 全绿后才转为可审查并写“请审查”，由用户合并。原 PR #72 已关闭，原分支保留；本次只 merge、不 rebase、不强推。先合入 main b4eb7fd（merge 3418159），再合入最新 origin/main 86779b1（merge 2fa1bc9）；存储实现提交 4342065。

用户明确决定：新写入 V3；V2 历史只读、可解码；旧 V2 head 拒绝继续写。当前没有正式或排练存储，现存仅测试数据（用户提供，本卡未访问用户数据）。本次不做 V2→V3 数据迁移。将来需要时由 **P5-26** 按 v2.0 §2.2 用规律 G 从 H 整段重放得到新 q/V 快照，**不另开迁移卡**。这一决定不是用默认 q/V 转换旧状态的授权。

## 交付物

- core/contracts/law.proto：EvidenceAllocationV3.9 actual_retention 保存历史 q；TendencyV3.18 evidence_absolute 保存独立 V；EvidenceGroupV3 保留 7/8，RSnapshotV3 保留 4/5。V2 标取代，原定义、标签和字节保留。
- core-types 的编解码与纯语义检查：q 有限且在 (0,1]，V 有限且 ≥|A|；绑定规律参数核对 B、π、r 的有限 binary64 计算次序；分配序号恰为 0…N−1。既有 serde_json 开启 float_roundtrip 保留 JSON 位值。
- 三语言生成类型、运行时及浏览器安全 Zod；黄金向量只追加 19 合法、35 非法，总计 464 合法、492 非法、18 Unicode。新分配/倾向/证据组/快照的合法与非法数分别为 6/10、5/11、4/7、4/7。p5-32 冻结基线保留。
- ADR 0021 §11、差异登记 V2-62/63 接续说明、docs/traceability/P5-34.md 与两份接续请求；不新增检测器/θ 字段或 G 算法。
- verify-protected.ps1、verify-main-supervisor.ps1、summarize_protected.py 与 verification/protected-20261026/：命令、时间、提交、退出码、日志 SHA-256、逐文件源码/基线哈希及 main 对照。旧运行目录不覆盖。

### 受保护路径 core-store（必须一并交叉审查）

这是用户专项批准的最小契约连锁改动，完整逐文件必要性见 [PROTECTED-SCOPE-20261026.md](PROTECTED-SCOPE-20261026.md)。没有恢复原 PR #72 的存储侧规律校验/重放回调扩围。

| 文件（相对 core/crates/core-store/src/） | 必要性 |
| --- | --- |
| objects.rs、store.rs | 专用 R 生产切至 V3；读取 NSI1 时保留 V2/V3 原解码；通用准备入口拒绝两代 R；旧 V2 head 在 writer 配置/回执修复前明确拒绝 |
| transaction.rs、publication.rs | staged R、可写 head 的 revision 和新发布的倾向/证据组索引切至 V3 |
| queries.rs、lib.rs | 历史证据查询返回包裹生成消息的 EvidenceGroupRecord::V2/V3，保留原代际；最新版本优先和完整预算查询逻辑不变 |
| tests/mod.rs、objects.rs、crash.rs、crash_harness.rs | 当前生产夹具与故障标签切至 V3；改变 A 的夹具同步声明 V≥|A|；原断言含义、等待与资源阈值不变 |
| tests/legacy_v2.rs、retention_versions.rs | 临时历史 V2 夹具和新旧边界回归；生产 API 无旧写入逃生口 |

共 12 个 core-store 文件。SQLite 格式版本/表结构、NSI1 物理布局、事务与预算行为未改变。新增逻辑对象使用独立 V3 标签和摘要；物理布局不变不等于完成语义迁移。受保护源码不含 G 实现、自动迁移、默认 q=1 或 V=B−c。

## 验收

本轮证据在 verification/protected-20261026/，每项 log/json 对记录原始命令、退出码及 SHA-256。Rust/Cargo 1.99.0 与仓库锁定一致；Node 24.21.0、pnpm 12.11.2；系统 Python 3.12.10，uv 项目 Python 3.12.14。缓存、编译、临时数据在 D 盘；重负载串行。routine_reporter（GPT-6 Luna/max）仅执行指定轻量检查/哈希/只读 CI 查询，不作验收或合并裁决；本轮未另行验证客户端是否自动加载 TOML 职责文件，不宣称该能力已验证。

| 命令或检查 | 结果 | 证据（相对 verification/protected-20261026/） |
| --- | --- | --- |
| python core/contracts/generate.py --check | 通过，0 stale | attempt-01/generate.log |
| python core/contracts/check_compat.py --ref origin/main | 合并后通过；p5-01、p5-21、p5-32、current、origin/main：0 违例 | attempt-02/compatibility.log |
| 旧黄金与新增字节独立复核 | 旧 445/457/18 前缀不变；新增 19 合法向量独立复算 0 差异 | attempt-01/bytes.log |
| 定向输入与 p5-32 基线 | 7 合法、23 非法、6 非有限对照；7 proto 与发布版本相同 | attempt-01/inputs.log |
| pnpm test:core-contracts（D10.8/SYS21 本卡范围） | 通过；12 schema 测试、每语言 11592 例、protoc 464、浏览器 Zod 106 | attempt-01/contracts.log |
| cargo test -p core-store p534_ | 通过，3/3；首次夹具构造错误 2/3 的原始失败保留，修复后复验 | attempt-01/storage.log、attempt-02/storage.log |
| cargo test --workspace --locked -j1 -- --test-threads=1 | 通过；kernel 45、store 34、types 31 及 2 个编译失败示例；store 1252.01 秒 | attempt-01/workspace.log |
| cargo clippy --workspace --all-targets -- -D warnings | 合并后复验通过 | attempt-01/clippy.log、attempt-02/clippy.log |
| cargo fmt --all -- --check | 通过，最后源码格式复核通过 | attempt-01/fmt.log、attempt-02/fmt.log |
| pnpm lint | 通过 | attempt-01/lint.log |
| pnpm typecheck / test / build | 全通过；Node 52 文件/580 项 | attempt-01/typecheck.log、test.log、build.log |
| IDL/生成 Python ruff、mypy --strict | 通过，mypy 12 文件 | attempt-01/ruff.log、mypy.log |
| 三 worker ruff、mypy --strict | 全通过，各 2 个源/测试文件 | attempt-01/{irl,drive,model-adapter}-{ruff,mypy}.log |
| 三 worker uv run pytest | 各 1 项通过；仅脚手架级用例 | attempt-01/*-pytest.log |
| 固定 main b4eb7fd 的三次完整 pnpm test | 3/3 通过，每次 52 文件/580 项；未复现 supervisor 失败 | main-test-04/05/06.log、supervisor-comparison.json |
| 最终 head CI | 转为可审查前须核实 Ubuntu 与手动 Windows 全部通过，跳过不算通过 | PR #95 checks；最终“请审查”评论固定 head 与 run |
| 完整 G 重放、AC2-08/19/21、I23、SYS17 | not_run，不用存储组件通过替代 | 本报告；各编号目录的本轮结果 |

合并 P5-25（86779b1）仅新增 core-kernel 动力学及其文档/报告。存储、core-types、契约、Cargo 清单/锁文件、packages、workers、任务卡/规则的 Git 对象逐项相同，证据见 main-refresh.json。合并后重跑全目标 Clippy、兼容门与 `cargo test --workspace --exclude core-store`：core-kernel 66 通过、1 个 main 已有的 V2-93 旧 Python 舍入诊断 ignored，core-types 31 通过。该 ignored 由 main 带入，本卡没有新增/修改忽略标记，不把它算作通过；证据为 attempt-01/workspace-without-storage.log。未变化的 34 项存储本地完整结果保留，最终 CI 仍在同一最终 head 执行完整 core-store 检查。

### supervisor 对照与原失败

先前本分支 pnpm test 在 supervisor.test.ts:199 观察到 PID 87292，579 通过/1 失败；随后 PID 退出不撤销断言时失败。原日志留在 verification/resumed-20261026/attempt-01/test.log。

main 的三次正确调用均通过，因此不声称“main 同样失败”或已定位根因。失败 head da945d5、main b4eb7fd 与当前实现的 supervisor 七文件及 package.json、pnpm-lock.yaml、vitest.config.ts 共十个 Git blob 完全相同；运行依赖为本地 supervisor 模块和 Node 内建 API，不调用本卡 IDL/Rust 存储逻辑。没有发现本卡引入该失败的代码证据。Ubuntu CI 会跑根 pnpm test 并包括 supervisor；Windows workflow 跑 transport/isolation/e2e/visual/W02，不跑根 Vitest。该原失败作为未定位的 Windows 退出检查波动保留，supervisor 代码、设置和断言均未修改。

### 本轮保留的失败与验证脚本问题

- storage attempt-01：旧历史夹具误以为可选 environment.law_version 必定存在，构造时 unwrap(None)；两项其他边界测试通过。夹具改为引用当前已准备 R 对象实际使用的 LawVersion；未改生产代码、断言或阈值。attempt-02 三项通过。
- main-test-01/02/03：报告验证脚本把单参数拆开，实际命令为 vitest run e s t；虽然都跑了 580 项，仍不作正式 main 对照。修正报告脚本参数数组后，按预先声明的三次固定运行执行 04/05/06，未改测试本身，未运行到成功才停。
- 合并后验证脚本新增带连字符的步骤名时未加 PowerShell 引号，解析提前失败，实际测试未启动；修正报告脚本的键名后通过。原错误摘录见 harness-errors.json。
- resumed-20261026 中的 Rust V2 新写入拒绝和 supervisor 失败，以及旧 PR #72 的全部证据保留。只以本轮各项实际结果判定当前实现。

## 规格对照

完整对照见 docs/traceability/P5-34.md 的当前分阶段追加节；ADR 0021 §11 是受保护存储范围的最新决定，§7/9 属旧 PR 历史。

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| v2.0 §5.2；ADR 0019 §5/6/8.5；ADR 0021 §2–4 | actual_retention；显式历史 q，不由当前支持度倒推 | q=0/>1/非有限拒绝、次正规数与相邻值、修改支持度后 q 保留、序号正反例 |
| v2.0 §5.2；ADR 0019 §1/8.2/8.5 | evidence_absolute；B=c+V、π 固定括号与顺序 | V<|A| 双符号、非有限、B 舍入丢失、小量 M/U、依恋分母分组 |
| ADR 0007/0016；ADR 0021 §11 | 四后继消息、reserved、V2 解码与 V3 写入分派 | 三基线/当前兼容门、旧字节复核、旧 R 平面/分段读取、旧 head 拒写后持久字节不变 |
| SYS21；施工合同 10.8 | 三语言共享描述符、有限 binary64、浏览器 Zod | 黄金、对抗、protoc 与语义正反例 |
| v2.0 §2.2；I23；SYS17 | 将来 P5-26 的 G/H 重放与迁移/失败回退 | 本卡 not_run；当前拒写保留旧数据不是完整迁移验收 |

I18：q/V 在显式版本化 IDL 中，无隐藏持久状态。I23：本卡证明字段/计算分组可表达和逐位保存，不证明完整历史重算。I25：契约序号/集合检查不代替 G 同源/分摊算法。I02：沿用原单写者与外层事务。I29：所有存储测试只用 TemporaryRoot，从未打开用户正式/排练存储。LAW-6、故障断言及验收阈值未改。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| 无新增；既有 serde_json 启用 float_roundtrip | 1.0.154 | MIT OR Apache-2.0 | 保持 JSON binary64 输入位值 |

无依赖或锁文件新增。超过 400 行的 semantic.rs 保留旧 V1/V2 检查并追加 V3；生成文件按既有生成器组织，避免重复维护跨进程类型。新存储文件均未超过 400 行。

## 待用户确认

- [ ] （阻塞合并）ADR 0021、V2-62/63 与具体 q/V 后继契约，包含上述 core-store 受保护路径，一起交叉审查并由用户合并。实现者不自行审查或合并。
- [x] 分阶段范围已获用户明确批准；不再请求迁移授权或另开迁移卡。
- [ ] （不阻塞本卡）后续维护者定位原 supervisor 退出检查波动；原失败不隐藏，测试不放宽。

## 后续建议（不在本任务范围内）

P5-26 需要迁移时，从完整有效 H 运行 G 重建 q/V，并完成备份、失败回退与 SYS17；不能把本卡结构夹具当成真实规律结果。新变化检测器/θ 状态与当前已有算法决定由所属卡处理。PR #65 共享 queries.rs 的追加接口尚未合入，后合并方按最新 main 保留双方改动；本 PR 不修改其分支。

## 下一步

本地验收已完成；同一最终 head 的 Ubuntu 与 Windows 全量 CI 确认通过后才转可审查并写“请审查”。保持 needs-user，等待用户安排独立交叉审查及合并。本卡不领取下一张卡。最终 CI 的提交与 run 链接固定在 PR 的交付评论，不能用旧提交的绿灯代替。
