# AGENTS.md — Nemu 3D 桌面伙伴

本文件是 Codex、Antigravity 与 Claude 三个执行者共用的项目规则。开始任何任务之前，先完整读完本文件、`docs/ARCHITECTURE.md`、`docs/CONTRACTS.md` 和 `docs/echo-core/README.md`。做 Echo Core 相关的卡（P5、P6）前，还要按 `docs/echo-core/README.md` 第 5 节读规格原文。

## 1. 项目一句话

Nemu 是一个常驻 Windows 桌面的原创二次元 3D 伙伴。

- **Echo Core：** 她的核心。用户设计的状态转移核心，Rust 实现，负责她的历史、短时状态、长期动力学和行动。
- **桌面应用：** 她的身体与感官。Electron + three.js + @pixiv/three-vrm 负责渲染与桌面集成，sherpa-onnx 与 Qwen3-TTS 负责声音。
- **说话：** 三种模式：本地严格、云端投影、起草改写。
- **助手：** 经 Codex app-server 接入。

角色资产是 VRM 1.0，必须满足 `docs/CONTRACTS.md` 中的资产契约。

架构已经定稿。不要提议更换引擎、框架、核心或角色管线。如果你认为某个决定有问题，按第 7 节写 ADR，然后继续按现有设计完成任务。

## 2. 认领、目录与建议执行者

任务卡不固定由谁完成。每张卡头部写有「建议执行者」，它是默认人选：熟悉相关目录，也有验收需要的环境（Windows、显卡、浏览器等）。用户派活时可以点名让别的执行者接手（`tasks/README.md` 第 2 节）。

**认领：** 开工时先在 `task/<任务号>-<短名>` 分支上开一个草稿 PR（draft），标题写成 `<类型>(<任务号>): <标题>`，例如 `feat(P5-10): …`。草稿 PR 就是认领标记：看板据此显示「进行中」，其他执行者看到就跳过这张卡。做完后把草稿转为可审查。放弃时关闭草稿 PR，并在评论里说明原因。

**目录：** 每张任务卡的「目录」一栏列出它可以修改的范围，卡里没写到的目录不能改。**「目录」有重叠的两张卡不能同时在做**，不论是谁在做。两个目录相同，或者一个是另一个的上级目录，就算重叠。比如 `core/crates/core-runtime/` 与 `core/crates/core-runtime/src/metrics/` 重叠，`core/crates/fact-view/` 与 `core/crates/gateway/` 不重叠。需要改卡外的目录时，在 `tasks/requests/` 下新建一个请求文件，写明原因和期望的接口，然后继续做不受影响的部分。

以下共享文件不受这条限制，任何卡都可以改，冲突按规则处理：

- **工作区清单与锁文件：** 根目录 `package.json`、`pnpm-workspace.yaml`、`pnpm-lock.yaml`，`core/Cargo.toml`、`core/Cargo.lock`，各 `pyproject.toml` 与 `uv.lock`。锁文件冲突时重新生成，不手工合并。
- **模块登记行：** 所在包入口文件（`lib.rs`、`mod.rs`、`index.ts`、`__init__.py`）里登记新模块的一两行。只追加；冲突时保留双方。
- **根目录脚本与 CI 配置：** 只追加，不改别人加的条目。
- **只追加的记录：** `docs/traceability/diff-register.md`、`reports/`。

下表是各目录通常由谁做，以及特别规则。执行者一栏是建议；括号里的特别规则对所有执行者都有效。

