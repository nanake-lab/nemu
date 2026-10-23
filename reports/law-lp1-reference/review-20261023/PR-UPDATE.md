ADR 0018的参考形式会在弱支持到来时降低整步π。本PR提出按新信号限制衰减的参考形式，固定依恋先验与复杂度基数，并保存历史实际q，以保持新增及撤回的单调方向。规律仍为提议，`needs-user`，不合并。

2026-10-23根据阻塞审查修复了两处参考实现问题：回溯中新候选的首次分配从cf中省去；份额乘法正数下溢显式报错并使整个事件原子失败。补入候选自身随机检查、独立cf oracle、仅回溯/混合事件及七个下溢边界，原失败与旧源快照完整保留。

[ADR 0019 §11](https://github.com/nanake-lab/nemu/blob/codex/law-lp1-reference/docs/adr/0019-law-lp1-reference.md)独立说明失去有限记忆上界的行为影响及A–D可选做法，**尚未选择放弃或修改推论6**。开放上界构造仅为新增步的局部分析，未实现或宣称解决完整分摊/重放约束。原LAW-6仍失败：3,000序列、3,064状态违例。

已同步 `origin/main=876cff7211389f7058668dfb2b9592634d517abc` 并完整复跑，四个内部命令退出码0/1/0/0；旧LAW判据不变，其余九项通过。整步80,000步0违例；原随机带无修订/新增边际/撤回9,000/12,000/5,019次均0违例；另3,600回溯场景和候选首次分配检查0违例；原6,000及新增3,600前缀0重放差异，7文件strict mypy通过。四世界对照仅报告，不设门槛。

详见[修复报告及失败见证](https://github.com/nanake-lab/nemu/blob/codex/law-lp1-reference/reports/law-lp1-reference/review-20261023/REPORT.md)、[最新原始输出](https://github.com/nanake-lab/nemu/tree/codex/law-lp1-reference/reports/law-lp1-reference/verification/attempt-06)。原check_law.py、Rust、IDL、原规格和PR #49分支未改。真实核心/存储/θ/行为未验收，CI按docs/reports过滤无检查，不称全绿。已按用户要求逐条回复并请求复审，设计选项留待用户决定。
