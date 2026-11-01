# P0 基础与可行性（第 1–3 周）

出口条件（N0 技术验证，2026-11-08，你在 P0-09 评审）：

- 示例 VRM 在透明窗口里正确显示，点击穿透正确，窗口策略已按判定规则选定；
- Echo Core 基线与差异登记完成；
- 载体探针证明 J 的注入通路在 3080 上可行，并选定载体；
- Codex app-server 与 IRL 路径实测完成；
- 你根据实测数据确认完成标准第 8 项的延迟目标。

---

## P0-00 借鉴清单（参考项目实测）

- **建议执行者：** Antigravity；你花约 30 分钟亲自试用
- **难度：** 低
- **里程碑：** N0 技术验证
- **依赖：** 无
- **工期：** 半天
- **分支：** `task/P0-00-borrow-list`
- **目录：** `reports/P0-00/`（只写报告）

### 目标
路线已经定了，这不是决定路线的评估。目的是在开工前看清楚：同类开源项目里，哪些零件可以直接借用，哪些坑可以提前绕开。

### 步骤
1. 在你的 Windows 机器上安装并运行 AIRI 桌面版（moeru-ai/airi，MIT）和 Mate-Engine（AGPL-3.0），各载入一个示例 VRM，试用 15 分钟。
2. 阅读 AIRI 仓库中与本项目对应的部分：three.js VRM 舞台、音频流水线、Electron 桌面窗口。
3. Mate-Engine 只观察交互设计（拖拽、坐窗口、待机动作、设置项），**不阅读、不复制源代码**，避免 AGPL 牵连。
4. 打开 Codex 桌面应用的宠物（`/pet`），观察它怎样在不打扰的前提下表现代理正在做什么、需要你、做完了。只看交互。

### 交付物（`reports/P0-00/borrow-list.md`）
- 可以借用的代码：AIRI 中的文件或模块路径、用途、许可证、建议放进本项目的哪张任务卡。
- 值得学习的交互设计：从三者中共列 8 条，标明对应的任务卡。
- 要避开的问题：从 AIRI、Mate-Engine 的 issue 区里，分别找出与透明窗口、VRM 渲染、语音延迟相关的高频问题各 3 条，写明本项目的应对。
- 你的试用感受（由你补充 3–5 句话）：哪里让你觉得「对，就是这个」，哪里让你觉得「不是我想要的」。

### 验收
- 清单中每条可借用代码都有可访问的仓库路径和许可证。
- 你在报告末尾补充了试用感受。

---

## P0-01 单仓脚手架与 CI

- **建议执行者：** Codex
- **难度：** 中
- **里程碑：** N0 技术验证
- **依赖：** P0-05 第一部分
- **工期：** 3 天
- **分支：** `task/P0-01-monorepo`
- **目录：** 整个仓库（本卡开工时没有其他卡在做）
- **派任务时告诉 Codex：** 旧原型仓库在你电脑上的路径。

### 目标
建立三种语言的工作区：pnpm（应用）、Cargo（Echo Core）、uv（worker 与旁路服务），并让它们都能构建、测试、在 CI 中运行。

### 交付物
- **根目录与 pnpm 工作区：**
  - `package.json`：`packageManager` 字段，以及 AGENTS.md 第 4 节的全部命令，暂未实现的命令先占位并给出说明；
  - `pnpm-workspace.yaml`：包含 `apps/*`、`packages/*`、`executors/codex-bridge`、`tools/*`、`spikes/*`；
  - `tsconfig.base.json`（strict）、ESLint flat config、Prettier、`.nvmrc`（Node 24）、`.gitignore`；
  - `.gitattributes`：二进制测试夹具和视觉回归基准图走 LFS。
- **Echo Core 工作区 `core/`：**
  - `rust-toolchain.toml`、`Cargo.toml`；
  - crate 骨架：`core-types`、`core-policy`、`core-kernel`、`core-store`、`core-runtime`、`core-replay`、`fact-view`、`semantic-core`、`gateway`、`action-gate`、`task-layer`、`echoctl`；
  - 依赖方向按总方案第 3 节：runtime → store、kernel、policy、types；kernel 不依赖 store、runtime；
  - `core/contracts/`（IDL 占位）、`core/tests/acceptance/`（按验收编号建目录的说明）。
