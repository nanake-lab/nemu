# 机械性子代理规则 — 2026-10-17

## 结果

完成规则追加、项目角色配置和一次真实只读任务试用。未触发 CI，未启动 runner，未修改冻结规格。P0-07 继续执行；本报告不批准载体选型或任务验收。

## 配置依据与交付物

- OpenAI 官方 [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents)（由 developers.openai.com/codex/subagents 跳转）：项目角色放在 `.codex/agents/`，独立 TOML 指定 name、description、developer_instructions，并可配置 model / model_reasoning_effort。权限继承父代理，不另加不受支持的角色级 sandbox 覆盖。
- `AGENTS.md` 新增第 9 节，16 行追加、0 行删除；原 188 行在统一换行后仍是完整前缀。
- `.codex/agents/routine-reporter.toml`：`routine_reporter`，`gpt-6-luna`，`max`。当前平台给该模型的最高可用推理档是 max，不静默降档。
- 不人为限制累计子代理数量；仍服从平台实际并发槽位。重负载全局串行，GPU 与 runner 互斥。规格解读、验收判断、选型及合并由主代理负责。
- 物理内存读取值为 34,282,377,216 bytes，按用户升级后的 32 GB 主机记录；不因内存升级放宽互斥约束。

## 真实试用

主代理通过 `collaboration.spawn_agent` 显式选择 GPT-6 Luna / max，fork_turns=none，并要求读取该角色文件。任务是读取已完成的 P0-07 十项矩阵：验证 TOML、核查 AGENTS.md 追加、重算样本统计及 p50；没有重新运行模型或测试。

当前 spawn 工具没有自定义角色名参数，因此本次验证的是**显式相同模型/强度并读取角色指令的执行**，不是按角色名自动加载；后者未冒充已验证。启动参数与官方依据见 `configuration-evidence.json`。

### 原始日志与复现配方

- 主代理提供的固定只读配方：`verify-trial.py`。
- 命令：`& 'D:\Dev\nemu\workers\model-adapter\probe\.venv\Scripts\python.exe' 'D:\Dev\nemu\reports\agent-delegation\2026-10-17\verify-trial.py'`。
- 原始 stdout：`trial-v3.stdout.log`；原始 stderr：`trial-v3.stderr.log`（0 bytes）。
- 子代理报告的进程退出码：0；外层执行 UTC 09:19:23.3810325–09:19:23.5537043，脚本自身起止时间另在 stdout 中。
- 配置 SHA-256：`fe68a73c2ed296d4844fc35248e0036179199d0dbba699603937b284ce0a824f`。

### 主代理核对结论

1. 已读原始日志，并独立读取配置 SHA-256，与子代理一致；git diff 确认只有追加行。
2. 十项矩阵均具有三个退出码，都是 0；每项有 20 个非预热样本；重算 p50 与存储值差为 0。两项只有 19 次完成二十字，其他八项为 20 次。主代理抽查 Qwen3-4B bf16 的第 11 次样本：5 汉字、time_limit、36817.50619999366 ms，与“19 次完成”一致。
3. 退出码为 0 仅证明测量脚本执行结束，**不等于 P0-07 验收通过**。本轮全部 p50 都大于 600 ms；这是主代理按原任务书作出的判断，未委派给子代理。
4. 子代理按要求保留失败并等待主代理指令，没有修复、改测试、推送或合并。试用成功的范围是固定配方执行与证据返回；不据此声称子代理能可靠猜测任意未知 JSON 结构。

## 失败历史（保留，不用成功覆盖）

- 第一次，2026-10-17 UTC 09:16:20.7014143–09:16:21.0008444：Get-Command 返回多个 Python 路径，ProcessStartInfo 启动失败，Python 未执行。外层 shell exit=0 不能代表核查成功。完整工具错误保留在子代理会话历史；此处未重造缺失的原始 stderr 文件。
- 第二次，经主代理指定固定 Python 路径，UTC 09:17:15.6554512–09:17:16.1123281，Python exit=1：子代理临时核查脚本在 concurrent.json 里猜测退出码和 sentence_boundary 布尔字段，没有使用 matrix.json 的三个退出码和样本 stop_reason。输出的“完成数为 0”及退出码缺失不是模型测量结论，已由主代理拒绝采用。其 p50 重算结果正确。stderr 原文保存于 `trial-v2.stderr.log`；完整命令/stdout 在子代理返回的会话记录中。
- CLI 试探 `codex --strict-config -c 'model_reasoning_effort="max"' features list` 返回 1：该子命令不支持 --strict-config；不能把这个调用错误解释为角色配置无效。原始错误见 `cli-attempt.stderr.log`。
- 最后由主代理提供固定字段和路径的 `verify-trial.py`，明确授权一次纠正后的核查以及两个全新日志输出，得到上面的 v3 原始成功记录。

## 后续

继续 P0-07。后续机械性测试/日志收集使用此子代理；主代理先提供确切命令与输出路径，授予独占重负载窗口后才可跑 GPU/编译/Electron。需要 Windows CI 时另行提前通知用户，不在本次触发。

## 2026-10-17 客户端角色发现后的补充试用

客户端本轮更新的 spawn_agent 已列出 routine_reporter，声明固定 model=gpt-6-luna、reasoning=max。主代理随后实际调用 agent_type=routine_reporter、fork_turns=none，返回 /root/native_role_check。由此补充验证“按角色名发现并启动”；上文未验证的说明保留为当时状态。

真实小任务是读取 Phi 方向原始 token 和固定 generation_config，核查两个 EOS 标记后的继续生成。日志 native-role-trial.log：Python exit=0，三语言各 20/20 样本在首个 EOS 后仍有 token。主代理核对 generation_config 确有 [200020,199999]，与脚本只检查 tokenizer 单 EOS 的缺陷吻合。日志中的 SHA-256 是 generation_config 文件的哈希，不是角色文件哈希。

限制和失败如实保留：目标样本筛选误用 modulation 字段（实际是 coefficient）而得到空数组，不能解释为样本缺失；全部样本列表仍有效。外层 New-Item -LiteralPath 不被支持，导致首次日志保存失败、外层 exit=1；原 stdout/stderr 在明确授权后写入新日志，没有重跑模型或覆盖证据。主代理承担诊断与代码修正，子代理未修复或作验收决定。
