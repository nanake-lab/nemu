# P5-71 v2 首次运行清单

登记于首次探针执行之前。预注册为 `e58542622da8a33a17d12f26dd42a5b4e699a5cc`，内容 SHA-256 `79abd4bbd7d331ad0255d2cd9d382256e17c261c3b4fe7fb7eb0bddce3cf5709`，未修改规模、判据或资源。

代码提交 `af9c71b63596779b856e92e6b35a51cf2937bdd1`；合入 main `74f7bcaf41ec45bb31063fc079eeeae49927ac82`（自冻结基线起仅阅读规则/代理与任务说明变动，核心代码仍是获批基线）。Rust 1.99.0 `b940084d7`，Cargo 1.99.0 `5f94df478`，Python 3.12.10，libm 0.2.16。普通 Cargo test profile，未优化、含调试信息；不把它的成本当 release 性能。

本清单先授权 §8 六条成本探针。完整矩阵已按 `(ID,层次,世界,分支,版本/变异,N)` 展开成本计划；全量历史目前保留为固定预注册规则与参数元组，未全部物化，剩余适配器也未完成。若超过用户的四小时门槛，在继续全量实现/运行之前停止；不能把这个阶段称为整卡实现或机制验收完成。

编译前已核对 R-STEER-01 停在 C3 预注册、会话 idle，且无对应测量 worker。未使用 GPU、模型 API、真实历史或存储。

## 固定输入与算术来源

完整 JSON 清单（含逐文件长度、全部测试工具 SHA-256、Cargo.lock、实际二进制、工具链及运行量）暂存 `D:/Dev/cache/P5-71/prepared-v2-r1/RUN-MANIFEST.json`，归档到私有证据库；本次清单 SHA-256：`b96f07979e1293a0c77ab72fb0996784d7fd3ee115582ad1f3cc885763e212f8`。

| 文件 | SHA-256 |
| --- | --- |
| C.json | `ce1d4fc7e59b99f3157e1a2d41248663928d0e1efefd36b15f42470e745762f5` |
| C-alarm.json | `bd7e089cc5a07b3125ec7b856ef1780ee46bfbc4fbd9c259cd2931e2707aaefe` |
| F.json | `8afe66b4e89cecc3a73bfa33594f2be0dccf185212a387d51312a0899615b9fb` |
| G.json | `a5e22008024a3ad07dc33887e4803aaf767c33d7e5568ace37daebf52f7cf488` |
| probe-inputs.json | `b417ea7e97fa91e497d3c3e5b69273a7b1e45d5d84dc9ca241039b5ef3ade3f9` |
| full-plan.json | `0c9ba4bd7ca6f73c9fb5fcfc540e19b10649986f70468ea55d3e1a773699cdf2` |
| 复用来源 dynamics/tests/d07_oracle.py | `7409611234e102456a1a1943a95580f9fe0f5754d4bfb92352eba4e0dd258333` |
| 复用来源 fast/contribution/tests/generate_oracle.py | `ea3a871909f705c3af5722c7feedb9e179d8b21bd19f687b96af695e6081261c` |
| 实际纯算术文件 tests/p571/arithmetic.py | `0a087babba6780cd153fc6150282863a9ba79532cd131e4a375700f82875f657` |
| 测试二进制 p571-5beb611af570f188.exe | `496860085eca68bf7bb7604eafeee16dc4f788e43cbee6b892386ec641634c51` |

两个来源文件锁定 `8839dcae73c98437f61222f49d08cafd9ba8f4e6`；只提取获批 Fraction、位转换、单次与方向舍入。手算边界向量在探针前运行，失败即中止。C/F 递推另写；libm 原语经独立位模式 RPC 调用，不返回/接收 DUT 状态，缓存键仅为函数名和输入位模式。DUT 增量/重算比较完整规范字节；C oracle 比较所有估计器与 θ 字段，F 比较 x/槽/目标/锚点，适配器保留完整规范字节。探针只测成本，不签发机制通过结论。

## 全量成本计划

以下是计划执行量，尚未运行。已冻结 `not_run` 的 DUT 行保留在 full-plan.json，执行量为零；研究参考/变异没有生产入口，不冒充 DUT。09-18 每对初态的两侧分别计一次 N=1；G 的 N=5 含准备前缀；四种 F 参考共享一个 G 轨迹，未虚增四套 G 计算。候选模型的四版本（正常及三单一变异）单列。