- **Python 工作区：** `workers/irl`、`workers/drive`、`workers/model-adapter`、`services/voice-sidecar`，用 uv 管理（`pyproject.toml`、`uv.lock`），各带一个示例测试。
- **执行器骨架：** `executors/codex-bridge`（TypeScript）、`executors/office-connector`（.NET 10 + `global.json`）。
- **其他目录：**
  - `profiles/structural_fixture/`、`profiles/deployment/`：空的 profile 和它们的 schema 占位；
  - `docs/traceability/`：`baseline.md`、`diff-register.md` 的模板；
  - `config/models.lock.json`：含 `codex` 条目，以及它的 JSON Schema。
- **`content-template/`：** 内容仓 `nemu-content` 的初始结构，按 CONTRACTS.md 1.7 建好目录、空的 `manifest.json`、`build.yaml` 样例、`LICENSE.md` 占位，以及内容仓自己的 `.gitattributes`（`*.vrm *.vrma *.vroid *.wav *.png *.psd *.kra` 走 LFS）。你把它复制进 `nemu-content` 仓库即可。
- **应用骨架：**
  - `apps/desktop`：electron-vite 工程，主进程 + preload；启动透明的舞台窗口（占位）和普通的面板窗口；
  - `apps/stage`、`apps/panel`：Vite + TypeScript，panel 用 React 19；stage 可以在 Electron 里运行，也可以通过 `pnpm dev:stage` 在普通浏览器里单独运行；
  - `packages/*`：每个包有 `package.json`、`src/index.ts`、一个示例测试；包名统一为 `@nemu/<目录名>`；另建 `@nemu/log`。
- **CI：** `.github/workflows/ci.yml` 分两组 job：
  - GitHub 托管的 Ubuntu 机器：Rust、Python、TypeScript 的静态检查和单元测试。核心的传输层先抽象，Linux 上用 Unix socket 替身。
  - 你机器上的自托管 Windows runner：命名管道、低完整性进程、Electron 端到端、视觉回归。
  - 在报告里估算每月托管分钟数；超过免费额度 80% 时，在 CI 日志里提醒。
- `.github/pull_request_template.md`：引用 AGENTS.md 的报告模板。
- **旧原型迁移：** 盘点旧仓库，把可复用的聊天界面组件移入 `apps/panel/src/legacy/`，在报告里列出迁移了什么、丢弃了什么。

### 验收
- `pnpm i; pnpm lint; pnpm typecheck; pnpm test; pnpm build` 通过。
- `cargo test --workspace`（在 `core/`）通过；`uv run pytest`（各 Python 工程）通过。
- `pnpm test:e2e` 冒烟测试通过：启动应用，两个窗口都出现，舞台窗口透明。
- CI 两组 job 首次运行全绿（报告里附运行链接）；自托管 runner 的安装步骤写进 `docs/SETUP.md`。

### 禁止
- 不引入 Nx、Turborepo 等额外编排工具。
- 不要在这张卡里实现任何业务逻辑。

---

## P0-02 应用契约包

- **建议执行者：** Codex
- **难度：** 中
- **里程碑：** N1 认得出是她
- **依赖：** P0-01
- **工期：** 2 天
- **分支：** `task/P0-02-contracts`
- **目录：** `packages/contracts/`（`src/generated/` 只建占位，内容由 P5-01 生成）

### 目标
在 `packages/contracts` 里实现 CONTRACTS.md 第 1、2、3、7 节与第 4.1 节：TypeScript 类型、zod schema、流式表演标记解析器、三语分句器。第 4.2–4.4 节的类型将由 P5-01 从 IDL 生成，本卡只建好生成目录和占位导出。

