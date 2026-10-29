# P5-06 语义核心 — 报告

## 当前状态

部分完成，存在正式契约缺口（见 [BLOCKED.md](BLOCKED.md)）。独立部分为语义结构预检，不是完整 IRL/概念注册表。基线 B 与本轮再次 fetch 核对的 M 均为 `f24f1256259c998bc213a9dc93103fcfb9cfe89a`；无新增 main 影响。PR #111 保持草稿。用户要求完成后交独立审查，不自行合并、不领取下一卡。

## 开工核对

- 2026-10-28：P5-04 / #65 已合并（8845567376e4726cc63d6f148cedd46ec563cd34）；P5-21 / #36 已合并（9042c59455efa340e194ce7a53fa397c09b324af）；两个提交均为本基线祖先，当前无 unverified 标签。
- 开放 PR #108、#104、#97、#96、#64、#49、#45 的任务范围和文件清单未占用 core/crates/semantic-core/。
- 独立 worktree：D:/Dev/nemu-wt/P5-06；分支 task/P5-06-semantic-core。
- 已读最新 AGENTS、ARCHITECTURE、CONTRACTS、Echo Core README、echo-core-work 技能、P5-06 卡、v2.0 全文、ADR 0005/0008–0011、保留的旧规格及相关差异登记。

## 范围

仅 semantic-core 实现及本卡报告、规格对照、接口请求；必要时追加共享 workspace 登记。使用正式 core-types，不修改契约、生成类型、存储、gateway 或编排。缺接口准确登记，继续独立部分。

## 验收

本包 18 项单元测试全部通过，Clippy 零警告。检测到 P5-36 的 Cargo/core-store/fact-view 测试后排队等待；其 Cargo 退出后才开始本包编译，之后串行运行 Clippy。构建输出限定 `D:/Dev/cache/P5-06/target`；使用仓库锁定的 Rust 1.99.0，合成 fixture，不打开任何用户存储、不调用模型。没有重复测试或失败重跑。开发中的初次格式检查提示待格式化，执行格式化后最终检查通过；不把它算作语义测试失败。

| 检查 | 状态 | 证据 / 边界 |
| --- | --- | --- |
| `cargo +1.99.0 test -p semantic-core --offline` | passed：18/18；doc-test 0 | [test-01.log](evidence/test-01.log) |
| `cargo +1.99.0 clippy -p semantic-core --all-targets --offline --locked -- -D warnings` | passed | [clippy-01.log](evidence/clippy-01.log) |
| `cargo +1.99.0 fmt --manifest-path core/Cargo.toml -p semantic-core -- --check` | passed | [fmt-02.log](evidence/fmt-02.log)；仅本包 |
| `git diff --check` | passed | [diff-check-02.log](evidence/diff-check-02.log)；Windows CRLF 提示不改变语义 |
| SYS02、SYS04 | not_run | 未有完整 source/span、registry 与阶段接口，预检测试不能替代 |
| SYS03 结构部分 | not_run（整项） | 否定/模态/引语/纠错/歧义/新词的完整结构覆盖缺正式 operand 与映射资源；语义准确率由 P5-07 承担 |
| 16.15、16.16 | not_run | 缺跨度验证、正式概念生命周期、修订与历史身份升级验收 |
| 全 workspace / Node / Python / Windows E2E / GPU | not_run，不重复运行 | 这些实现未变；没有为本卡开启真实运行或测量 |

## 交付物