| 路径 | 建议执行者与特别规则 |
| --- | --- |
| `core/**`（Echo Core：Rust 工作区、IDL、黄金向量） | Codex（`core/contracts/` 只在标注「契约变更」且有 ADR 的任务里修改） |
| `core/crates/core-policy/**`、`core/crates/core-kernel/**` | Claude |
| `workers/**`（irl、drive、model-adapter） | Codex |
| `executors/**`（codex-bridge、office-connector） | Codex |
| `services/**`（voice-sidecar） | Codex |
| `apps/desktop/**` | Codex |
| `apps/stage/**` | Antigravity |
| `apps/panel/**` | Antigravity |
| `packages/contracts/**` | Codex（只在标注「契约变更」且有 ADR 的任务里修改） |
| `packages/terminal/**`（终端客户端） | Codex |
| `packages/avatar-runtime/**` | Antigravity |
| `packages/body/**`（物理状态机、反射、读出应用） | Antigravity |
| `packages/desktop-sense/**` | Codex |
| `packages/voice/**`（含 `packages/voice/web/**` 麦克风采集模块） | Codex |
| `tools/**` | Codex（`tools/status/` 看板状态脚本：Claude） |
| `config/**` | Codex（`opportunities.json`、`helpers.json` 的修改需要 ADR） |
| `profiles/**`（Echo Core 结构夹具与部署配置） | Codex |
| `spikes/**` | 由任务卡指定 |
| `content/**` | 用户。代理可以提交管线生成的产物：`nemu.vrm`、`nemu.profile.json`、`lipsync/*.json`、`motions/*.vrma` 及其清单条目。其余内容在用户确认之前只能放进 `content/_candidates/`：设定集、源文件、`build.yaml` 里的审美参数、定稿的音色、协议 p、genesis 清单、语气样例库 |
| `docs/**`、`tasks/**` | 只能追加报告、请求和 ADR，不能改已有内容；`docs/echo-core/original/` 永远不改。`tasks/cards.json`（每张卡的建议执行者、难度、里程碑、依赖）的修改需要用户批准。2026-10-19 用户一次性授权整理了 `tasks/README.md` 和各任务卡头部 |

## 3. 黄金规则

1. **一次只做一张任务卡。** 任务卡以外的发现写进报告的「后续建议」，不要顺手做。
2. **契约是唯一真相源。** 跨进程的类型、事件名、标记语法只能从 `@nemu/contracts` 或 Echo Core IDL 生成的类型导入，不允许在别处重复定义。
3. **Echo Core 是唯一权威。** 她的历史、状态、成长和行动只能经核心的外层事务提交。worker、执行器、终端、界面、模型都只能提出 Proposal 或上报事件，不得直接写权威存储。
4. **不准制造隐藏状态。** 任何会影响她未来行为的持久信息，只能存在于 H、x、R、θ、L、Q 或显式版本化的资源里（不变量 I18）。终端里不得保存情绪、好感、亲密度之类的变量；缓存可以加速，但不得改变权威结果。
5. **不准放宽验收。** 不能为了通过而修改阈值、跳过测试、删除断言。Echo Core 的验收编号（F01…、SYS…、I01…）按原文执行。确实做不到时，写 `reports/<任务号>/BLOCKED.md`，说明实测数据和备选方案。
6. **不准补写规格。** Echo Core 规格里没有定义、或依赖缺失的「Review_and_Remediation」增补的内容，不得凭经验自行补写；写进差异登记，标为「待用户提供」或「待用户确认」。
7. **美术不靠猜。** 涉及外观的改动，一律产出候选和截图，交给用户选择。不要安排需要用户手绘或手工修图的步骤：贴图修整用 krita-ai-diffusion 局部重绘或脚本完成。
8. **依赖要登记。** 新增运行时依赖时，在报告里写明名称、版本和许可证。禁止在应用代码里引入 GPL、AGPL 依赖；不能复制 Mate-Engine 的代码。
9. **资产要登记。** 每个模型、动作、声音、贴图、核心配置都必须在对应的 `manifest.json` 里登记来源、许可证和能否分发。
10. **Windows 优先。** 命令用 PowerShell 写；路径一律用平台 API 拼接；涉及坐标的代码必须考虑每显示器 DPI；进程间通信用命名管道。
11. **不碰密钥。** 不提交任何密钥；本地配置放 `.env.local`（已加入 gitignore）；应用里的密钥用 Electron `safeStorage` 加密保存。
12. **Blender 只走无头脚本。** 资产管线只能用 `blender -b --python`。Blender MCP 只能在交互检查时用，必须开安全模式，并且只能打开仓库里的文件。
13. **卡住时不停工。** 把卡住的部分写进 `BLOCKED.md`（问题、已试过的办法、2–3 个选项），然后继续做其余部分。
14. **她的数据不是测试数据。** 开发、测试、基准一律使用临时存储，不得打开用户的排练存储或她的正式存储（ADR 0005）。诞生之后：
    - 升级核心前先做本机备份（`echoctl checkpoint`）；
    - 改变存储格式的修改必须带迁移、回退和迁移测试（SYS17），迁移失败时保留旧系统。