### 交付物
- `src/asset.ts`：`NemuProfile`（v2）、`AssetManifestEntry`。
- `src/motions.ts`：30 个动作槽位 id、类别、「可表演」与「可请求」标记。
- `src/stage.ts`：舞台协议全部命令与事件、`StageMessage` 外壳、三条通道各自允许的消息类型，以及 `PhysicalState`、`BodyAction`。
- `src/performance.ts`：`PerformanceCue`、`PerformanceSentence`、`Emotion`、`Lang`；`createPerformanceParser()` 输入流式文本块，输出 `PerformanceSentence` 的异步序列，严格遵守 3.2 的规则（含三语分句）。
- `src/appbus.ts`：`AppBusEvent` 外壳和全部事件类型。
- `src/readout.ts`：`BodyReadout` 与取值范围校验。
- `src/generated/`：IDL 生成类型的目录与占位。

### 验收
- 解析器单元测试 ≥ 60 个用例，至少覆盖：
  - 标记跨文本块断开、未闭合标记超过 48 字符、未知 kind、每句超过 3 个标记、句末标点后的标记；
  - 中英日混合标点、中文 40 字强制切分、中文首句 20 字快速切出、英文 25 词切分、英文首句 8 词快速切出；
  - 英文缩写与小数点不切（≥ 10 例）、省略号、空句、连续标点。
- `performance.ts` 行覆盖率 ≥ 90%。
- 全部 schema 都有「合法样例通过、非法样例拒绝」的测试。

### 禁止
- 不得自行增加 CONTRACTS.md 里没有的字段；发现缺漏时写进报告，并按 AGENTS.md 第 7 节提 ADR。

---

## P0-03 透明舞台技术验证

- **建议执行者：** Antigravity
- **难度：** 中
- **里程碑：** N0 技术验证
- **依赖：** P0-01
- **工期：** 3 天
- **分支：** `task/P0-03-stage-spike`
- **目录：** `spikes/stage-overlay/`（本卡允许在该目录里写主进程代码）

### 目标
用实测数据回答三个问题：

1. 透明叠加窗口 + 小画布方案的性能和兼容性是否达标；
2. 逐像素点击穿透是否可靠；
3. alpha 边缘是否干净。

然后按判定规则选定窗口策略。

### 步骤
1. 用 three-vrm 官方示例模型（许可允许测试使用的 VRM 1.0 模型），在 Electron 透明窗口中渲染。
2. 实现策略 B：叠加窗口覆盖当前显示器工作区，角色大小的画布用 CSS transform 移动；画布可拖动。
3. 实现策略 A：角色大小的窗口，拖动时移动窗口本身。
4. 两种策略都实现逐像素穿透：`setIgnoreMouseEvents(true, { forward: true })` + 光标移动时（最多 30 Hz）用 WebGL2 异步读回取光标处像素的 alpha。另外实测一次「每帧同步 `readPixels`」的帧时间，作为对照写进报告。
5. 写基准脚本：待机 30 fps 与活动 60 fps 各跑 5 分钟。采样工具：
   - `process.getCPUUsage()`；
   - `app.getAppMetrics()`；
   - `nvidia-smi --query-gpu=utilization.gpu,memory.used --format=csv -lms 500`。
6. 在 2560×1440 的 100% 和 150% 缩放下各跑一遍；有 4K 显示器则加测。
7. 分别在白、黑、彩色壁纸上截图，检查描边和半透明边缘。

### 判定规则（写死，不讨论）
- 策略 B 同时满足以下四条 → **选 B**：
  - 待机 CPU ≤ 3%、GPU ≤ 8%；
  - 拖动时无撕裂和残影；
  - 穿透测试 100% 正确；
  - 两种缩放下都没有渲染异常。
- 否则，如果策略 A 满足以上条件 → **选 A**。
- 两者都不满足 → 写 BLOCKED.md，附数据和 3 个优化方向，交给你决定。

### 交付物
- `spikes/stage-overlay/`：可运行的验证代码。
- `reports/P0-03/benchmark.md`：两种策略的数据表、截图、结论。
- `reports/P0-03/decision.md`：按判定规则得出的选择，一句话结论放在第一行。

### 验收
- 你能用 `pnpm --filter stage-overlay-spike start` 复现。
- 报告中每个数字都有原始采样数据（CSV）作为证据。

---

## P0-04 环境检查脚本

