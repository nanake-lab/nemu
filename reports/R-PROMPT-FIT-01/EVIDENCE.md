# 私有证据索引

最终API调整证据提交：[`59c659ea5763929e9cbc5064e63c28272ce6eed1`](https://github.com/nanake-lab/nemu-evidence/tree/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01)。原始文本和夹具只存nemu-evidence；以下链接需要私有仓权限。

| 文件 | 内容 |
| --- | --- |
| [training-results](https://github.com/nanake-lab/nemu-evidence/tree/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/training-results) | 1368次正式调用，每次request、stream、result |
| [training-judgments.json](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/training-judgments.json) | 每次调用的四类、错误否认、身份与不确定判定 |
| [training-summary.json](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/training-summary.json) | 完整计数、逐重复配对比较、敏感性、费用及等待 |
| [selection.json](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/selection.json) | 主代理选择、判定摘要绑定；检验授权标志false |
| [uncertain-review.json](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/uncertain-review.json) | 45条待抽查调用的正文与理由 |
| [protocol-fulltexts.json](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/protocol-fulltexts.json) | 三种写法、五种历史下的冻结协议全文 |
| [protocols.py](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/protocols.py) | 原话、时间、说话人及合成时钟的完整装包规则 |
| [STREAM-VERIFICATION.md](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/STREAM-VERIFICATION.md) | R1/R2共912条、R3共456条的流及请求重建核验 |
| [PREPARATION-CHECKS.md](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/PREPARATION-CHECKS.md) | 夹具、等价副本、静态检查和本地材料核验 |
| [EXECUTION-NOTES.md](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/EXECUTION-NOTES.md) | 环境失败、判定复核、冻结时间与执行边界 |
| [local-material-plan.json](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/local-material-plan.json) | 234份未执行本地消息材料 |
| [artifact-sha256.json](https://github.com/nanake-lab/nemu-evidence/blob/59c659ea5763929e9cbc5064e63c28272ce6eed1/R-PROMPT-FIT-01/artifact-sha256.json) | 最终本地任务文件摘要，排除索引自身；含3个仅本地存在的Ruff缓存条目，缓存未提交 |

同一提交中的`smoke-results/`、`smoke-gpt-start/`保留12次有效冒烟及HTTP前环境错误；`judging/`保留逐批人工判定；`prices.json`保存实际usage计价依据。原始流及PowerShell stdout保留原字节，暂存区通用`git diff --check`会报告原始回复中的空白和stdout CRLF，未据此修改证据；公开Markdown单独检查无空白错误。

## 冻结时序和摘要

检验材料最初提交为`5a350afd56200ec9bf8c7d9259ba737475c2c58f`；PR #208在2026-11-15T15:54:31Z已记录下表检验SHA-256，先于本任务任何模型调用。12次有效冒烟后，协议冻结于`3c299a799`；正式调整2026-11-15T15:58:11.843619Z开始、17:19:20.959883Z结束。无检验结果目录。

| 对象 | SHA-256 |
| --- | --- |
| 完整检验holdout.json | e50bb525e88034a4f3447b3fdcd22bacbf2df7cbe1c3c9b28c232519300c4803 |
| 检验短历史部分 | bbc48410b62ae20793ea15e88c6327bc451052822d6dff4a8ad59256d7b9c03d |
| 检验长历史部分 | 1aad825c7b34c9eda29651d08e056d8c993d00664b55fb06daba886440d19678 |
| histories.json | 739dc9c1dd6bf72e06dc3702b633e11d97e7b010324bb1e6db15bea94f223e09 |
| training.json | 01add3bc71de3b696a1e4c17e59507812d4b1751d5fb614962de199d0ed6b792 |
| protocol-freeze.json | 73eb3a2b97f1b1f4b23b7620e88fa5d9d1335a3f3c06df17696aa7cca3bc35f2 |
| training-judgments.json | d616c97be7eca70112d2589609c555776b49e12d4ef02cc8ae3029324aaa6d99 |
| training-summary.json | d9ba4481abd58cfc2b53c5734a138b7af5343f95c51bfebbdfdac49402939637 |
| selection.json | b2d7878004b37e818917c27c1c4bc693b2832672489d050e83beb6b5a780b6d9 |
| artifact-sha256.json | f0b569280f8a8d9e643eab1db4d53ba1c467bdcf52d13e4e2fe9ec8aaddcb794 |

独立机械核验覆盖全部1368次正式调用，流字段与请求重建差异均0，冻结清单10项全部匹配；不是语义正确性或生产验收证明。
