验证档位：重档；本 PR 修改 Echo Core IDL、规范字节与三语言生成类型。

# P6-17 触摸停留契约 — 报告

## 当前状态

契约实现完成，本机验收通过。已 merge `origin/main=67cc296c8b960ae429aceb1f1e0110ef0dbc2f9e`，合并提交 `11e7c639efb271762cb0ad8352b38cfc12e81b0d`。契约实现提交为 `72d5e7fbf0979c5d72ad705aed00146ddf3075f9`；最终源码为 `e92a9be22d9da5cba1a1cfc8783751c8abf106f9`，只追加了取代消息登记表测试修正，生产源码与冻结字节不变。部分检查开始时是相同源码的未提交工作树。P5-34 #95、P5-36 #108 已合并，原目录互斥阻塞解除。

交付前 main 增量核对：实际验证基线 B=`67cc296c`，最新 M=`ee5c2b6068b9a2200ad5fd48ebbc8e337458c0d5`，B 为 M 祖先。新增 #127 仅记录条件载体切换、P0-12 实测与 P5-13 依赖，不改变本卡触摸契约、代码、测试、工具链或验收要求；按 AGENTS §6 不再合入或重跑，实际验证基线仍为 B。[核对记录](acceptance-20261101/main-impact.json) 与 [完整增量](acceptance-20261101/main-increment.patch)。

PR 保留 `needs-user`。独立交叉审查与用户合并批准尚未完成；按本次指示，本机验收通过后推送、转为可审查并写「请审查」，由用户安排独立会话。执行者不自行合并。

## 交付物

- `core/contracts/terminal.proto`：`TouchObservationV2` / `TouchKindV2`；停留开始、持续、结束均携带非空 `contact_id` 和有限、非负的累计 `elapsed_ms`。单条记录的重复投递仍用自己的 `stable_source_key`。
- 新冻结标签 `echo.touch-v2`，`schema_version=1`；保留字段 1、2、4，旧 kind 的字段 3 reserved，新 kind=5、contact_id=6、elapsed_ms=7。旧枚举的 1–6 语义不变，新枚举增加 7–9；旧 `TouchKind` 不增改，旧 `TouchObservation` 标为取代并保留解码。
- Rust、三个 worker 的 Python、TypeScript 生成物由同一 IDL 生成；验证复用既有 descriptor/cases/min/nonempty 边界，没有新增手写业务校验层。
- `fixture_touch.py` 与黄金向量追加 12 合法、26 非法；覆盖全部九种 kind、三种停留的缺字段/空标识/负时长、完整消息中的 NaN/±Inf、新旧 Protobuf/CanonicalV1 混用。`check_touch_browser.ts` 加入既有验收入口。
- `docs/CONTRACTS.md` 只扩充 §4.2 的 touch 行，注明 ADR 0033 §3 的鼠标停留不自动躲开；[规格对照](../../docs/traceability/P6-17.md)。

## 本机验收

命令、时间、退出码与原始日志摘要见 [commands.jsonl](acceptance-20261101/commands.jsonl)，最终 18 个源码文件的 blob 与字节哈希见 [源码快照](acceptance-20261101/tested-source-final.json)；[交付核对](acceptance-20261101/delivery-audit.json)、[证据哈希清单](acceptance-20261101/manifest-sha256-final.json)。以下使用临时合成数据；没有打开排练或正式存储。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | 0 stale | [generate](acceptance-20261101/generate.log) |
| `python core/contracts/check_compat.py --ref origin/main` | p5-01/21/32/34/35、current、origin/main 全部 0 违例 | [compat](acceptance-20261101/compat-final.log) |
| `pnpm test:core-contracts`（D10.8 / SYS21） | Rust/Python/TypeScript 各 13,488 例通过；527 protoc 向量、14 schema 测试、32 项新浏览器边界检查通过 | [conformance-02](acceptance-20261101/conformance-02.log) |
| 同套件的 P5-36 回归 | 150 项预算检查、65 个实际状态前缀与 260 份跨语言返回状态恢复/续算通过 | [原始状态与结果](acceptance-20261101/theta-restore/result.json) |
| `python reports/P6-17/verify_bytes.py` | 旧 515 合法/597 非法向量保持严格前缀；12 个新合法向量的 CanonicalV1、Protobuf 与域分离摘要独立逐字段复核一致 | [independent-bytes](acceptance-20261101/independent-bytes.json) |
| 契约与生成 Python 的 Ruff / strict mypy | 通过 | [ruff](acceptance-20261101/ruff.log)、[mypy](acceptance-20261101/mypy.log) |
| Rust workspace fmt / Clippy `--all-targets -- -D warnings` | 测试断言修正后重新通过 | [fmt](acceptance-20261101/rust-fmt-02.log)、[Clippy](acceptance-20261101/rust-clippy-02.log) |
| Rust workspace tests（分段完成） | kernel 173、store 51、core-types 34、其余 crate 75、文档测试 5 项通过；kernel 保留 1 项既有 ignored | [kernel/store](acceptance-20261101/rust-test-02.log)、[修正后的 core-types](acceptance-20261101/rust-remaining.log)、[其余 crate](acceptance-20261101/rust-remaining-02.log)、[全部文档测试](acceptance-20261101/rust-doc-all.log) |
| TypeScript lint / typecheck / tests / contracts coverage | 全通过；工作区 52 文件/580 测试，契约包 8 文件/354 测试；既有 coverage 门槛通过 | [lint](acceptance-20261101/pnpm-lint.log)、[typecheck](acceptance-20261101/pnpm-typecheck.log)、[tests](acceptance-20261101/pnpm-test.log)、[contracts](acceptance-20261101/pnpm-contracts.log)、[coverage JSON](acceptance-20261101/ts-coverage-summary.json) |
| 三个 worker 的 Ruff / strict mypy / pytest | 9 项命令全部通过；每个 worker 的导入 smoke test 各 1 项通过 | [irl](acceptance-20261101/irl-test.log)、[drive](acceptance-20261101/drive-test.log)、[model-adapter](acceptance-20261101/model-adapter-test.log)；静态检查命令与摘要见 commands.jsonl |