- **建议执行者：** Codex
- **难度：** 低
- **里程碑：** N0 技术验证
- **依赖：** P0-01
- **工期：** 1 天
- **分支：** `task/P0-04-doctor`
- **目录：** `tools/doctor.ps1`、`tools/doctor.Tests.ps1`、`docs/SETUP.md`

### 目标
写一个 PowerShell 脚本，检查你机器上的全部开发与运行依赖，缺什么就给出安装命令。

### 交付物
- `tools/doctor.ps1`，逐项检查并输出表格（项目、要求、实际、状态、修复命令）：
  - Git、Git LFS；Node 24、pnpm（corepack）；Python 3.12、uv；rustup 与 MSVC 目标；VS Build Tools 的 C++ 工作负载（用 `vswhere`）；.NET 10 SDK。
  - NVIDIA 驱动与 CUDA（`nvidia-smi`）；当前空闲显存；在 uv 环境里 `torch.cuda.is_available()`。
  - Codex CLI：版本与 `config/models.lock.json` 一致；已登录（只检查状态，不读取凭据）。
  - Blender：版本 ≥ 5.1，与 `tools/asset-pipeline/blender-version.txt` 一致；VRM Add-on 已启用。
  - VRoid Studio 已安装；Microsoft Office 桌面版（读注册表）。
  - `%LOCALAPPDATA%` 位于本机 NTFS 卷，且没有被 OneDrive 等重定向（E3）。
  - 能创建只允许当前用户访问的命名管道；能以低完整性级别启动一个测试进程，且该进程写入中完整性目录会被拒绝（E1、E2）。
  - Windows 版本与版本类型（家庭版没有 Windows Sandbox，P7-02 需要知道）。
  - 磁盘剩余空间：模型约 25 GB；另有本地动作生成需求时再加约 30 GB。
- 修复命令优先使用 `winget` 的包 id。
- `docs/SETUP.md`：首次开发环境搭建说明（中文）。

### 验收
- 缺少必需项时退出码非 0。
- 用 Pester 写 ≥ 8 个单元测试，模拟「已安装」「版本过低」「未安装」三种情况。
- 在你机器上运行的输出保存为 `reports/P0-04/doctor.txt`，由你在 P0-05 第二部分提交。

---

## P0-05 仓库创建与环境安装（你）

- **负责：** 你
- **依赖：** 第一部分无依赖，**必须在派出任何任务之前完成**；第二部分依赖 P0-04
- **工期：** 第一部分约 1 小时，第二部分约 2 小时

### 第一部分：开工前
- [ ] 在 GitHub 上建两个私有仓库：`nemu`（代码）和 `nemu-content`（内容包）。留意 GitHub 的 LFS 存储额度。
- [ ] 克隆 `nemu` 到 `D:\dev\nemu`，把本任务书包的全部文件（包括隐藏的 `.agents/` 目录和 `docs/echo-core/original/` 里的规格原文）放进仓库根目录并提交。
- [ ] 安装代理开工必需的基础工具链（在 PowerShell 里执行；包名以 `winget search` 的结果为准）：
  ```powershell
  winget install Git.Git GitHub.GitLFS Schniz.fnm Python.Python.3.12 astral-sh.uv Microsoft.DotNet.SDK.10 Rustlang.Rustup
  winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive"
  # 让 fnm 在每个 PowerShell 窗口里生效（只需做一次），然后重开 PowerShell
  Add-Content $PROFILE 'fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression'
  fnm install 24; fnm default 24; corepack enable
  git lfs install
  rustup default stable
  npm install -g @openai/codex   # Codex CLI，供 P0-08 和 codex-bridge 使用；装好后运行 codex login 用你的 ChatGPT 账号登录
  ```
- [ ] 在 Codex 和 Antigravity 中打开仓库，确认它们都能读到 AGENTS.md。
- [ ] 在 Codex 里确认可以同时开三个线程（见 `tasks/README.md` 第 2 节）。

