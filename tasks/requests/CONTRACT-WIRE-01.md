# CONTRACT-WIRE-01：说话与独处思考的契约缺口（契约变更）

- **线：** 交付线。**难度：** 高。**档位：** 重档（改 `core/contracts/`），要交叉审查。
- **标签：** `契约变更`、`needs-user`（新 ADR）。
- **分支：** `task/CONTRACT-WIRE-01`，从 `origin/main` 新建。工作区 `D:\Dev\nemu-wt\CONTRACT-WIRE-01`。
- **编号：** ADR 0046，差异登记 V2-187。开 PR 前、交付前各 `git fetch` 核对一次，撞号就顺延，在报告里写明原号与新号。
- **目录：**
  - `core/contracts/`（`.proto`、生成、基线、夹具、黄金向量、校验脚本）；
  - 生成端：`core/crates/core-types/`、`packages/contracts/`，以及 `generate.py` 写到的 worker `echo` 包；
  - `docs/adr/0046-*.md`（新建）；`docs/traceability/diff-register.md`（只追加）；
  - `tasks/requests/CONTRACT-WIRE-01.md`（本卡原文）；`reports/CONTRACT-WIRE-01/`。
- **不碰：**
  - `core/crates/task-layer/`、`core-store/`、`core-runtime/`、`action-gate/`、`gateway/`；
  - `tools/cause-demo/`、`tasks/cards.json`、其他 ADR 与规格、`content/`。
  - 别处按消息类型穷举的表缺行时，按 AGENTS 直接追加一行，在报告里列出。

## 背景

ADR 0045 已经批准。它的 §4「契约缺口」列了四项，§5 写了云端念头回执必须能核对的事实。本卡只定义类型：

- 不接生产路径：没有任何运行代码开始写新消息；
- 现有消息的字节一律不变。

谁来产出这些新消息，后继卡已经定好：

- 云端回执：P5-38、P5-39；
- 记录关系：P5-43、P5-53；
- 投递范围：P6-04。

## 先读

1. `AGENTS.md`，`core/contracts/README.md`。
2. ADR 0007、ADR 0030；ADR 0042 中与关系和计量有关的各节；**ADR 0045 全文**，重点是 §3 映射表、§4、§5。
3. `core/contracts/` 下的 `cognition.proto`、`state.proto`、`records.proto`、`terminal.proto`、`terminal_ingress.proto`。
4. `core/crates/core-types/src/semantic/cognition.rs`；`core/crates/task-layer/src/cause_projection/types.rs`（只看组包器要的输入形状）。
5. **先例：** PR #164（P5-42 / ADR 0036）怎样新增消息、冻结基线、加黄金向量、补穷举表。本卡照它的做法走。

## 1. 先判定，再设计（写进 ADR 0046 §1，然后才动 IDL）

对 ADR 0045 §4 的四个缺口，逐个回答：**能不能用现有字段表达？** 能，就不加字段，在 ADR 里写明换算或还原规则，并标出由哪张后继卡实现；不能，就写缺什么、为什么现有字段不够（引文件和行号）。

| 缺口 | 判定方法 |
| --- | --- |
| A. 云端内部文字的回执 | ADR 0045 已确认现有 `CognitiveResultAndReceipt` 表达不了：`model_hash`、`tokenizer_hash`、`modulation` 是必填项，云端拿不到。直接进入设计。 |
| B. 原记录到作者及引语层级的受信绑定 | 有条件的缺口。按来源种类逐个查：内部产物、终端用户文字、用户语音、经 `source_refs` 或 `source_span` 转述的第三方、教导标注。看现有受信字段能不能唯一定出作者和引语层级。只有定不出的种类才需要新类型。 |
| C. 带类型的记录间关系、生效范围与完整性 | ADR 0045 已确认 `parents` 和通用 `source_refs` 不区分关系种类。直接进入设计。 |
| D. 部分投递的精确范围 | 有条件的缺口。核对 `SpeechDelivery.played_chars` 和 `delivered_chunks`，加上 `OutputChunk.text` 与绑定的编码，能不能唯一换算成组包器使用的字节半开区间。要查清 `played_chars` 按什么计数（码点、UTF-16 单元，还是字素），契约或代码里有没有写明。写明了且能唯一换算，就不加字段。 |

判定表放进 ADR 0046 §1，每行一个结论：「需要新增」或「不需要，规则是……，由某卡实现」。

## 2. 设计约束（只针对判定为「需要新增」的项）

**通用约束：**

- 只增不改：
  - 改动的消息做成带新冻结 tag 的后继，写 `@supersedes`；
  - 旧字段号保持原样或标为 `reserved`，新字段号放在前驱最大号之后；
  - 旧消息继续可解码，不加 `@superseded`，因为它的生产者还没换；
  - `check_compat.py` 必须通过。
- 契约记录事实，不记录政策。比如关系的种类写「回复」「正文引用」「更正」「撤回」这类事实；它对应组包器五类中的哪一类，由 ADR 0045 §3 的映射资源决定，不写死进契约。
- 能用类型挡住的，就不留给运行时检查。

