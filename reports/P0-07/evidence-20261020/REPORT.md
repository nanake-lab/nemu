# P0-07 路线 1 补证 — 2026-10-20

## 结果

部分完成。三项缺口均补充了原始证据，但数值容差、连续显存峰值与三语方向判断仍未闭合，不选定载体，不放宽 600 ms / 9 GB，不启动下一卡。

用户确认 runner 已停后才开始；开始前无 Listener/Worker、其他 Nemu 探针或 doctor。GPU 工作串行执行。结束后检查测量进程和 5174 监听均已退出，已通知用户恢复 runner。所有模型离线加载，不访问她的任何存储。

## 交付物与复现

- `kernels-v1/`：四个模型，三语各一次 prefill + 八步 decode；完整 reference/native logits 存为 NPZ，指标、实际提示与 token 存为 JSON。诊断深度固定，即使遇到 EOS 也继续教师强制，不作为生成质量或速度验收。
- `memory-v1/`：真实 TTS Base GPU 常驻、RTX 3080 舞台占位并载；WDDM 全部进程计数器、测量进程树和各模型时间窗。窗口来源是本次新建 `path.log` 与 `stop` 文件的 NTFS 创建时间，已记录来源，不伪称来自进程内精确打点。
- `qwen-no-watcher-v1/`：Qwen3-4B 无显存采样器的同协议对照。
- `direction-neutral-v1/`：三款模型各 63 条，三语、每语言七档系数 × 三 seed；完整 JSON、日志和本地 `comparison.html`。
- `checks/`：Python 和仓库静态检查、CPU 测试日志。`source-hashes.json`：实际脚本与 manifest 摘要。
- 探针新增 `kernel_probe.py`、`memory_watch.ps1`、`summarize_supplement.py`、`direction_html.py`、`run_supplement.py`；`direction_probe.py` 默认行为保留，另加显式 `--neutral-dose`。

在激活已有 probe 环境之后，可用 `pnpm probe:supplement --output reports/P0-07/supplement-NEW` 顺序复现补证。也可使用 `--phase kernels|memory|direction`，每次必须指定全新目录。本次实际执行是四个 kernel 子命令、memory phase、单独无采样器矩阵、direction phase，均 exit 0；没有把新增 all 入口声称为已从头完整执行。历史 bf16/int8 数据保留，本轮按路线 1 不再投入。

## 跨内核数值

固定 NF4/bf16、eager；原模型与原生 RMS 分别重算 KV，decode 使用参考路径选出的同一 token 序列。下表是每语言九个位置的最大绝对 logits 差；不是生产容差。

| 模型 | 中文最大差 | 英文最大差 | 日文最大差 | argmax 改变位置 / 27 |
| --- | ---: | ---: | ---: | ---: |
| Qwen3-4B | 0.53125 | 0.2890625 | 0.21875 | 0 |
| Instruct-2507 | 0.34375 | 0.4375 | 0.546875 | 0 |
| Phi-4-mini | 0.640625 | 0.5 | 0.375 | 2（均英文） |
| Qwen3-8B 对照 | 0.5625 | 0.75 | 0.625 | 0 |

证据：`kernels-v1/summary.json`、各 `result.json`、`*-logits.npz`。旧五 token 夹具的零误差仍是事实，但不能推广到真实输入；“优先 Qwen 因跨内核为零”的旧理由不再成立。没有 argmax 改变也不证明任意采样分布或任务能力不变。

routine_reporter 独立从 NPZ 重算最大差和 argmax，全部匹配。独立 NumPy p99 对照初设 1e-6，四个模型/语言组合未满足该核对容差，最大差约 6.10e-6；没有删除或改写这份失败对照，见 `independent-audit.json`。原始 p99 来自 torch float32 分位插值；另保留 CPU 上按原 torch float32 和双精度插值的重算 `quantile-recheck.json`。这些是派生统计的插值精度比较，不是跨内核 logits 的生产容差。

本轮矩阵路径的同内核零调制 prefill/decode 与 changed-vs-fresh 仍为 0，缓存失效 true；三款 4B 注入误差 0，8B 最大 0.001953125。它们与跨内核对照是不同问题，不互相替代。

## 并载延迟和进程显存

固定二十汉字复制夹具；greedy、eager、NF4/bf16、原生 RMS、rank8、既有层号和掩码、一次预热加 20 个独立 KV 请求、96 token / 30 s。各项 path/concurrent/render 均退出 0，各 20/20 匹配。TTS 仅常驻用于负载，不选音色。

| 模型 | 含 WDDM 采样器 p50 ms | 进程专用显存最高观测 bytes | 共享显存最高观测 bytes |
| --- | ---: | ---: | ---: |
| Qwen3-4B | 603.1069999953615 | 5807128576 | 288575488 |
| Instruct-2507 | 592.0561500024633 | 5804998656 | 270987264 |
| Phi-4-mini | 670.2396999971825 | 6005956608 | 268890112 |
| Qwen3-8B 对照 | 636.3789500028361 | 9311465472 | 341766144 |