### 第二部分：P0-04 完成后
- [ ] 把 P0-01 生成的 `content-template/` 复制进 `nemu-content` 仓库并提交，再把它作为子模块挂到 `nemu/content/`。
- [ ] 运行 `tools/doctor.ps1`，按提示安装其余软件（Blender、VRM Add-on、VRoid Studio、NVIDIA 驱动等），直到全部通过。
- [ ] 按 `docs/SETUP.md` 注册 GitHub 自托管 runner（只给 `nemu` 这个私有仓库用）。
- [ ] 把 `doctor.ps1` 的输出提交为 `reports/P0-04/doctor.txt`。

---

## P0-06 Echo Core 基线与差异登记（W00）

- **建议执行者：** Codex
- **难度：** 中
- **里程碑：** N0 技术验证
- **依赖：** P0-01
- **工期：** 2 天
- **分支：** `task/P0-06-echo-baseline`
- **目录：** `docs/traceability/`
- **必读：** `docs/echo-core/README.md` 全文；五份规格的目录与第 1 节；总方案第 1、2、26、31 节

### 目标
完成总方案的 W00：固定规格版本、登记差异、锁定构建环境、列出 v1.0 范围清单。之后所有 Echo Core 任务卡都以此为准。

### 交付物
- **`docs/traceability/baseline.md`：**
  - 五份规格原文（`docs/echo-core/original/`）的文件名、文档内标题、SHA-256；
  - 优先级顺序；
  - 缺失的「Review_and_Remediation」增补（架构规格第 20–27 节、施工合同附录 B）及其影响范围。
- **`docs/traceability/diff-register.md`：**
  - 把 `docs/echo-core/README.md` 第 4 节的 E1–E14 逐条登记，每条写明规格原文位置、Nemu 的做法、是否改变冻结边界、状态（已批准、待确认）；
  - 通读时发现的规格内部冲突或歧义（例如 v0.6.4 与 v1.0 对同一对象的不同表述），逐条登记，并写出建议的最保守解释。
- **`docs/traceability/build-lock.md`：** Rust、Python、Node、.NET 版本；SQLite 版本（rusqlite 内置构建，需包含 WAL 相关修复）；Windows 版本；CPU 指令集配置。
- **`docs/traceability/scope-v1.md`：** v1.0 范围清单。工作包对应到任务卡；验收编号对应到任务卡；范围外项目标明「v1.x」。

### 验收
- 五份原文的摘要可以用 `Get-FileHash` 复核。
- 差异登记里的每条都有原文引用（文件 + 节号）。
- [ ] 你确认差异登记里新发现的冲突（阻塞项）。E5、E13、E14 你已于 2026-10-13 确认，登记为已批准。

### 禁止
- 不得修改 `docs/echo-core/original/` 里的任何文件。
- 不得在登记中「顺便」决定规格未决的问题；只列出选项和最保守的默认做法。

---

## P0-07 载体探针（W12 可行性）

> **高难度卡：** 由 Claude 与 Codex 协作完成。一方实现，另一方交叉审查；审查结论写在 PR 评论里，通过后才能合并（AGENTS.md 第 6 节）。

- **建议执行者：** Codex
- **难度：** 高
- **里程碑：** N0 技术验证
- **依赖：** P0-01
- **工期：** 4 天
- **分支：** `task/P0-07-carrier-probe`
- **目录：** `workers/model-adapter/probe/`
- **必读：** 《核心算子》第 12–14 节；总方案第 2 节最后一段、第 19 节

### 目标
在你的 3080 上验证 J 的注入通路：冻结的开源 decoder 能否在一个固定残差位置接收低秩激活加法，零调制时与基准一致，注入差值可测，缓存在 m 改变时正确失效；同时测出显存和延迟，选定 v1.0 的载体。这是整个项目最大的技术未知，必须最先做。

### 步骤
1. 选 2–3 个支持中、英、日的 4B 级开放权重模型作为候选（当时可获得的版本，记录仓库与修订哈希），以及 1 个 8B 级模型用于对照。
2. 用 PyTorch 写显式 forward 包装：在配置的层号、残差相加之后、下一归一化之前注入 `U·a`。U 的算子范数 ≤ 1；prefill 最后位置和生成位置的掩码规则固定。
3. 测量：
   - 零调制路径与原模型 logits 的最大差；
   - 注入后实测的 `h' − h` 与 `U·a` 的误差；
   - m 改变时 KV 缓存是否被正确判为失效。
