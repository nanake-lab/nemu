# P6-02 1/2：终端客户端与文字聊天

仓库：GitHub `nanake-lab/nemu`。先完整读一遍 `AGENTS.md`，再读 `tasks/P6-presence.md` 中 P6-02 的卡面，包括它的追加说明。

**投入：** token 预算无上限，不要吝惜 token。该读的文件读全，该测的情况测全，证据给全。不要为了省 token 跳过步骤或简化验收。

- **线：** 交付线，中档。
- **分支：** `task/P6-02-terminal`，从 `origin/main` 新建。
- **目录：**
  - `packages/terminal/`
  - `apps/desktop/src/terminal/`（新建）
  - `apps/panel/src/chat/`（新建，最小聊天界面）
  - `tasks/requests/P6-02-chat.md`（本卡原文）
- **不碰：** `core/`、契约、其他 app 目录。缺核心侧接口时，写 `tasks/requests/P6-02-<主题>.md`。
- **预授权：** 目录内的实现层决定由你自己做，在 PR 描述里逐条写「决定 / 理由」。要新增第三方包、改契约或放宽检查时，停下来问。
- **不等待：** 本卡做完写「请审查」就停下。不要为了等别的 PR 合并、等 CI 或等回复，让会话一直跑。

## 为什么只做这一半

下一个里程碑是「第一次真的聊上」：她在桌面上用文字和用户聊天，全部经过核心。本卡只做聊天需要的终端部分。P6-02 卡面里的触摸、停留、桌面观察、Codex 会话、身体动作和语音放到 2/2。

核心进程由 HOST-01 并行在做，两边按下面的约定对接：
- 管道名按 `core/crates/gateway/src/pipe.rs` 的规定。
- 终端令牌放在 `%LOCALAPPDATA%\Nemu\terminal.token`，由核心第一次启动时生成。文件内容是 DPAPI 密文原样：没有附加熵，加密范围是当前用户，见 `core/crates/gateway/src/windows.rs` 的 `protect`。

## 要做的

1. **管道客户端（`packages/terminal/`）**
   - 帧格式、认证消息和各类请求，一律用 `packages/contracts` 的生成类型，并对照 `core/contracts/golden/` 的黄金向量，不手写协议。
   - 令牌用 Windows 自带的能力解密，例如通过 PowerShell 调用 `System.Security.Cryptography.ProtectedData`，不新增第三方包。令牌不进命令行参数、环境变量和日志，只在认证请求里出现。
   - 断线后带退避重连。核心不可用时，在界面上显示原因。
2. **发件箱：** 按 P6-02 卡面执行。
   - 只保存还没收到 `RECEIVED_DURABLE` 的事件；
   - 重发时复用原来的 `stable_source_key`；
   - 收到回执就删除；
   - 不保存任何别的内容（I18）。
3. **主进程接入（`apps/desktop/src/terminal/`）**
   - 主进程持有唯一的核心连接。
   - 应用内总线的 `chat.send` 转成 `user_text`。
   - 核心下发的 `output.chunk`、`output.end` 放到总线上，给聊天界面用。
   - `run_state` 转成 `core.runState`。
   - `approval.request` 和 `approval.decision` 经总线往返。
   - 界面上的「停止」发 `output.cancel`。
   - 网关现在还没有从核心向终端推送的通道，由 P5-53 补。下行这部分先按契约写好，用下面的假核心测试，不用等它。
4. **最小聊天界面（`apps/panel/src/chat/`）**
   - 一个输入框，加上一列消息。
   - 她的回复按收到的块逐段显示。只显示核心实际下发的块，不显示任何未经核心的文字。
   - 解释失败、生成失败这类回合结果，以及核心不可用的状态，用一行灰字如实显示。
   - 做得朴素就行，正式面板由 P6-10 接手。
5. **结构检查：**
   - 终端里没有以情绪、心情、好感命名的持久变量；
   - 发件箱里只有还没确认的事件。

## 验收

- **单元测试：**
  - 帧编解码与黄金向量逐字节一致；
  - 发件箱的保存、重发、去重、删除；
  - 令牌不出现在日志和参数里。
- **脚本化的假核心：** 放在 `packages/terminal` 的测试里。
  - 认证、发 `user_text`、回 `RECEIVED_DURABLE`；
  - 下发 `output.chunk` 序列时，界面逐块显示；
  - 中途发 `output.cancel` 后，只保留已经收到的块；
  - 断线后重连，发件箱重发。
- **与真实核心的端到端测试不在本卡。** HOST-01 1/2 合并后另派会话补测。如果它在你交付前已经合并，可以顺手做：连续发 20 条消息，全部收到回执；在回执前、后各杀一次核心，0 丢失。
- `pnpm lint`、`pnpm typecheck`、`pnpm test` 通过。Windows 检查按 AGENTS 第 6 节。

## 交付

开草稿 PR，标题 `feat(P6-02 1/2): 终端客户端与文字聊天`。PR 描述按代码卡格式写，并附上「决定 / 理由」和 2/2 的范围。写「请审查」后停下。卡住就写 `BLOCKED.md`，然后停下。