无采样器 Qwen3-4B 对照：592.8167500023847 ms，20/20 匹配，path/concurrent/render 均 exit 0。单次顺序对照不能证明约 10.3 ms 差异全部来自采样，也不能证明稳定通过 600 ms；不删除 603.1 ms 的未达线结果。2026-10-18 的约 547 ms 只代表此前测量配置与环境。

显存来自本次根进程及其子进程的 WDDM 专用/共享计数器，按 PID 与 adapter 实例分别记录后求和，包含载体/TTS Python 和 Chromium GPU 进程。两列各自取最大值，不把它们相加冒充同一时刻峰值。845 个快照，无采样器错误，最大采集耗时 0.46755 s、最大起始间隔 0.755445 s；123 个快照没有 owned GPU counter，按缺少观测记录，不补零。模型窗口内最大起始间隔约 0.400–0.439 s，仍受 provider 自身刷新限制。全卡读数、torch allocator 峰值也保留在原 JSON，互不替代。

这些数据改善了归因，但不覆盖采样间的短时分配、未及时发现的短命进程或所有驱动开销，因此仍不能认证连续 Nemu 峰值。8B 单是已观测专用显存就有 9.311465472 GB，超过 9 GB（9,000,000,000 bytes）；不把 9 GB 改写成 9 GiB 来通过。

## 信息性方向观察与主代理阅读

沿用三对合成平静/兴奋提取样本，冻结 U；评估使用未参与提取的中性“桌上有书和水”三语提示。七档 -4/-2/-1/0/1/2/4、每档 seed 731–733；独立探索 beta=4，旧 beta=1 和生产配置不改。保留 48 token / 25 s、温度 0.7、完整 EOS 集合。各 63 条均保存，三模型及 HTML exit 0。

主代理逐条阅读全部 189 条：

- 中文：三个模型在每个匹配 seed 的七档剂量下文字均不变，不能签发“三语均看到变化”。Phi seed732 的“桌上的书和杯子里有水”在基线也存在，不能归因于正调制。
- 英文：有措辞和扩写变化，但多数仍是 quiet/calm/peaceful 描写，未见稳定朝兴奋增强的单调趋势。Qwen3-4B / Instruct / Phi 分别 10 / 21 / 20 条触及 token_limit；这些不是完整句子质量结论，不增加预算来消除失败。Phi seed731 的 +4 描写空杯等内容，存在与场景不符的细节。
- 日文：变化主要是“ありました/あります”、置放措辞或少量扩句；Phi 正侧甚至增加“静かで落ち着いた雰囲気”，不能直接当作兴奋方向成功。

共 138 条 EOS、51 条 token_limit、0 条 time_limit。没有宣称该观察完成 P5-31、AC2-16 或通用能力无损伤验证；也不以样例偏好选择参数。完整对照供用户阅读 `direction-neutral-v1/comparison.html`。

## 验收与规格对照

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| Python Ruff | 通过 | `checks/ruff.log` |
| mypy --strict | 14 文件通过 | `checks/mypy.log` |
| CPU pytest | 6 passed | `checks/pytest.log`；含归因算术及模型文本 HTML 转义 |
| pnpm lint / typecheck / test | 通过；21 文件、403 tests | `checks/pnpm-*.log` |
| 新增 PowerShell 语法与执行 | 解析通过，实际 watcher exit 0 | `memory-v1/memory-watch.log`、exit-codes |
| 新补证 GPU 子进程 | 全部 exit 0，非完整卡验收通过 | 各 phase exit-codes / matrix.json |
| 600 ms / 9 GB、生产容差、方向条件 | 未全部满足 | 本报告及 BLOCKED.md |

| 规格条款 | 实现位置 | 核验 |
| --- | --- | --- |
| O §13、S §19：冻结 decoder、显式注入、固定位置/掩码/请求内 m | 现有 injection/path/latency，不改注入实现 | 本轮 matrix/path.json |
| O §13.2：m 改变则 KV 失效 | 现有 RequestCache | CPU 测试及 path.json；无跨请求调制状态 |
| v2.0 §9、I17：模型/语言/数值资源绑定 | kernel JSON、完整 logits、manifest、source-hashes | 三语分别测，不自行冻结容差 |
| P0-07 2026-10-20 追加 | 独立中性剂量观察 | 信息性；正式提取/能力验证留 P5-31 |
| I18、I29、AGENTS 黄金规则14 | 全部为独立合成提示和本地临时运行态 | 未读取排练或 LIFE 存储，无权威状态写入 |

## 新增依赖

无新增包或版本变化。NumPy 使用既有 probe 环境中随依赖安装的版本，新增 PowerShell 脚本只用 Windows 自带 CIM；模型与许可登记不变。根 package.json 只追加 `probe:supplement` 命令。

## 待用户确认与下一步

- [ ] （阻塞）载体最终选择；当前没有满足全部条件的选定项。
- [ ] （阻塞）数值容差和现有显存观测是否足够，不能由本报告自行批准。

建议保留原 RMS 参考路径，继续路线 1 的数值/实现核验与严格计时；不依据旧零误差理由批准当前原生 RMS。是否投入更高分辨率的显存观测，或另开范围更小的数值实现实验，由用户决定。没有改正式锁文件、没有启用 P5-31 或下一卡；PR #27 保持草稿，等待用户及后续交叉审查。
