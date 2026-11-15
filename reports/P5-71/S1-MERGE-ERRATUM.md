验证档位：重档；修复预注册研究的分析读数，另存勘误，不执行DUT或矩阵。

# P5-71 S1 别名合并局部误差勘误

接受[复审P2](https://github.com/nanake-lab/nemu/pull/154#issuecomment-6845150184)，已修复合并舍入残差漏记在本步读数的问题。S1中48个合并分析步骤已逐个核对，24个局部读数更正，另外24个仍为零，其余分析字段全部一致。005136新增检查先在旧代码上失败、修复后通过。S1原件及标签保留，勘误另存；请原审查者做增量复审，保持草稿。

## 改动与核对

- 修复提交：`33bef6c44c460419701e319118bbfd24a92e1f3b`。`s1_analysis.py`合并分支把同一个`ideal - actual`残差同时加入`local[j]`与`self.error[j]`。依据：S1补充预注册§4的局部/累计读数要求及ADR0035§4.6–4.7。
- `s1_checks.py::merge_005136`取冻结09-11输入生成器的sign=+1、coordinate=0、variant=6，前三步为1s/2s的binary64 0.1/0.2及合并。固定半步算术夹具验证前一步分解残差为−2⁻⁵⁶、合并后为0，本步首坐标区间为[2⁻⁵⁶,2⁻⁵⁶]，其余坐标为[0,0]，累计首坐标仍为[0,2⁻⁵⁴]。该检查经既有reference_checks入口进入正常CI。
- 勘误按代码路径选取S1分析白名单中所有包含merge的可运行行，包含零残差样本，不按结果挑行。先绑定48份原分析、源轨迹与输入哈希，再由已存的合并前后状态及上一步分析状态调用修复后的单步`Analysis.observe`；没有调用状态递推、Φ/libm原语或DUT。逐个断言除local_error以外全部字段与旧分析相同，并用精确Fraction槽和独立核对局部残差。
- 005136第3步从[0,0]更正为[1/72057594037927936,1/72057594037927936]。全部48步中24步非零并更正；96个分析/源轨迹绑定在计算后再次匹配。原分析先与S1归档FILES中的SHA-256/长度核对，原件没有改写。

## 检查与证据

| 检查 | 结果 |
| --- | --- |
| `python core/crates/core-kernel/tests/p571/s1_checks.py` | 修复前exit 1，失败位置为新增local_error断言；修复后exit 0；两份日志保留 |
| `python core/crates/core-kernel/tests/p571/reference_checks.py` | exit 0，手算与工具自查通过，不启动正式driver |
| 锁定Ruff check / 两个改动文件format --check | exit 0 |
| 锁定mypy --strict，p571全部30个Python文件 | exit 0 |
| 私库勘误脚本`correct_merge_readouts.py` | exit 0；48步核对、24步更正、其他字段相等 |
| `git diff --check`、新Markdown相对链接、JSON解析 | 通过 |

Python 3.12.10、Ruff 0.15.6、mypy 1.20.2。此次只改Python分析及检查，未编译、未运行Cargo测试或DUT；旧Rust测试/CI的成功不冒充本次head的检查结果。当前head远端CI由正常工作流处理，提交报告时尚未确认其完成。

原始勘误、选择清单、逐行读数、复现脚本、源码快照及检查日志在私库提交[`18dcf77e6b30e4a376a260f4d3d1c7f8bf3e294c`](https://github.com/nanake-lab/nemu-evidence/tree/18dcf77e6b30e4a376a260f4d3d1c7f8bf3e294c/P5-71/20261115-S1-merge-erratum-01)。[证据索引](EVIDENCE-INDEX-S1-MERGE-ERRATUM.json)列16文件的SHA-256与长度；已核对本地原件、复制件和该提交Git blob逐字节一致。原S1继续使用[既有索引](EVIDENCE-INDEX-S1.md)，不将本次勘误写回S1结果表。

main影响核对：从上次CI实际main父提交`f39f5e9feb4bce7ed56774645b8166e483e7ce7c`到本次fetch的`8acfe0b6e2906809fc44cbf9c0638c2b10a77a48`，新增为文档/报告/规则、两线研究划分及独立的r-steer工具；core、依赖与CI工作流无变化。README新增经历模型说明，不改本轮冻结数值规则；按最新AGENTS/skill读法继续，未合并无关main变化。

## 限制与未完成

本轮是既有证据的局部读数勘误，不是新一轮实验，也不签发整卡通过。分析见证缩减仍未完成（`not_run`），按用户指示本轮不补。S1原有09-04的8/16灵敏度缺口、A0的8条分析失败、未知分解、理想连续系统误差界及真实体验等边界均保留。没有修改生产核心、参数、契约或判据，没有新增依赖。等待原审查者增量复审；不合并、不领下一卡。
