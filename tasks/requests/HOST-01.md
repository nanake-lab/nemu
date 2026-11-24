# HOST-01 1/2：nemu-core 核心进程

仓库：GitHub `nanake-lab/nemu`。先完整读一遍 `AGENTS.md`。

**投入：** token 预算无上限，不要吝惜 token。该读的文件读全，该测的情况测全，证据给全。不要为了省 token 跳过步骤或简化验收。

- **线：** 交付线，重档，要交叉审查。
- **分支：** `task/HOST-01`，从 `origin/main` 新建。
- **目录：**
  - 新建 crate `core/crates/nemu-core/`（二进制名 `nemu-core`），并加进 `core/Cargo.toml` 的 workspace members；
  - `apps/desktop/src/supervisor/process-config.ts` 里 `nemu-core.exe` 这一项的启动参数和健康探测；
  - `tasks/requests/HOST-01.md`（本卡原文）；
  - `reports/HOST-01/`（只放证据索引，代码卡不写 REPORT）。
- **不碰：** 其他 crate 的源码、契约 `.proto`、阈值和期限。别的 crate 缺接口时，写 `tasks/requests/HOST-01-<主题>.md`，说清要什么、为什么，不要自己改。
- **预授权：** 目录内的实现层决定由你自己做，在 PR 描述里逐条写「决定 / 理由」。新增第三方包、改契约、放宽任何检查，或者碰到「她是谁」的问题，停下来问。
- **不等待：** 本卡做完写「请审查」就停下。不要为了等别的 PR 合并、等 CI 或等回复让会话一直跑。2/2 会另外派。

## 背景

- 桌面应用的进程守护（`apps/desktop/src/supervisor/process-config.ts`）会启动 `core/target/release/nemu-core.exe`，但仓库里没有这个二进制。`docs/ARCHITECTURE.md` 第 88 行列了它的职责。
- 需要的部件都已经是现成的库，只是还没人把它们组装成一个进程：
  - `gateway`：`Gateway::new`、`serve`、`Installation`、`TerminalToken`、`CoreHandler` trait；
  - `core-store`：`Store::<Rehearsal>::startup`；
  - `core-runtime`：事件处理流水线。
- 下一个里程碑是「第一次真的聊上」：她在桌面上用文字和用户聊天，独处时自己想事情，全部经过核心。所有部件最后都要接进这个进程。

## 拆分（写进 PR 描述）

- **1/2（本次）：起得来、连得上、收得稳。** 进程生命周期、网关、终端认证、事件持久接收、重启恢复。
- **2/2（另派）：** 换上 P5-53 的运行时 CoreHandler，事件真正处理并提交。说话回合、生成器、独处思考调度，由各自的卡直接接进本进程。本卡不为它们预留接口或插件层。

## 要做的

1. **二进制与配置。**
   - 命令行参数只要谱系种类和日志级别。
   - 这一阶段只接受排练谱系（rehearsal）；正式谱系一律拒绝创建（ADR 0010 第 3 条）。
2. **存储。**
   - 用 `Store::<Rehearsal>::startup` 创建或打开排练谱系。它的位置由 `core-store` 固定在 `%LOCALAPPDATA%\Nemu\core\` 下（见 `core-store/src/lineage.rs` 的 `production_path`），本卡不改。
   - 谱系头怎么构造，以现有测试和 P5-08、P5-30 的用法为准，不发明新字段。
3. **终端令牌。**
   - 固定放在 `%LOCALAPPDATA%\Nemu\terminal.token`。P6-02 的终端按这个路径读取。
   - 第一次启动时用 `TerminalToken::create_protected` 生成，以后用 `load_protected` 加载。
   - 令牌不进日志、命令行参数和环境变量。
4. **网关。**
   - 在 `gateway::pipe` 规定的管道名上启动 `Gateway::serve`。
   - Installation 的能力只开 `INGRESS_EVENT`、`INGRESS_QUERY`、`INGRESS_APPROVAL`、`INGRESS_CANCEL`。文字聊天用不用得上 `INGRESS_UPLOAD`，你按网关代码判断后写明理由；`INGRESS_GROWTH` 关闭。
5. **生产用 CoreHandler（本阶段的最小版本）。**
   - `authorize_event`：已认证的本机终端会话发来的 `user_text`，按 P5-05、P5-40 已有的权限谓词授权。找不到现成谓词时，不要自己造授权规则：写请求文件，并让这类事件明确拒绝。
   - `process_next`：还没有初始根，本阶段不处理事件，返回空。不能伪装成处理成功。
   - 其余回调保持 trait 默认的拒绝。
6. **生命周期。**
   - 健康探测用 `process-config.ts` 已支持的 `stdout-heartbeat`：进程就绪后定期输出心跳行。
   - 收到停止信号时优雅退出：不再接新连接，已经收到的事件保持持久。退出时限由你定，写明理由。
   - 崩溃或被杀后重启：已回 `RECEIVED_DURABLE` 的事件一条不少；重复上传按 `stable_source_key` 去重，沿用网关现有逻辑，不另写。
7. **运行状态。** 网关目前没有从核心向终端主动推送的通道。本卡不做推送，状态只通过心跳行和日志如实反映，例如「已连接、等待初始根」。推送通道归 P5-53。
8. **两个平台。** 网关是 Windows 专用（`cfg(windows)`）。`nemu-core` 在 Ubuntu 上也要能编译，非 Windows 平台启动时明确报「只支持 Windows」并退出。

## 验收

- Windows 本机，用一个脚本化测试终端（放在本 crate 的 tests 里）：
  - 启动 `nemu-core`，终端认证成功，发 20 条 `user_text`，全部收到 `RECEIVED_DURABLE`；
  - 在收到回执前、后各杀一次进程，重启后已回执的事件一条不少，未回执的重发后按原 key 去重；
  - 错误令牌、未开放的能力、超大帧都被拒绝，拒绝原因准确；
  - 停止信号下在你定的时限内退出，退出后再启动能正常连上。
- `cargo fmt --check`、`cargo clippy -p nemu-core --all-targets -- -D warnings`、`cargo test -p nemu-core` 通过；Ubuntu CI 通过。
- 桌面应用能把它拉起来，心跳探测通过。
- Windows 检查按 AGENTS 第 6 节：草稿阶段只跑相关组，审查通过后全量跑一次。

## 交付

开草稿 PR，标题 `feat(HOST-01 1/2): nemu-core 核心进程`。PR 描述按代码卡格式写：开头白话三行，正文不超过十行，再加上拆分计划和「决定 / 理由」。写「请审查」后停下。卡住就写 `BLOCKED.md`，然后停下。
