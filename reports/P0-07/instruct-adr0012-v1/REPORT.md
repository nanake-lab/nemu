# P0-07 Instruct-2507 — ADR 0012 收尾实测报告

## 结果

ADR 0012 的实测口径满足；等待 Claude 交叉审查和用户批准，尚未合并 P0-07。只测 Qwen/Qwen3-4B-Instruct-2507，修订 cdbee75f17c01a7cc42f958dc650907174af0554，不新增候选。PR #38 已合并为 bd5d9d8，并以 merge 提交 e861ffd 合入本分支；未 rebase、未强推。

固定配置：NF4，bf16 compute，无 double quantization，eager，原生 RMS；零起算第 17 层 attention residual 后、post_attention_layernorm 前，rank 8，固定 last-prefill/every-decode 掩码。PyTorch 2.10.0+cu128、Transformers 4.57.3、bitsandbytes 0.50.2；内核 aten::_fused_rms_norm，经 torch.nn.functional.rms_norm / native_norm.NativeRMSNorm 调用。CPU 测试和依赖核对见 ../instruct-adr0012-preparation/。

## 安静窗口与复现

用户明确停 runner 后，2026-10-20 23:41:14 +08:00 确认 Runner.Listener/Runner.Worker 均不存在，再执行。剂量计划已先提交 03071ae101d39f9a90d0de7e0bd4c5e52509c496，代码提交 27e01214de591d548c45637f5fd9a396569b991e。未运行其他测试、下载或交互聊天。窗口结束核对：测量/渲染进程为 0，5174 无监听，已通知恢复 runner；证据在父目录 instruct-adr0012-window-start.json/end.json。

一条命令复现本轮，须另约 GPU 窗口且使用全新输出目录：

```powershell
workers/model-adapter/probe/.venv/Scripts/python.exe workers/model-adapter/probe/instruct_finish.py --output reports/P0-07/instruct-adr0012-NEW
```

只使用本机固定修订缓存；脚本强制离线，检查 runner、已提交的剂量计划及归档 U 的绑定；三个独立矩阵子进程各重新加载载体和 TTS，再执行方向观察。TTS 是同协议的真实 Base 权重驻留，舞台为 RTX 3080 硬件渲染，不声称正在合成音频或通过正式音色验收。

## 通路、延迟与显存

三次均：同内核零调制 prefill/decode 最大 logits 差 0；注入观测误差 0；m 改变时缓存丢弃，changed/fresh logits 差 0。每次 path/concurrent/render 和 WDDM 采样器退出码均 0。每次排除一个 warmup，正式 20/20 条实际输出与固定二十字句完全匹配，均以句界停止；自回归生成，非 teacher forcing。p50 为中位数，p90 用 numpy linear 分位。

| 独立运行 | p50 ms | p90 ms | 正式样本 | 专用最大观测 bytes | 共享最大观测 bytes | 两项之和 GB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 540.5973000015365 | 551.8742300017038 | 20 | 5805043712 | 293908480 | 6.098952192 |
| 2 | 548.7046999987797 | 564.4164599987562 | 20 | 5804998656 | 270987264 | 6.075985920 |
| 3 | 545.4835000055027 | 553.569240005163 | 20 | 5804998656 | 270987264 | 6.075985920 |

GB = 10^9 bytes。每次 p50 ≤ 600 ms，显存上界 ≤ 8 GB，分别满足 ADR 0012 第 2.3、2.2 条；原 600 ms / 9 GB 不变。9 GB 中的 1 GB 余量按 ADR 保留。三次 allocator allocated/reserved 峰值分别都是 5040005120 / 5261754368 bytes，包含载体和驻留 TTS、排除舞台的 allocator 外分配；WDDM 则覆盖测量进程树的全部已归属进程。不能把 allocator 与 WDDM 再相加。