- `core/crates/semantic-core/src/objects.rs`：正式类型的 CanonicalV1 类型/摘要/尺寸核验、完整 ProposalBindingV2 规范比较、producer/version 校验；触摸、桌面、执行回执、语音送达、Codex 会话、维护 payload 的纯结构核验。
- `graph.rs`：来源集合与认识论标签检查、ConceptRef 精确成员检查、跨类型 ID 重复检查、角色引用闭包固定点、非法边隔离、合法独立子图保留、CanonicalV1 集合排序；有序边不重排。复用 codec 的 bytes/items/nesting depth 上限。不是已验证 span、图路径深度或语义准确率证明。
- `records.rs`：各解释记录 schema/数值/绑定/生产者/目标事件检查；引用存在性、已知人/倾向、伪造教导说话人和自身叙述证据检查；不归一化方向、不补三语、不猜说话人。重解释只检查 ID 关系；同类同目标的正式对象读取仍缺失。
- `aliases.rs`：有界显式有向映射、未知端点/重复后继/环拒绝、终点代表与输入顺序无关；不做字符串相似推断，不重写历史，不持久化。
- `source_rules.rs`：固定预检版本 `p5-06-source-partition-preflight-v1`，检查 V2-71/EV3 的已批准分区以及显式账号归人；不推断 EV9 可核实性，不生成可提交属性。
- `lib.rs`：有型失败与已有 `IRL_DEGRADED`/`IRL_FAILED` 正式状态转换，不附伪造 z 或提交能力。
- `tasks/requests/P5-06-formal-semantic-boundaries.md`：G01–G05 精确接口缺口及验收目录请求。
- `docs/traceability/diff-register.md`：只追加 P5-06 缺口及分区，不改旧条目。新增登记仍为阻塞。

## 规格对照

| 规格 / 不变量 | 当前实现与验证 | 未满足边界 |
| --- | --- | --- |
| F §3.2–3.3、D I §4.3；SYS02、16.15 | graph 的结构/来源标签/引用/预算/排序反例 | span、provenance、predicate operand 缺正式对象；完整验收未运行 |
| F §3.4、D I §4.4、SYS04、16.16 | aliases 的多语 ID、环、显式终点用例；ConceptRef 原值不变 | 没有注册表持久化、staged 分配或版本迁移 |
| v2 §1/3/4/7、EV1/EV6 | records 复用 core-types 数值约束和 1e-12 单位方向规范；保留输入值 | A/M 的生成与生产验收不在本卡 |
| EV2、EV3、EV9、V2-71 | source_rules 的分区反例；model_inference 不升格用户自述；教导 speaker 不能由模型任填 | 来源根链、认证与 verifiability 生产未接通；不签发证据资格 |
| I02/I18 | 无存储依赖/写接口；临时集合不序列化、不作为隐藏权威状态 | 事务接入由后继负责 |
| I14 | 完整 binding 的规范字节比较，含环境与读集 | expected 必须来自可信 dispatch；该读取接入未做 |
| I23、SYS03 结构 | 纯函数，无模型/时钟/随机；规范化输出顺序一致 | 不宣称完整系统重放 |
| I29 | 测试只用合成数据，无用户数据库 | 无生产/排练验收 |

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| serde_json（本包直接依赖；工作区已有锁定版本） | 1.0.154 | MIT OR Apache-2.0 | 复用 core-types Schema 的正式摘要计算；没有创建 JSON 权威编码 |

没有 IDL/生成代码/黄金向量变更。Cargo 自动更新的共享 Cargo.lock 只有 semantic-core 直接依赖列表新增 serde_json 一行，没有变更任何包版本。没有超过 400 行的本卡 Rust 文件。没有修改 #108 的 contracts/types/store 或 #64 的 gateway，也没有触碰其他 worktree。完整验收状态在各原编号子目录 result.json 中保留为 not_run；源文件 SHA256 与工具链信息保存在 evidence/。

## 待用户确认

- [ ] （阻塞）安排 G01–G05 的正式契约/资源与可信读取接口；不在本卡越界补协议。
- [ ] （阻塞）新增差异登记，以及核心验收目录的后续注册范围。
- [ ] （阻塞）整卡完成后由独立 GPT-6 Astra 最高推理会话审查；本轮未签发审查通过或整卡完成。

## 后续建议（不在本任务范围内）

由接口所属卡提供正式对象、黄金向量和只读边界；P5-06 再继续完整阶段实现。P5-19/P5-29 承担编排，不由本 PR 代做。

## 下一步

保留草稿，等待正式接口；未获缩卡授权，不把“语义预检完成”改写成“P5-06 完成”。不自行合并、不领取下一卡。