4. 量化：每个候选分别测 bf16、8 bit、4 bit。在同时加载 Qwen3-TTS 0.6B 和渲染占位的条件下记录总显存，以及 20 字回答的首句延迟、每秒 token 数。
5. 信息性的方向检查（不是验收）：用少量对比提示（例如「平静」与「兴奋」的同一句话）提取一个方向，在中、英、日三种语言下各生成 20 次，看输出是否朝预期方向变化、是否损害流畅度。
6. 8B 级模型在 4 bit 下重复第 4 步，为是否加装显卡提供数据。

### 判定规则
- 选定的载体必须同时满足：
  - 通路检查通过（零调制一致、注入差值在容差内、缓存失效正确）；
  - 与 TTS 0.6B 和渲染同时运行时，Nemu 合计显存 ≤ 9 GB；
  - 20 字回答的首句延迟 p50 ≤ 600 ms；
  - 三种语言的方向检查都能看到变化，且没有明显的流畅度损失。
- 满足的候选中选首句延迟最低的。
- 都不满足 → 写 BLOCKED.md，附数据和选项（更小的模型、TTS 改用 CPU 或云端、加装显卡、默认使用模式 B），交给你决定。

### 交付物
- `workers/model-adapter/probe/`：可复现的探针脚本。
- `reports/P0-07/benchmark.md`：全部数据表。
- `reports/P0-07/decision.md`：第一行写一句话结论，包括选定的模型、hook 位置、rank、量化方式。
- `config/models.lock.json` 中写入选定载体的条目。

### 验收
- 你能用一条命令复现所有测量。
- 报告中每个数字都有原始数据。

> **2026-10-20 追加（v2.0）：** 第 5 步的方向检查仍是信息性的，判定规则不变。v2.0 正式的方向提取与验证（四个天性方向、按概念提取学得方向、通路与能力验证、换模型）在 P5-31，本卡不扩大范围。

> **2026-10-20 追加（ADR 0012）：** 用户决定只围绕 Qwen3-4B-Instruct-2507 收尾，不再加候选。判定规则与阈值不变，口径按 ADR 0012 第 2 条：零调制的基准是同一载体配置（含内核）不装 hook 时的输出，跨内核差记录在案、不算通路失败；显存以专用与共享显存最高观测值之和为上界，≤ 8 GB 通过；延迟以含采样器的测量为准，3 次独立运行的 p50 都 ≤ 600 ms；方向检查是信息性的，建议只对 Instruct-2507 按第 ℓ 层 ‖h‖ 的比例重跑一次第 5 步。ADR 0012 合并之前，`decision.md` 不写选定结论。

---

## P0-08 Codex app-server 与 IRL 路径验证

- **建议执行者：** Codex
- **难度：** 中
- **里程碑：** N0 技术验证
- **依赖：** P0-01
- **工期：** 2 天
- **分支：** `task/P0-08-codex-irl-spike`
- **目录：** `spikes/codex-app-server/`
- **必读：** ADR 0003、ADR 0004；v0.6.4 第 3.1–3.3、19.1 节

### 目标
实测四件事：模式 B 的首个文本增量延迟、IRL 结构化输出的延迟与合格率、app-server 的打断与审批行为、额度消耗。据此决定 IRL 的默认后端，并修订完成标准第 8 项的延迟目标。

### 步骤
1. 在独立的 `CODEX_HOME` 下启动 `codex app-server`（版本写入 `config/models.lock.json`），用你的 ChatGPT 登录；用同版本的 `generate-ts` 生成类型。
2. 列出当前账号可用的模型，核对 ADR 0004 的三个档位，写入 `config/helpers.json` 草稿。
3. 「只能说话」线程：配置按 ADR 0003（无工具、只读沙箱、无网络、`personality: none`、`base_instructions` 为一份样例协议 p）。在「日常」档、低推理强度下，三种语言各 50 次短回答，测首个 `item/agentMessage/delta` 的延迟。
4. IRL 结构化输出：用 `outputSchema`（SemanticCandidate 的 JSON Schema 草稿）对 200 条三语测试句测延迟和 schema 合格率。
5. 用 P0-07 选定的载体，在本地以 m = 0、受约束解码的方式完成同样的 IRL 任务，测延迟和合格率。
6. 验证 `turn/interrupt`、`turn/steer` 的行为；在一个开启工具的测试线程里触发命令执行和文件修改的审批请求，确认它们会交给客户端处理。
7. 记录 100 次短回答和 100 次 IRL 调用消耗的额度（以账号显示为准）。

