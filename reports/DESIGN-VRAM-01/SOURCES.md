# 来源、证据口径与待批准读法

阅读/设计日期：2026-11-02（Australia/Perth）。仓库必读以 main **ee5c2b6068b9a2200ad5fd48ebbc8e337458c0d5** 为基线；未改旧报告、原始证据、规格或配置。下列旧数据均为引用，不是本卡测量。

## 仓库依据

| 来源 | 本卡使用的内容 |
| --- | --- |
| [AGENTS.md](../../AGENTS.md) §§5–6、9 | 简单优先、有出处的数值/规则；轻档与设计卡白话总览；高难度交叉审查、needs-user 与草稿；资源测量窗口 |
| [ARCHITECTURE](../../docs/ARCHITECTURE.md) §§2–6 | 单卡基准、现行性能要求、原则 8、结构化读取优先、进程与权威边界；旧显存估算不能代替同驻实测 |
| [CONTRACTS](../../docs/CONTRACTS.md) §§4–8 | 真实观察与回执、生成输入、行动/权限；当前列出的消息不自动具有图像/近看新语义 |
| [Echo Core README](../../docs/echo-core/README.md)、[v2.0](../../docs/echo-core/echo-core-v2.0-first-principles.md)、[0008](../../docs/adr/0008-echo-core-math-clarifications.md)、[0009](../../docs/adr/0009-echo-core-v2-first-principles.md)、[0010](../../docs/adr/0010-v1-scope-birth-and-lineages.md)、[0011](../../docs/adr/0011-network-life-boundaries.md) | 权威写权限、真实记录、m=0 判断、证据来源、重放与谱系；后续已定修订优先 |
| [ADR 0005](../../docs/adr/0005-rehearsal-and-birth.md) 与 [ADR 0025](../../docs/adr/0025-desktop-perception-and-history.md) | 临时测试存储；原文/大对象保留；外源、桌面、自致来源区分；标题授权不直接推出截图授权 |
| [ADR 0012](../../docs/adr/0012-continuity-carrier-mobile.md) | 载体连续性、判断一致性、显存采样口径、执行配置变化须验证 |
| [ADR 0026](../../docs/adr/0026-continuous-cognition.md) | 当前材料与 H/Q 接续、本地 m 选择、独立 m=0 解释、公平资源交接与真实暂停；结合末尾有效性说明阅读 |
| [ADR 0027](../../docs/adr/0027-voice-model-breeze-target-gsv-interim.md) | 目标 Breeze / 过渡 GPT-SoVITS；约 3.5 GB 是第三方估计，未实测；本卡不重选声音 |
| [ADR 0031](../../docs/adr/0031-carrier-engine-and-injection.md) | llama.cpp b11669、Q4_K_M、直接库优先；状态清除、server 禁缓存、原 wire 异步 IRL、E2 分阶段启用 |
| [ADR 0032](../../docs/adr/0032-presets-as-starting-points.md) | 起点可改变、保险丝只防工程错误；不恢复统一回落；结果时间与执行凭证分开；真实来源与身体调控 |
| [ADR 0034](../../docs/adr/0034-qwen35-4b-conditional-carrier.md) | 新视觉载体有条件切换与既定回退；测量完成不等于通过，不静默改模型/引擎 |
| [DOC-PRESETS-01 索引](../../docs/traceability/DOC-PRESETS-01.md) 与 [差异登记](../../docs/traceability/diff-register.md) V2-71/87/97/117 | 来源分工；维护记录与直接经历分开；成长故障/结构故障/报警分开；不复活旧主动机会和旧行为预算 |

## P0-12：固定于 PR #128 的 a7cd1876

本卡从固定 Git 对象阅读以下文件，并核对该 PR 最新评论没有模型切换批准；不把报告内部保留的旧 head 描述当作远端当前 head。

