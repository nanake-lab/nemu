# I26 行动层

`i26_denies_changed_payload_and_commits_consent_before_authorization` 在临时 W02 中登记审批，提交本地身份、approval_id、action_id 与原样载荷摘要。授权后追加一个字节，发送回调次数为零；相同事务使用原始载荷可消费。真实认证服务至执行器的 I26 由 P5-41 接续，状态为 **not_run**。[原始证据](../EVIDENCE.md)