## 4. 常用命令

```powershell
pnpm i                      # 安装 Node 依赖
pnpm dev                    # 启动应用（会拉起 nemu-core、worker、旁路服务）
pnpm dev:stage              # 在浏览器里单独跑舞台（供 Antigravity /browser 验证）
pnpm dev:review             # 评审台
pnpm lint; pnpm typecheck   # 静态检查（TS）
pnpm test                   # 单元 + 集成测试（Vitest）
pnpm test:e2e               # 端到端测试（Playwright for Electron）
pnpm test:visual            # 视觉回归
pnpm bench                  # 性能与延迟基准（只能在用户的 GPU 主机上跑）
pnpm asset:build            # 运行 Blender 资产管线
pnpm asset:lint <file.vrm>  # 资产契约校验
cargo test --workspace      # Echo Core 测试（在 core/ 下）
cargo run -p echoctl -- <inspect|replay|verify|checkpoint|restore|birth>   # Echo Core 管理命令（birth 只由用户在 N3 执行）
uv run pytest               # worker 与旁路服务测试（在 workers/、services/ 下）
pnpm accept:core            # Echo Core M2 范围验收套件（P5-18）
pnpm eval:helpers; pnpm eval:speech; pnpm eval:memory
```

这些命令在 P0-01 里建立骨架，由各卡补齐。如果某张任务卡需要新的命令，把它加进根目录的 `package.json`（或 `core/` 的 Cargo 别名），并在报告里说明。

## 5. 代码规范

- **TypeScript：** `strict`，禁止 `any`（确实需要时用 `unknown` 再收窄）；全部使用 ESM。进程边界上的输入一律用 zod 校验。
- **Rust：** `#![forbid(unsafe_code)]`（命名管道与低完整性启动等必须用 unsafe 的地方，集中放在一个模块并逐处注释）；`clippy -D warnings`；算子是纯函数，不访问网络、数据库或实时时钟（总方案第 2 节）；写权限用类型能力表达（`StagedX`、`StagedR`、`CommitCapability` 等）。
- **Python：** 类型注解 + `mypy --strict`；worker 只读输入、只产出 Proposal，不打开权威存储。
- **数值：** 权威核用有限 binary64，拒绝 NaN、Inf，固定求和次序，不启用 fast-math（总方案第 6 节）。
- **日志：** 统一用 `@nemu/log`（TS）、`tracing`（Rust）、`structlog`（Python）；日志里不能出现密钥、完整的对话内容和文件内容。
- **组织：** 模块保持小而单一；超过 400 行的文件需要在报告里说明理由；测试和代码放在同一个包里。
- **提交：** 提交信息使用 Conventional Commits；每张任务卡一个分支：`task/<任务号>-<短名>`。

## 6. 任务完成标准与报告

一张任务卡只有同时满足以下条件才算完成：

- [ ] 任务卡列出的全部验收命令通过；
- [ ] 所涉及语言的静态检查和测试全部通过（`pnpm lint`、`pnpm typecheck`、`pnpm test`、`cargo test`、`uv run pytest`）；
- [ ] 报告写好，放在 `reports/<任务号>/REPORT.md`；
- [ ] 截图、基准数据、日志、验收证据放在 `reports/<任务号>/` 下；涉及 Echo Core 验收编号的，按编号建子目录；
- [ ] 需要人工评审的项目，已在报告里列成待用户确认的清单。

报告模板：

```markdown
# <任务号> <标题> — 报告

## 结果
一句话：完成 / 部分完成（见 BLOCKED.md）。

## 交付物
- 文件或目录：作用

## 验收
| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |

## 规格对照（Echo Core 相关的卡必填）
| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |

## 新增依赖
| 名称 | 版本 | 许可证 | 用途 |

## 待用户确认
- [ ] （阻塞）...
- [ ] （不阻塞）...

## 后续建议（不在本任务范围内）
- ...

## 下一步
已合并 / 等待用户（写明在等什么）；接下来取的卡：<任务号>
```

### 审查与合并