### 判定规则
- IRL 默认后端：本地后端 p50 ≤ 400 ms 且合格率 ≥ 95% 时选本地；否则远程满足同样条件时选远程；都不满足时选较快的一个。
- IRL 截止时间按所选后端 p90 设定，写入部署 profile 草稿。
- 完成标准第 8 项：用实测数据算出三种模式的预计 p50、p90，写进报告，交给你在 P0-09 确认。

### 交付物
- `spikes/codex-app-server/`：可复现的测量脚本。
- `reports/P0-08/benchmark.md`、`reports/P0-08/decision.md`（第一行写结论）。

### 验收
- 每个数字都有原始数据；审批请求的处理有录屏或日志。
- [ ] 你在 P0-09 确认延迟目标（阻塞项）。

---

## P0-09 N0 评审：技术验证（你）

- **负责：** 你
- **依赖：** P0-03、P0-06、P0-07、P0-08、P0-05 第二部分
- **工期：** 约 1 小时

### 步骤
- [ ] 读 `reports/P0-03/decision.md`，确认窗口策略。
- [ ] 读 `reports/P0-07/decision.md`，确认载体：模型、hook 位置、rank、量化方式。
- [ ] 读 `reports/P0-08/decision.md`，确认三件事：
  - IRL 的默认后端与截止时间；
  - `config/helpers.json` 的三个档位；
  - 三种说话模式的预计延迟。
- [ ] 冻结完成标准第 8 项的延迟目标：照 P0-08 的预计值确认，或者写下你要改成的数字（代理按 ADR 流程写进 ARCHITECTURE.md）。
- [ ] 确认差异登记（P0-06）里新发现的冲突。E5（身体读出，包括姿态和表情）、E13、E14 你已于 2026-10-13 确认，不需要再确认。

### 决策
- 全部确认 → N0 通过。
- 载体在 3080 上不可行 → 在 P0-07 报告给出的选项里选一个：
  - 更小的模型；
  - TTS 改用 CPU 或云端；
  - 加装显卡：模式 A 的载体工作（P5-13 起）在新卡到位后进行，按到货日期重排；
  - 默认使用模式 B，模式 A 推迟到 v1.1：需要一份 ADR，完成标准第 26、29 项的范围随之调整，由你签字。

---

## P0-10 载体推理引擎实测（2026-10-22 追加，N0 决定第 5 项）

- **建议执行者：** Codex
- **难度：** 中
- **里程碑：** N2 她在了（排练版）
- **依赖：** P0-07
- **工期：** 3 天
- **分支：** `task/P0-10-carrier-engine`
- **目录：** `spikes/carrier-engine/`
- **必读：** `reports/P0-09/DECISION.md`、ADR 0012、`reports/P0-07/decision.md`、`reports/P0-08/REVIEW-R1-REPORT.md`、v2.0 第 9.1、9.5 节

### 目标
P0-08 实测本地载体去掉约束后只有约 21 token/s，瓶颈在 transformers 逐步执行和 NF4 运算核，不在显卡。本卡在 3080 上实测换用 llama.cpp 能快多少、能否保住注入，作为 IRL 架构、模式 A 延迟和 P5-13 选引擎的依据。只测量，不选型。

