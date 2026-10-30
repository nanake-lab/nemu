# AGENTS.md — Nemu 3D 桌面伙伴

本文件是 Codex、Antigravity 与 Claude 三个执行者共用的项目规则。开始任何任务之前，先完整读完本文件、`docs/ARCHITECTURE.md`、`docs/CONTRACTS.md` 和 `docs/echo-core/README.md`。做 Echo Core 相关的卡（P5、P6）前，还要按 `docs/echo-core/README.md` 第 5 节读规格原文。

## 1. 项目一句话

Nemu 是一个常驻 Windows 桌面的原创二次元 3D 伙伴。

- **Echo Core：** 她的核心。用户设计的状态转移核心，Rust 实现，负责她的历史、短时状态、长期动力学和行动。
- **桌面应用：** 她的身体与感官。Electron + three.js + @pixiv/three-vrm 负责渲染与桌面集成，sherpa-onnx 与 TTS 模型负责声音（ADR 0027：目标 Breeze TTS 2，过渡 GPT-SoVITS V5）。
- **说话：** 三种模式：本地严格、云端投影、起草改写。
- **助手：** 经 Codex app-server 接入。

角色资产是 VRM 1.0，必须满足 `docs/CONTRACTS.md` 中的资产契约。

架构已经定稿。不要提议更换引擎、框架、核心或角色管线。如果你认为某个决定有问题，按第 7 节写 ADR，然后继续按现有设计完成任务。

## 2. 认领、目录与建议执行者

任务卡不固定由谁完成。每张卡头部写有「建议执行者」，它是默认人选：熟悉相关目录，也有验收需要的环境（Windows、显卡、浏览器等）。用户派活时可以点名让别的执行者接手（`tasks/README.md` 第 2 节）。

**认领：** 开工时先在 `task/<任务号>-<短名>` 分支上开一个草稿 PR（draft），标题写成 `<类型>(<任务号>): <标题>`，例如 `feat(P5-10): …`。草稿 PR 就是认领标记：看板据此显示「进行中」，其他执行者看到就跳过这张卡。全部验收完成后才把草稿转为可审查；部分完成时保持草稿（GitHub 不允许合并草稿）。放弃时关闭草稿 PR，并在评论里说明原因。

**检查 main 变化与按影响同步（2026-10-27 用户修订）：** 规则、任务卡和 ADR 随时可能在 main 上更新，执行者主动读取最新内容；是否把 main 合入分支，按第 6 节的实际验证基线与影响判断，不要求一律同步。

- **开新卡：** 先 `git fetch origin`，从 `origin/main` 新建任务分支。本文件、任务卡和必读文档都以这个版本为准。不依赖本地 `main`，也不需要检出它。
- **继续做已有的卡**（新会话、隔天、收到用户的新指示时）：先 `git fetch origin`，读取本文件、本卡及相关 ADR 在 `origin/main` 上的变化，按最新规则继续；再按第 6 节判断是否需要同步。需要时用 merge，不 rebase、不强推。
- **以最新状态为准：** 本地文件与 `origin/main` 不一致时，以 `origin/main` 为准。回复用户或继续工作前，先读 PR 的最新提交、评论和审查结论，不凭会话里的旧记忆。
- **合并前：** 照第 6 节核对冲突、当前提交的必要检查与所需审查，并检查最近一次有效验证以来 main 的全部新增变化；仅在需要时同步并运行受影响的检查。

