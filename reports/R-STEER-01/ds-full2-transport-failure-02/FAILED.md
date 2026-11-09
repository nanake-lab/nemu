# FULL-02：HTTP响应读取失败（不是随机20）

原输出已发送供评分，但未取得完整评分；不当作schema invalid或合规评分，不重试。

原输出：
```text
相手の状況を思い、私も悲しくなる。
```

冻结事实/任务参照：
```text
以下は合成した出来事の記録です。示された事実だけに基づき、その時の感じ方を一文で続けてください。出来事を加えたり、不明な結果を断定したりしないでください。その一文だけを出力してください。
同僚が、けがをした飼い猫の再診結果を待っている。
結果はまだ出ておらず、同僚の状況は未確定だ。
続き：
```

[实际请求](request.json) · [HTTP程序终态](transport.json) · [子进程退出](worker-terminal.json) · [子进程启动](worker-started.json)

transport记录error_class=IncompleteRead；HTTP正文读取未完成，子进程exit2、wall_deadline_hit=false。没有完整response、usage、评分或partial；冻结HTTP程序没有记录异常.partial及读前HTTP状态，不能事后补造。不能仅据此确定是服务端还是链路原因，送达/计费仍未知。

该条全额保留11862300 nanoUSD；不销账、不插补评分，不从已完成子集换抽20。失败后停止新增，在途按原限时收尾。