**A 云端回执：**

- 必须能如实写下 ADR 0045 §5 表里的每一项：
  - 提供者；
  - 请求的模型标识，以及服务方实际返回的模型字符串；
  - thinking 关闭；
  - 生成参数和格式资源的引用；
  - 映射和计量的版本；
  - 容量与输出预留；
  - 输入摘要，以及可取回的实发请求对象；
  - 返回原文对象、终止原因、用量；
  - 状态：正常、截断、取消、服务故障、内容被拒、发送前终止并记本地原因；
  - 采纳结果：采纳范围、拒收或失效。
- 拿不到的供应方版本或摘要，要有「如实标明不可得」的写法，不能用填充值冒充本地权重或 tokenizer 摘要。
- 云端分支在结构上不能携带 `next_focus`、`queries`、`action_plan`，而且要写明获准的输出种类只有内部文字。
- **候选：**
  - 新增 `RemoteCognitiveReceipt`；
  - 再做一个 `CognitiveResultAndReceipt` 的后继，用 oneof 区分本地载体（原有字段原样）和云端载体；
  - `InternalActivityRecord` 及向上的容器是否要跟着出后继，按「活消息不能内嵌被取代类型」的规则推出来。
  - 执行者可以给出更简单的形状，在 ADR 的备选方案里写明取舍。

**B 作者绑定**（只做判定为需要的来源种类）：

- 能绑定确切的原记录、发言主体，以及可见的引用区间。
- 通道、设备、session、显示名、文字里的自称、模型返回的编号，都不能单独定作者。

**C 记录关系：**

- 一条关系至少写清这几样：
  - 关系种类；
  - 两端的记录 ID 和内容版本；
  - 用途；
  - 生效前沿。
- 种类的枚举值从 ADR 0045 §3 映射表的每一行逐行推出，并在 ADR 里写成对照表；表里标「未核实」的行，照样保留为候选种类。
- 在某个前沿查询关系时，要能证明已经查全，即有可核对的完整性标记。**候选：** 查询结果绑定前沿和计数，或绑定一个摘要。
- 治理撤权的生效范围要能表达。

**D 投递范围**（只在判定为需要时做）：

- 能重建出实际呈现的对象版本和半开边界。
- 未投递的部分不会被当成原话。

## 3. ADR 0046

文件名用 `0046-speak-think-contract-gaps.md`，内容如下：

- **背景：** 链接到 ADR 0045 §4、§5，不复述原文。
- **§1：** 第 1 节的判定表。
- **§2：** 每个新增或后继消息逐字段的说明，每个字段写它对应 ADR 0045 的哪一条要求。
- **§3 备选方案：** 至少写一个更简单的形状，以及为什么不选。
- **§4 迁移：**
  - 哪些旧消息仍然由谁产出、产出到哪张卡为止；
  - 后继卡接上以后，旧消息怎样标 `@superseded`（只写计划，本卡不做）。
- **§5 后果：** 按 AGENTS 黄金规则 17 的三问作答。

差异登记追加 V2-187，引用 ADR 0045 和 ADR 0046。

## 4. 实现

1. 改 `.proto`，跑 `python core/contracts/generate.py`，提交所有生成文件。
2. 照 #164 冻结新基线，为每个新消息加黄金向量，同时覆盖 CanonicalV1 和 Protobuf 两种编码。冻结前后的字节差在报告里给出证据链接。
3. 在 `core-types` 的语义层加校验：
   - 云端分支不能携带后继选择与行动字段；
   - 不可得的摘要只能用「不可得」的写法，不能填零字节；
   - 关系两端齐全；
   - 状态和采纳结果组合合法。
4. **测试：**
   - Rust、TypeScript、Python 三端往返；
   - 旧黄金向量逐字节不变；
   - 反例都能被拒，至少包括：云端回执带 `action_plan`、用零字节冒充 hash、关系缺一端、截断却标「完整采纳」。
5. **必过命令：**
   - `python core/contracts/generate.py --check`
   - `pnpm test:core-contracts`
   - `cargo test -p core-types`
   - `pnpm test:contracts`
   - 当前 head 的 CI

## 5. 交付

- `reports/CONTRACT-WIRE-01/REPORT.md` 按重档模板写。
- 开头白话三行：做了什么；对她意味着什么；需要用户做什么。第三行写：批准 ADR 0046，合并即批准。
- 报告里列：
  - 四个缺口的判定结果；
  - 新增和后继的消息清单；
  - 补了哪些穷举表的行；
  - 检查命令和结果；
  - 未做的事：生产路径都没接。
- 开 draft PR，标题 `feat(CONTRACT-WIRE-01): 说话与独处思考的契约缺口（ADR 0046）`，加上 `契约变更`、`needs-user` 两个标签。
- 写「请审查」后停下。不合并，不取下一张卡。卡住就写 `BLOCKED.md`，然后停下。
