# 本机测试批准与 P7 集中待决（2026-11-03）

本表按用户本日明确决定更新，不修改上游文件许可。数据文件哈希沿用 resume-08/bundled-data-hashes.json 和固定代码清单。

| 项目/文件 | 来源与许可依据 | 原因/影响面 | 本卡决定 | P7 前处理 |
| --- | --- | --- | --- | --- |
| NLTK CMUdict 0.6 / corpora/cmudict.zip | 固定 nltk_data 550b6625bcef1f2abff2ff770a5a0d272c9c6b2a；原文允许研究和商业用途、不限使用并希望署名 | g2p_en 初始化的英文发音字典 | 用户批准按宽松许可登记，保留 Carnegie Mellon University 出处声明 | 保留许可/出处随包 |
| wordsegment unigrams.txt/bigrams.txt/words.txt | wordsegment 1.3.1；词频源自 Google Web Trillion Word Corpus，经 Norvig 整理，数据本身许可未明确 | 中英混合中英文罕见词的拆分；不是所有中文合成核心权重 | 仅批准本机测试；P7 前待决 | 用户选择：保留并写明风险 / 替换为许可清楚词频表 / 关闭该功能 |
| GPT-SoVITS 派生词表（cmudict-fast.rep、engdict-hot.rep、姓名缓存、opencpop-strict.txt、polyphonic*.rep 及派生缓存） | 固定仓库 04a44eb977586645f33594b72af13a035a2056c6 的 MIT 声明 | 中文拼音/多音字和混合英文热词/姓名处理 | 按用户决定以固定仓库 MIT 为本机使用依据，不再逐文件追溯 | P7 集中复审 |
| lid.176.bin / 包内 lid.176.ftz | fastText 模型 CC-BY-SA-3.0，代码 MIT 不替代权重许可 | 中文/混合分段语言识别 | 已批准：不改模型、独立文件、署名及许可声明 | P7 打包复审 |
| frozendict 2.5.1；FFmpeg LGPL shared；既有 soxr/soundfile 条件项 | 前阶段许可表及用户批准 | 运行时独立组件 | 保留原批准条件，不扩大为任意 LGPL 许可批准 | P7 打包集中复审 |

本卡本机测试两层规则：GPL/AGPL 代码、非商用/禁止再分发等明确限制仍停止；宽松但不在旧清单、来源不清或灰色数据，记录文件/来源/原因/影响面为 P7 前待决，然后继续。此规则不声称解决了第三方的权利问题，也不构成正式分发批准。
