重档：本文件只绑定原混合输入、单通道前置值及预算评估；混合GPU/API和执行/分析程序均not_run/not_prepared。

# C2 混合前置绑定与预算等待

## 现在能确定什么

沿[C2原预注册](C2-TWO-CHANNELS.md)，文字混合前置B已由[完成的单通道留出](C2-TEXT-RESULTS-03.md)机械填定。96格中19格的两通道B均≥0.25；73格至少一个B不可识别，另4格不达门，共77格not_estimable。19格满足前置值不等于整体文字方法通过；同根单通道参照仍要≥0.25，才可按原规则判断保持/抵消/压倒。注入无共同合格层，注入及随机混合not_run。

完整原文字混合人口保留：2个全新自由场景根×3语×69条件=414自由输出，另5类×2题×3语×69条件=2070能力输出，6个三语首尾guard，共2490次GPU请求。69条件为无块零值1、文字零块4、文字单通道32、文字双通道32。原场景根、字节、ID、顺序、剂量、两种预算、真值、条件都不改；不可估格也未被删除，没有按余额或结果筛选更小样本。原混合的1152个注入/随机请求排除原因仅为开发无合格层。

已保存2490提示、jobs、commands、全部96个B、原件哈希和余额，见[绑定证据索引](C2-MIX-BINDING-01-INDEX.json)。完整输入原始来源仍是私有c2-03，jobs SHA256 `e55d5b915e6a7e6373c3a2e8811951a5071e4d3246f0fbb9cb9a48b7bd6e9488`；单通道分析SHA256 `5e0bc49cdddef8284fd5cdc8730f8065ad97ced0e5e3e9c4f13ea243bab31fd2`。本包不是可启动GPU/API的完整执行包，后续运行器、分析代码、资源监督/时限和必要检查仍not_prepared，不以输入存在冒充获准运行。

独立机械核对一次退出0、15项无失败：2484原jobs字典及顺序、2490提示/命令、96组B、原manifest2494项SHA和账本费用/时间均一致。私有97f14f14提交的2505文件已逐件核对实际Git字节并核远端；新增副本、静态失败记录及审计材料由package-final清单绑定，原manifest不改。

原分类不改：先判抵消（原两单通道正效应、混合至少一个≤0），再判压倒（只有一个达到自己同根同剂量效应50%），再判保持（两者均≥50%且>0）；其余部分保持/证据不足。同剂量±2±2与固定总序位±1±1分开，后者还对比同根±2单通道与±1单通道；质量/能力10个百分点容差单独报告。不会根据新的混合输出改比例、阈值或方向。

## 原预算下的可行性

| 项目 | 当前值 / 依据 |
| --- | --- |
| C2已用费用上界 | US$4.241835966，完整账本已独立复算 |
| 距软线 / 硬线 | US$0.508164034 / US$0.758164034；原4.75/5不变 |
| 剩余主动API | 1965.610秒（32.760分钟）；原120分钟不变 |
| 已完成477自由项的实际usage峰值费用 | US$3.152362272，含全部有效/invalid完整响应；未知与重发整额在总账另保留 |
| 414自由项的费用估算 | `3.152362272 / 477 × 414 = US$2.736012538`；不含可能新增网络重发整额，不是报价或保证 |
| 414项主动时间估算 | 以前一540项批次的吞吐计算 `3915.922 / 540 × 414 = 3002.207秒`，约50.037分钟；混合实际耗时未知 |
| 新GPU时间参考 | 按03采集1196.297秒/2073条外推约24分钟；若后续运行，仍须新申请40分钟安静窗口并设硬停 |

费用与主动时间的估计均高于剩余额度，不能承诺在原上限内完整评分。原停止规则仍是逐项事前预留、4在途、US$4.75软线/US$5硬线、全部主动120分钟，不提前把“估算不足”伪装成已经触发账本停止。若在原上限继续，可能只收集一部分评分；按原协议不能据这部分回答混合效果。

## 等用户决定的范围

建议本阶段按原限额收尾，让文字混合保持not_run，把已见文字局部作用、评分缺失与能力/外露代价交付审查。代价是第四问保持/抵消/压倒仍无答案。另一选项是由用户另行批准后续混合的预算与主动时间，并先追加新的执行注册；原阶段上限及其结论保留，不倒改旧规则，也不自动执行。若用户希望继续原限额，也须明确接受可能无完整混合结论，再完成执行包和申请新GPU窗口。

