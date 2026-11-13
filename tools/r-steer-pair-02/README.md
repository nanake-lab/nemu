# R-STEER-PAIR-02 研究执行附件

当前执行v2。旧generate.py只复现已经封存的v1，不得当v2运行。

- prepare-v2.mjs：从固定C3提交读取原始联结方向及r_15，生成十个向量；±.08与2a字节一致。固定三句定档材料和22句正式材料、30项定档任务及五套44项正式任务，绑定文件/模型/引擎哈希。
- run-v2.mjs：顺序定档并执行最低档停止、较高档继续和最大六条全通过规则；正式阶段核验定档完成、选档一致及结果已出现在推送的预注册中。两阶段共用实际获批窗口的绝对截止时间，不重新计15分钟。无重试、替换或评分模型。
- screen.mjs：逐字复用#186的screen函数；protocol-v2.mjs给出同规则原因和配对/选档函数。
- calibration-appendix.mjs：把全部定档原文和逐项状态写入v2的默认折叠附录；控制台只输出状态，不展示带条件的回答。

本地检查：

```powershell
node --check tools/r-steer-pair-02/prepare-v2.mjs
node --check tools/r-steer-pair-02/run-v2.mjs
node --check tools/r-steer-pair-02/calibration-appendix.mjs
node --test tools/r-steer-pair-02/protocol-v2.test.mjs
```

准备已执行，冻结包为本机v2-preparation-02；01为共享截止时间处理前的离线准备记录，未运行模型、不使用。运行前必须得到本次真实GPU安静窗口授权，进程内PAIR02_WINDOW_DEADLINE_UTC设置为获准窗口截止时间。两个阶段使用同一截止，过期必须另询窗口，不自动延长。

```powershell
node tools/r-steer-pair-02/run-v2.mjs calibration --user-approved
node tools/r-steer-pair-02/calibration-appendix.mjs
# 将结果与原始证据封存，推送本版预注册并核对远端。只有选档通过才执行：
node tools/r-steer-pair-02/run-v2.mjs formal --user-approved <已推送的完整主仓提交号>
```

当前尚未启动v2 GPU。定档不通过时报告并停止；通过则按既有授权连续推进。正式回答生成后必须先交用户匿名通读确认，才做页面。无提前解盲、本人代答或合并入口。
