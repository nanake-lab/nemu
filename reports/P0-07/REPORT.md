# P0-07 载体探针 — 开工记录

## 结果

已领取，尚未完成实现或测量。2026-10-16 从 main d93f97ce1945b800a5e5c3d5a00713fa298e5ef2 创建 task/P0-07-carrier-probe。

## 依赖与目录占用

- 任务依赖 P0-01：PR #1 已合并。
- 排序前置 P5-01：PR #4 已合并，最终头双平台 CI 通过。
- 检查打开 PR（无）、本地工作树及已知 Codex 线程，未发现 workers/model-adapter/probe/ 被另一张卡占用；Antigravity 的工作树为 D:/Dev/nemu-antigravity，不修改其文件。
- 实现目录严格限制为 workers/model-adapter/probe/；报告追加到 reports/P0-07/。未选定模型前不写入正式载体锁条目。

## 已读要求

tasks/P0-foundation.md 的 P0-07；《核心算子》§§12–14；总方案 §2、§19；echo-core-work 技能。保留固定残差位置、显式 forward 包装、固定低秩 U 和掩码、零调制基准、实际注入差值与缓存失效边界。不得用文本提示冒充严格注入，方向检查不冒充 M7 行为验收。

## 只读环境检查

| 项目 | 实测结果 | 方法 |
| --- | --- | --- |
| GPU | NVIDIA GeForce RTX 3080，10240 MiB，驱动 610.88 | nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader |
| D 盘可用空间 | 146521448448 字节（检查时快照） | Get-Volume -DriveLetter D |
| model-adapter 的 Python 环境 | torch、transformers、accelerate、bitsandbytes、qwen-tts 均未安装 | uv run --project workers/model-adapter --locked python；importlib.metadata 检查分发元数据 |

上述仅为环境信息，未加载模型，未分配模型 GPU 显存，未运行 Electron、Playwright 或视觉测试。缺少探针依赖是待准备工作，不是模型可行性失败。

## 验收

模型候选、许可证与修订 SHA 尚待核验；没有选择最终载体。零调制、注入误差、缓存失效、三种精度、TTS/渲染并载、显存、首句延迟和三语方向检查均为 **not_run**。没有性能数字或验收通过结论。冻结规格、ADR 0006、C01/C02 均不变。

## 新增依赖

无，本轮未安装模型运行依赖或下载权重。

## 下一步

核验 2–3 个 4B 级及 1 个 8B 级候选的来源、许可和不可变修订；准备隔离的探针环境与可复现脚本。runner 在线期间不同时运行本机 Electron、Playwright、视觉测试；并载 GPU 测量待 runner 空闲且用户关闭/注销后开展。近期不为本开工记录触发新 CI；完成 P5-01 合并后的 main CI 后将明确通知用户可关闭 runner。

## 2026-10-16 docs/test 独立支线合并及探针准备追加

用户授权的独立 PR #8 已在最终提交 2fc036d 上通过 Ubuntu、Windows（run 38835329796），确认包含当时最新 main d93f97c，报告无阻塞项，转 ready 后 squash 合并为 ab282dc。P0-07 分支已同步此 main。E12 当前状态为“用户确认不可得（2026-10-14 决定）”，不重建缺失内容；C01、ADR 0006 和 NO-GROWTH 不变。

探针隔离环境安装完成，锁定 torch 2.10.0+cu128、torchaudio 2.10.0+cu128、Transformers 4.57.3、accelerate 1.12.0、bitsandbytes 0.50.2、qwen-tts 0.1.1。仅为探针环境，未修改生产 worker 环境。完整依赖清单与许可仍待整理，安装成功不代表 GPU 模型通路成功。

新增 prepare.py：按候选清单的固定修订下载文件并生成 SHA-256 清单，默认只准备配置、tokenizer 和许可；权重需显式传入 --weights。脚本 ruff、mypy --strict 通过；配置准备正在运行。尚无模型测量、选型或验收通过结论。

