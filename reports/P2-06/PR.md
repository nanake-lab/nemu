P2-06 的候选已准备：8 个自定义表情各 3 档、12 种 emo 各 2 个组合、3 组 MToon 在 3 种背景下对照。62 张单图及 23 张对照页见 reports/P2-06/review.html 和 sheets/，可以与 P2-05 一起挑选。

全部候选保留在 content/_candidates/P2-06；P2-05 tail、springs 候选与原始源文件未修改。内容仓使用独立 task/P2-06-expressions-materials 分支。此提交为候选预览，不将临时值写入正式 build.yaml。

验证：三个预览模型全规则 asset-lint 均为 0 错误、2 警告；62 个画面双截图一致；三模型结构和源数据保全通过；重复组装所有候选文件 SHA256 一致；Python 21 tests、Vitest 443 tests、lint/typecheck/mypy/ruff 通过。警告和失败尝试日志完整保留。

草稿保持：等待用户选择与 12 种最终组合签字，之后需正式 Blender 完整构建与舞台叠加检查，并同步最新 origin/main 重跑检查。报告与阻塞项见 reports/P2-06/REPORT.md 和 BLOCKED.md。中难度按用户本次指示不另请审查；最终由用户合并。
