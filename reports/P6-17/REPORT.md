# P6-17 契约变更：触摸观察加入「停留」 — 报告

## 当前状态

部分完成，仅预读与阻塞登记，**未开始契约施工**（见 [BLOCKED.md](BLOCKED.md)）。基于最新 `origin/main` 的 `2bc04bad4902657eef49d3225cafa7945e7cfae6`。P5-34 的新 PR #95 仍占用本卡目录；不能因为旧 #72 关闭而并行开工。PR 保持 draft + needs-user。下面基线检查不能作为新消息已完成的证明；暂无其他获授权且不受阻塞的实现部分。

## 交付物

- 本报告、BLOCKED.md 与当前 PR 状态证据。
- 完成 ADR 0007、0025，V2-71/72，CONTRACTS 4.2、P6-02 的预读；v2.0、AGENTS 和其余指定文档在 P5-07 阶段已读，主线同步未改变它们。
- 拖动同次接触标识的只读自查，见下文。未分配消息标签/字段号，未写门限或转换逻辑。

## 验收

| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `python core/contracts/generate.py --check` | passed：现有基线 0 过期生成物 | `verification/generate.log`、`light-exit-codes.json` |
| `pnpm test:core-contracts` | not_run：资源预检发现已有其他 Cargo 全工作区测试，未并行启动编译 | `verification/resource-preflight.json` |
| `python core/contracts/check_compat.py --ref origin/main` | passed：p5-01、p5-21、p5-32、current、origin/main 全部 0 违例；137 消息、51 枚举 | `verification/compat.log`、`light-exit-codes.json` |
| 新消息 3 合法/3 非法及定向向量、三语言一致 | not_run：未改契约 | — |
| 用户批准本次新字节，交叉审查 | not_run：无新字节 | — |

现有依赖离线安装与 `uv sync --locked` 均通过；编译输出预留在 `D:/Dev/cache/P6-17/`。资源预检发现 Cargo PID 81008/87276 正在执行 `test --workspace --locked --offline`，按 AGENTS §9 重负载串行规则，没有启动完整三语言套件；未停止或干预其他任务。未运行套件不能记为通过。没有启动模型、访问用户谱系或修改用户/系统设置。

## 规格对照（Echo Core 相关的卡必填）

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| ADR 0025 §4.3；V2-72：停留开始/持续/结束、同次标识、已持续毫秒 | 尚未实现，受目录互斥阻塞 | 新消息向量 not_run |
| v2.0 §4 EV2、§8；V2-71/72：同次对 G 一组，对 C 只计一次 | 本卡补类型，未来 P6-02 转换、Core 规则执行 | 不能以现有基线验收代替 |
| ADR 0007 §§2–9：旧字节保留、后继类型、有限值、缺字段拒绝 | 现有定义原样保留；新类型尚未分配 | 只读基线检查 |
| AGENTS §2；本卡开工条件 | 没有修改重叠目录 | `git diff origin/main` 仅本报告目录 |

### 拖动关联标识自查

结论：拖动开始与结束需要**可核验的同次交互关联**，否则仅凭两条独立事件，不能可靠证明是否同一起因（EV2）。当前 `TouchObservation` 只有 region、kind、speed；`EventEnvelope` 虽有 parents，但现行 touch 合同没有规定父引用必须如何表示同一次拖动。不能靠部位相同、时间接近或重投 key 推断同次接触：重投 key 只解决同一记录幂等。

建议由用户决定是否把后继的同次接触标识也用于 dragStart/dragEnd。若同卡加入，应在新后继上定义开始/结束的关联要求与定向向量；若不加入，保留现行拖动行为，另列后续卡。**本次未实施任何一个选项，也未把该建议补写成规格。**

## 新增依赖

无运行时依赖。基线核验只使用仓库锁定的现有工具链。

## 待用户确认

- [ ] （阻塞）#95 结束，或明确批准目录互斥例外。
- [ ] （阻塞）实际契约修改完成后，批准新字节；契约范围需要独立交叉审查。
- [ ] （不阻塞基线预读；实施该扩展前须决定）拖动开始/结束是否也使用同次接触标识。

## 后续建议（不在本任务范围内）

- 门限、报告节奏与 stage 事件转换按 P6-02 做；本卡不得提前实现。