| 层次/家族 | ΣN | Σ N(N+1)/2 |
| --- | ---: | ---: |
| DUT/C | 19536 | 5994184 |
| DUT/CF | 9216 | 230912 |
| DUT/F | 17072 | 314088 |
| DUT/G | 200 | 600 |
| DUT/固定率单步及私有候选 | 0 | 0 |
| independent_research/C | 130479 | 46587172 |
| independent_research/CF | 20416 | 514272 |
| independent_research/F | 47424 | 938768 |
| independent_research/G | 200 | 600 |
| independent_research/算术单步 | 20 | 20 |
| sensitivity_model/候选 | 2400 | 2400 |

成本探针：C 四轨迹 N=32，ΣN=128、逐前缀重算量 2112；F 两轨迹 N=30，ΣN=60、逐前缀重算量 930。DUT 与独立研究各有这六条轨迹；F 的两个 DUT 参考在一次桥接调用内执行，各自推进/重算一次，没有增添探针样本。

## 运行入口与停止门槛

已完成 `cargo test -p core-kernel --test p571 --no-run --message-format=json` 编译准备；日志另存，不运行正式矩阵。首次测量入口：

```powershell
python core/crates/core-kernel/tests/p571/probe.py D:/Dev/cache/P5-71/prepared-v2-r1 D:/Dev/cache/P5-71/target/debug/deps/p571-5beb611af570f188.exe D:/Dev/cache/P5-71/probe-01
```

按各家族实测的逐前缀重算纳秒/操作乘全量增量与重算量；CF 用 C+F。另记每提交时间、每前缀成本、峰值工作集与证据字节。G、候选检查、单步算术、长史有理数分子增长和完整证据序列化尚不能由六条探针直接估计，不把未知成本当零。若已知部分已超过 14400 秒，立即按用户指示报告并等待；若未超过，不能仅凭已知部分宣称完整预算通过，还须交代剩余成本。禁止缩样、改判据或自动启动全量。

## 第一次工具故障后的登记（仍在完整探针前）

`probe-01` 在首条 C 轨迹的空前缀导出时报 `Invalid("theta law binding")`；资源引用的版本字符串未与 C 的 law_id 对齐。未完成任何轨迹，没有完整探针成本或机制结果。算术手算检查已通过，失败原日志保留。只修正适配器绑定并加入桥接子进程的峰值工作集测量；没有改变已物化输入、资源、枚举数量和判据。

重跑登记：代码 `6b94540c2dd926992446cbd178f01de205655dfe`；实际二进制 SHA-256 `4fe7b2c5e8074a46537fdcaa75ba202faebcbf4995306ecf677105c4b7436a66`；新清单 `prepared-v2-r2/RUN-MANIFEST.json` SHA-256 `8ec43893307fe3c302cd7eb152a3187c5536fada5ff65cc3001f001d6e71c1e6`。其中所有输入/资源文件哈希与 r1 相同，代码哈希已更新。入口仅将 prepared 目录改为 `prepared-v2-r2`、输出改为 `probe-02`；不覆盖 probe-01。

## 工具验证后的最终探针登记

r2 未执行；单步工具检查修正了日志句柄及 F 的 θ 引用类型，Python 补齐类型与格式检查。F 引用为代数夹具绑定，所指 F 资源含真实 θ 值；不是可恢复 ThetaSnapshotV3，不声称覆盖 H 解析或 09-19 引用验收。没有修改资源或冻结输入。

本次正式成本探针代码 `bdd52e9fdb0d9dee7a908bcd6b4555d54a21eac4`；二进制 SHA-256 `668c1a6c3f1300788d07674db1c73a7be3a1b8cc91019cfee41b560f5b0b8cf1`；实际算术文件 SHA-256 `ab549e7c533bec311fb7cdea38803f0caac811c502953782761ff38eaecce40d`；清单 `prepared-v2-r3/RUN-MANIFEST.json` SHA-256 `fa3f215fc4c9170e57428c10290dfb97f5c6854f6eb6465ba2466623230c8503`。输入/资源/成本清单字节仍与 r1 相同。使用 r3 输入、输出 `probe-02`，不覆盖失败探针。运行前已完成 Rust 编译、新工具三项检查、clippy、fmt、Python mypy strict 与 ruff；原有 kernel 单元测试 273 passed、1 ignored，原全包命令因新工具故障退出，工具修复后单独重验。


## 全量执行登记（16 小时授权）