原始数据：run-1/2/3/matrix/*/concurrent.json、path.json、render.json、matrix.json，各进程原始日志和 memory.jsonl；每次 summary.json 逐项记数。WDDM 的 provider 查询、200 ms sleep 和系统调度使采样非连续。三次最大起始间隔 6.015322 / 0.426681 / 0.416919 秒；专用/共享最大值可来自不同快照，保守相加。保留所有未归属 GPU counter 的快照、外部进程及驱动范围限制，没有声称精确连续峰值。

## 跨内核属性

既有真实三语证据 ../evidence-20261020/kernels-v1/Qwen--Qwen3-4B-Instruct-2507/：原生 RMS 对 Transformers 4.57.3 Qwen3RMSNorm 的最大 logits 差 zh=0.34375、en=0.4375、ja=0.546875，27 个位置 argmax 变化 0。原 NPZ 与 JSON 保留；本轮未重跑此项。按已合并 ADR 0012 第 2.1 条，这是载体配置属性，与同内核零调制检查分开。已写入锁文件候选条目；生产只用该内核，更换按 v2.0 第 9.5 节 / I17 迁移，不批准跨内核任意替换。

## 信息性范数与剂量观察

复用原 U，SHA-256 3173bb95b4f94029f4a1a6ae008e0e5be9e0d6d7984d252b2facbff2ee00534b；方向实际范数 0.9990000009987084。没有改提取样本、拟合方向或按输出偏好挑参数。预注册计划的文件 SHA-256 为 034638c683c27de5e004d6fea5f1bddd18ad9531328e9b1447c36b8c47986b52。

| 语言 | 最后 prefill 位置 h 范数中位数 | decode 位置 h 范数中位数 |
| --- | ---: | ---: |
| zh | 41.82356466481255 | 49.85668451192784 |
| en | 42.52559240686032 | 51.02159645683774 |
| ja | 43.634513447314866 | 47.899637549257605 |

先在 m=0 下用相同中性提示、每语三个 seed 测出以上六个中位数，其六者中位数 45.76707549828623 为统一参考。原始各位置范数和 9 个 m=0 输出在 direction/norms.json，写完该文件后才开始非零剂量。固定比例 -0.4/-0.2/-0.1/-0.05/0/0.05/0.1/0.2/0.4，对应 a[0] 约 -18.325155/-9.162578/-4.581289/-2.290644/0/2.290644/4.581289/9.162578/18.325155，其余七维为 0；beta=18.325155336349354，仅本次信息性研究预算。

81 条完整输出在 direction/direction.json 和 [comparison.html](direction/comparison.html)。每种语言 27 条，三个 seed=731/732/733，fresh KV，48 token / 25 秒、temperature .7 不变。主代理逐条阅读了全部输出：

- 中文在 +0.4 三个 seed 均变化，其余八档各 seed 相同；变化同时扩写了书页翻动、水杯微倾或冒泡等场景，不能只看“热闹”字样就判定方向合格。
- 英文各比例有文本变化，但多数样本为描述性扩写；27 条中 25 条到 token_limit，2 条 EOS。不得把截断当完整自然回答或延长预算掩盖。
- 日文在 +0.2/+0.4 等剂量有变化；+0.4 seed732 出现原场景没有的咖啡气味，seed733 混入 uiten，暴露流畅度/语义风险。

zh/ja 各 27 条均 EOS。本轮没有 time_limit；全部 stop 与 token IDs 保留。相同剂量英/日已变而中文仍不变的低剂量区间，以及高剂量扩写/混杂文字，作为 P5-31 风险记录；不宣称三语稳定、单调的兴奋方向，也不由这些样例选生产参数。按 ADR 0012 第 2.4 条，本项是信息性的。

## 手动试聊

在 D:/Dev/nemu-p007 根目录由用户启动：

```powershell
pnpm probe:chat
pnpm probe:chat --ratio 0.1 --norms reports/P0-07/instruct-adr0012-v1/direction/norms.json
```

默认 m=0；非零只能取预注册比例，绑定实测 reference/U/修订/计划摘要，沿同一归档方向。示例不是推荐剂量。每轮显示首句延迟；48 token / 25 秒内无句界会明确提示。/reset 清 RAM 上下文，/exit 退出；程序不保存对话，不打开任何 Core、排练或正式存储。手动入口不由测量或 CI 启动，CI 拒绝加载模型；帮助与拒绝路径已验证，实际交互留给用户试聊，不算验收。

## 依赖、范围与待确认

无新增运行时依赖；原探针许可证登记保留。原多候选历史数据及失败样本保留，未扩展到 P5-31，未访问她的存储。

- [ ] （阻塞合并）Claude 交叉审查。
- [ ] （阻塞合并）用户批准 P0-07 选定条目与交付。
- [ ] （不阻塞）用户手动试聊；不替代上述测量或 P5-31。

下一步：选定条目仅作为本 PR 可审查交付，保留 needs-user，等待 Claude 与用户，不合并、不启动下一卡。Windows CI 必须在最终收尾提交后通过；状态在 PR 上核对。

## 最终锁文件验证（追加）

models.lock 的 selected 状态、固定修订与必需 selection 记录已通过 schema 校验；更新后的 pnpm test 再跑 443 tests 全部通过，日志 pnpm-test-selected.log。前面的 lint/typecheck/pytest/Mypy/Ruff 记录在准备目录。测试生成的八个 fixture 日志归档于 cpu-fixture-logs/，内容不含真实对话。

## 机械独立重算（追加）

routine_reporter 的 CPU 只读重算与本报告数字一致：3 次 p50/p90、所有 60 个正式句子、WDDM 两项 maxima、allocator、path/渲染/TTS 元数据、9 个 m=0 样本、81 个剂量组合、六个范数中位数、原 U/预注册计划字节 SHA、全部系数和 HTML 行均核对。有效检查点及审计脚本错误原样保留在 audit/；前期字段/序列化错误属于机械审计脚本，不是 GPU 测量失败，没有据此重跑 GPU。

run-1 最大 6.015322 秒采样间隔在 sampler 第 1/2 条，早于 path/渲染/并载载体加载记录，时间戳见 audit/README.md。仍保留原 summary 与采样限制。子代理仅报告证据，是否满足 ADR 口径和选定/合并判断由主代理承担。