### 失败与中断

- 首轮契约套件的 Rust、Python 已通过，TypeScript 子进程启动失败：旧 `.test-build` 指向 D 盘缓存，缓存中的模块解析找不到仓库已安装的 `zod`。仅补齐该缓存到现有 `packages/contracts/node_modules` 的目录链接，同一套件重跑通过；没有增加依赖或更改全局配置。[原始失败](acceptance-20261101/conformance.log)、[明确错误](acceptance-20261101/ts-probe-diagnostic.log)、[修复位置](acceptance-20261101/cache-link-repair.json) 均保留。
- 首轮全工作区 Rust 测试的执行会话消失，过程记录未写回退出码，相关测试进程已不在。部分日志含 kernel 173 通过/1 个既有 ignored，未见断言失败，但不能据此判定整个工作区通过；原因未确定。保留 [原始部分日志](acceptance-20261101/rust-test.log)，完整套件重新执行并单列日志。
- 第二轮全工作区 Rust 测试在旧 `superseded_registry` 数量断言失败（实际 48、期望 47）；[原始失败](acceptance-20261101/rust-test-02.log) 保留。`core-types/tests/v2_contract.rs` 把新 TouchObservation 前后继状态纳入断言并更新数量，没有删除旧断言或放宽契约校验。该测试重跑通过；已通过且源码未变的 kernel/store 单元结果继续有效，其余 crate 全套与全部文档测试另行通过。
- 修正后的续跑在宿主更新后再次失去会话（工具返回 stale host generation / unknown process id），没有退出码，也没有存活测试进程。[部分日志](acceptance-20261101/rust-remaining.log) 保留：core-types 的单元及四组集成测试已全部通过，fact-view 未完成；整条命令不计通过。未完成的 crate 用 `cargo test --workspace --exclude core-kernel --exclude core-store --exclude core-types --locked --offline -- --test-threads=1` 通过；随后 `cargo test --workspace --doc --locked --offline -- --test-threads=1` 通过。此分段覆盖全部工作区，没有取消未运行的测试。
- 暂存检查曾把原始 stdout 的对齐空格、尾部空行及 diff 上下文空格报为格式问题，[原始输出](acceptance-20261101/staging-whitespace.log) 保留。按仓库已有原始日志规则，仅在本次证据目录对 `.log` / `.patch` 设置 `-whitespace`；证据字节不改，源代码与文档仍做空白检查。

## 边界、迁移与拖动自查

停留接触 ID 为 G/EV2 归组和 C/N_e 单次计数提供输入；跨记录的同 ID、同部位、累计时长顺序及实际归组/计数由后继终端/核心路径落实。本卡没有运行 G/C 行为验收，不能据三语言字节一致宣称 AC2-03/14 的停留端到端通过。门限、节奏和事件转换属于 P6-02；鼠标停留的实际身体调控属于 ADR 0033 的身体后继，均未在本卡实现。

拖动开始/结束也需要可核验的同次关联，才能避免仅凭部位或时间推断同一起因；现有 `parents` 未规定这一关联，重投 key 只识别同一条记录。建议后续由用户决定是否给拖动扩充关联。**本卡未加入该扩展**：非停留不带 contact_id/elapsed_ms，相关拒绝向量明确锁住当前范围。这一选择不阻塞已授权的停留契约交付。

上线时必须先部署支持新标签的接收方，再由 P6-02 启用新生产者；依 ADR 0007 做本机备份。旧记录继续按旧标签解码，不重写历史摘要。回退时停止新生产者、使用兼容接收方或恢复升级前备份，保留后继字节，禁止将它们当旧 touch 解码。本卡只增加类型，没有存储格式或生产写入路径变更，也未执行真实存储迁移；不签发 SYS17。

## 其他说明与待确认

- 没有新增运行时依赖，没有新增 ADR 或差异登记编号。
- 原有超过 400 行的 `terminal.proto`、`fixture_inputs.py` 与自动生成文件沿用既有结构；本次定向夹具单独放在 `fixture_touch.py`，避免继续扩充夹具主体。
- [ ] （合并前）独立交叉审查本次新字节及兼容边界。
- [ ] （合并前）用户批准契约变更并合并 PR。
- [ ] （后续可选）决定拖动关联是否另开契约后继。

旧 2026-10-26 的预读证据保留在 `verification/`，不作为本次新契约验收。当前剩余项见 [BLOCKED](BLOCKED.md)。
