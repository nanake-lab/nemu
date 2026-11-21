重档。本卡包含 RTX 3080 10 GB 上的真实语音合成首包、取消、30 分钟连续压力复核与 CUDA Graph 独立对照测量。

本轮在无 GPU 与 RTX 3080 独占窗口内完成了内存稳定性修复复核、确定性标点分句流式、预热内核初始化，以及 V5 `torch_static_cuda_graph` 独立对照。\
她现在长时间说话的内存与显存已通过 30 分钟连续压力复核（尾 10 分钟进程树私有提交斜率 −2.01 MiB/h，专用显存首尾持平），原 40 句首包 P50 从 1,053 / 759 ms 降到 885 / 537 ms，多句长句首包缩短 3.1–4.8 倍。\
原 40 条无句内标点短句的首包 P50 仍未达到 250 ms，且配对多分句样音待用户试听确认；PR #45 保持草稿与 `needs-user`。

# P4-02 交付报告（2026-11-21 第二轮）

**部分完成（内存稳定项已通过，首包 250 ms 仍未达标）。** 内存与显存稳定性已通过无 GPU 10,000 次平台期验证和 RTX 3080 上 30 分钟连续合成复核；取消与子进程故障恢复继续通过。原 40 句短句在默认基准配置下中英文首包 P50 分别为 **885 ms、537 ms**，未达到 250 ms 阈值；单独测试的 V5 `torch_static_cuda_graph` 未缩短总首包且增占显存，本轮保持默认关闭。未完成项见 [BLOCKED](BLOCKED.md)。

## 第一步：内存稳定性（无 GPU 定位 + 30 分钟 GPU 复核）

### 根因与修复（无 GPU）

在无 GPU 的真实 HTTP/WS + 子进程 Pipe 路径上执行了 3,000 次 `tracemalloc` 分配跟踪与 4,000 / 10,000 次连续建连、合成、断开对照（[汇总数据](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/round2-summary.json)）：

- **根因（引用循环滞留 + 回环 PCM 默认开启 zlib 压缩，非请求对象真泄漏）：** `uvicorn` + `websockets.legacy.server` 的 `WebSocketProtocol` 在连接关闭后通过 Python 引用循环滞留到第 2 代 GC 回收（约每 1,100–2,200 次请求一次）。原 [main.py](../../services/voice-sidecar/src/main.py) 使用默认的 `ws_per_message_deflate=True`，每个未回收连接都挂着 `zlib.compressobj` 与 `zlib.decompressobj`（约 300 KiB 堆与句柄），形成约 **12.3 MiB 锯齿**（4,000 次对照：`43.55 → 55.86 MiB`，句柄 `269 → 301`，显式 `gc.collect()` 后回落至 `44.42 MiB`、`traced_current = 2.65 MiB`，请求对象无残留）。上一轮 30 分钟压测（1,856 次）的最后 10 分钟恰好落在锯齿上升沿，造成 HTTP 进程 `+17.67 MiB/h`。
- **修复：** 在 [main.py](../../services/voice-sidecar/src/main.py) 显式设置 `ws_per_message_deflate=False`（本机 `127.0.0.1` 回环原始 float32 PCM 不应做 zlib 压缩）。
- **无 GPU 平台期证据：**

| 无 GPU 测量臂 | 请求数 | HTTP Private Bytes（首 → 末 / 峰值 / GC 后） | 后半段斜率（MiB / 1k 次） | 句柄（首 → 末） | 子进程 Private（首 → 末） |
| --- | ---: | --- | ---: | --- | --- |
| 修复前对照（`deflate=True`） | 4,000 | 43.55 → 52.77 / 55.86 / 44.42 MiB | −0.94（12.3 MiB 锯齿） | 269 → 301 | 16.14 → 16.14 MiB |
| 修复后对照（`deflate=False`） | 4,000 | 38.76 → 42.48 / 43.16 / 42.14 MiB | −0.25 | 243 → 243 | 16.15 → 16.15 MiB |
| 修复后 `tracemalloc` | 3,000 | 46.24 → 53.78（含 tracemalloc 表）；GC 后 `traced = 2.50 MiB` | — | 243 → 243 | 16.20 → 16.09 MiB |
| 修复后长程平台期（`deflate=False`） | 10,000 | 41.20 → 41.43 / 43.26 / 40.80 MiB（区间 40.86–43.26 MiB） | **+0.048** | **243 → 243** | **16.24 → 16.24 MiB** |

### 30 分钟真实 GPU 复核（通过）

在 RTX 3080 上沿用上轮 40 条固定短句循环，连续运行 **1,800.7 s**，完成 **1,799 次**真实 WebSocket 合成，40/40 个 case 各自逐字节一致（聚合 RTF 0.562，压测首包 P50 796 ms，P95 1,106 ms）：

