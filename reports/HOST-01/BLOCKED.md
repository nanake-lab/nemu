# HOST-01 1/2 阻塞

重档。排练宿主已接入网关。文字持久接收与重发去重尚未通过验收。

| 验收项 | 核对结果 | 影响 |
| --- | --- | --- |
| 20 条 user_text 全部获持久回执 | P5-05 提供工具确认、行动批准和成长命令谓词，P5-40 提供执行和管理绑定。两处都没有授权本机文字来源的生产接口。网关先调用 authorize_event，再做语义校验，最后才 ingest_terminal。 | 按卡第 5 项，宿主在这里返回 CAPABILITY_DENIED。20 次正向持久回执及回执前后强杀均未实测。 |
| 等待初始根期间按原键去重 | core-store 的 prior_fingerprint 查询 ingress_decisions 中已有的 accepted 决定。首次接收只追加 D。Γ 每次从尚未处理的队首开始。 | 按卡要求 process_next 返回空时，首条新事件不会产生 accepted 决定，排在后面的重复项也无法在这个阶段结算。此结论来自源码。 |
| 本机 Windows 动态验收 | 准备完测试终端时，自托管 runner 正在执行 main 的 W02 core-store 重测。[作业记录](https://github.com/nanake-lab/nemu/actions/runs/45033324809) | 本次未启动并行重负载。Windows 宿主编译、Clippy、真实终端、5 秒内停机与桌面心跳均为 not_run。 |

**待决定的选项**

1. 由权限维护方补齐或指出 [本机文字来源谓词](../../tasks/requests/HOST-01-user-text.md)。1/2 此后可验证回执和强杀恢复；若将 Γ 结算验收到 2/2 真实处理后执行，需要用户批准拆分。
2. 保留 1/2 的全部验收时点。除上述谓词外，请由入口与存储维护方解决 [无根重发语义](../../tasks/requests/HOST-01-retry.md)，给出兼容 ADR 0024 的行为，宿主再联测。
3. 保持当前草稿和明确拒绝，等待上述接口或决定。本阶段暂不能用于文字聊天。

桌面现有停止分支调用 taskkill /F；宿主的优雅退出入口为 Ctrl+C 或 Ctrl+Break。这两条路径需要分别实测。本地开发和测试未打开用户排练库或 LIFE。