用户批准原冻结 v2 全量一次，不做工具优化；此登记发生在正式执行之前。代码提交 `8205b16fb3e5cbcda49165684b6620db89b036d9`，验证日志 full-kernel-test-01 / full-clippy-01 / full-fmt-check-01 全部退出 0。全量输入已物化，原准备文件保留。主内核源码 tree 与冻结基线相同：`4db8d68c86ce465d4ccdffec1d726b3d37448417`。

完整清单暂存 `D:/Dev/cache/P5-71/prepared-full-v2-02/RUN-MANIFEST.json`，SHA-256 `a0bd02cb20b34b624b84d28af15fa0638aa3f7eb957c94c83a81a64c65a7a0a0`；正式运行入口逐文件核对其中的实际代码、完整输入/资源和二进制哈希。清单包含 Rust/Cargo 完整版本、Python、Cargo.lock、复用源和实际原语，以及逐行执行计划。

| 项目 | SHA-256 | 字节 |
| --- | --- | ---: |
| matrix.jsonl | `bcdd35560845ca6db644b261e94ffac4379d99250b28fa896fd319c58c078483` | 85634810 |
| C.json | `ce1d4fc7e59b99f3157e1a2d41248663928d0e1efefd36b15f42470e745762f5` | 494 |
| C-alarm.json | `bd7e089cc5a07b3125ec7b856ef1780ee46bfbc4fbd9c259cd2931e2707aaefe` | 520 |
| F.json | `8afe66b4e89cecc3a73bfa33594f2be0dccf185212a387d51312a0899615b9fb` | 224 |
| F-interaction.json | `104a181b6e49e980607b30f17e4dc74464276716251670c0ec588956bda7826b` | 236 |
| G.json | `a5e22008024a3ad07dc33887e4803aaf767c33d7e5568ace37daebf52f7cf488` | 847 |
| 实际 arithmetic.py | `ab549e7c533bec311fb7cdea38803f0caac811c502953782761ff38eaecce40d` | 2352 |
| 测试二进制 | `c074f02f64968bfadf7ce0dd1520d5585b21063e29f50b161e94b88e11c26d63` | 5359104 |

| 层次/家族 | 计划行 | 固定 not_run 行 | ΣN | 逐前缀重算量 |
| --- | ---: | ---: | ---: | ---: |
| DUT/C | 560 | 432 | 19536 | 5994184 |
| DUT/CF | 728 | 400 | 9216 | 230912 |
| DUT/F | 21968 | 13368 | 17072 | 314088 |
| DUT/G | 40 | 0 | 680 | 2040 |
| DUT/arithmetic | 20 | 20 | 0 | 0 |
| DUT/candidate | 2 | 2 | 0 | 0 |
| independent_research/C | 559 | 0 | 130479 | 46587172 |
| independent_research/CF | 728 | 0 | 20416 | 514272 |
| independent_research/F | 21968 | 56 | 47424 | 938768 |
| independent_research/G | 40 | 0 | 200 | 600 |
| independent_research/arithmetic | 20 | 0 | 20 | 20 |
| independent_research/baseline_subpath | 10 | 0 | 5375 | 2886380 |
| production_wiring/handoff | 13 | 13 | 0 | 0 |
| sensitivity_model/candidate | 2400 | 0 | 2400 | 2400 |

G DUT 的正例含四条 F 路线各自的 G 调用，负对照仅 A1；07-10 baseline_subpath 是同一历史中分报的额外基线计算，不增加独立样本。固定 not_run 行的计算量为零。16 小时计时从正式 START 开始，包含矩阵与见证缩减；准备、编译和只读汇总不计为实验执行。中断、故障或到限不自动重启，保留每个已刷盘前缀，未完成的矩阵/缩减不报通过。

## 本次执行结束记录

`full-run-01` 于 2026-11-11 07:36:54.131164 UTC 启动，09:54:29.815616 UTC 因已确认的 D09-B 参考资格工具缺陷停止，实际 8255.684452 秒，退出 2。没有达到 16 小时，没有第二次正式执行。375 条完整轨迹、1 条部分轨迹（独立研究 A 的 07-09，到 620/1280）全部保留。计划文件把 07-09 提前到 07-05 之前的偏差亦已登记，不回写清单。

执行代码、输入、资源、原语与二进制仍绑定上述原始哈希。运行后只读汇总修正提交 `0bf0c92210d0d7ef69561bcee04e32be6045c000` 独立留存；原执行源码副本已先行保全。私有证据提交 `f7f473a2c40cbdc04c69e2a24fbb790e80e64cad`，逐文件见 [EVIDENCE-INDEX](EVIDENCE-INDEX.md)。状态与边界以 [REPORT](REPORT.md) / [BLOCKED](BLOCKED.md) 为准。

