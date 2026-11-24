# SYS12

`sys12_speech_partial_delivery_preserves_the_terminal_extent` 与 `sys12_complete_speech_is_settled_from_its_actual_receipt` 分别检查部分和完整回执。完整回执结算为成功。部分用例顺序发送两块，重启后读回部分投递回执，保留一块、两个已播放字符。`sent_body_can_fail_and_pending_action_can_cancel` 覆盖已发失败和在途前撤销。拒绝与超时记录原因；丢回执与迟到结算见 [X04](../X04/result.md)。[原始证据](../EVIDENCE.md)