**工作区放 D 盘（2026-10-23）：** 在用户电脑上工作时，用 `git worktree add` 在 `D:\Dev\nemu-wt\<任务号>` 新建工作区；虚拟环境、各类缓存、模型和编译产物放 `D:\Dev\cache\`，不在 C 盘新建工作区或缓存。

**目录：** 每张任务卡的「目录」一栏列出它可以修改的范围，卡里没写到的目录不能改。**「目录」有重叠的两张卡不能同时在做**，不论是谁在做。两个目录相同，或者一个是另一个的上级目录，就算重叠。比如 `core/crates/core-runtime/` 与 `core/crates/core-runtime/src/metrics/` 重叠，`core/crates/fact-view/` 与 `core/crates/gateway/` 不重叠。需要改卡外的目录时，在 `tasks/requests/` 下新建一个请求文件，写明原因和期望的接口，然后继续做不受影响的部分。

以下共享文件不受这条限制，任何卡都可以改，冲突按规则处理：

- **工作区清单与锁文件：** 根目录 `package.json`、`pnpm-workspace.yaml`、`pnpm-lock.yaml`，`core/Cargo.toml`、`core/Cargo.lock`，各 `pyproject.toml` 与 `uv.lock`。锁文件冲突时重新生成，不手工合并。
- **模块登记行：** 所在包入口文件（`lib.rs`、`mod.rs`、`index.ts`、`__init__.py`）里登记新模块的一两行。只追加；冲突时保留双方。
- **根目录脚本与 CI 配置：** 只追加，不改别人加的条目。
- **只追加的记录：** `docs/traceability/diff-register.md`。`reports/` 的规则见第 6 节。
- **后继消息带来的穷举表追加（2026-10-25）：** 契约新增后继消息后，别处按消息类型穷举的表或样例表（例如面板 `schema-validator.ts` 的样例）缺一行导致检查失败时，可以直接追加这一行，不需要请求文件；只追加、不改旧行，并在报告里列明文件和行。

下表是各目录通常由谁做，以及特别规则。执行者一栏是建议；括号里的特别规则对所有执行者都有效。

| 路径 | 建议执行者与特别规则 |
| --- | --- |
| `core/**`（Echo Core：Rust 工作区、IDL、黄金向量） | Codex（`core/contracts/` 只在标注「契约变更」且有 ADR 的任务里修改） |
| `core/crates/core-policy/**`、`core/crates/core-kernel/**`、`core/crates/experiment-world/**` | Claude |
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
| `docs/**`、`tasks/**` | 只能追加报告、请求和 ADR；`docs/echo-core/original/` 永远不改。改已有文字或 `tasks/cards.json` 需要用户批准（PR 加 `needs-user`） |

## 3. 黄金规则

1. **一次只做一张任务卡。** 任务卡以外的发现写进报告的「后续建议」，不要顺手做。
2. **契约是唯一真相源。** 跨进程的类型、事件名、标记语法只能从 `@nemu/contracts` 或 Echo Core IDL 生成的类型导入，不允许在别处重复定义。
3. **Echo Core 是唯一权威。** 她的历史、状态、成长和行动只能经核心的外层事务提交。worker、执行器、终端、界面、模型都只能提出 Proposal 或上报事件，不得直接写权威存储。
4. **不准制造隐藏状态。** 任何会影响她未来行为的持久信息，只能存在于 H、x、R、θ、L、Q 或显式版本化的资源里（不变量 I18）。终端里不得保存情绪、好感、亲密度之类的变量；缓存可以加速，但不得改变权威结果。
5. **不准放宽验收。** 不能为了通过而修改阈值、跳过测试、删除断言。Echo Core 的验收编号（AC2-01…、F01…、SYS…、I01…）按原文执行。确实做不到时，写 `reports/<任务号>/BLOCKED.md`，说明实测数据和备选方案。
6. **不准补写规格。** Echo Core 规格（以 v2.0 为准）没有定义的内容，不得凭经验自行补写；写进差异登记，标为「待用户确认」。v2.0 标「参考」的写法可以换成实验中更好的写法（v2.0 第 0 节第 4 条），但换写法就是发布新的规律版本：要登记，并在实验谱系里检验。
7. **美术不靠猜。** 涉及外观的改动，一律产出候选和截图，交给用户选择。不要安排需要用户手绘或手工修图的步骤：贴图修整用 krita-ai-diffusion 局部重绘或脚本完成。
8. **依赖要登记。** 新增运行时依赖时，在报告里写明名称、版本和许可证。禁止在应用代码里引入 GPL、AGPL 依赖；不能复制 Mate-Engine 的代码。
9. **资产要登记。** 每个模型、动作、声音、贴图、核心配置都必须在对应的 `manifest.json` 里登记来源、许可证和能否分发。
10. **Windows 优先。** 命令用 PowerShell 写；路径一律用平台 API 拼接；涉及坐标的代码必须考虑每显示器 DPI；进程间通信用命名管道。
11. **不碰密钥。** 不提交任何密钥；本地配置放 `.env.local`（已加入 gitignore）；应用里的密钥用 Electron `safeStorage` 加密保存。
12. **Blender 只走无头脚本。** 资产管线只能用 `blender -b --python`。Blender MCP 只能在交互检查时用，必须开安全模式，并且只能打开仓库里的文件。
13. **卡住就停下来问。** 同一个验收项认真试过一种方法仍失败，或下一步要换工具、下载新模型、引入依赖、改目录范围时，不再自行尝试替代方案：在 `BLOCKED.md` 简要写明卡在哪、试过什么、2–3 个选项及代价，等用户决定；不受影响的部分照常做。涉及硬件性能的新组件，先做最小的可行性测量，再写完整实现。
14. **她的数据不是测试数据。** 开发、测试、基准一律使用临时存储，不得打开用户的排练存储或她的正式存储（ADR 0005）。诞生之后：
    - 升级核心前先做本机备份（`echoctl checkpoint`）；
    - 改变存储格式的修改必须带迁移、回退和迁移测试（SYS17），迁移失败时保留旧系统。
15. **不替用户操作。** 不操作浏览器或其他应用去请求审查、联系其他执行者或替用户做事。需要审查时在 PR 里写「请审查」，然后停下。
16. **不改用户级、系统级设置（2026-10-25）。** 包括注册表、开机启动项、系统或用户环境变量、全局的 git/npm/pnpm/uv 配置、防火墙、电源与通知设置。测试和工具需要时，只用进程内或仓库内的配置；确实要改的，写进 `BLOCKED.md` 由用户决定并由用户本人执行。发现以前的代码会改这类设置（例如测试路径写开机启动项），只读检查、写进报告，不自行删除。

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
cargo run -p echoctl -- <inspect|replay|verify|checkpoint|restore|birth|growth|experiment>   # Echo Core 管理命令（birth 只由用户在 N3 执行；growth 只由用户执行）
uv run pytest               # worker 与旁路服务测试（在 workers/、services/ 下）
pnpm accept:core            # Echo Core v2.0 验收套件（P5-18）
pnpm experiment:core        # 实验谱系：在合成世界里跑 v2.0 第 14 节的实验（P5-30）
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
- [ ] 证据放在 `reports/<任务号>/` 下，涉及 Echo Core 验收编号的按编号建子目录。证据要适量：Echo Core、契约和正式验收测量的原始数据全部保留（含失败）；其余只放关键日志和结果摘要，不提交大批过程日志。报告随时可以改写，保持简洁、只写当前状态，历史靠 git；原始证据文件不得删改，失败结果必须在报告里写明；
- [ ] 需要人工评审的项目，已在报告里列成待用户确认的清单。

### 验证与报告按风险分级（2026-10-30 用户决定）

验证的分量跟着出错的代价走。档位按 PR 改动的文件判定；一个 PR 涉及多档时，按其中最重的一档执行。

**重档：** PR 改动以下任一路径，或包含正式验收测量、GPU 测量、预注册研究的实际运行：`core/crates/core-kernel/`、`core/crates/core-policy/`、`core/crates/experiment-world/`、`core/crates/core-runtime/`、`core/crates/core-store/`、`core/contracts/`、`packages/contracts/`、`core/crates/action-gate/`、`executors/`；以及本节「不论难度，以下改动也要交叉审查」所列的其他范围。重档保持本文件现有的全部要求，不作任何简化。

**轻档：** PR 只改动以下路径中的文件：`docs/`（`docs/echo-core/original/` 永远不改）、`tasks/`、`reports/`、`AGENTS.md`、`.agents/skills/`。轻档的检查只做这些：
- `git diff --check`；
- 本次新增或改动的 Markdown 相对链接，逐个确认所指文件存在；
- 改了 JSON、YAML 或 TOML 文件时，用解析器完整读一遍；`tasks/cards.json` 的改动另由 Docs guard 自动检查；
- 不跑全工作区测试，不做证据归档或哈希核对，不要求规格对照表。改变规格或契约读法的，每一处写一行「位置 / 依据 / 新读法」。

**中档：** 不属于重档、也不属于轻档的改动（例如应用、终端、worker、旁路服务、工具脚本、CI 配置）。按本节完成清单跑所涉语言的测试与静态检查；证据只放关键日志与结果摘要。

**拿不准属于哪一档时，按较重的一档执行。**分级只调整验证与报告的分量，不放宽任何验收阈值或审查要求，黄金规则第 5 条不变。

**设计卡与研究卡**（任务号以 `DESIGN-`、`R-` 开头）：报告内容本身就是交付物，按内容需要写。开头必须有白话总览；其中实际运行的部分按重档执行。

**报告写什么（各档通用）：**
- 必须写：所用档位及理由（一句话，放在第一行）；当前状态与结论；改了什么（到文件或模块）；做了哪些检查，命令与结果；失败、阻塞与未完成项；需要用户做什么。
- 不写：工作过程的叙述；对规格、ADR、任务卡原文的复述；证据文件里已有的数据（写链接即可）；对未改动内容的逐项确认（任务卡明确要求的除外）。
- 篇幅由必须写的内容决定，不设行数。失败、阻塞与未完成项必须写在报告正文，不得为了简短而省略或只放进附件；其余长明细放证据文件，正文给链接。

**白话三行：** 每个 PR 说明与「请审查」评论的开头，先用三行白话写：做了什么；对她或对项目意味着什么；需要用户做什么（没有就写「无」）。术语与细节放在后面。

报告模板：

```markdown
# <任务号> <标题> — 报告

## 当前状态
一段话：完成 / 部分完成（见 BLOCKED.md）；在等什么；接下来取的卡。

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
```

### 审查与合并

每张卡头部写有难度（低、中、高），它决定 PR 合并前要经过谁的审查：

| 难度 | 审查 | 合并 |
| --- | --- | --- |
| 低、中 | 不需要审查。 | 满足下面三条，执行者自己合并 |
| 高 | 交叉审查：实现方之外的审查方审查。近期由用户另开的 Codex 新会话审查（2026-10-23 用户决定），用 GPT-6 Astra、最高推理强度。 | 满足下面三条且审查通过，执行者自己合并 |

**不论难度，以下改动也要交叉审查**（2026-10-22 用户决定）：契约与 IDL（`core/contracts/`、`packages/contracts/`）、账本与存储格式（`core/crates/core-store/`）、行动闸门与权限（`core/crates/action-gate/`、`executors/`）、网络生活等对外行动、改规格或契约的 ADR。这几处出错，要么诞生后改不回来，要么伤到仓库之外。

合并条件，不论难度：

- PR 无合并冲突，且已按下述规则完成 main 增量影响核对；
- 当前提交的必要检查已通过，任务卡列出的验收命令全部通过，所需审查已完成；只改文档时按第 9 节的检查范围执行，不要求本来不适用的 CI/Windows 检查；
- 报告里没有「阻塞」类的待确认项。

**main 增量影响核对：**

1. `git fetch origin` 后固定本次核对的 main 提交 M。对各项必要检查，从最近一次有效验证的运行记录或原始证据确定当时的 PR head、实际被测提交、实际使用的 main 基线 B 和结果；不同检查使用不同基线时分别核对。PR 合成合并提交的 B 是该次实际 checkout 的合并提交的 main 父提交，须核实另一个父提交对应被验证的 PR head；直接测试分支或本地提交时，依据当时的 checkout 与合入历史确认 B。不能拿当前 PR base、当前合成合并提交、分支落后数量或最后一次提交代替当时的基线。
2. 检查 B 到 M 的全部新增提交及文件 diff（如 `git log B..M`、`git diff B M`），结合 PR 的构建、运行、测试和依赖关系判断影响。先确认 B 是 M 的祖先；基线证据缺失、历史关系异常或无法确认相关性时，在报告或 PR 说明中列明不确定点，不得按「无影响」放行。仅作无影响核对不产生新的实际验证基线，下一次仍从 B 检查。
3. 若只有不影响产品构建、运行和测试的文档、报告或代理配置变化，不要求同步 main，也不因此重跑 CI/Windows。不能只凭目录或扩展名认定无影响：契约、验收要求、生成输入等即使写在文档里也要检查其含义；代理配置若影响测试命令或环境，同样不能直接豁免。
4. 若涉及相关源码、依赖、契约、生成器、工具链、测试或工作流变化，则 merge main，并运行受影响的检查，验证 PR 与 main 的新组合；原有验收标准与必要审查不变。存在合并冲突时必须先同步并解决冲突，再按解决后的实际改动运行受影响的检查；不能以文档变化为由忽略冲突。
5. 在现有报告或 PR 说明中简记 B、M、验证证据、变化范围、影响判断及需要运行的检查。合并前若 main 又有变化，继续核对；PR head 改变时核验新 head 的必要检查与审查，不把旧 head 的绿灯当成新 head 的结果。仅因 main 前进且核对为无影响，不使当前 head 的有效检查失效。

**合并前核对四样（2026-10-25）：** 不论执行者自己合并还是用户合并，合并前对**当前 head** 逐项核对：CI 结论（看这个提交的检查，不拿旧提交或本机的通过代替）；报告的「当前状态」；没有勾选的验收项与待确认项；`reports/<任务号>/BLOCKED.md` 是否还有未解除的项。判断 PR 状态时先读最新提交和最后一条评论，不只看更新时间。

合并一律用 squash。提交信息里不写 `[skip ci]`：squash 会把它带进 main，main 的检查就被跳过（只改文档的提交本来就不跑 CI，见第 9 节）。

**审查结论**写成一条 PR 评论，第一行固定是「审查结论：通过」或「审查结论：有阻塞问题」，下面逐条写问题：文件和行、原因、建议。看板按第一行判断状态。有阻塞问题时，执行者修改后在 PR 里回复，审查方复审，再写一条新的结论。审查方只审查，不替执行者改代码。

审查方由用户安排，看板的「等你处理」会提示；执行者只在 PR 里写「请审查」。

**未核验就合并：** PR 在审查完成前被合并时，给它加 `unverified` 标签。审查方照常补完审查，发现的问题另开修复 PR，全部解决后去掉标签。依赖这张卡的卡在标签去掉之前不能开工。

以下情况属于「阻塞」，不论难度：PR 加上 `needs-user` 标签，由用户合并，合并即批准；执行者不自己合并。等用户做决定时 PR 保持草稿；转为可审查并带 `needs-user`，表示其余合并条件都已满足，只差用户合并。

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

**编号（2026-10-25）：** ADR 编号和差异登记编号（`V2-NN`）由任务卡预先占用；卡里没写的，取 `origin/main` 与所有开着的 PR 都没用过的下一个号。开 PR 前、合并前各 `git fetch` 一次，号被占了就顺延，并在报告和 PR 说明里写明原号与新号。不改别人已经用掉的号。

## 8. 技能文件

| 技能 | 说明 | 何时读 |
| --- | --- | --- |
| `.agents/skills/stage-visual-check/SKILL.md` | 如何在浏览器里确定性地渲染舞台并做截图对比 | 做渲染、动画相关任务前必读 |
| `.agents/skills/blender-headless/SKILL.md` | 如何编写和运行 Blender 无头脚本 | 做资产管线相关任务前必读 |
| `.agents/skills/echo-core-work/SKILL.md` | 如何把 Echo Core 规格条款对应到代码、测试和报告 | 做 P5、P6 的卡前必读 |

Antigravity 会自动发现这些技能；Codex 请在开始相关任务时手动读取。

## 9. 子代理与资源调度

- 主代理可以把机械性工作交给子代理（项目配置 `.codex/agents/routine-reporter.toml`，模型与推理强度以配置为准，不得静默降档或换模型）：跑测试、收集日志、校验哈希、整理报告素材、只读查询 CI。子代理只报告证据，不修复、不提交、不推送、不合并、不做判断或决定，也不再委派；主代理核对后入库。
- 重负载一次只运行一个：编译、Electron/Playwright/视觉测试、Blender、GPU 测量。子代理不启动重负载；只读的轻量整理可以并行，但不能干扰安静测量窗口。
- Windows runner 默认常驻在线（2026-10-19 用户调整）。runner 空闲时只是一个等待作业的小进程，不影响本机测试；冲突只发生在 CI 作业正在执行的那几分钟，表现为单实例锁、端口或命名管道被占用、置顶窗口挡住视觉截图。因此：
  - 本机运行 Electron、Playwright 或视觉测试前，先看有没有 CI 作业在执行（存在 `Runner.Worker` 进程就是有）。没有就直接运行；有就等它结束再开始。
  - 测试中途遇到 CI 作业开始执行而失败的，等作业结束后重跑，不当作代码问题，也不为此改测试或阈值。
- GPU 测量需要用户安排的安静窗口：主代理提前告诉用户要测什么、大约多久，用户停掉 runner 后再开始。测量前确认 `Runner.Listener` 和 `Runner.Worker` 都没有在运行，在运行或无法确认就停止并报告。窗口内不运行其他测试或 GPU 工作；测完通知用户重新启动 runner。
- PR 需要 Windows 检查而 runner 不在线时，由主代理通知用户，并列出相关的 PR 和提交。

**CI 的范围（2026-10-20 按用户要求追加）：**

- 只改文档、任务卡、报告或 `content` 子模块指针的 PR 和提交，不跑 CI（`ci.yml` 的 `paths-ignore`）。
- `docs/echo-core/original/` 或 `tasks/cards.json` 有改动时，由 Docs guard（`docs-guard.yml`）检查：前者一律不允许改，后者检查能否解析、依赖和里程碑是否存在。
- Windows 检查在 `windows.yml` 里，在你的电脑上运行，按改动只跑相关的部分：桌面（传输、隔离、端到端、视觉回归）和存储（W02），对应的目录写在该文件开头。纯 Linux 就能验证的改动（如 core-kernel 的数学部分、文档、研究报告）不触发它。草稿 PR 不跑 Windows 检查，转为可审查时自动运行；草稿阶段需要时，用 `gh workflow run windows.yml --ref <分支>` 手动触发，手动触发默认全量。
- 转为可审查以后还要连续修改时，先转回草稿，改完再转回，避免每推一次就在用户电脑上跑一轮。
- 新增一类 Windows 检查时，同时把它涉及的目录加进 `windows.yml` 开头的路径列表和「Decide scope」步骤。

## 10. Claude（2026-10-19 用户追加，同日按新的第 2、6 节整理）

用户已决定增加第三个执行者 Claude（Claude Code 云端会话）。Claude 遵守本文件全部规则，另有以下几点：

- **Claude 的卡：** `tasks/cards.json` 中建议执行者为 Claude 的卡，即 `core/crates/core-policy/`、`core-kernel/`、`experiment-world/` 里不依赖 Windows 和显卡的 Echo Core 部分。其他卡由用户在派活时点名。
- **审查：** 用户转来时审查（第 6 节）。
- **环境：** Linux 云端临时会话，没有 Windows、显卡和用户的机器，也不保存本地状态。验收需要这些的卡，Claude 不认领。所有进度以推送到 GitHub 的分支、PR 和报告为准。
- **工具链：** Claude 的环境可能无法安装 `core/rust-toolchain.toml` 锁定的版本。此时 Claude 可以用环境内可用的 stable 版本做本地开发和测试，但以 CI 中按锁定版本运行的结果为准；报告里写明本地实际使用的版本。
- **Windows CI：** Claude 的 PR 同样会触发 Windows 作业。需要 Windows runner 时，Claude 提前通知用户，不自行判断不需要。

## 11. 云端执行（2026-10-25，按 P5-25 云端试点整理）

云端会话（Codex 云端、Claude 云端）没有用户的电脑，状态只以推送到 GitHub 的内容为准。试点中遇到过：

- 无法完整克隆仓库，只能经 GitHub 接口逐个文件提交；
- 大文件上传失败，2.4 MB 的证据对象丢失，导致实现当时没有生成提交；
- 会话中途失效。

所以：

- **适合放云端的：** 研究、ADR 草案、文档、只需 Linux 的小范围检查。
- **留在用户电脑上的：** 实现卡，尤其需要全工作区编译测试、Windows 或显卡验收的卡。
- **云端提交后**核对远端文件的 blob 哈希与本地一致；超过约 1 MB 的证据文件先确认能上传，必要时无损分片，并记录完整文件和各分片的哈希。
- **会话可能随时中断**，每完成一个可独立核对的步骤就推送一次，不攒到最后。