## 修复审查准备（不是新一轮执行登记）

用户选择修复后重新审查。代码 `6062a36038752e19f41d3213c5b51d013e872612` 已修正 B 资格、行序及参考自查发现的问题；私有 `repair-registration.json` 仅记录软件验证时的代码/工具链/文件哈希，`formal_run_started=false`。重新物化矩阵 SHA-256 为 `d34c3c9017cbc9f922cfb51d47748b8ef2ba669023300aef109a32b1b8daceea`，数量/成本/资源不变，只有行序和 B 研究资格表改变。该文件没有发送给正式执行入口。

本节不授权重跑，不覆盖前述运行登记。待用户审查后重新批准，才在新的输出目录从头执行冻结矩阵并再次登记当时真实代码、二进制、资源、输入与原语哈希；上限16小时，旧部分结果不拼接。

## full-run-02 执行前登记

用户已批准修复与自查，并授权当前 head CI 通过后，从头运行冻结 v2 一次，最多57,600秒。旧 full-run-01 只作记录，不参与新结果或前缀。CI [43050075490](https://github.com/nanake-lab/nemu/actions/runs/43050075490) 已 success，实际 checkout `7d2c740832a7d59b217712cf42dc9b672c8488b9`，两父提交分别为 main `88be8131d1240209da0a1c9428f2e0875c0e4380` 和获批 head `d650aba57bb8b414f293d7ad7df1a457a50834bc`。本次没有修改运行工具或判据。

代码绑定 d650aba5，32份测试源码逐字节匹配；新目录 `prepared-full-v2-03` 的全部物化文件与获批修复准备目录逐字节一致。新清单 SHA-256 `aee05f49a76fa10aafb21e96cc9ce27c0d5d03df809ef8c85cd266a42c69ec27`（10,322字节）；matrix SHA-256 `d34c3c9017cbc9f922cfb51d47748b8ef2ba669023300aef109a32b1b8daceea`；测试二进制 SHA-256 `08f700e24a26d3b3bc9db762f7b6c60da41292d631ee8bec8b6ea495efb4f9c3`（5,409,280字节）。五资源和实际 arithmetic.py 哈希保持前述登记值。

工具链仍为 Rust/Cargo 1.99.0、Python 3.12.10、libm 0.2.16，Cargo test普通未优化构建；32份源码、二进制、Cargo.lock、工具链文件、算术复用来源、输入/资源和分层 totals 全部登记到新私有清单。分层 ΣN/逐前缀重算量沿用上表，49,046行，34,755 ready、14,291预定 not_run。16小时包含正式矩阵和见证缩减，超时/故障/中断保留已完成前缀，剩余 not_run，不重试或续跑。

本机已合入的构建基线是 a6a0567f。此次 fetch 的 main `02372ff6` 相对该基线只有报告、指定隐私清理及其 Docs guard例外；没有本卡源码、输入、测试命令或CI数学检查变化，按影响规则不另合并。最新 AGENTS 的一次性docx元数据清理授权已读取，不能扩展用于本卡。运行前未发现项目GPU测量或其他项目重负载。实际启动/结束时间由新 `full-run-02/START.json`、`END.json` 记录。


## full-run-02 结束记录（只读收尾）

执行代码 d650aba57bb8b414f293d7ad7df1a457a50834bc 未变。UTC 2026-11-11 15:52:34.411575 至2026-11-12 07:52:34.408347，时间戳差57,599.996772秒，单调时钟16小时deadline退出2。49,046行遍历完成，34,755条ready轨迹均完整，14,291条保持冻结not_run；无部分矩阵轨迹。额外见证缩减只有123份流程完成、11份谓词未复现，其余保留首差异。

实际DUT ΣN=46,504、逐前缀重算6,541,224；独立研究198,539与48,040,832；候选模型2,400与2,400，与预先登记的可运行矩阵量一致。独立baseline_subpath子路径另列，不重复算矩阵行。完整逐家族统计及真实失败见REPORT与RUN02-SUMMARY。

运行后49/49文件/清单绑定核对一致，全部原件归档见EVIDENCE-INDEX。停止后只进行JSON读取汇总、源码字节保全、归档与报告，不执行任何递推、编译、测试或缩减。旧full-run-01原件与结果不变，不拼接。
