# P5-36 θ 估计器状态的存储 — 报告

## 当前状态

首次完整审查指出的 Rust θ 固定 Limits 不一致已修复，配置贯通语义边界、参数摘要、kernel 桥接及存储入口。写锁改为显式释放并保留原始错误；新增 Linux 描述符生命周期反例和 128 次并发 spawn/drop/reopen 验证。65 前缀真实存储恢复/续算、完整 Rust 工作区及静态检查均通过。当前提交 Ubuntu/Windows 必要 CI、Linux 原始锁证据及复审请求以 [PR #108](https://github.com/nanake-lab/nemu/pull/108) 最终交付评论为准，不为回填 CI 结果反复提交触发检查。等待用户安排独立复审；本会话不自行批准、合并或取下一卡。首次审查和修复逐项说明见 [review-fix](review-fix/README.md)。

继续原分支 `task/P5-36-theta-estimator-idl`，已合入 main / #110 `f24f1256259c998bc213a9dc93103fcfb9cfe89a`。本轮 fetch 后 M 仍等于各项验证基线 B，没有新 main 增量。其他开放 PR 的实际路径没有占用 core-store/fact-view；#49 的 kernel law 子目录与本卡 storage 测试不重叠。见本次 [范围核对](review-fix/scope.json)（旧阶段记录仍保留）。旧 B1/B2 已解除。

## 交付物

- `core/contracts/theta.proto` 与三语言生成物：5 个新消息，ThetaSnapshotV2 内嵌完整参数、摘要和估计器。旧 θ 字段、标签及规范字节保留，旧消息 superseded；冻结 `baselines/p5-35`。
- `core-types/src/semantic/theta.rs` 及 Python/TS 对应实现：有限性、布局、质量、计数、时间和完整参数摘要校验。结构解码不能替代显式语义检查。
- 已批准的 kernel 三路径：完整导出/恢复逐字段复制已存位值；核对独立传入的完整规律引用和 law/fuses，不从旧状态/H 回填，不重新计算中间量。C 算法和字段可见性未改。
- `core-store`：新对象写入/读取调用完整 θ 语义检查；暂存/当前头绑定使用 V2；按生成类型标签识别旧/新快照，未知 θ 类型拒绝。旧头在可写连接打开前以只读预检明确拒绝，保留原 DB/WAL/历史字节；保留 superseded、ReadTicket/CommitCapability 和 SQL 原子提交边界。
- `core-store` / `fact-view` 夹具同步 V2。验证复用原 C 轨迹、私有字段位比较器、历史发布夹具和故障点，没有增加生产写入绕过入口或复制递推算法。
- ADR 0029、V2-107–109、接口请求及规格对照追加批准的存储范围。原始失败证据不删改；报告按当前 AGENTS §6 整理为当前状态，旧叙述可查 Git 历史。

`dynamics/storage/tests.rs` 为 565 行：在唯一获批 kernel 测试文件内保留完整结构解构/逐字段 `to_bits`、原轨迹、续算及跨语言/存储返回模式；新增状态字段会迫使断言更新，未另建第二套位比较器。本次增加的大事件/规律 ID 资源预算回归也复用这些私有字段断言。

## 验收

已有契约/桥接证据来自 `6be25832`（后继纯报告提交 `4ceb7e69`）。此前 B3 的影响判断保留在 [storage/validation-impact.json](storage/validation-impact.json)。本次修改 Rust 语义入口/桥接/存储以及验证脚本，故已重跑契约套件、真实恢复链及完整 Rust 工作区。IDL、生成物、黄金、基线和依赖未改；新生成的跨语言原始状态与既有证据逐字节相同，哈希见 review-fix/cross-language-reuse.json。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | 通过；0 stale，本次重跑 | `review-fix/contracts.log` |
| `python core/contracts/check_compat.py`，全部冻结基线 | 通过；p5-01/21/32/34/35/current 0 违例 | 同上 |
| `pnpm test:core-contracts` | 通过；三语言各 13,224 例、protoc 515 向量、14 schema 测试 | `review-fix/contracts.log` |
| 每种新消息 ≥3 合法、≥3 非法 | 通过；参数 3/18、时间 3/6、末事件 3/6、估计器 3/27、快照 3/12 | `implementation/message-coverage.json` |
| V2-109：真实状态跨两种编码及 Python/TS 返回后恢复续算 | 通过；65 前缀、260 返回状态，全部状态位/变化/报警/界一致 | `review-fix/contracts.log`、`cross-language-reuse.json`；原始数据复用 `V2-109/` |
| V2-109：`storage/verify-storage.ps1 -Output <新证据目录>` | 通过；65 真实前缀全部提交/关闭重开/读取，原私有字段逐位比较及全后缀续算一致 | `review-fix/storage-roundtrip/`；复用原 `storage/verify-storage.ps1` |
| 旧 θ 原字节读取与拒写；非法新快照/旧暂存/错误类型；提交前故障 | 通过；含有/无未 checkpoint WAL、物理维护入口不得绕过；失败无部分发布 | `review-fix/workspace.log`；原失败日志仍在 `storage/` |
| `cargo fmt --all -- --check`；`cargo clippy --workspace --all-targets --locked -- -D warnings` | 最终检查通过 | `review-fix/fmt-final.log`、`clippy.log` |
| `cargo test --workspace --locked --no-fail-fast -- --test-threads=1` | 通过，exit 0；store 51/51、fact-view 18/18、kernel 90 通过/1 既有 ignored；全部文档测试通过 | `review-fix/workspace.log`、`workspace-result.json` |
| `pnpm lint` / `pnpm typecheck` / `pnpm test` | 通过；52 文件、580 测试；本次未改 TS，沿用原证据 | `implementation/pnpm-lint-01.log`、`pnpm-typecheck.log`、`pnpm-test.log` |
| 契约 Python ruff/mypy strict；三个 worker 静态检查/pytest | 通过；各 worker 1 个既有骨架测试；worker/生成实现未改，沿用其证据 | `implementation/ruff-01.log`、`mypy-02.log`、`workers-results.json` |
| 共同 4 MiB 预算边界：1 MiB / 1 MiB+1，encode 与两种 decode | 通过；三语言新增 150 检查，窄预算和错误摘要仍拒绝 | `review-fix/contracts.log` |
| 大事件/规律 ID 的导出、参数摘要、恢复及续算；大快照实际存储 | 通过；窄预算拒绝时无对象/部分发布 | `review-fix/kernel-limits.log`、`store-limits.log` |
| 新验证脚本 ruff / mypy strict | 通过；初次 mypy 错误已修正，失败日志保留 | `review-fix/python-ruff-final.log`、`python-mypy.log`、`python-mypy-02.log` |
| 写锁释放与单写者门禁 | Windows 单写者及完整崩溃套件通过；Linux 生命周期/128 次并发原始结果见当前提交 CI | `review-fix/workspace.log`、[CI 入口](review-fix/CI.md) |
| 当前提交 Ubuntu / Windows CI（含 NTFS 存储） | 发布门槛：必须全绿；固定 head 的最终结果记 PR 交付评论，旧 head/draft skip 不算通过 | [CI 证据入口](review-fix/CI.md) |

原 B3 导致 core-store 31 失败/15 通过、fact-view 17 失败/1 通过，保留在 `implementation/rust-workspace-01.log`、`rust-other-crates.log`。该阶段首编译将字符串资源 ID 误填数字版号字段，已改为正确的 ObjectRef 资源版本绑定（`storage/targeted-initial.log`）。第二轮负质量被结构层提前拒绝，测试却尝试生成规范对象；现分别验证结构拒绝和结构合法的语义反例，失败与修复日志都保留。没有放宽阈值、取消门禁或跳过失败检查。

审查反例在重新构建前的旧 Rust 探针上复现，见 `review-fix/limits-before-rebuild.log`（预期失败）；修复后共同配置下的完整边界矩阵通过。新增 Python 回归脚本首次 mypy 报循环导入常量推导/Path 参数问题，已修正，原始日志保留。旧 Ubuntu 的 writer lock 失败未拿重跑覆盖：本次保留原始 TryLockError，并增加显式 unlock 和 Linux 定向机制验证；缺少历史现场持锁进程记录，不能把该机制确定为那一次失败的唯一根因。

全部测试使用 TemporaryRoot 和 D 盘任务缓存，未访问用户排练/正式存储；重负载串行。既有 V2-93 的 1 个 ignored 为原数值诊断，本卡不改。新冻结 `baselines/p5-35/growth.proto` 保留 main 的 EOF 空行，先前 whitespace 提示与冻结源字节一致，未为消除提示改写冻结内容。

## 规格对照

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| ADR 0007；V2-107 | theta.proto、superseded、p5-35 基线、store theta 分派 | 旧向量不变；旧字节可读、新写只用 V2、未知类型拒绝 |
| V2-94/95/98/103、I18 | 完整估计器/运行时间；kernel storage | 正/零时长、负 log、停机、开放段、检测与慢漂；全部私有字段位比较 |
| V2-100/108 | 参数摘要、restore 可信资源匹配、store 语义边界 | 错资源/摘要/版本/计数/质量/时间拒绝，无默认填值 |
| I23、V2-109 | 原轨迹 + 跨语言桥接 + 存储提交重开链 | 每前缀恢复及全后缀状态/变化/报警/界比较；variance 与 sigma² 不等位正控 |
| I11/I24、AC2-05/14；V2-96/97、AC2-20/21 | 原 stage 不改 | 无事件/排除来源恒等、重复/倒序拒绝、Jump/SlowDrift、大 ordinal/次正规数 |
| 原子性与权限边界 | 原能力类型、stage/commit、只读打开检查 | 非法状态无发布，BeforePublish 回滚；既有崩溃/隔离/能力测试 |
| I29 | 合成资源/事件、TemporaryRoot | 不声明个体校准、资源来源认证或整系统验收 |

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| 无 | — | — | 复用既有依赖；本轮未改 Cargo 清单/锁文件 |

## 待用户确认

- [x] 内嵌方案、三个 kernel 路径和 B3 存储消费者适配均获用户批准。
- [x] 65 前缀真实存储恢复/续算、完整 Rust 工作区与静态检查通过。
- [ ] （合并门槛）首次完整审查修复后的独立复审及用户最终契约合并；实现会话不自行审查批准或合并。

## 后续建议（不在本任务范围内）

- 旧 θ 头仍不可写；后继明确完整估计器来源与旧历史迁移/回退方案，不得补零、猜造或自动升级，本卡不签发 SYS17。
- P5-26 独立完整 H 重算、P5-29 实际资源/H/停机来源认证和完整运行时编排仍未交付；存储消费者及恢复接口的验证不替代这些后继验收。
