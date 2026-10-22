# P0-08 复审修订进度

针对 Claude 评论 6332417953，当前保持草稿。

- max 是用户明确指定的对照，原始数据不替换；补测 low 的三语各50条和IRL 200条，其余远程配置、输入、顺序、Schema及计时口径不变。
- 已把 decision 第一行改为“规则输出，不可用作部署值”；部署草稿的操作字段已清空，机械规则结果另存，交 P0-09 作架构决定。
- 新增逐条远程时间分段和 token 汇总，不以边际百分位相减。
- 本地固定50输入做有/无约束配对计时，用户已批准远程完成后的30–45分钟GPU窗口；开始前须再次确认 runner/Blender/P4测量没有运行。
- Context7 额度不可用，已查阅 PyTorch 2.10 的 CUDA Event / forward hook 文档及 Transformers 4.57.3 的 streamer 文档和本地源码。
- 新增 CPU 计时测试3项和远程分段测试2项通过。初次 strict mypy 发现 CUDA Event 缺类型声明，显式 Protocol 后3文件通过，失败原日志保留。

完整修订和实测完成后，再 merge 最新 origin/main、重跑检查，按用户本次明确指令转为可审查并等待 Claude 复审。#29 的 unverified 不提前去掉。
