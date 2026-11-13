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

## 2026-11-13 单项获批修订与续行

仅删除han < 6，其余screen保持#186原文。当前准备包v2-preparation-03，定档目录v2-calibration-02，正式目录v2-formal-02。复用先前唯一输出并按新规则重筛，原停止与字节不变；剩余29条才调用模型。calibration-appendix.mjs追加第9节，不覆盖第8节。四项测试含精确单项删除核验通过；现等新的GPU窗口，旧截止不得延长。

## 最终页面与取消时限修订

上述等待GPU/通读为历史状态；两阶段生成及用户可理解性确认现已完成。最终页面为本机D:/Dev/cache/R-STEER-PAIR-02/delivery-v2-unlimited/comparison.html，版本R-STEER-PAIR-02-v2-no-deadline。用户作答前取消时限：没有倒计时、定时器或超时停止，只在提交结果中记录elapsedMs。

build-page.mjs核验匿名材料后生成独立HTML与承诺；app.js处理质量、选择、识别标记与下载。app.test.mjs六项离线测试通过；browser-page-check.mjs使用合成22题完成浏览器检查，正式页只检查首页。输出目录已存在时不会覆盖，不应为重复验证直接重跑。原真实计时检查已按用户要求停止并封存，不记通过，也不再要求真实时间验证。

```powershell
node --test tools/r-steer-pair-02/app.test.mjs
```

用户结果返回前不解密或分析；页面证据与承诺见reports/R-STEER-PAIR-02/PAGE-V2-UNLIMITED-INDEX.json。