每张卡头部写有难度（低、中、高），它决定 PR 合并前要经过谁的审查：

| 难度 | 审查 | 合并 |
| --- | --- | --- |
| 低 | 不需要审查。Claude 的每日审查事后抽查。 | 满足下面三条，执行者自己合并 |
| 中 | 一方实现，Claude 审查；Claude 实现的卡由 Codex 审查。 | 满足下面三条且审查通过，执行者自己合并 |
| 高 | Claude 与 Codex 协作：一方实现，另一方交叉审查。Codex 审查时用 GPT-6 Astra、最高推理强度。 | 同上 |

合并条件，不论难度：

- 分支已经更新到最新的 `main`，并在更新后重跑了测试；
- CI 全绿，任务卡列出的验收命令全部通过；
- 报告里没有「阻塞」类的待确认项。

合并一律用 squash。

**审查结论**写成一条 PR 评论，第一行固定是「审查结论：通过」或「审查结论：有阻塞问题」，下面逐条写问题：文件和行、原因、建议。看板按第一行判断状态。有阻塞问题时，执行者修改后在 PR 里回复，审查方复审，再写一条新的结论。审查方只审查，不替执行者改代码。

需要 Codex 审查时，由用户把 PR 转给 Codex，看板的「等你处理」会提示。需要 Claude 审查时，Claude 的每日审查会处理，用户也可以直接让 Claude 审。

**未核验就合并：** PR 在审查完成前被合并时，给它加 `unverified` 标签。审查方照常补完审查，发现的问题另开修复 PR，全部解决后去掉标签。依赖这张卡的卡在标签去掉之前不能开工。

以下情况属于「阻塞」，不论难度：PR 加上 `needs-user` 标签，等用户批准后再合并。

- 任何偏离 `docs/echo-core/` 规格、`docs/CONTRACTS.md`、`docs/ARCHITECTURE.md` 的改动，以及新的 ADR；
- W00 差异登记中新增的「重建」「待用户确认」条目；
- 放宽验收阈值；
- 更新已有的视觉回归基准图；
- 新增的运行时依赖不是宽松许可（MIT、Apache-2.0、BSD、ISC、CC0 等）；
- 把候选内容定稿，也就是从 `content/_candidates/` 移进正式目录，包括音色、协议 p、genesis 清单、语气样例库、审美参数等；
- 修改 `config/opportunities.json`、`config/helpers.json`。

其余待确认项（例如「呼吸幅度是否合适」）标成「不阻塞」，照常合并；用户之后提出的意见，作为新的小改动处理。

合并之后，按 `tasks/README.md` 第 2 节的规则取下一张卡，开草稿 PR 接着做。没有可以开工的卡（依赖还没合并、需要用户参与、或者「目录」与正在做的卡重叠）时停下，在报告的「下一步」里写明在等什么。

## 7. ADR（架构决策记录）

以下情况都必须先写 ADR：

- 修改契约或放宽阈值；
- 引入重量级依赖；
- 对 Echo Core 规格的任何扩展或例外。

ADR 放在 `docs/adr/NNNN-<短名>.md`，包括背景、决策、备选方案、后果。ADR 需要用户批准；批准之前按原设计继续工作。

## 8. 技能文件

| 技能 | 说明 | 何时读 |
| --- | --- | --- |
| `.agents/skills/stage-visual-check/SKILL.md` | 如何在浏览器里确定性地渲染舞台并做截图对比 | 做渲染、动画相关任务前必读 |
| `.agents/skills/blender-headless/SKILL.md` | 如何编写和运行 Blender 无头脚本 | 做资产管线相关任务前必读 |
| `.agents/skills/echo-core-work/SKILL.md` | 如何把 Echo Core 规格条款对应到代码、测试和报告 | 做 P5、P6 的卡前必读 |

Antigravity 会自动发现这些技能；Codex 请在开始相关任务时手动读取。

## 9. 机械性任务委派与资源调度（2026-10-17 用户追加规则）

本节只追加规则，前文保留。用户已授权把简单、按既定规程即可得到结果的任务交给子代理：跑测试、收集日志、校验哈希、整理报告素材、只读查询 CI 状态。主代理负责明确任务范围、输入、命令、证据格式和资源窗口，然后核对返回结果。

