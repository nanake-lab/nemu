# SYS11

`sys11_request_and_outbox_publish_together_and_only_once` 在 W02 发布前和 SQLite 提交后分别注入故障，重新打开临时库。未提交时保留原 PLANNED 和待处理输入，成功发布时 H、Q、行动账本一同可见。第二个请求无法重复登记。后续授权、消费可跨越自身记账提交继续运行。[原始证据](../EVIDENCE.md)
