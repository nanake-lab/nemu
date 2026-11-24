# X04

`x04_sys12_body_sent_receipt_lost_reopen_no_resend_then_late_success` 将原样身体动作送到合成子进程。子进程解析挥手动作，记录收到后以退出码 86 退出。父进程没有收到协议回执。重开真实 W02 临时库后将 DISPATCHING 记为 UNKNOWN，拒绝第二次发送，同一行动后来收到成功回执并结算。发送计数为 1。[原始证据](../EVIDENCE.md)
