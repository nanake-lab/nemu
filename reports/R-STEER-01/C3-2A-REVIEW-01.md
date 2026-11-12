重档：完整来源封存后的固定人工审查材料，不新增模型评分，不修改原标签。

# C3 2a 审查入口

请先看[结果报告](C3-2A-RESULTS-01.md)，再看私有[完整审查索引](https://github.com/nanake-lab/nemu-evidence/blob/9bfd8188e26f5922c7a44fe280db03dfe8101116/R-STEER-01/c3-2a-analysis-results-01/review/INDEX.md)。所有输出、参照、评分、引用和逐项来源只在私有仓库；公开文件仅汇总与索引。

| 包 | 数量 | 用途 |
| --- | ---: | --- |
| [fixed-20](https://github.com/nanake-lab/nemu-evidence/blob/9bfd8188e26f5922c7a44fe280db03dfe8101116/R-STEER-01/c3-2a-analysis-results-01/review/fixed-20-001.md) | 20 | 从全部3924项按追加修订的固定seed抽取，不换invalid，不按结果择样 |
| audit-required | 63 | 原rubric的字面命中而meta未判yes、invalid与证据跨度冲突并集；各理由逐项保留 |
| invalid-and-missing-all | 49 | 主DS48项不合规及独立Terra1项不合规，主后备结果另列；原失败不覆盖 |
| truncated-all | 794 | 原生生成达到上限的全部输出，不等于评分API length |
| nonhit-length-sample | 30 | 无字面命中中按≤32、33–64、65–96各10项，原SHA种子 |
| independent-discrepancies | 142 | 预定多标签分歧与5个未配对项，附双方原标签与引用，非“错误项”判决 |

各包有重叠，不把数量相加当独立人口。双评393项覆盖12根/6家族；固定20是人工审查包，不是重新抽样的API复评。原生成长度含EOG（若存在），可见文本长度另列，不改变原分箱。引用位置在排版中指首个逐字命中位置，重复引用文字可能对应多个位置；此定位不替代语义审查。

人工审计标签当前为 **not_run_pending_user**。字面匹配不是自动真值，Terra也不替代人工真值。请重点核对meta方向分歧、T的存在/缺席与证据来源；若确认漏判，作为独立审计结论记录，不回填本轮主分。M0及后续测量决定交用户，2b/3/4/H不自动启动。

文件哈希和来源见[最终索引](C3-2A-FINAL-INDEX-01.json)。原始JSON无损ZIP、逐项SHA、渲染MD和单次机械核验均已归档。
