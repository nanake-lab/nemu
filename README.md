# Nemu 任务书包

这是交给 Codex、Antigravity 和 Claude 执行的全部材料。完整的设计理由、选型对比和风险分析在《Nemu 3D 桌面伙伴 · 总体工程设计 v2.0》文档里，这里只放代理需要的部分。

Nemu 的核心是你设计的 Echo Core。桌面应用是她的身体、感官和声音；Codex 是她可以求助的助手，不是她本人。

**已经开工了？** 直接看 [`tasks/README.md`](tasks/README.md) 的「从这里开始」：进度看板、等你处理的事、怎么派活。

## 怎么用

1. **你先做 P0-05 的第一部分**（`tasks/P0-foundation.md`，约 1 小时）：
   - 建两个私有仓库：`nemu`（代码）和 `nemu-content`（Nemu 本人：模型、动作、声音、人设、genesis）；
   - 把本文件夹的全部内容放进 `nemu` 根目录并提交，包括隐藏的 `.agents/` 目录和 `docs/echo-core/original/` 里的规格原文；
   - 装好基础工具：Git、Node、pnpm、Python、uv、.NET、Rust、VS Build Tools、Codex CLI。
2. **在 Codex 和 Antigravity 里打开这个仓库。** 它们都会自动读取 `AGENTS.md`。
3. **按 `tasks/README.md` 的提示词派第一批任务：**
   - Codex：先开一个线程做 P0-01（单仓脚手架）。它合并后再开另外两个线程，三个线程按 `tasks/README.md` 第 2 节的规则自己取卡；
   - Antigravity：先做 P0-00（借鉴清单），P0-01 合并后接 P0-03（透明舞台技术验证）；
   - 你：P2-01（设定集与人设种子）。P0-04 完成后做 P0-05 的第二部分；N0 的报告齐了以后做 P0-09。

之后代理每合并一张卡就接着做下一张。你只需要处理看板「等你处理」里的事，以及五次里程碑评审：

| 里程碑 | 承诺日期 | 内容 |
| --- | --- | --- |
| N0 技术验证 | 2026-11-08 | 窗口策略、载体探针、Codex 与 IRL 路径实测，你冻结延迟目标 |
| N1 认得出是她 | 2026-12-13 | 身体 V1 站上桌面 |
| N2 她在了 | 2027-01-17 | 排练版：真实的核心与记忆、她的声音和身体；你开始每天使用 |
| N3 诞生 | 2027-02-28 | Echo Core 通过 M2 阶段门，你签字 genesis，执行诞生 |
| N4 发布候选 | 2027-03-28 | 30 项完成标准逐项签字；v1.0 目标 2027-04-11 |

## 目录

| 路径 | 内容 | 主要读者 |
| --- | --- | --- |
| `AGENTS.md` | 三个执行者共用的规则：认领与目录规则、14 条黄金规则、命令、报告模板、审查与合并 | 代理 |
| `docs/ARCHITECTURE.md` | 架构说明与 30 项完成标准 | 代理 |
| `docs/CONTRACTS.md` | 资产契约、舞台协议、表演协议、终端协议、生成接口、权限分级、身体读出、机会规则（v2.1） | 代理 |
| `docs/adr/` | 0001 基线架构，0002 Echo Core 作为核心，0003 三种说话模式，0004 Codex 助手，0005 排练版与诞生 | 你和代理 |
| `docs/echo-core/` | 你的 Echo Core 规格：原文（`original/`）、供代理阅读的转换版、优先级与 v1.0 范围、集成扩展登记 E1–E14 | 代理 |
| `tasks/README.md` | 从这里开始（看板、等你处理的事）、派任务的提示词、取卡规则、依赖、里程碑、你的时间 | 你 |
| `tasks/P0-…P7-*.md` | 70 张任务卡，头部写明建议执行者、难度、里程碑；标「你」的卡是给你的 | 代理 |
| `tasks/cards.json`、`tools/status/` | 每张卡的静态信息，以及从 GitHub 推算状态、生成看板数据的脚本 | 代理 |
| `.agents/skills/` | 三个技能：舞台视觉验证、Blender 无头脚本、Echo Core 施工方法 | 代理 |

## 四条底线

- **Nemu 只属于你。** 仓库默认私有。代码、Nemu 本人、Echo Core 规格三者分开管理，哪天想公开，可以只公开其中任何一部分，不会被依赖或素材卡住。
- **美术由你定。** 代理只做候选、脚本和截图，不替你判断好不好看；整个项目不安排手绘，也不委托外部建模。
- **路线不推翻。** 代理觉得哪里不对，写 ADR 交给你批准；批准前照原设计推进。
- **她的数据不是测试数据。** 排练存储和她诞生后的正式存储都只属于她和你，代理开发和测试时一律使用临时存储。

## AI Contributors

Nemu 由 [nanake-lab](https://github.com/nanake-lab) 发起并拥有。架构、Echo Core 规格、美术取舍、诞生与所有定稿决定都属于项目所有者。

下列 AI 编码代理按 `AGENTS.md` 的规则参与施工。它们只修改任务卡允许的目录，所有改动都经 PR 提交，按卡的难度审查后合并（`AGENTS.md` 第 6 节）；需要用户确认的事项交由项目所有者决定。

| 协作者 | 提供方 | 通常做的卡 |
| --- | --- | --- |
| [Codex](https://openai.com/codex/) | OpenAI | Echo Core（`core-policy`、`core-kernel` 除外）、worker、执行器、旁路服务、桌面应用、契约与工具链；审查 Claude 实现的中、高难度卡 |
| [Claude](https://claude.com/product/claude-code)（Claude Code） | Anthropic | Echo Core 核心数学链：`core-policy`、`core-kernel`（P5-09、P5-10、P5-12）；审查其余中、高难度卡，每日抽查 |
| [Antigravity](https://antigravity.google/) | Google | 舞台、评审台、角色运行时与身体（`apps/stage`、`apps/panel`、`packages/avatar-runtime`、`packages/body`） |

每张卡的建议执行者以 `tasks/cards.json` 和任务卡头部为准，规则见 `AGENTS.md` 第 2、6、10 节。