- [REPORT.md](https://github.com/nanake-lab/nemu/blob/a7cd18764a5d557818a2d9d5be9ee650db0b3af8/reports/P0-12/REPORT.md)：七项结果，重点⑤⑥；官方模型修订 851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a、固定引擎、F16 mmproj 与配置。
- [显存摘要](https://github.com/nanake-lab/nemu/blob/a7cd18764a5d557818a2d9d5be9ee650db0b3af8/reports/P0-12/continuation-02/memory-summary.json)：全会话专用 9,240,739,840 bytes、共享 115,343,360 bytes，两独立峰值之和 9,356,083,200 bytes；⑥自身之和 9,345,597,440 bytes。属于 owned process tree 采样，未加载 TTS/舞台。
- [图像计时边界](https://github.com/nanake-lab/nemu/blob/a7cd18764a5d557818a2d9d5be9ee650db0b3af8/reports/P0-12/continuation-02/image-timing-boundaries-v2.json)：1024² 七图 TTFT P50 618.35 ms；编码区间 P50 339.45 ms；完整 prompt 包含子阶段，不重复相加。PNG 预处理没有独立计时，未验 OCR 准确率。
- [raw-02 映射](https://github.com/nanake-lab/nemu/blob/a7cd18764a5d557818a2d9d5be9ee650db0b3af8/reports/P0-12/raw-02/raw-filemap.json) 中 **items-5-7/server/server.stderr.log**；对应[原有压缩对象](https://github.com/nanake-lab/nemu/blob/a7cd18764a5d557818a2d9d5be9ee650db0b3af8/reports/P0-12/raw-02/objects/0a3bbccad0a3d3733992007906f7059574718b853b44e30060acf9c530b9bd5d.gz)。本卡只读原有缓存日志与该映射，未新建证据归档或作全量哈希核验。

该日志 231–247 行给出：image_size 768、patch_size 16、n_merge 2、image_min_pixels 8192、image_max_pixels 4194304；两次 dummy warmup 为 **1472×1472**，随后视觉 CUDA compute buffer 为 **4628.88 MiB**，视觉 Flash Attention disabled。

因此，“默认最大约 2048²”来自最大面积，“4628.88 MiB”来自实际初始化日志。不能把它改写成“2048² 实际编码峰值”，也不能用名义 image_size 768 断言所有图片实际缩到了 768²。日志中的模型、KV、循环状态与 decoder compute 是另外的分量；buffer 日志不是完整 WDDM 峰值。

P0-12 的 9/21 图片达到输出上限、预处理计时缺口、三语来源检查 100/200 与 mypy 未运行等限制继续保留。这里只把来源检查不足列为条件切换风险，不替该卡判定通过/失败或触发 Qwen3-VL 回退。

## P0-10：主线原测量

- [REPORT](../P0-10/REPORT.md) 与 [BENCHMARK-REPORT 的载体显存](../P0-10/BENCHMARK-REPORT.md)：旧 Qwen3 context 8192、m=0；Q4 两独立峰值和 4,723,433,472 bytes，Q8 为 6,506,024,960 bytes。不是新 Qwen3.5 的单载体底数。
- [INJECTION-REPORT](../P0-10/INJECTION-REPORT.md)：context 512 注入实验之和 3,930,730,496 bytes；不能与长上下文数值互换。
- [P0-09 选项中的同驻部分](../P0-10/P0-09-OPTIONS.md)：GPT-SoVITS 未部署、同驻未测；禁止把约 3.5 GB 估算直接相加宣布通过。

## 组件能力的外部查证

先用 Context7 的官方 llama.cpp 文档定位入口，再查看 **620d2225cc436ab0b001e3200e590e3b286ef9ab** 固定源码。当前文档中的选项不能不经核对就套用到固定版本；以下仅为静态能力依据，本卡未执行这些选项。

| 固定来源 | 静态结论及限制 |
| --- | --- |
| [common/arg.cpp](https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/common/arg.cpp)；本机 P0-10 已保存副本交叉阅读 | 有禁视觉 GPU offload、图像 token 上限及视觉 batch 相关参数；参数存在不保证峰值或准确率 |
| [tools/mtmd/clip-model.h](https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/tools/mtmd/clip-model.h)，set_limit_image_tokens / set_warmup_n_tokens | 输入上限与 warmup 有关联，部分预热可受自定义上限约束；实际分配形状仍须记录，不能只按最大像素推算 |
| [tools/mtmd/clip.cpp](https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/tools/mtmd/clip.cpp)，get_dummy_batch / warmup / reserve_compute_meta | 有按 dummy 图建图和计算缓冲预留路径，以及视觉 Flash Attention 支持/回退处理。由此提出测量候选，不宣称本机已支持或能省固定容量 |
| [tools/mtmd/mtmd.h](https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/tools/mtmd/mtmd.h)，mtmd_context_params / mtmd_init_from_file / mtmd_free | 视觉 context 有 CPU/GPU 选项与独立生命周期。研究局部装卸有接口依据；不等于现有 server 可热装卸，也不证明完整视觉/decoder状态处理正确 |
| [Microsoft DXGI_QUERY_VIDEO_MEMORY_INFO](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/ns-dxgi1_4-dxgi_query_video_memory_info) | OS 的预算与当前使用不同，超预算可能有性能代价；用于解释为何要测全桌面余量，不提供本机预测 |

## 位置 / 依据 / 新读法或待批准扩展

下表不是新的规范。已定修订按依据阅读；图像专用新语义只是本卡待批准设计。用户限定写入范围，因此本卡不向 ADR、IDL 或差异登记追加编号；若选择实施，由后继卡先发布所需 ADR/契约及登记。

| 位置 | 依据 | 本设计的读法与边界 |
| --- | --- | --- |
| REPORT 看清边界、ROUTES A/B | 用户本卡目标；ADR 0032 §1 | 原图可完整读取是能力；选区/是否继续是她的选择。建议提供两者，未定默认流程、尺寸或心理门槛 |
| REPORT 近看行动 | CONTRACTS §§4–6；ADR 0026 §§4–5 | 新图像对象、覆盖及近看请求/结果需后继契约；不把 stage gaze 或旧 BodyAction 偷换为图像读取 |
| REPORT IRL 与关注 | EV6；ADR 0012、0031、0034 | 独立 m=0 解释不由 m 下的选区代替；每请求状态隔离，原图解释未完成要明确，异步 IRL 不新增阻挡回复门 |
| REPORT 来源与成长 | EV1/2/3/5/8；ADR 0025、0032 §3 | 原图与派生链可追溯；自身近看不改标直接外源，重复调用不增独立根。图像后继事件精确分类仍需批准，不在此编新 G/C 规则 |
| REPORT 资源不足记录 | 原则 8；ADR 0032 §1；V2-87 | 建议诊断为维护记录、真实行动结果另归自致；V2-87 不因类比自动扩容。后继发布可知能力状态，不新增心理通道 |
| REPORT 认知接续 | ADR 0026；ADR 0032 §§5、7；V2-117 | 局部等待不等于她选择停止或必须回落；按发生/观察时间保留真实后果，执行凭证另计；不复活 thinning 或旧主动间隔 |
| ROUTES 精度、模型、声音 | ADR 0012、0027、0031、0034；v2.0 §9.5–9.6 | 全部只是候选，不借显存不足跳过选型、能力/连续性或迁移要求，不暗换精度 |
| MEASUREMENTS 判据 | AGENTS §§5–6；现行验收 | 新尺寸、耗时、预算待实测与预先登记；既有门槛不改，文档检查不签发硬件或行为通过 |