| 指标 | 全窗 30 分钟（n=1,799） | 尾段 10 分钟（n=609，599.4 s） | 上一轮尾段 10 分钟 |
| --- | --- | --- | --- |
| 进程树 Private Bytes | 9,640.37 → 9,665.89 MiB（+25.52 MiB，+11.42 MiB/h） | 9,663.31 → 9,665.89 MiB（+2.59 MiB，**−2.01 MiB/h**） | +21.28 MiB/h |
| HTTP 进程 Private Bytes | 38.55 → 42.32 MiB（+3.77 MiB，+6.72 MiB/h） | 41.74 → 42.32 MiB（+0.58 MiB，**+4.01 MiB/h**） | +17.67 MiB/h |
| 模型子进程 Private Bytes | 9,601.82 → 9,623.57 MiB（+21.75 MiB，+4.70 MiB/h） | 9,621.57 → 9,623.57 MiB（+2.00 MiB，**−6.02 MiB/h**） | +3.61 MiB/h |
| 进程树 RSS | 4,830.39 → 4,869.69 MiB（+39.30 MiB，+11.94 MiB/h） | 4,869.31 → 4,869.69 MiB（+0.38 MiB，**−0.69 MiB/h**） | 工作集收缩 |
| HTTP 进程句柄数 | 244 → 242（全程 242–244） | 恒定 | 未单记 |
| WDDM 专用显存（7,237 样本） | 2,204.52 → 2,204.52 MiB（区间 2,180.52–2,226.52 MiB，全窗峰值 2,508.52 MiB） | +1.74 MiB/h，首尾差 0.00 MiB | 首尾相同 |
| WDDM 共享显存 | 恒定 72.00 MiB | 恒定 72.00 MiB | 恒定 72 MiB |

尾段 10 分钟内进程树私有提交与 RSS 斜率均为负（−2.01 / −0.69 MiB/h，远低于 12 MiB/h 复测门限，且未出现上轮工作集收缩干扰），专用显存首尾一致，内存与显存稳定性验收通过。

## 第二步：首包延迟（基准组 vs CUDA Graph 对照组）

新增确定性标点分句模块 [clauses.py](../../services/voice-sidecar/src/nemu_voice_sidecar/clauses.py)（按中英标点切分、保护小数点、合并短于 5 非空字符的片段以与上游预切分一一对应），在 [gsv_worker.py](../../services/voice-sidecar/src/nemu_voice_sidecar/gsv_worker.py) 中对多分句启用 `return_fragment=True`（`batch_size=1`、`fragment_interval=0.15`）逐分句流式送出，并在 `/warmup` 中用临时合成正弦波预热 CUDA 内核。默认配置显式保持 `"use_cuda_graph": False`。

### 原 40 句短句（中英各 20 句，真实 WebSocket 建连到首块 PCM）

两组分开实测，均保留原 40 句语料与 250 ms 阈值：

| 测量组 | 中文 P50 | 中文 P95 | 中文最大 | 英文 P50 | 英文 P95 | 英文最大 | 250 ms 验收 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 上一轮（2026-11-21 首轮） | 1,053 ms | 1,438 ms | 4,238 ms | 759 ms | 1,723 ms | 1,795 ms | 未通过 |
| **本轮基准组（关 CUDA Graph，含内核预热 + 分句流式）** | **885 ms** | **1,110 ms** | **1,579 ms** | **537 ms** | **1,034 ms** | **1,409 ms** | **未通过** |
| **本轮 CUDA Graph 对照组（`rich==13.9.4`，开 `torch_static_cuda_graph`）** | **1,139 ms** | **1,449 ms** | **1,868 ms** | **548 ms** | **1,413 ms** | **1,470 ms** | **未通过** |

- **基准组结论：** 内核预热与关闭回环压缩使中文 P50 降 168 ms、最大首包从 4,238 ms 降到 1,579 ms，英文 P50 降 222 ms、P95 降 689 ms。但由于原 40 句全是 5–9 字或 3–6 词的单分句、不含句内标点，标点分句对其均为 1 段，单分句端到端推理仍需 537–885 ms，未达 250 ms。
- **CUDA Graph 对照组结论（不建议进默认路径）：** 按用户批准在 `gsv-v5-venv` 安装固定版本 `rich==13.9.4`（MIT）后，日志与健康状态证实 `torch_static_cuda_graph` 正常激活（`accelGraphEnabled=True`，`accelGraphFailed=False`）。虽然 AR 阶段 P50 确降至 272 ms（中）/ 312 ms（英），但中文 V5 CFM+Vocoder 阶段 P50 反升至 **891 ms**（基准约 220 ms），导致中文总首包 P50 退化至 1,139 ms（+254 ms），英文 P50 基本持平（548 vs 537 ms），两者 P95 均变差，WDDM 专用显存峰值增加 122 MiB（2,630.5 vs 2,508.5 MiB），且 40 句 PCM 与基准组 0/40 逐字节一致。因此本轮保持默认 `"use_cuda_graph": False`。

### 多分句长句切分前后对比与试听样音

