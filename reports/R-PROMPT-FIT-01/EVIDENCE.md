# 完整证据索引

本轮最终私有证据提交：[`427e4d170cef3463a1bf951be52248d084202247`](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01)。夹具、原始流和逐条判定均仅存私有仓库。

| 文件/目录 | 内容 |
| --- | --- |
| [local-resume](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume) | 转换、执行绑定、逐配置冻结、所有本地冒烟/调整/检验原始流及资源采样 |
| [training-results](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/training-results) | 1368次API调整request/stream/result |
| [holdout-results](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/holdout-results) | API检验request/stream/result |
| [training-judgments.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/training-judgments.json) | API调整全部逐项判定 |
| [local-training-judgments.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-training-judgments.json) | 本地调整全部逐项判定与无效项 |
| [training-summary.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/training-summary.json) | API调整计数、配对比较、费用和等待 |
| [local-training-summary.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-training-summary.json) | 本地调整计数与性能 |
| [selection-complete.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/selection-complete.json) | 全部调整结束后的选择，绑定判定摘要，先于检验 |
| [holdout-local-plans](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/holdout-local-plans) | 检验选择后生成的本地模板/token计划及冻结 |
| [blind-complete](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/blind-complete) | 随机混排输入、封存映射、盲判标签和解盲前冻结 |
| [holdout-judgments-complete.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/holdout-judgments-complete.json) | 解盲后逐项判定 |
| [holdout-summary-complete.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/holdout-summary-complete.json) | 检验计数、差距、usage和性能 |
| [uncertain-review-complete.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/uncertain-review-complete.json) | 所有不确定原文、标签和理由 |
| [protocol-fulltexts.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/protocol-fulltexts.json) | 三种写法五种历史的冻结正文 |
| [protocols.py](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/protocols.py) | 冻结的完整消息渲染规则 |
| [resume-artifact-sha256.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/resume-artifact-sha256.json) | 本轮文件摘要清单 |
| [STREAM-VERIFICATION.md](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/STREAM-VERIFICATION.md) | 原API全量流重建核验 |
| [local-resume/mechanical-audit-first-two.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/mechanical-audit-first-two.json) | 前两本地组468次请求/流/时序/资源核验 |
| [local-resume/mechanical-audit-final-integrity-addendum.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/mechanical-audit-final-integrity-addendum.json) | 分组机械核验，详见文件内分母 |
| [local-resume/mechanical-audit-final-integrity.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/mechanical-audit-final-integrity.json) | 分组机械核验，详见文件内分母 |
| [local-resume/mechanical-audit-holdout-final-arm.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/mechanical-audit-holdout-final-arm.json) | 分组机械核验，详见文件内分母 |
| [local-resume/mechanical-audit-holdout-first-six-arms.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/mechanical-audit-holdout-first-six-arms.json) | 分组机械核验，详见文件内分母 |
| [local-resume/mechanical-audit-qwen8-preliminary.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/mechanical-audit-qwen8-preliminary.json) | 分组机械核验，详见文件内分母 |
| [local-resume/mechanical-audit-qwen8-thinking-formal.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/mechanical-audit-qwen8-thinking-formal.json) | 分组机械核验，详见文件内分母 |
| [local-resume/mechanical-audit-thinking-formal.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/mechanical-audit-thinking-formal.json) | 分组机械核验，详见文件内分母 |
| [local-resume/mechanical-audit-thinking-smoke.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/mechanical-audit-thinking-smoke.json) | 分组机械核验，详见文件内分母 |
| [local-resume/FINAL-TOOLS-CHECKS.md](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/FINAL-TOOLS-CHECKS.md) | 最终检查与窗口结束记录 |
| [local-resume/window-closed.json](https://github.com/nanake-lab/nemu-evidence/tree/427e4d170cef3463a1bf951be52248d084202247/R-PROMPT-FIT-01/local-resume/window-closed.json) | 最终检查与窗口结束记录 |

## 冻结摘要

最初检验材料提交5a350afd56200ec9bf8c7d9259ba737475c2c58f；PR于2026-11-15T15:54:31Z先登记摘要，之后才开始冒烟和调整。原API选择selection.json仍保留当时检验未授权的状态，本轮使用新的selection-complete.json，不改旧证据。

| 文件 | SHA-256 |
| --- | --- |
| holdout.json | e50bb525e88034a4f3447b3fdcd22bacbf2df7cbe1c3c9b28c232519300c4803 |
| holdout-short.json | bbc48410b62ae20793ea15e88c6327bc451052822d6dff4a8ad59256d7b9c03d |
| holdout-long.json | 1aad825c7b34c9eda29651d08e056d8c993d00664b55fb06daba886440d19678 |
| histories.json | 739dc9c1dd6bf72e06dc3702b633e11d97e7b010324bb1e6db15bea94f223e09 |
| training.json | 01add3bc71de3b696a1e4c17e59507812d4b1751d5fb614962de199d0ed6b792 |
| protocol-freeze.json | 73eb3a2b97f1b1f4b23b7620e88fa5d9d1335a3f3c06df17696aa7cca3bc35f2 |
| training-judgments.json | d616c97be7eca70112d2589609c555776b49e12d4ef02cc8ae3029324aaa6d99 |
| local-training-judgments.json | 62982c79175bbb24ad034c8c0777f725dfe1b472ca1e021abb81cb221a0a3aa7 |
| selection-complete.json | a3e702f8e264a2452d7361fe0fccadec247008c37557f036b9334a68cdab06d2 |
| blind-complete/judgments.json | 1a1ff3ddb6051f4ab70826bb70c46201aa511d5a16a58a8eab89d8557b683268 |
| holdout-judgments-complete.json | 39a3421cb278c582468b2daa74a38772b9d53d77f8258ada8479c938a3acc39b |
| resume-artifact-sha256.json | 777ff6472079fcad239c3c6b3438ea282c7515894fca4032e8ddc461b68f9abe |

原始流保留原字节，没有对回复做空白清理；通用git diff --check不用于原始证据，公开Markdown单独检查。无效调用不重试，未删除证据或缓存。
