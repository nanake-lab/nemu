# 正式结果的封存与解盲证据

私有仓 `nanake-lab/nemu-evidence`，分支 `task/R-STEER-PAIR-01`。下列提交均已推送；[逐文件索引](EVIDENCE-INDEX-RESULTS-V2.json)记录最终固定提交的10个文件、共21,867字节，各Git blob已核对本机字节一致。

| 顺序 | 固定提交 | 内容 |
| --- | --- | --- |
| 1：未读结果、未解盲 | [`8a7d98982d3f00c8610a1e9b3752081d1ea07cb9`](https://github.com/nanake-lab/nemu-evidence/tree/8a7d98982d3f00c8610a1e9b3752081d1ea07cb9/R-STEER-PAIR-01/post-answer-v2/) | 先封存用户原话与说明 |
| 2：未解盲 | [`22ab8585fcddc3e3638ab75720c753fbc6af0009`](https://github.com/nanake-lab/nemu-evidence/tree/22ab8585fcddc3e3638ab75720c753fbc6af0009/R-STEER-PAIR-01/post-answer-v2/) | 原样封存最终结果JSON、SHA-256及用户授权记录 |
| 3：解盲与分析完成 | [`73d1e4af27af165bc26ae8eff177927a213165b3`](https://github.com/nanake-lab/nemu-evidence/tree/73d1e4af27af165bc26ae8eff177927a213165b3/R-STEER-PAIR-01/post-answer-v2/) | 解封答案、逐题分类、汇总、分析源码及独立计数复核 |

## 文件与作用

- `observation-before-unblinding.json`：用户作答后、解盲前原话；不作质量改判。
- `result-original.json`：原JSON的逐字节副本，未格式化或修正。
- `result-seal-before-unblinding.json`：结果/观察哈希、v2与题单摘要匹配及授权说明。
- `analysis-source/`：解盲计数程序、3项边界测试、解盲前推送与测试核验摘要。
- `analysis/answers-unsealed.json`：原封存答案明文，仅在私有证据；SHA-256与原承诺一致。
- `analysis/trial-classifications.json`：匿名位置到四类/未答/屏蔽的分类。
- `analysis/summary.json`：严格按v2的总数、六家族描述、状态与判定。
- `analysis/independent-check.json`：Python逐类逐家族复核、质量正常10及原JSON未变。

原观察和结果在各自先行提交与最终提交中的字节完全相同；本轮只追加 `post-answer-v2/`，未改第一版、第二版或原研究证据。custody密钥未归档、未打印；用于本地解密后只保存私有明文答案及分析，不公开逐题答案或原回答。

当前汇总见[报告](REPORT.md)与[机器可读结果](RESULTS-V2.json)。冻结材料见[第二版证据索引](EVIDENCE-V2.md)，原答案承诺仍由[COMMITMENT-V2.json](COMMITMENT-V2.json)绑定。累计作答耗时不含下载保存时间；技术检查不能替代用户阅读感受。
