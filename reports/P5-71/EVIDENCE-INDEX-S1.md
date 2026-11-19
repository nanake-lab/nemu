# P5-71 S1证据索引

本轮是审查后静态白名单子集，不是run02续跑或重标。结果及限制见 [S1-REPORT](S1-REPORT.md)，汇总见 [S1-SUMMARY](S1-SUMMARY.json)，运行前登记见 [S1-RUN-MANIFEST](S1-RUN-MANIFEST.json)。

私库归档提交 [`73b36fbb5e6206e63f20aa0278d7f26a7e7849ee`](https://github.com/nanake-lab/nemu-evidence/tree/73b36fbb5e6206e63f20aa0278d7f26a7e7849ee/P5-71/20261113-S1-01/archive)。执行源码为`86bde0073bcb861ec4f866cfd7e27f301739b876`，预注册/白名单先推送为`5f736a91`，输入编码勘误为`310b5b62`。

共18,251个归档条目、232,618,498字节；gzip文件24,213,725字节，SHA-256 `8560f8553828631660962835d457ecbfd641762c180d5c7ab7f0db663ec09b4b`。不分片。压缩时重新核对源字节，解包后18,251/18,251逐件SHA-256与长度匹配。`FILES.json`为完整原件清单，SHA-256 `20139941c0209b9f825fd606d84a15463c1a84dbff3232f3eb1272b01513ab33`；`VERIFICATION.jsonl`保留各条核验。

[逐文件CSV](EVIDENCE-INDEX-S1.csv)共18,265行：18,251条归档内原件、6个归档/验证文件及8个收尾核验文件。archive_entry非空时，private_path指向ARCHIVE.json，解开同目录evidence.tar.gz后按archive_entry定位。CSV的retained表示保全，不表示实验通过；原始结果标签、见证及minimum状态须按报告分开解读。私库P5-71的`* -text`保留原始字节；主仓汇总文本的LF转换不替代私库原件hash。

收尾补充提交 [`6095697b08db3f902df9006e8030adbd7f1fda02`](https://github.com/nanake-lab/nemu-evidence/tree/6095697b08db3f902df9006e8030adbd7f1fda02/P5-71/20261113-S1-01/supplement) 保留复制核验、6个已提交归档文件的Git blob逐字节核验，以及报告中29项数字/时长、8个本地链接和最初18,257条CSV的机械核对。核验记录明确未核事项，不充当参考实现审查。追加的8个收尾文件亦与该提交Git blob逐字节匹配，CSV记录其hash及长度；这次追加不改变18,251个原件及归档hash。

归档范围：

- `prepared-s1-01`：执行前完整清单、旧run02逐文件绑定、实际执行源码与锁定文件。
- `supplement-S1-01`：全部824条执行轨迹、分析记录、结果表、配对距离、逐步引用破坏、07-11-S1读数、29份新执行见证/缩减/最小结果、START/END。
- `s1-final-evidence`：优化二进制、编译/测试/static日志（含失败后修复记录）、CI完整日志及实际checkout证据、运行窗口、补充预注册/白名单/新输入、run02运行后36,611文件零差异核验、公开汇总生成脚本。
- `s1-final-summary`：只读汇总代码、基础/完整汇总及一次汇总输入解析失败堆栈；失败不是实验重跑。
- `s1-inventory`：执行前机械性清单/哈希核验。

原冻结v2的完整矩阵、资源与run02原件继续定位到 [`0f2878ded77850e5b367fc47b0ce706cff8f27ae`](https://github.com/nanake-lab/nemu-evidence/tree/0f2878ded77850e5b367fc47b0ce706cff8f27ae/P5-71/20261111-full-v2-run02/archive)；S1清单逐件绑定所读原件，不重复上传或改写旧证据。旧run01/run02标签与原预注册不变。
