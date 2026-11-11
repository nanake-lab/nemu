档位：重档，契约语义边界修复；不改 IDL 或已发布字节。

# P5-42 B1 修复

## 判断与改动

B1 成立：原 V3 只查引用类型，重绑摘要不能补足内部创世约束。Rust 将既有创世规则抽成共享纯字段函数，V2/V3 共用且复用 check_law_parameters；Python/TS 按同一组既有规则实现共享 V2/V3 入口。三语言资源组合检查强制调用，formal=false 也检查。

保留 codec / check_j_message 的已发布向量；它们不代表完整创世语义。单独消费 V3 必须再调用明确的纯创世语义入口，详见 ADR 0036 的 B1 补充。没有拼造退役字段，也没有改 IDL、生成类型、原黄金或基线。

## 两条非阻塞建议

1. **采纳重复调用删除**：同一输入上连续调用两次无额外约束，删除一次，不加抽象或镜像测试。
2. **采纳 formal 正反例**：增加一个非 fixture 声明的结构正例，防止正式分支一律拒绝；prefill NONE / decode NONE 在重绑摘要后拒绝。合成声明不获得真实资格。

## 测试与证据

一个合法创世对照，以及 tau0_s=0、sigma0=0、成事初始增益=2、n0=0、g_min>g_max 五个反例；每个反例重新绑定摘要并验证引用相等，检查具体语义错误。三语言同时检查独立创世入口、资源组合入口，以及六组输入仍被 codec 接受。

被测源码 `02a4dc4c4c0dfe0a5897424523970f0e2b1107b1`：

| 检查 | 结果 |
| --- | --- |
| `pnpm test:core-contracts` | 通过；每语言原 16,773 用例通过，另有 B1 六组 codec 接受及独立/资源语义检查、formal 正例及掩码反例通过 |
| `cargo test -p core-types --locked --offline` | 通过，包含 P5-12 数值往返 |
| `cargo clippy -p core-types --all-targets --locked --offline -- -D warnings` | 通过 |
| 生成器 `--check`、Ruff、严格 mypy | 通过 |
| IDL、黄金、基线、夹具、已发布生成类型相对 6f552607 的 diff | 空，旧内容未改 |
| Ubuntu CI 43067969814 | 通过；实际 checkout 6d777b4c，双亲为 main 02372ff6 / 源码 02a4dc4c |
| Windows CI | not_run：Runner.Listener 离线，未发起工作流；涉及 #164 / 02a4dc4c |

初轮 5e04340f / 6f552607 的 CI 不用于替代本次修复的验证。Windows runner 恢复后仍需核对当前提交的必要 Windows 检查。

无新增依赖/规则/阈值。semantic.rs 保留既有公共语义模块组织，只提取共享字段函数，避免另立重复的 V3 规则。

## 待完成

- 原审查者复审 B1；实现方不自行解除审查阻塞。
- ADR 0036 / V2-151 用户批准仍保留。保持 draft / needs-user，推送请求复审后停止。

## main 增量

本机实际基线仍为 a6a0567f。fetch 到的 main 为 02372ff6；a6a0567f 之后为研究/隐私报告、两份 DOCX 的个人元数据清理、相应 Docs guard 例外与基线登记。已核对新规则只授权这一次元数据清理；DOCX 变动成员仅 docProps/core.xml 和 docProps/custom.xml，没有本卡规格正文、源码、生成器或当前 CI/Windows 流程变化，故按影响规则不 merge。Ubuntu 的实际合成 checkout 在 CI 证据中另行绑定。

## 本轮证据归档

原始日志与 CI 产物存于私有证据提交 [dcd25fab697c7dec98b3622fa74c58846e444bbf](https://github.com/nanake-lab/nemu-evidence/tree/dcd25fab697c7dec98b3622fa74c58846e444bbf/P5-42)，新增 27 文件、431,966 字节；Git blob 与本机原件逐字节相等，远端提交已核对。逐文件 SHA-256 见 [B1-EVIDENCE.csv](B1-EVIDENCE.csv)。

源码提交之后的收尾提交仅修改 reports/P5-42 报告与证据索引，不改变源码、依赖、测试、生成物或工作流。上述 CI 结论绑定源码 02a4dc4c 及合成 checkout 6d777b4c；不冒充之后报告提交的 CI 结论。