# 订阅内Codex评判：能力与条款核验（2026-10-29）

结论：协议已改为ChatGPT订阅内非交互`codex exec`，Anthropic API路径已删除。**当前不能证明同时满足用户要求的严格隔离与实际模型版本记录；批量研究评分的条款适用性未核实。实际启动硬阻断，未进行任何评判。** 本轮只读取CLI帮助、官方网页和额度快照，不读认证文件/密钥、不改用户或系统配置。

## 本机与官方可证实的能力

本机`codex-cli 0.162.1`，路径`C:/Users/nanake/AppData/Local/Programs/OpenAI/Codex/bin/codex.exe`。`exec --help`列出新任务入口、`--cd`、`--ephemeral`、`--ignore-user-config`、`--ignore-rules`、`--json`、`--output-schema`、`--sandbox read-only`。只运行帮助没有创建推理会话。一次`features list --ignore-user-config`查询被CLI拒绝（该子命令不支持该参数），未以其他设置绕过。

[官方非交互文档](https://learn.chatgpt.com/docs/non-interactive-mode)支持脚本运行、JSONL事件与已有CLI登录；`--ephemeral`不持久化本地会话rollout。此处使用新exec，不用resume/fork/共享会话；只使用ChatGPT登录，不读出/复制凭据、不使用API key。

## 仍缺的执行证明

| 用户要求 | 已有准备 | 当前缺口/处置 |
| --- | --- | --- |
| 每次独立新会话 | 每次随机新目录，只写`anonymous.json`；真值私有清单放在目录外；未来检查唯一thread ID | 空目录只隔离材料布局，不能证明目录外不可读 |
| 无仓库/真值/其他材料读取 | 独立目录，无仓库挂载计划 | [只读模式](https://learn.chatgpt.com/docs/sandboxing)允许文件检查，不能等同仅能读取cwd。尚未核验所有自动注入AGENTS/记忆/技能及文件访问面均被封闭 |
| 无网络或工具 | 拟关闭所有模型工具与自主联网；只允许宿主必要的认证/推理传输 | [配置参考](https://learn.chatgpt.com/docs/config-file/config-reference)可关闭shell/记忆等，但命令网络限制不覆盖web/apps/MCP。没有实际工具目录、权限边界与否定测试证据，不能用“不要调用工具”提示或事后无调用记录替代 |
| 实际模型名和版本 | 计划保存每次可信服务端模型ID/修订号及原始证据哈希，另存CLI版本/哈希 | 官方JSONL示例有thread/turn/item/usage，未建立每次提供不可混淆实际后端修订号的接口。`-m`请求别名、模型自报、CLI 0.162.1都不能代替；缺失/变化即停 |
| 不保留上下文 | 新exec、ephemeral、无resume | 还须证明没有会话外记忆/配置材料注入；ephemeral不等于供应商零数据留存，不宣称远端数据保留政策改变 |

`codex_judge.py`可离线导出任务，校验新thread/单turn/无工具事件和拟议的实际模型回执；这些是必要条件，不是已验证执行能力。`CodexJudge`构造时无条件阻断，无自填`verified=true`开关，未启动进程；`evaluate.py --mode run`不能绕过。将来需在本卡允许范围内取得可核验执行方案，再另行实现/验证。不得改用户设置、安装隔离组件、伪造版本或换付费接口补缺口。

## 使用条款结论：未核实本研究批量评分适用性

已查阅2026-01-01生效的[Terms of Use](https://openai.com/policies/terms-of-use/)及2026-10-16更新的[Service Terms](https://openai.com/policies/service-terms/)。一般条款禁止自动/程序化提取数据或输出、规避限制；服务条款的Codex条目主要涉及代码生成许可证，没有找到针对本研究数百至数万次独立非代码评分的明确例外或专项许可。另一方面，官方明确提供非交互脚本接口，不能据此武断认定所有exec自动化都违规。

因此区分：**技术上官方支持exec自动化已核实；本研究这种批量研究评分是否符合订阅使用条款未核实。** 需要平台对具体用途的明确说明或其他适用依据后再启动批量评判；本卡不代用户联系平台、不作法律批准、不绕限额。用户授权研究不等于服务商授权。

## 订阅额度与收费边界

[订阅说明](https://help.openai.com/en/articles/11369540-using-codex-with-your-chatgpt-plan)确认ChatGPT登录使用计划额度；没有固定“一个exec等于一个标准消息”的换算，模型、上下文、推理和其他工作都会影响消耗。本次只读快照：周窗口10080分钟已用37%、剩约63%，显示重置2026-11-04 16:00:33 +08:00；credits余额0。它不是可完成调用数承诺，也没有使用重置额度。

[积分说明](https://help.openai.com/en/articles/12642688-using-credits-for-flexible-usage-in-chatgpt-personal-plans)指出套餐额度耗尽后可能使用积分。因此“评判新增API费用0”只指已含订阅额度内，不等于无限或保证免费；禁止自动购买/使用额外付费额度、兑换重置、换账户或降换模型。额度不足或收费状态无法确认时停，保留未完成，不追加请求。

试点688个新会话、正式6304–25024个新会话；很可能需要跨额度周期，当前无法保证任何一个方案完成。跨周期若实际模型版本变化，同批不能混合。会话数/功效/费用对照见[COSTS](COSTS.md)。
