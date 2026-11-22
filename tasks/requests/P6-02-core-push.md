# P6-02 文字聊天的核心推送接线

P6-02 1/2 消费现有 IDL 的 `OutputChunk`、`OutputEnd`、`RunStateV2` 和 `ApprovalRequestV2`。现行网关的认证后管道只收 `TerminalRequest`、发成对的 `TerminalResponse`。请 P5-53 提供经认证的主动下行入口，将这四类消息交给 desktop 的 `bindTerminal`。错误回合需要可区别的解释失败、生成失败与空输出信息，并关联到对应回合。`OutputEnd(ERROR)` 目前只够显示通用输出失败。

终端沿用 appbus 的 `output.chunk`、`output.state`、`core.runState` 和 `approval.*.v2`。面板的停止按钮调用本地 IPC，载荷为既有 IDL `OutputCancel`。如需让总线消费者直接订阅 `output.end` 或发布 `output.cancel`，请在后继契约卡登记这两个事件及错误状态。

HOST-01 1/2 的联测请使用安装标识 `desktop`、当前用户 SID 管道及当前用户 DPAPI 令牌。认证会话需要 `INGRESS_UPLOAD` 和 `INGRESS_EVENT`。新文本及其来源由两次 canonical 对象上传传入，随后提交 `TerminalEvent`。P5-53 接通回合后，请用临时运行存储核验真实推送、取消、故障与重启；现阶段的脚本化下行从生成类型回调注入。
