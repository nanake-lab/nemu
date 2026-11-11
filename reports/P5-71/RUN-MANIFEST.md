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