Windows runner 已由用户重新上线并执行 PR #8，随后承担合并后的 main CI（run 38836093496）。期间没有在本机启动 Electron、Playwright 或视觉测试。今后凡 PR/main 会触发 Windows 作业，直接说明 runner 需求；不能把尚未开始执行判断为不需要 runner。GPU/渲染测量须待用户关闭/注销 runner 后开展。

配置准备完成追加：四个固定修订的配置、tokenizer、许可文件下载均成功，逐文件 SHA-256 清单已保存于 preparation/。尚未下载权重，未加载模型；模型通路、显存和延迟仍为 not_run。

main CI 完成追加：run 38836093496（ab282dc）Ubuntu、Windows 均成功。已通知用户本轮 runner 作业结束，可以关闭并注销；本机渲染与 GPU 并载测量仍等待 runner 注销。

## 2026-10-16 通路实现与负载准备追加

用户已确认注销 runner；随后检查未发现 Runner.Listener / Runner.Worker，才进行本机硬件渲染。当前仅恢复 P0-07，不创建或触发新的 PR/Windows 作业。

新增显式 forward 包装、固定请求调制、包含模型/tokenizer/prompt/carrier/m/mask/数值配置的缓存键，以及实际 logits/激活观测脚本。默认不跨请求复用缓存；更换 m 时先清空缓存再前向。使用 torch inference_mode、冻结模型参数，不接入任何权威存储或生产运行路径。

CPU 机制测试 3 项通过；完整小型 Qwen3 随机夹具的零调制 prefill/decode logits 最大差均为 0，m 改变后的新计算与 fresh prefill 差为 0。非零注入保存 before/after/delta，误差见 benchmark.md。这是结构/实现验证，不是真实 4B/8B 载体验收；生产容差仍未冻结。

渲染负载复用现有 sample.vrm。硬件渲染证据确认 RTX 3080 / D3D11，帧计数在观察窗口从 1 增至 13；浏览器和 Vite 服务在 stop-file 后正常退出。未修改舞台代码、视觉基准或角色资产。此结果不代表并载显存、帧率、首句延迟或语言方向验收。

### 依赖登记补充（仅隔离探针环境）

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| torch | 2.10.0+cu128 | BSD-3-Clause（包元数据） | 显式模型前向与张量测量 |
| torchaudio | 2.10.0+cu128 | BSD-2-Clause（随包 LICENSE） | TTS 音频工具 |
| transformers | 4.57.3 | Apache-2.0 | 固定 Qwen3 实现 |
| accelerate | 1.12.0 | Apache-2.0 | 模型加载 |
| bitsandbytes | 0.50.2 | MIT | 8/4 bit 候选量化 |
| qwen-tts | 0.1.1 | Apache-2.0 | 0.6B TTS 并载准备 |

这些版本未进入生产 worker 依赖或应用分发。完整 uv.lock 保存传递依赖；不据此声称所有随包二进制或 CUDA 组件可重新分发。包装中借鉴的 Transformers forward 顺序保留来源版权及完整 Apache-2.0 许可。

### 下载状态与下一步

Hugging Face Python 客户端下载权重时发生 SSL unexpected EOF。系统 curl 的固定修订 Range 请求成功；已实现四路并发、8 MiB 分段续传，逐段校验 Content-Range 和长度，最后校验仓库 LFS SHA-256 与大小。当前下载仍在进行，未获得完整模型验收结果。完整权重未就绪前不运行真实模型测试，不把网络失败认作模型失败。

benchmark.md 和 decision.md 明确保留全部未执行项。下一步是在首个候选权重完整且哈希通过后执行真实 GPU 通路，继而完成其余候选、TTS/渲染并载和三语检查。尚未选型，config/models.lock.json 不变。暂不需要 Windows runner；未来若创建 PR 会明确通知其 Windows CI 需要 runner，并停止本机 GPU/渲染工作。
