# P5-34 分配的实际保留率与倾向的证据量 — 报告

## 当前状态

部分完成，草稿PR #72保持draft/needs-user，等待复审与用户批准，不合并、不取下一卡。本轮按[10-24审查](https://github.com/nanake-lab/nemu/pull/72#issuecomment-6379570379)修复两项阻塞并补回归。PR #79已合并，当前分支merge了origin/main `7f962b2f9cc77983cb5318c690b5ac30c7b06422`，包含35分钟Ubuntu CI上限。验证实现提交为`3f57afc`，之后只补报告和证据。真实有效H的G重放器尚未接入，生产迁移、AC2-08/19/21、I23和SYS17仍未签发。

## 交付物

- 四种V3后继消息保存q/V，V2定义/标签/字节不改；两个权威字段分别为EvidenceAllocationV3.9 actual_retention、TendencyV3.18 evidence_absolute。容器继承reserved；ADR0021、V2-62/63保持原编号。潜在变化检测器字段仅列出，未自行增加。
- Rust/Python/TypeScript及浏览器安全Zod生成类型；q∈(0,1]、V≥|A|及绑定规律的B/π/r检查。既有serde_json 1.0.154启用float_roundtrip，保留JSON相邻值与次正规数位模式。
- V3存储新写入、V2原字节历史读取、版本与规律绑定、完整H的存储重放入口；不默认q=1或V=B−c，不访问用户存储。
- 只追加19合法/35非法黄金，当前464合法/492非法/18 Unicode；旧前缀与p5-32基线不改。真实G仍由可信核心实现，替身仅验证存储边界。

## 本轮审查修复

| 问题 | 修复 | 回归测试 |
| --- | --- | --- |
| 全库distinct group_id被当作单消息预算 | 普通发布先编码校验本次CoreTransactionV2集合，再按待发布R的当前倾向查询最新关联组并核对序号；SQL先选最新版本再筛关联，逐行处理，不设置历史总组数上限。完整重放对仍有分配的旧组要求结果/空后继，已有空后继无须重复发布 | r_review::retired_group_history_exceeds_message_budget_without_blocking_commits：max_items=32，9批累计36个历史ID，每批仅4个活跃组；继续全部移出、正常事实提交、恢复与历史查询；超限的本次33项结果集合仍在发布前拒绝，head不变，随后可继续提交 |
| 重放丢失staged_concepts及独有资源 | RReplayEvent逐事件保留既有CoreTransactionV2.staged_concepts，纳入摘要验证资源闭包；历史与待提交概念均可读，仍排除旧R统计 | replay_keeps_each_events_staged_concepts_and_atomic_publication：2个历史事件加待提交事件，各自唯一概念及signature/grounding/relations均由staged_concepts可达；核对关联与字节，回调/发布失败时head和概念可见性不变，成功后恢复可读。replay_hash_checks_staged_concept_resources_before_calling_kernel：保持长度和合法编码但改变摘要，明确拒绝且不调用回调 |

这两项不新增wire字段、依赖、表或规律；ADR0021 §9追加说明。新测试文件少于400行；semantic.rs原403行仍为保留V1/V2接口并追加V3纯检查的组织方式。

## 验收

本轮原始证据：`verification/review-20261024/`；此前Echo Core/契约原始证据全部保留在`verification/idle-20261024/`等目录，未删改。每项命令JSON记录开始/结束、HEAD/main、参数、退出码与原始日志SHA-256。重负载由主代理串行执行；routine_reporter仅做只读CI/日志核验，当前AGENTS §9不委派重负载。

| 命令/检查 | 当前结果 | 证据 |
| --- | --- | --- |
| 定向r_review回归 | 3项通过，0失败；其余37项由完整回归覆盖 | attempt-01/regression.log/.json |
| workspace Clippy，all-targets，-D warnings | 通过，退出0 | attempt-01/clippy.log/.json |
| cargo test --workspace --locked -j1，单测试线程 | 退出0；101项通过（含core-store 40项及2项compile-fail文档测试），0失败/忽略 | attempt-01/workspace.log/.json |
| Rustfmt | 通过，退出0 | attempt-01/fmt.log/.json |
| generate --check、12项schema、pnpm test:core-contracts（本卡SYS21/施工合同10.8范围） | 0 stale、12项schema通过；三语言各11592用例、protoc464向量和Node中的Zod106项通过 | attempt-01/contracts.log/.json |
| check_compat --ref origin/main | p5-01、p5-21、p5-32、current、origin/main均0违例 | attempt-01/compatibility.log/.json |
| 独立旧黄金/新增字节复核 | 旧445合法/457非法/18 Unicode前缀未变；新增19合法向量独立计算0差异 | attempt-01/bytes.log/.json |
| 新提交CI（包含PR #79的35分钟上限） | 以[PR #72 checks](https://github.com/nanake-lab/nemu/pull/72/checks)为准；本轮修复head使用PR #79合入的35分钟配置 | 本PR最新head的GitHub运行链接 |
| 生产G迁移、AC2-08/19/21、I23、SYS17 | not_run | BLOCKED.md及对应编号结果；不把本轮存储回调测试当作生产重放证明 |

此前未变的TS/Python实现已在实现提交8d9694e的本机窗口验证：pnpm lint/typecheck、535项单测、契约包350项、Python静态检查与四项目脚手架测试通过；原始日志仍在idle-20261024。当前完整远端CI另覆盖这些项目，旧结果不冒充最新head的CI通过。

### 失败和取消的读法

- 旧CI run40002333162 attempt2是20分钟超时取消，不是测试断言失败；原始annotation明确写着`The job has exceeded the maximum execution time of 20m0s`，见prior-ci-timeout.json。合并已获批准的PR #79后采用35分钟配置，不重跑仍带20分钟配置的旧head。
- 首次黄金冻结曾错误接受V=0.9999999999999999、A=-1；复现确认Rust默认JSON解析舍入为1，启用既有float_roundtrip并补位模式测试后修复。旧黄金未重写，原始失败/诊断及成功重跑均保留。
- 此前fmt包装器无输出日志问题、全目录mypy模板上下文错误、pnpm junction安装警告均保留，未作为通过证据；正式纠正后的运行另列。当前测试不放宽阈值、不跳断言。

## 规格对照

见docs/traceability/P5-34.md：q追溯ADR0019 §5/6/8.5，V追溯§1/8.2/8.5，共同对应v2.0 §5.2；存储边界追溯ADR0021 §7–9。I02/I29通过类型能力、原事务和TemporaryRoot约束；序号与旧版本读取有运行时及回归检查。I23真实重放与SYS17不由替身签发。原LAW-6及规律判据不改。

## 新增依赖

本轮无。既有serde_json 1.0.154（MIT OR Apache-2.0）的float_roundtrip特性不增依赖；Cargo.lock未改。

## 待用户确认

- [ ] （阻塞）ADR0021、V2-62/63的批准及本轮交叉复审。
- [ ] （阻塞）真实有效H重放器可用后接入并完成生产迁移验收。
- [x] core-store扩围已获授权，不重复请求授权；本轮两项审查修复未扩大到其他生产目录。

## 后续建议（不在本任务范围内）

- P5-23/P5-26接入真实G并验证来源更正、容量及迁移路径。
- P5-33的变化检测器状态/资源版本留待用户决定，不在本卡追加字段。