### 步骤
1. **速度：** 同一个 Qwen3-4B-Instruct-2507 修订，取 2–3 种量化（例如 Q4_K_M、Q8_0），测 prefill、首 token、decode token/s；与 P0-07、P0-08 的 transformers 基线用同一批输入对比。
2. **注入：** llama.cpp 的 control vector 加在层输出的残差上，位置与 P0-07 的 hook（第 17 层 attention 残差之后、post_attention_layernorm 之前）不同。
   - 先如实比较两种位置的差别；
   - 再评估按 v2.0 第 9.5 节在新位置重新提取方向的可行性；
   - 测零调制时与原模型逐 token 的一致性；
   - 确认能否按每次请求更新注入向量（x 每个事件都会变）：服务端文档没有写明，库接口要实测。
3. **受约束输出：** 用 P0-08 的 200 条 IRL 输入和 SemanticCandidate 的 JSON Schema，测 llama.cpp 的 JSON Schema 约束解码的完成时间和合格率；另测一个紧凑版 schema（编号与枚举代替长字符串）的输出 token 数和耗时。
4. **显存：** 与 P4-02 的 TTS 0.6B 和渲染同时驻留，按 ADR 0012 §2.2 测合计显存。

### 验收
- 以上数字全部有原始记录，失败结果保留；与基线的对比表写进报告。
- 列出给 P0-09 待补项的结论草稿：可选的引擎、注入方案、IRL 能否在可接受的时间内完成。不自行选型，由用户在 ADR 中决定。
- 需要 GPU 安静窗口，按 AGENTS.md 第 9 节执行，与其他测量错开。
- 新增依赖登记版本与许可（llama.cpp 为 MIT）。

---

## P0-11 --cache-ram 0 TTFT 补测

- **建议执行者：** Codex
- **难度：** 低
- **里程碑：** E3 她有声音（前置测量）
- **依赖：** P0-10
- **工期：** 一个 GPU 窗口及报告
- **分支：** `task/P0-11-cache-ram-zero-ttft`
- **目录：** `reports/P0-11/`（使用既有 `spikes/carrier-engine/` 测量入口，不改生产实现）
- **必读：** [ADR 0031](../docs/adr/0031-carrier-engine-and-injection.md)；`reports/P0-10/BENCHMARK-REPORT.md`、`benchmark-run-03/README.md` 与原始文件映射；AGENTS.md 第 9 节。

### 目标

在下一个用户安排的 GPU 安静窗口，确认 llama-server 禁用内存提示缓存后的客户端 TTFT。只测量，不选型、不实施载体迁移、不设置新的延迟或截止阈值。本卡是 E3 前置，P6-20 等待本卡完成；不新增 E2 依赖。

### 测量与交付物

- 沿用 P0-10 的 llama.cpp b11669、Q4_K_M 与模型修订 `cdbee75f17c01a7cc42f958dc650907174af0554`，使用既有固定输入和对应运行配置，在启动命令中显式设置 `--cache-ram 0`，请求显式设置 `cache_prompt=false`；m = 0。
- 覆盖既有无约束与原 wire schema 输入，分别记录客户端 TTFT、原生 prefill 和两者的逐请求差值；与 `benchmark-run-03` 的相同输入对比。沿用原报告计时边界与分位数口径，预热单列，失败和慢样本保留，不用分位数相减代替逐请求差值。
- 保存实际命令、配置与输入绑定、server 日志、逐请求原始计时及失败记录；核对日志中的内存提示缓存确已关闭。报告 P50/P90、样本数与实际差异，写入 `reports/P0-11/REPORT.md`；尚未测得的结果写未测。
- 不重跑 Q8_0、紧凑 schema、方向提取或注入实验；GPT-SoVITS V5 与渲染同驻显存仍是独立未决事项，不纳入本卡。

### 验收

- 命令与日志证明 `--cache-ram 0`，请求证明 `cache_prompt=false`；引擎、量化、模型修订、输入与对比基线可追溯。
- 客户端 TTFT 与原生 prefill 分别报告，原始记录完整，失败不删除；如差距仍在，如实记录，不据此自动更换方案或放宽阈值。
- 按 AGENTS.md 第 9 节取得 GPU 安静窗口，测前确认 `Runner.Listener`、`Runner.Worker` 均未运行；只用合成输入，测完通知用户恢复 runner。
- 交付测量与结论即为本卡范围；不以本卡签发 P5-31、完整语音延迟或同驻显存验收结论。
