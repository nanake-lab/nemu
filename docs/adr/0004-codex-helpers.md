# ADR 0004：通过 Codex app-server 接入助手模型

- 状态：已采纳
- 日期：2026-10-13

## 背景

用户希望 Nemu 通过 Codex 暴露的接口来驱动，并且不绑定任何一个模型：日常运行用 GPT-6 Luna，工作、游戏和更难的任务交给更强的模型，以后有更好的模型就换。

在 Echo Core 里，模型分两类：

- 承载她状态的载体（模式 A、模式 C 的改写步骤）；
- 只做事情、不承载身份的执行者与 worker。

后者可以随时替换。

Codex app-server 是 Codex 官方客户端自己使用的 JSON-RPC 接口，具备本项目需要的能力：

- 线程与轮次（`thread/start`、`turn/start`）；
- 流式文本（`item/agentMessage/delta`）；
- 中途打断与追加（`turn/interrupt`、`turn/steer`）；
- 把审批请求交给客户端（`item/commandExecution/requestApproval`、`item/fileChange/requestApproval`）；
- 结构化输出（`outputSchema`）；
- 自定义基础指令。

OpenAI 把 app-server 标为实验性接口，而且生成的类型只对应生成它的那个 Codex 版本。

## 决策

1. **桥接进程：** `executors/codex-bridge`（TypeScript）负责启动并独占一个 `codex app-server`，设置以下三项：
   - 使用独立的 `CODEX_HOME`，与用户开发用的 Codex 配置分开；
   - 认证使用用户的 ChatGPT 登录，或 API 密钥；
   - Codex 版本锁定在 `config/models.lock.json` 的 `codex` 条目中，TypeScript 类型用该版本的 `codex app-server generate-ts` 生成。
2. **四种用途：**

   | 用途 | 线程配置 | 结果去向 |
   | --- | --- | --- |
   | 模式 B 生成、模式 C 起草 | 只能说话：不开工具、只读沙箱、不联网、无 MCP；`base_instructions` 为协议 p | 回到任务层，随生成回执记录 |
   | IRL worker 的远程后端（P0-08 决定是否采用） | 只读、`outputSchema` 为 SemanticCandidate 的 JSON Schema | 作为 Proposal 交给 Trusted Core 校验 |
   | 助手任务（`helper.ask`、`helper.task`） | 在为该任务新建的临时工作目录里运行；需要的权限由 Echo Core 行动闸门逐项决定 | 执行结果事件写入 H |
   | 以后的游戏连接器 | 同助手任务 | 同上 |

3. **审批：** 助手任务中，Codex 发来的命令、文件修改、权限申请，一律转给 Echo Core 的权限判定（W22）。需要用户时，在面板显示审批卡，结果回复给 Codex。Codex 自己的审批策略设为最严格，桥接层不自动同意任何请求。
4. **模型档位**（只是配置，`config/helpers.json`）：

   | 档位 | 默认模型 | 用途 |
   | --- | --- | --- |
   | `everyday` | GPT-6 Luna | 模式 B、模式 C 起草、IRL 远程后端、简单助手问答 |
   | `work` | GPT-6.1 Sol | 办公任务、代码、需要推理的助手任务 |
   | `hard` | GPT-6 Astra | 用户明确要求或任务卡标注的困难任务 |

   换模型 = 改配置 + 重跑助手评测，不需要改代码，也不影响她的身份。其他供应商可以通过 OpenAI 兼容接口的适配器接入，接口与 Codex 桥接相同。
5. **额度：**
   - 桥接层记录每个档位的用量，并在面板显示。
   - 她自主发起的助手调用有独立预算。
   - 额度耗尽时：助手不可用；模式 B 按 ADR 0003 回退；她会说明原因。
6. **安全：**
   - 助手线程的工作目录不得包含 `%LOCALAPPDATA%\Nemu\core\`。
   - 协议 p、c_task 以外的核心内容不会发给远程模型。
   - 远程模型的输出作为执行结果进入 H 时，带 `model_inference` 一类的认识论标记，不成为独立证据根。
7. **用户自己的 Codex 会话（可选）：** 用户可以在自己的 Codex 配置里安装 Nemu 提供的钩子。钩子只把 `SessionStart`、`PermissionRequest`、`Stop` 三类事件的最小元数据经本机命名管道发给 Nemu，作为观察事件。默认关闭，需要用户授权。

## 备选方案

- **直接调用 OpenAI 的 HTTP API：** 需要 API 密钥按量计费，用不上用户的 ChatGPT 订阅额度；作为适配器保留，不作主路径。
- **Codex TypeScript SDK：** 封装 `codex exec`，不支持把审批交给客户端，也不支持中途追加。否决作为主路径。
- **把 Codex 作为她的「心智」：** 文本 API 不能承载 Echo Core 的状态（ADR 0003），否决。Codex 是她可以求助的能力，不是她本人。

## 后果

- 依赖 app-server 的实验性接口。锁定版本、单点适配、录制回放测试三项一起降低风险；升级 Codex 必须走一张任务卡并重跑测试。
- 用户的 Codex 订阅额度同时被开发和 Nemu 使用，需要在面板里看得见。
