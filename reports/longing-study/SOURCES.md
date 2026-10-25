# 来源与读取范围

所有仓库结论固定到下列提交；行号以对应远端原文为准。报告目录中的建议不覆盖来源，也不表示采纳其他未合并 PR。

## 仓库来源

基线 B = 0b5412e95caab447c70148c473e544dbab28c57c。
独立研究 P = 25e8474958c7ccd9d0d37266a881c8df733f7058。

| 来源 | 读取内容/与本研究关系 |
| --- | --- |
| [AGENTS.md](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/AGENTS.md) | 全文：目录、真实数据禁用、报告、草稿与审查边界。用户本次的研究范围优先，不套用认领任务卡/正式 ADR/继续取卡规则。 |
| [ARCHITECTURE.md](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/docs/ARCHITECTURE.md) | 全文：运行进程、唯一核心、状态和模型输入隔离、桌面感知、当前自主接线。 |
| [CONTRACTS.md](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/docs/CONTRACTS.md) | 全文，重点 §4–8：观察/结果类型、重放不重发、生成输入、预算与权限。 |
| [echo-core/README.md](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/docs/echo-core/README.md) | 全文：v2.0 Markdown 为原文，优先于冲突的旧规格。 |
| [v2.0 原文](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/docs/echo-core/echo-core-v2.0-first-principles.md) | 全文；L92–170 状态/顺序，L204–230 证据/规律，L255–296 推论/x，L306–345 θ/主动，L473–524 不变量/验收。 |
| [数学审计](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/docs/traceability/echo-core-math-audit.md) | 重点 P-1、D-1、§7.1–7.3、§7.9、v2.0 适用范围追加；不是把旧问题当现状。 |
| [ADR 0008](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/docs/adr/0008-echo-core-math-clarifications.md) | 全文，特别 §1 及 v2.0 追加：事件非扩张、T_x、只允许抑制型依赖。 |
| [P5 卡片](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/tasks/P5-echo-core.md) | P5-20 L762–797 的调度与三个问题；P5-33 L1282–1314 的 a/b/c 和待用户决定。 |
| [opportunities.json](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/config/opportunities.json) | 全部实际字段；不照抄旧差异登记里的“尚不存在”。 |
| [diff-register.md](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/docs/traceability/diff-register.md) | V2-54 及后续补充，未修改或新增登记。 |
| [ADR 0019](https://github.com/nanake-lab/nemu/blob/0b5412e95caab447c70148c473e544dbab28c57c/docs/adr/0019-law-lp1-reference.md) | 核对参考规律、§11 已选 A 与其余批准范围。报告不以旧推论 6 给内部循环提供稳定保证，不复写其 R 递推。 |
| [PR #73](https://github.com/nanake-lab/nemu/pull/73) | 状态、基线、最新 head、说明及审查评论；与 main 分开。 |
| [P5-33 METHODS](https://github.com/nanake-lab/nemu/blob/25e8474958c7ccd9d0d37266a881c8df733f7058/reports/P5-33/METHODS.md#L55-L63) | a/b/c 的观察窗口、同源归组、未知、回归摘要及局限。未重新运行此 PR 的实验。 |
| [PR #73 审查通过评论](https://github.com/nanake-lab/nemu/pull/73#issuecomment-6370155995) | 研究结论通过，但采纳/缺席语义/发布仍待用户。 |

没有完整审计产品代码；任何关于“可行、需扩展”的结论都是规格层设计分析，不是现有实现符合性的断言。

## 文献目录与证据口径

以下均为原始研究，通过作者全文、期刊全文或原始摘要核对。未取得全文的条目不声称审查过完整统计方法。本报告没有依赖科普文章推导脑机制。

1. Diamond, L. M., Hicks, A. M., & Otter-Henderson, K. D. (2008). Every time you go away: Changes in affect, behavior, and physiology associated with travel-related separations from romantic partners. *JPSP, 95*, 385–403. DOI: [10.1037/0022-3514.95.2.385](https://doi.org/10.1037/0022-3514.95.2.385). 原始摘要；自然观察，不作因果/单调增长结论。
2. Poore, J. C., et al. (2012). Prediction-error in the context of real social relationships modulates reward system activity. *Frontiers in Human Neuroscience, 6*, 218. DOI: [10.3389/fnhum.2012.00218](https://doi.org/10.3389/fnhum.2012.00218). 全文。研究局限包括样本小、评价反馈任务；BOLD 与评分变化量未显著相关，不能由任务后评分关联推导变化因果。
3. Tomova, L., et al. (2020). Acute social isolation evokes midbrain craving responses similar to hunger. *Nature Neuroscience, 23*, 1597–1605. DOI: [10.1038/s41593-020-00742-z](https://doi.org/10.1038/s41593-020-00742-z). [MIT 作者稿与摘要](https://hdl.handle.net/1721.1/138313)。Nature 入口在本次获取中失败，采用机构存档核对；不外推为特定对象的依恋。
4. Nolen-Hoeksema, S., & Morrow, J. (1993). Effects of rumination and distraction on naturally occurring depressed mood. *Cognition & Emotion, 7*, 561–570. DOI: [10.1080/02699939308409206](https://doi.org/10.1080/02699939308409206). 出版记录与作者上传原文。特定短时反刍实验，不泛化为所有内在活动。
5. Stanton, S. C. E., Campbell, L., & Loving, T. J. (2014). Energized by love: Thinking about romantic relationships increases positive affect and blood glucose levels. *Psychophysiology, 51*, 990–995. DOI: [10.1111/psyp.12249](https://doi.org/10.1111/psyp.12249). 原始摘要；不从题名扩大为长期依恋结论。
6. Hudson, N. W., & Fraley, R. C. (2018). Moving toward greater security: The effects of repeatedly priming attachment security and anxiety. *Journal of Research in Personality, 74*, 147–157. DOI: [10.1016/j.jrp.2018.04.002](https://doi.org/10.1016/j.jrp.2018.04.002). 作者全文，尤其 §3.2.1 和局限。内部表征加工与较长期改变相容，不证明自然思念的独立效应。
7. van der Helm, E., et al. (2011). REM sleep depotentiates amygdala activity to previous emotional experiences. *Current Biology, 21*, 2029–2032. DOI: [10.1016/j.cub.2011.10.052](https://doi.org/10.1016/j.cub.2011.10.052). [PMC 作者稿](https://pmc.ncbi.nlm.nih.gov/articles/PMC3237718/)。
8. Baran, B., Pace-Schott, E. F., Ericson, C., & Spencer, R. M. C. (2012). Processing of emotional reactivity and emotional memory over sleep. *Journal of Neuroscience, 32*, 1035–1042. DOI: [10.1523/JNEUROSCI.2532-11.2012](https://doi.org/10.1523/JNEUROSCI.2532-11.2012). [PubMed 摘要](https://pubmed.ncbi.nlm.nih.gov/22262901/)；PMC 获取出现验证码，使用可获取的期刊/摘要记录，不声称绕过。
9. Sbarra, D. A., & Emery, R. E. (2005). The emotional sequelae of nonmarital relationship dissolution: Analysis of change and intraindividual variability over time. *Personal Relationships, 12*, 213–232. DOI: [10.1111/j.1350-4126.2005.00112.x](https://doi.org/10.1111/j.1350-4126.2005.00112.x). 原始摘要；分手不同于暂时分离。

没有一篇被用来证明 Nemu 会产生人的主观体验、必然更依恋用户、或一个无参数的“越久越想”定律。数学充分条件由本研究推导，引用仓库审计是为了标明继承与扩大证明范围的区别。

