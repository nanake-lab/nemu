# P5-71 证据索引

## full-run-02（16小时到点结束）

正式执行代码 `d650aba57bb8b414f293d7ad7df1a457a50834bc`，冻结预注册 v2 未改；旧 full-run-01 保留且不拼接。本轮执行到 UTC 2026-11-12 07:52:34，34,755 条可运行轨迹均完成，14,291 条固定 not_run。C-02 位差、07-11 未检出及见证缩减缺口见 [REPORT](REPORT.md) / [BLOCKED](BLOCKED.md)。

执行前登记私库提交 [`b15bb91f9dd83db0d9588291f91a84d7749954cc`](https://github.com/nanake-lab/nemu-evidence/tree/b15bb91f9dd83db0d9588291f91a84d7749954cc/P5-71/20261111-full-v2-run02/preflight)，含实际二进制、CI成功与checkout父提交、构建/物化日志、资源窗口和完整运行清单，先于正式执行存档。

完整原件私库提交 [`0f2878ded77850e5b367fc47b0ce706cff8f27ae`](https://github.com/nanake-lab/nemu-evidence/tree/0f2878ded77850e5b367fc47b0ce706cff8f27ae/P5-71/20261111-full-v2-run02/archive)。[逐文件索引](EVIDENCE-INDEX-run02.csv) 记录归档内路径、SHA-256、字节数、执行代码、ID/层次/版本/行号及不可变证据提交。CSV的archive_entry非空时，private_path指向实际ARCHIVE.json；按该清单拼接14片后，在tar中查找archive_entry。轨迹status是比较标签，见证行写retained，必须查看原件里的缩减status；不将原件保全等同于机制通过。

共36,696个归档内文件、3,484,073,999字节；无损压缩为873,779,789字节。顺序拼接 `evidence.tar.gz.part000`…`part013` 得完整tar.gz，SHA-256 `2148707ced73aa934e8364e95682b729ddf2c87ba1a13046d0de1f2b05518815`。ARCHIVE.json列14片各自长度与哈希；FILES.json登记所有原件；VERIFICATION.jsonl记录解包后36,696/36,696逐件匹配，拼接整体也匹配。原始full-run-02目录自身为36,611文件、3,363,088,478字节，余项为完整物化输入/资源、源码快照、登记日志、只读汇总与机械核验，不是新增样本。

归档包括32份绑定执行提交的测试源码，以及Cargo.lock、rust-toolchain和冻结预注册；运行START/END、全部前缀、结果、1,559个mutation-witness、22个first-failure、134个minimum、缩减日志均保留。minimum中123份流程completed、11份未复现谓词not_run；其余不称最小见证。压缩前后各核对一次源字节，20个归档元数据/分片Git blob再次逐字节核对。

Git blob与只读收尾核验补充提交 [`dbb3c6c8e9d55d65621f82080510e26256c51097`](https://github.com/nanake-lab/nemu-evidence/tree/dbb3c6c8e9d55d65621f82080510e26256c51097/P5-71/20261111-full-v2-run02)。其中GIT-VERIFICATION.json绑定归档提交的20个原始Git blob；supplement保留文档检查、复制/Git核验脚本、远端归档SHA和上传日志。15个补充文件也与该提交的Git blob逐字节匹配。逐文件CSV共36,743行：36,696个归档条目、20个归档文件、12个执行前登记文件、15个收尾补充文件。

主仓 RUN02-SUMMARY / RUN02-DELAYS 是只读汇总副本；Git 对主仓文本统一 LF 时，JSON 值不变，原始字节哈希以本索引指向的私库归档内文件为准。私库 P5-71 的 `* -text` 保留原始换行与字节。

以下各节保留历史时点的状态和原件定位，不作为本轮结果，也不以本轮新状态改写旧记录。
## 修复与自查（未开始第二轮实验）

私有证据提交 [`6d2226a5efc05ded2ad9960a91840e5176485966`](https://github.com/nanake-lab/nemu-evidence/tree/6d2226a5efc05ded2ad9960a91840e5176485966/P5-71/20261111-repair-review)，绑定修复代码 `6062a36038752e19f41d3213c5b51d013e872612`。[逐文件索引 EVIDENCE-INDEX-repair.csv](EVIDENCE-INDEX-repair.csv) 记录归档内原始路径、SHA-256、字节数、代码/证据提交、ID和状态。

归档65条原件，共92,743,027字节，压缩为 `evidence.zip` 4,293,536字节，SHA-256 `693ba322abcac8754cfa1582b2fb62b30179075b94ae6a9091e3973d718c1e7b`；流式解包65/65逐条哈希匹配。包含修复时32份测试源码、冻结预注册与构建锁定文件、重新物化的完整输入/资源、counts/totals、新旧多重集合对照、源码绑定和工具链登记、手算与正常测试/static检查日志。非锁定Ruff失败日志也保留，不以最终成功覆盖它。

索引另列7个归档/核验文件，共72行。6个原始归档文件的 Git blob 与工作区原字节一致；核验补充提交 [`3d93acfd51aef19d69a301935d91ecbdf05dfcf3`](https://github.com/nanake-lab/nemu-evidence/blob/3d93acfd51aef19d69a301935d91ecbdf05dfcf3/P5-71/20261111-repair-review/GIT-VERIFICATION.json)。

`repair-registration.json` 明确 `formal_run_started=false`，只是审查准备的文件登记，不是新的执行清单。32份源码与修复提交逐字节相同；counts/totals、五个资源与旧物化一致；除排序及 B 研究资格表之外，全部规范输入行多重集合一致。第二轮正式矩阵、成本探针、见证缩减均未启动；旧 `full-run-01` 原件及索引不变，不拼接。

自查结论和局限见 [REFERENCE-SELF-AUDIT](REFERENCE-SELF-AUDIT.md)，软件验证见 [REPORT](REPORT.md)。下列旧标签只表示旧代码曾执行的检查；本轮发现的参考缺陷不会通过修改旧原件来掩盖。

## 正式执行 full-run-01（中断，部分结果）

私有证据提交 [`f7f473a2c40cbdc04c69e2a24fbb790e80e64cad`](https://github.com/nanake-lab/nemu-evidence/tree/f7f473a2c40cbdc04c69e2a24fbb790e80e64cad/P5-71/20261111-full-v2-run01)，分支 `evidence/P5-71-probe`。原始执行代码 `8205b16fb3e5cbcda49165684b6620db89b036d9`；运行后只读汇总修正 `0bf0c92210d0d7ef69561bcee04e32be6045c000`。

[逐文件索引 EVIDENCE-INDEX-full.csv](EVIDENCE-INDEX-full.csv) 共 620 行：601 个原始归档条目加 19 个归档/补充文件。每行记录私有路径、归档内路径、SHA-256、字节数、证据提交、代码绑定、ID 和状态。原始轨迹的状态来自只读覆盖校正，不修改原 results.jsonl；变异 failed 不表示所选规律失败，详见 REPORT。历史工具日志保留原过程，不把后来的代码提交强加给更早日志。

原始条目 1,022,697,525 字节，无损压缩为 210,638,104 字节；按编号拼接四片 `evidence.tar.gz.part000`…`part003`，完整 SHA-256 为 `7dce1d6e13e9b131ba48e5f8d1461708025eb8954095d878f46311f2cdd67702`。各片 SHA/长度见 ARCHIVE.json；FILES.json 与 VERIFICATION.jsonl 记录 601/601 条流式解包哈希验证。最大原始文件超过 100 MB，因此保留原字节并分片归档，不截断轨迹。

执行源码/二进制已先于运行后汇总改动保全；29 份源码、Cargo.lock 与二进制匹配运行清单，toolchain 文件匹配执行提交。另存源代码缺陷摘录、STOP/END、原始结果、全部已刷盘前缀、99 个未缩减首差异、输入/资源/原语、工具失败与最终验证日志。supplement 包含执行顺序审计、额外基线路径成本核对及汇总边界检查；正式矩阵没有重跑。

当前结果是工具故障后的部分执行，不是整卡通过。既定接口/接线缺口与中断剩余项分别保留 not_run；真实体验未运行。

## 原成本探针（历史，不替代正式矩阵）


私有仓库 `nanake-lab/nemu-evidence`，提交 [`707bee9dfc656626b617f1df6822b4cf20712c49`](https://github.com/nanake-lab/nemu-evidence/tree/707bee9dfc656626b617f1df6822b4cf20712c49/P5-71/20261111-probe-v2)，分支 `evidence/P5-71-probe`。

正式成本探针代码 `bdd52e9fdb0d9dee7a908bcd6b4555d54a21eac4`；预注册 `e58542622da8a33a17d12f26dd42a5b4e699a5cc`。旧清单各自记录对应代码。DUT/独立研究的六条探针是成本证据，全量 E07/E09/交互为 not_run。

归档已逐文件核对 Git 提交中 blob 与原始文件字节相同，72/72。初次归档发生 Git 换行规范化，后续提交设置任务目录 `* -text`，恢复原始字节；本索引只指向核验后的提交。两个过程提交均保留。

所有下列路径相对于该私有仓库；ID 由文件名和对应 full-plan/probe-inputs 清单定位。registration 只登记、不代表运行；failed-retained 保留工具失败；cost_only 不签发机制通过；passed-tooling 仅指工具检查。

| 私有路径 | SHA-256 | 字节 | 状态 |
| --- | --- | ---: | --- |
| `P5-71/20261111-probe-v2/ARCHIVE.json` | `52e109458b75709960732f031b3b14a908d8ecbd32a8d3e942ce3b53c54ec88a` | 11996 | registration |
| `P5-71/20261111-probe-v2/bin/p571.exe` | `668c1a6c3f1300788d07674db1c73a7be3a1b8cc91019cfee41b560f5b0b8cf1` | 3340800 | registration |
| `P5-71/20261111-probe-v2/build-01.log` | `512adefa063eee10328092f367fce3840bd79dc6e84fa368061b2c8ce49078eb` | 28232 | passed-tooling |
| `P5-71/20261111-probe-v2/build-02.log` | `fa99499d2fa167019178168fa8da8ac7449fa6e29fda175a53389bfb84908212` | 27473 | passed-tooling |
| `P5-71/20261111-probe-v2/clippy-01.log` | `ff5448fe22436850b3808edee6b779d75d2b4091f169c142911dc49dfe1e2433` | 722 | passed-tooling |
| `P5-71/20261111-probe-v2/doc-test-01.log` | `44a298144ef61a47b657d4245fb1561dd9a74864c0605f8d4452fcde1ccce36d` | 218 | passed-tooling |
| `P5-71/20261111-probe-v2/fmt-01.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | 0 | passed-tooling |
| `P5-71/20261111-probe-v2/kernel-test-01.log` | `e489be0dfcd88fafb2d4422ff19edbed62f41ae406915e17b611053193eb5f5c` | 29360 | failed-retained |
| `P5-71/20261111-probe-v2/mypy-01.log` | `6aa8738c96eaae00c92a7ecd7c129d75977aafdd01b750ff372927e06c1378ee` | 1205 | failed-retained |
| `P5-71/20261111-probe-v2/mypy-02.log` | `1349375aa999a8c5cccf6d1afda33360d95c575fc0ab2473c7043407a7805b68` | 261 | failed-retained |
| `P5-71/20261111-probe-v2/mypy-03.log` | `5e07a0d1e3531bd3118cd64d4798e8535b4a2afc7caa59995ba2e358d1b15b91` | 211 | failed-retained |
| `P5-71/20261111-probe-v2/mypy-04.log` | `90377c8fd97127580c93ac5740d596c9ebc9db94dafa6560ee11fe16b4460c62` | 44 | passed-tooling |
| `P5-71/20261111-probe-v2/prepared-v2/C-alarm.json` | `bd7e089cc5a07b3125ec7b856ef1780ee46bfbc4fbd9c259cd2931e2707aaefe` | 520 | registration |
| `P5-71/20261111-probe-v2/prepared-v2/C.json` | `ce1d4fc7e59b99f3157e1a2d41248663928d0e1efefd36b15f42470e745762f5` | 494 | registration |
| `P5-71/20261111-probe-v2/prepared-v2/F.json` | `8afe66b4e89cecc3a73bfa33594f2be0dccf185212a387d51312a0899615b9fb` | 224 | registration |
| `P5-71/20261111-probe-v2/prepared-v2/full-plan.json` | `1308ac354e5ad6ab8d3d3f9b3b5400faddef06b86f8c363cb5753f0695f51816` | 12719800 | registration |
| `P5-71/20261111-probe-v2/prepared-v2/G.json` | `a5e22008024a3ad07dc33887e4803aaf767c33d7e5568ace37daebf52f7cf488` | 847 | registration |
| `P5-71/20261111-probe-v2/prepared-v2/plan-summary.json` | `c5d810c02c336d43b2a91d46434affafbdd493db042e71bf45d3bfb25bc9ea28` | 1493 | registration |
| `P5-71/20261111-probe-v2/prepared-v2/preparation.json` | `c24ba6ce1412628fb2614189b939543bad952b8008ae86e85ce8755de6095115` | 1132 | registration |
| `P5-71/20261111-probe-v2/prepared-v2/probe-inputs.json` | `b417ea7e97fa91e497d3c3e5b69273a7b1e45d5d84dc9ca241039b5ef3ade3f9` | 27399 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r1/C-alarm.json` | `bd7e089cc5a07b3125ec7b856ef1780ee46bfbc4fbd9c259cd2931e2707aaefe` | 520 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r1/C.json` | `ce1d4fc7e59b99f3157e1a2d41248663928d0e1efefd36b15f42470e745762f5` | 494 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r1/F.json` | `8afe66b4e89cecc3a73bfa33594f2be0dccf185212a387d51312a0899615b9fb` | 224 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r1/full-plan.json` | `0c9ba4bd7ca6f73c9fb5fcfc540e19b10649986f70468ea55d3e1a773699cdf2` | 21138240 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r1/G.json` | `a5e22008024a3ad07dc33887e4803aaf767c33d7e5568ace37daebf52f7cf488` | 847 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r1/plan-summary.json` | `41b05e2309943ca128c1c7146bf3e1455b0aeb09cd0698da469260b1dabb357f` | 1494 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r1/preparation.json` | `9fc5f41510a016f24d4d0643c0696dd0885ed993d14d6af72f18f81b27face57` | 1132 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r1/probe-inputs.json` | `b417ea7e97fa91e497d3c3e5b69273a7b1e45d5d84dc9ca241039b5ef3ade3f9` | 27399 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r1/RUN-MANIFEST.json` | `b96f07979e1293a0c77ab72fb0996784d7fd3ee115582ad1f3cc885763e212f8` | 6537 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r2/C-alarm.json` | `bd7e089cc5a07b3125ec7b856ef1780ee46bfbc4fbd9c259cd2931e2707aaefe` | 520 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r2/C.json` | `ce1d4fc7e59b99f3157e1a2d41248663928d0e1efefd36b15f42470e745762f5` | 494 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r2/F.json` | `8afe66b4e89cecc3a73bfa33594f2be0dccf185212a387d51312a0899615b9fb` | 224 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r2/full-plan.json` | `0c9ba4bd7ca6f73c9fb5fcfc540e19b10649986f70468ea55d3e1a773699cdf2` | 21138240 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r2/G.json` | `a5e22008024a3ad07dc33887e4803aaf767c33d7e5568ace37daebf52f7cf488` | 847 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r2/plan-summary.json` | `41b05e2309943ca128c1c7146bf3e1455b0aeb09cd0698da469260b1dabb357f` | 1494 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r2/preparation.json` | `9fc5f41510a016f24d4d0643c0696dd0885ed993d14d6af72f18f81b27face57` | 1132 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r2/probe-inputs.json` | `b417ea7e97fa91e497d3c3e5b69273a7b1e45d5d84dc9ca241039b5ef3ade3f9` | 27399 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r2/RUN-MANIFEST.json` | `8ec43893307fe3c302cd7eb152a3187c5536fada5ff65cc3001f001d6e71c1e6` | 6766 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r3/C-alarm.json` | `bd7e089cc5a07b3125ec7b856ef1780ee46bfbc4fbd9c259cd2931e2707aaefe` | 520 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r3/C.json` | `ce1d4fc7e59b99f3157e1a2d41248663928d0e1efefd36b15f42470e745762f5` | 494 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r3/F.json` | `8afe66b4e89cecc3a73bfa33594f2be0dccf185212a387d51312a0899615b9fb` | 224 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r3/full-plan.json` | `0c9ba4bd7ca6f73c9fb5fcfc540e19b10649986f70468ea55d3e1a773699cdf2` | 21138240 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r3/G.json` | `a5e22008024a3ad07dc33887e4803aaf767c33d7e5568ace37daebf52f7cf488` | 847 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r3/plan-summary.json` | `41b05e2309943ca128c1c7146bf3e1455b0aeb09cd0698da469260b1dabb357f` | 1494 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r3/preparation.json` | `9fc5f41510a016f24d4d0643c0696dd0885ed993d14d6af72f18f81b27face57` | 1132 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r3/probe-inputs.json` | `b417ea7e97fa91e497d3c3e5b69273a7b1e45d5d84dc9ca241039b5ef3ade3f9` | 27399 | registration |
| `P5-71/20261111-probe-v2/prepared-v2-r3/RUN-MANIFEST.json` | `fa3f215fc4c9170e57428c10290dfb97f5c6854f6eb6465ba2466623230c8503` | 6817 | registration |
| `P5-71/20261111-probe-v2/probe-01/arithmetic-hand-vectors.json` | `90a4125137ebba67a0784fa311643c615af7a17c04527aa028aa7961c742720c` | 99 | failed-retained |
| `P5-71/20261111-probe-v2/probe-01/bridge.log` | `d4b5cc5c49476347ecd013df7f450291e36737d015f38c29fe5c24e54a9f9f3a` | 457 | failed-retained |
| `P5-71/20261111-probe-v2/probe-01-command.log` | `a0170f42719b9f8e556bd6c3a6e122d31fc42319a0524be334a78e708979aeac` | 1173 | failed-retained |
| `P5-71/20261111-probe-v2/probe-02/07-01-p2-b-0.25-A-dut.json` | `a65c37b6d428daf92c15da68afe9bf7c1ddc5e807c394c34b27b5c1932f2189a` | 336247 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/07-01-p2-b-0.25-A-independent.json` | `8c659729cbf4797747ad8ca466a3877ce13fd816b88320d52d94560c6f027a32` | 277753 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/07-01-p2-b-0.25-old-dut.json` | `20ec2cec1c5e05167729e22399f470fd483b531e2caea565ca9dede0a45afd85` | 333495 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/07-01-p2-b-0.25-old-independent.json` | `28190b6b2e19cdbe98a09ba0394e69190379a019ac7a85098efbcb6b13ae102c` | 275313 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/07-01-p2-b0.25-A-dut.json` | `225fbb850c8acfc7937b505021089a056352fad6ca69c3cfdd1bc1d814afe702` | 336005 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/07-01-p2-b0.25-A-independent.json` | `b301711990566e7f9c4778bd4a3ddf13b04a517d3a86d09ccc6f54098ec89dfb` | 277761 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/07-01-p2-b0.25-old-dut.json` | `eb063ccfe7970aeb35c773fe743f2a826a1c0df72d07895d85e14a462b212132` | 333364 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/07-01-p2-b0.25-old-independent.json` | `8e29757d514ee6708474d208cb1c4c50e8f6b4997d604b235f70b09bb8e7e9c6` | 275320 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/09-06-plus-j0-123-A1-independent.json` | `70699f51dc2cc8f7bac27a6e8f86bc7a6b31fe280b407e24a71a0daa215e65cd` | 72299 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/09-06-plus-j0-123-old-dut.json` | `4502585324852c76d6572a79653cb29d9f1d7f1b3cac6db292284109e67300ac` | 172357 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/09-06-plus-j0-123-old-independent.json` | `bf6361f203173fb30783429c4072a69b7a4427291926a0f7a8cc88d6b4e2fdd6` | 16064 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/arithmetic-hand-vectors.json` | `90a4125137ebba67a0784fa311643c615af7a17c04527aa028aa7961c742720c` | 99 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/bridge.log` | `572ef5e211b76a8937807737053342ec0cd3bead5fe87340ba5ef4eabce07c98` | 150 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/primitive-cache.json` | `c7cef1b8662228cff2413c9916df1efe6e6c49f5cc2ffc4ecd21d29801d86c1c` | 7183 | cost_only |
| `P5-71/20261111-probe-v2/probe-02/SUMMARY.json` | `374e87f242dc9bd0115b7f10646c53ad711baf64b0ee920d8e7b44758b74b8a4` | 3398 | cost_only |
| `P5-71/20261111-probe-v2/probe-02-command.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | 0 | passed-tooling |
| `P5-71/20261111-probe-v2/probe-tooling-test-02.log` | `acf084d1fa1045e4e6900ea1d16fdf623524de1877001404d9e87dc1b2ed8695` | 1762 | failed-retained |
| `P5-71/20261111-probe-v2/probe-tooling-test-03.log` | `f40d71f8a7c64884fe14efac68c86360bedc8ce3e6246c16dd13c87d03bcc76a` | 2228 | failed-retained |
| `P5-71/20261111-probe-v2/probe-tooling-test-04.log` | `3021eb09f52e5573100dc320b76473ef9beeff78d6b8c90c2301a774936df8e0` | 789 | passed-tooling |
| `P5-71/20261111-probe-v2/python-format.log` | `b055b90273e8663cdc5a9f0b62833edf98eb5e8e8473bda5d271a93ced319cd3` | 21 | passed-tooling |
| `P5-71/20261111-probe-v2/ruff-01.log` | `a4443afdcfb6d7363adb285762515ccf7cf50473b1a05c20c1a50f6bed4d26b0` | 20 | passed-tooling |
| `P5-71/20261111-probe-v2/ruff-02.log` | `a4443afdcfb6d7363adb285762515ccf7cf50473b1a05c20c1a50f6bed4d26b0` | 20 | passed-tooling |