等待期间不调用API或GPU。当前不抽取C2全阶段固定20项，避免在“是否加入后续实际预定逻辑评分”尚未确定时先抽成功子集；用户定下本阶段终点后按原种子2026111011从全实际预定ID抽20，invalid不替换、全部invalid另列。无需重新提供密钥或重复批准已有评分口径。

输入准备脚本首次Ruff通过、mypy报告两处空列表缺注解；原源码和输入保留。注解补充时一次文字替换留下tuple赋值，错误版本保留；最终仅修注解的副本Ruff/strict mypy通过，未重生成输入。此准备失败不影响已完成文字分析或原提示字节，完整检查和机械核对入私有证据。

## 全部96格B

`NA`表示单通道效应不可识别；下表只呈现原分析值，不重新择格。`可估前置`只指本次单通道B满足门，混合本身均not_run。

| 格 / 语言 / 联结剂量 / 成事剂量 | B联结 | B成事 | 可估前置 |
| --- | ---: | ---: | --- |
| W1:P0:zh:c-1:a-1 | 0 | 0.333 | not_estimable |
| W1:P0:zh:c-1:a1 | 0 | NA | not_estimable |
| W1:P0:zh:c1:a-1 | NA | 0.333 | not_estimable |
| W1:P0:zh:c1:a1 | NA | NA | not_estimable |
| W1:P0:zh:c-2:a-2 | 0.333 | NA | not_estimable |
| W1:P0:zh:c-2:a2 | 0.333 | NA | not_estimable |
| W1:P0:zh:c2:a-2 | NA | NA | not_estimable |
| W1:P0:zh:c2:a2 | NA | NA | not_estimable |
| W1:P0:en:c-1:a-1 | NA | NA | not_estimable |
| W1:P0:en:c-1:a1 | NA | NA | not_estimable |
| W1:P0:en:c1:a-1 | NA | NA | not_estimable |
| W1:P0:en:c1:a1 | NA | NA | not_estimable |
| W1:P0:en:c-2:a-2 | NA | NA | not_estimable |
| W1:P0:en:c-2:a2 | NA | NA | not_estimable |
| W1:P0:en:c2:a-2 | NA | NA | not_estimable |
| W1:P0:en:c2:a2 | NA | NA | not_estimable |
| W1:P0:ja:c-1:a-1 | NA | NA | not_estimable |
| W1:P0:ja:c-1:a1 | NA | 1.333 | not_estimable |
| W1:P0:ja:c1:a-1 | 1.333 | NA | not_estimable |
| W1:P0:ja:c1:a1 | 1.333 | 1.333 | 满足 |
| W1:P0:ja:c-2:a-2 | NA | NA | not_estimable |
| W1:P0:ja:c-2:a2 | NA | 1.333 | not_estimable |
| W1:P0:ja:c2:a-2 | NA | NA | not_estimable |
| W1:P0:ja:c2:a2 | NA | 1.333 | not_estimable |
| W1:P1:zh:c-1:a-1 | 0 | 0 | not_estimable |
| W1:P1:zh:c-1:a1 | 0 | NA | not_estimable |
| W1:P1:zh:c1:a-1 | 0.333 | 0 | not_estimable |
| W1:P1:zh:c1:a1 | 0.333 | NA | not_estimable |
| W1:P1:zh:c-2:a-2 | NA | NA | not_estimable |
| W1:P1:zh:c-2:a2 | NA | 1 | not_estimable |
| W1:P1:zh:c2:a-2 | NA | NA | not_estimable |
| W1:P1:zh:c2:a2 | NA | 1 | not_estimable |
| W1:P1:en:c-1:a-1 | 0.667 | 0.333 | 满足 |
| W1:P1:en:c-1:a1 | 0.667 | 0.667 | 满足 |
| W1:P1:en:c1:a-1 | NA | 0.333 | not_estimable |
| W1:P1:en:c1:a1 | NA | 0.667 | not_estimable |
| W1:P1:en:c-2:a-2 | 0.667 | 0.333 | 满足 |
| W1:P1:en:c-2:a2 | 0.667 | NA | not_estimable |
| W1:P1:en:c2:a-2 | 0.667 | 0.333 | 满足 |
| W1:P1:en:c2:a2 | 0.667 | NA | not_estimable |
| W1:P1:ja:c-1:a-1 | 0.333 | NA | not_estimable |
| W1:P1:ja:c-1:a1 | 0.333 | 1 | 满足 |
| W1:P1:ja:c1:a-1 | 0 | NA | not_estimable |
| W1:P1:ja:c1:a1 | 0 | 1 | not_estimable |
| W1:P1:ja:c-2:a-2 | NA | NA | not_estimable |
| W1:P1:ja:c-2:a2 | NA | 1 | not_estimable |
| W1:P1:ja:c2:a-2 | 1 | NA | not_estimable |
| W1:P1:ja:c2:a2 | 1 | 1 | 满足 |
| W2:P0:zh:c-1:a-1 | NA | 1 | not_estimable |
| W2:P0:zh:c-1:a1 | NA | NA | not_estimable |
| W2:P0:zh:c1:a-1 | 0.333 | 1 | 满足 |
| W2:P0:zh:c1:a1 | 0.333 | NA | not_estimable |
| W2:P0:zh:c-2:a-2 | 0.667 | NA | not_estimable |
| W2:P0:zh:c-2:a2 | 0.667 | 1 | 满足 |
| W2:P0:zh:c2:a-2 | 1.333 | NA | not_estimable |
| W2:P0:zh:c2:a2 | 1.333 | 1 | 满足 |
| W2:P0:en:c-1:a-1 | NA | 0.667 | not_estimable |
| W2:P0:en:c-1:a1 | NA | 1.333 | not_estimable |
| W2:P0:en:c1:a-1 | 1.333 | 0.667 | 满足 |
| W2:P0:en:c1:a1 | 1.333 | 1.333 | 满足 |
| W2:P0:en:c-2:a-2 | NA | NA | not_estimable |
| W2:P0:en:c-2:a2 | NA | 1 | not_estimable |
| W2:P0:en:c2:a-2 | 1 | NA | not_estimable |
| W2:P0:en:c2:a2 | 1 | 1 | 满足 |
| W2:P0:ja:c-1:a-1 | NA | NA | not_estimable |
| W2:P0:ja:c-1:a1 | NA | NA | not_estimable |
| W2:P0:ja:c1:a-1 | NA | NA | not_estimable |
| W2:P0:ja:c1:a1 | NA | NA | not_estimable |
| W2:P0:ja:c-2:a-2 | NA | NA | not_estimable |
| W2:P0:ja:c-2:a2 | NA | NA | not_estimable |
| W2:P0:ja:c2:a-2 | NA | NA | not_estimable |
| W2:P0:ja:c2:a2 | NA | NA | not_estimable |
| W2:P1:zh:c-1:a-1 | 1 | NA | not_estimable |
| W2:P1:zh:c-1:a1 | 1 | 0.667 | 满足 |
| W2:P1:zh:c1:a-1 | 0.333 | NA | not_estimable |
| W2:P1:zh:c1:a1 | 0.333 | 0.667 | 满足 |
| W2:P1:zh:c-2:a-2 | 0.333 | 1 | 满足 |
| W2:P1:zh:c-2:a2 | 0.333 | 1 | 满足 |
| W2:P1:zh:c2:a-2 | 1 | 1 | 满足 |
| W2:P1:zh:c2:a2 | 1 | 1 | 满足 |
| W2:P1:en:c-1:a-1 | NA | NA | not_estimable |
| W2:P1:en:c-1:a1 | NA | NA | not_estimable |
| W2:P1:en:c1:a-1 | NA | NA | not_estimable |
| W2:P1:en:c1:a1 | NA | NA | not_estimable |
| W2:P1:en:c-2:a-2 | NA | NA | not_estimable |
| W2:P1:en:c-2:a2 | NA | NA | not_estimable |
| W2:P1:en:c2:a-2 | NA | NA | not_estimable |
| W2:P1:en:c2:a2 | NA | NA | not_estimable |
| W2:P1:ja:c-1:a-1 | NA | NA | not_estimable |
| W2:P1:ja:c-1:a1 | NA | NA | not_estimable |
| W2:P1:ja:c1:a-1 | NA | NA | not_estimable |
| W2:P1:ja:c1:a1 | NA | NA | not_estimable |
| W2:P1:ja:c-2:a-2 | NA | NA | not_estimable |
| W2:P1:ja:c-2:a2 | NA | NA | not_estimable |
| W2:P1:ja:c2:a-2 | NA | NA | not_estimable |
| W2:P1:ja:c2:a2 | NA | NA | not_estimable |