- 项目级子代理配置为 `.codex/agents/routine-reporter.toml`，名称 `routine_reporter`，模型 `gpt-6-luna`（GPT-6 Luna），推理强度 `max`（当前该模型支持的最高档）。不得静默降档或换模型；不支持时报告主代理。
- 子代理只报告证据，不修复、不改测试、不提交、不推送、不创建或修改 PR、不合并、不批准决定。报告素材交由主代理核对后追加入库；失败原始日志必须保留，不用自行重跑代替诊断。
- 解读规格、判断是否通过验收、选型、批准/合并条件判断和实际合并仍由主代理承担，不委派给子代理。
- 项目不人为限定累计子代理数量；可按任务需要分派。并发受当前平台实际槽位、服务限制和资源约束控制，超出槽位排队，不绕过限制，不用一个虚构的“无限”配置值。子代理不递归委派。
- 主代理统一调度重负载；编译、Electron/Playwright/视觉测试、GPU 测量等一次只运行一个。子代理未获本次明确独占窗口不得启动重负载。只读轻量证据整理可以并行，但不能干扰安静测量窗口。
- Windows runner 默认常驻在线（2026-10-19 用户调整）。runner 空闲时只是一个等待作业的小进程，不影响本机测试；冲突只发生在 CI 作业正在执行的那几分钟，表现为单实例锁、端口或命名管道被占用、置顶窗口挡住视觉截图。因此：
  - 本机运行 Electron、Playwright 或视觉测试前，先看有没有 CI 作业在执行（存在 `Runner.Worker` 进程就是有）。没有就直接运行；有就等它结束再开始。
  - 测试中途遇到 CI 作业开始执行而失败的，等作业结束后重跑，不当作代码问题，也不为此改测试或阈值。
- GPU 测量需要用户安排的安静窗口：主代理提前告诉用户要测什么、大约多久，用户停掉 runner 后再开始。测量前确认 `Runner.Listener` 和 `Runner.Worker` 都没有在运行，在运行或无法确认就停止并报告。窗口内不运行其他测试或 GPU 工作；测完通知用户重新启动 runner。
- PR 需要 Windows 检查而 runner 不在线时，由主代理通知用户，并列出相关的 PR 和提交。
- 用户主机内存已升级到 32 GB；这一容量更新不放宽重负载串行、GPU 测量窗口或任务验收要求。
- 当前客户端若不能按名称选择自定义角色，主代理必须显式传入相同的模型和推理强度，并让子代理读取该配置的职责指令；报告区分“显式参数试用”与“按角色名自动加载已验证”，不得声称未验证的自动加载能力。

配置依据：OpenAI 官方 [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents)。项目自定义角色使用独立 TOML 文件，权限继承父代理；角色文件不用于绕过父代理权限或平台并发限制。

## 10. Claude（2026-10-19 用户追加，同日按新的第 2、6 节整理）

用户已决定增加第三个执行者 Claude（Claude Code 云端会话）。Claude 遵守本文件全部规则，另有以下几点：

- **建议执行者为 Claude 的卡：** P5-09 → P5-10 → P5-12，即 `core/crates/core-policy/` 与 `core/crates/core-kernel/` 里不依赖 Windows 和显卡的 Echo Core 数学链。其他卡要不要交给 Claude，由用户在派活时点名，或改 `tasks/cards.json`。
- **审查：** Claude 是中、高难度卡的审查方之一（第 6 节）。Claude 每天早上例行审查一次：给等它审查的 PR 写审查结论，抽查前一天合并的低难度 PR。
- **环境：** Linux 云端临时会话，没有 Windows、显卡和用户的机器，也不保存本地状态。验收需要这些的卡，Claude 不认领。所有进度以推送到 GitHub 的分支、PR 和报告为准。
- **工具链：** Claude 的环境可能无法安装 `core/rust-toolchain.toml` 锁定的版本。此时 Claude 可以用环境内可用的 stable 版本做本地开发和测试，但以 CI 中按锁定版本运行的结果为准；报告里写明本地实际使用的版本。
- **Windows CI：** Claude 的 PR 同样会触发 Windows 作业。需要 Windows runner 时，Claude 提前通知用户，不自行判断不需要。