对 4 条含句内标点的中英文长句，比较同一句整句合成（`cut0`、不切分）与按固定标点切分流式送出的首包延迟及音频（配对 WAV 存私有证据仓，供用户试听）：

| 样例 | 分句数 | 切分前首包 | 切分后首包 | 首包提速 | 切分前 / 后时长 | 配对试听（私有证据仓） |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| `zh_multi_01` | 4 | 3,684 ms | **767 ms** | **4.80×** | 8.28 s / 7.12 s | [切分前](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/zh_multi_01_unsplit.wav) · [切分后](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/zh_multi_01_split.wav) · [CG 切分](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/zh_multi_01_cudagraph_split.wav) |
| `zh_multi_02` | 4 | 2,571 ms | **534 ms** | **4.81×** | 7.00 s / 6.28 s | [切分前](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/zh_multi_02_unsplit.wav) · [切分后](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/zh_multi_02_split.wav) · [CG 切分](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/zh_multi_02_cudagraph_split.wav) |
| `en_multi_01` | 4 | 3,148 ms | **1,016 ms** | **3.10×** | 8.52 s / 8.32 s | [切分前](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/en_multi_01_unsplit.wav) · [切分后](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/en_multi_01_split.wav) · [CG 切分](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/en_multi_01_cudagraph_split.wav) |
| `en_multi_02` | 4 | 2,735 ms | **644 ms** | **4.25×** | 7.36 s / 7.32 s | [切分前](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/en_multi_02_unsplit.wav) · [切分后](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/en_multi_02_split.wav) · [CG 切分](https://github.com/nanake-lab/nemu-evidence/blob/853b69fcc99662ab14234c9c32628e1d0605a179/P4-02/round2-20261121/en_multi_02_cudagraph_split.wav) |

## 取消、恢复与历史归因

- **取消回归（6/6 通过）：** 首包前 3 次均 0 块返回；收到首块后 3 次分别有 1–2 个在途块，最晚 **0.35 ms** 到达（阈值 ≤ 50 ms），连接均以关闭码 `1000` 结束。
- **故障恢复（通过）：** 首次 `/warmup` 耗时 58.44 s（含合成正弦波内核预热）；主动终止模型子进程后 `/health` 返回 `WORKER_EXITED`、`ready=false`，再次 `/warmup` 并恢复完整合成通过。
- **既有归因保留：** 首次英文延迟（4.233 s，已移入预热）、CUDA DLL 映像 560.883 MiB 与同精度 CUDA/CPU 加载净差 1.074 GiB、上游 `wns1` TorchScript 门函数首轮数值差异（本轮按排序决定不改上游数值路径）详见上一轮归档与 [证据索引](EVIDENCE-INDEX.md#2026-11-21-正式-provider-与归因)。

## GPU 窗口、依赖与验证

- **GPU 安静窗口：** **2026-11-21 14:34:50 +08:00 至 2026-11-21 15:11:14 +08:00**（UTC `06:34:50Z–07:11:14Z`）。开窗前确认 #240 Windows 作业（`44873820256`）及排队作业均已结束，停止 runner 并确认 `Runner.Listener` 与 `Runner.Worker` 均退出（0 个运行），写入 `D:\Dev\cache\gpu-window.txt`；开窗前背景总进程 640、内存占用 25,821.1 / 32,694.2 MiB、显存 1,275 / 10,240 MiB，无其他重负载。测完已删除 `gpu-window.txt` 并恢复 `Runner.Listener` 运行（关窗背景总进程 646、内存 24,217.9 MiB、显存 2,016 MiB）。
- **新增环境依赖（经用户批准，仅用于独立推理环境 CUDA Graph 对照）：** `rich==13.9.4`（MIT）及其传递依赖 `markdown-it-py==4.3.0`（MIT）、`mdurl==0.1.2`（MIT）、`pygments==2.21.0`（BSD-2-Clause），已登记在 [DEPENDENCIES.md](DEPENDENCIES.md)；应用代码目录无新增依赖。
- **同步 main 与本机静态检查：** 分支已 merge 最新 `origin/main`（`2954da2b`），仅修改卡面目录 `services/voice-sidecar/`（`src/main.py`、`src/nemu_voice_sidecar/clauses.py`、`src/nemu_voice_sidecar/gsv_worker.py`、`tests/test_clauses.py`）与 `reports/P4-02/`。Windows 本机 `uv run ruff check .`、`uv run mypy --strict src tests`（29 文件）、`uv run pytest -q`（58 通过，1 GPU smoke 跳过）、`pnpm tts:schema`、`pnpm lint`、`pnpm typecheck`、`pnpm test` 全部通过。
- **私有证据索引：** 本轮全部原件、12 份 WAV、SHA-256 与 41 个归档成员清单固定于私有仓库提交 `853b69fcc99662ab14234c9c32628e1d0605a179`，见 [证据索引](EVIDENCE-INDEX.md#2026-11-21-第二轮内存稳定性分句首包与-cuda-graph-对照)。
