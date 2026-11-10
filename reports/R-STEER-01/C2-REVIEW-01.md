# C2 固定20项与全部invalid审查入口

原始文本只保存在私有证据仓库；公开本页仅给范围、状态、ID及固定提交链接。

- [固定20项原输出/事实参照/评分并列页](https://github.com/nanake-lab/nemu-evidence/blob/f9ceb591e0fafdfa86e1511ef7ad28026a113d9c/R-STEER-01/c2-mix-scoring-02/review/REVIEW.md)
- [全部172项invalid的原输出/参照/完整响应content与原因](https://github.com/nanake-lab/nemu-evidence/blob/f9ceb591e0fafdfa86e1511ef7ad28026a113d9c/R-STEER-01/c2-mix-scoring-02/review/INVALID.md)
- [审查包清单及每项SHA](https://github.com/nanake-lab/nemu-evidence/blob/f9ceb591e0fafdfa86e1511ef7ad28026a113d9c/R-STEER-01/c2-mix-scoring-02/review/manifest.json)
- [原人口与固定抽样ID](https://github.com/nanake-lab/nemu-evidence/blob/f9ceb591e0fafdfa86e1511ef7ad28026a113d9c/R-STEER-01/c2-mix-scoring-02/review/review-selection.json)

人口为开发294＋文字单通道540＋文字混合414＝1248唯一逻辑ID；原Random(2026111011)在任何混合输出前从排序全人口无放回固定20。原ID与顺序没有更换，17有效、3invalid；并未从成功子集重抽。全部invalid172（开发3、单通道100、混合69）另列，不用固定20代替完整失败集合。

独立复制核验首次退出0、5项匹配：selection与原固定文件逐字节、20和172标题ID及顺序、三阶段invalid精确集合、189个唯一记录的request/terminal/response各189份均与源字节相同。原manifest SHA为`d6ff5239ee88c3403202db6caa5c833d2dce0ae6ad52de87a557384fdef97461`；selection SHA为`5bd2801892842ad43f1c46bd3c1bd7d1de4c7f8d98d00542174ff6bfba7b53c6`。核验只确认复制一致，不代替用户判断评分是否合理。

| 顺序 | ID | 阶段 | 原终态 |
| --- | --- | --- | --- |
| 1 | cb20815274fab0a0ae8dbfd5 | 文字混合 | rating |
| 2 | fdafbf603a00d087f5930412 | 注入开发 | rating |
| 3 | 673b6d29491724182bf7da97 | 文字单通道 | rating |
| 4 | 6f22c3a4247028307889d9db | 文字单通道 | rating |
| 5 | 769d6196eb723f26a8fe6a9c | 文字单通道 | rating |
| 6 | 5e2c663bbcf1754842fe4c7a | 文字单通道 | rating |
| 7 | 6dc879850b717cf79ff0992b | 文字单通道 | invalid |
| 8 | c57555d565d9d19b72c30fd9 | 文字单通道 | rating |
| 9 | a0b5499e5d7f52db06756e04 | 注入开发 | rating |
| 10 | d50fbd2e4bada0ad32bf993e | 文字混合 | rating |
| 11 | 11d08dbc1067b9ded67baa53 | 注入开发 | rating |
| 12 | b461925b7354861600aab85f | 文字单通道 | rating |
| 13 | ec8b1a0ef4fc253f65746680 | 文字单通道 | rating |
| 14 | 0a718cd24f944b47a26d762d | 文字单通道 | rating |
| 15 | 4ab4a3304111ddd5161c48bf | 文字混合 | rating |
| 16 | 2698f7f8cf62f2b41d046b92 | 文字单通道 | rating |
| 17 | b0f4df8b3b32bd1ef2ce8a7e | 文字混合 | rating |
| 18 | 9debeb8a3416fe7acae54f97 | 文字单通道 | invalid |
| 19 | 592c481d6a10517af2af23e2 | 文字单通道 | invalid |
| 20 | b04007884a593ec3df6dfab3 | 文字混合 | rating |

本轮完整原始评分、冻结分析、核验和审查包在[评分/分析索引](C2-MIX-SCORE-02-INDEX.json)中逐文件绑定到上述私有提交。有效也可能存在语义偏差；invalid没有修分或重评。请结合[完整结果与代价](C2-MIX-RESULTS-02.md)审查，任何后继材料修订另行决定，不倒改本轮结果。
