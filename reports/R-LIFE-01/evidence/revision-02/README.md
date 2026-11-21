# v2离线证据（不是行为测量）

2026-10-29。原v1证据不改；本目录只增加v2检查和解析规划，模型/API/GPU调用0。

- `checks-01/`：20项单元通过；mypy有5项类型错误、Ruff有3项E741命名错误，原始失败日志保留。
- `checks-02/`：修正类型收窄和变量名，并增加完整H1族Holm检查后，21项单元、mypy严格检查13文件、Ruff各执行一次均退出0。UTC06:58:34–06:58:38；每次前后源哈希一致。主代理核对输出，未由子代理裁决研究结论。
- `checks-02/diff.*`、`tracked.*`：`worlds.py`、`judging.py`、`carrier.cpp`、`carrier.py`、`build.ps1`相对v1提交`289cff97`无差异且均已跟踪。
- `prepare-check-01/`：UTC07:01:43–07:01:44，`run.py prepare`退出0。只生成计划，不调用任何模型。
- `prepared-01/`：完整世界清单、19个当前协议/代码绑定、实验标记、`behavior_result=not_run`与`analysis-plan.json`。计划含预注册抽样候选种子清单、10项H1族/8项H2族、明确玩具分布下H1a敏感性、H1b/H2b理想解析功效及费用/会话数；不是经验功效或实验结果。
- `ARCHIVE-MANIFEST.json`：上述54个原始文件的源路径、字节数和SHA256。逐一核对复制前后哈希相同；19项绑定与准备时当前文件一致。索引及清单本身不计入原始文件数。
- `prepare-check-02/`、`prepared-02/`：main合入P5-23后仅更新R-LIFE-02依赖状态，UTC07:06:16–07:06:17重新离线prepare，退出0。此处19项绑定为最终文档/代码版本，prepare-01仍保留此前版本。
- `main-impact.json`：实际测试基线B=`787fd2be`，收尾M=`740c2289`及merge `d3b4176a`；完整增量路径、必读/所用源码变化为空的核对、13个已验证Python文件仍一致。新增Rust规律引擎不被本研究调用，未重跑未受影响检查，不把M冒充新实际测试基线。
- `FINAL-ARCHIVE-MANIFEST.json`：第二次准备与main影响记录的9个新增原始文件，复制哈希均核对。两清单共63个原始文件，未覆盖第一次准备或失败日志。

未归档mypy/Ruff缓存数据库。机械核验由既有子代理按本项目配置显式GPT-6.1 Terra/max执行，不声称按角色名自动加载；无修复、推送、PR操作、CI或模型运行。Codex CLI/订阅与官方条款的核验结论另见[CODEX-JUDGE](../../CODEX-JUDGE.md)，不存在实际评判会话证据。
