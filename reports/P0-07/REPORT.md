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

## 2026-10-16 用户补充约束与资产登记追加

按用户明确要求，config/models.lock.json 新增 p0_07_probe 候选登记，覆盖两款 4B、8B 对照与 TTS 0.6B 的仓库、固定修订、角色、许可证、来源和可分发标记；status 为 candidates-only，不表示已选定正式载体。同步扩展 models.lock.schema.json，使现有 additionalProperties=false 校验仍有效；AJV 校验通过，四条登记与探针 manifest 完全一致。此前“暂不修改 models.lock.json”的说明为历史状态，由本次用户指令取代。

Git 核查：probe/.cache/ 和 probe/.venv/ 被忽略，git ls-files 在两目录下均无条目；权重和下载缓存不提交。2026-10-16 本次检查 D 盘可用 146161319936 字节，C 盘可用 26023874560 字节。权重保存在 D 盘，不迁入仓库跟踪路径。

正式延迟/显存测量前先通知用户进入安静窗口。用户注销 runner 且不运行其他 GPU 任务；代理只运行测量所必需的载体、TTS 和渲染占位，不同时启动 Electron 或其他测试。当前尚未开始该测量窗口。下载不足、CPU 夹具和渲染负载就绪均不构成延迟/显存验收。

磁盘空间不足时先追加 reports/P0-07/BLOCKED.md，记录可用空间、需求与处理选项，不删除用户其他文件。当前没有进行任何磁盘清理。

## 2026-10-16 候选架构扩展与 TTS 使用边界

按用户要求重新核查 Hugging Face 官方仓库的当前元数据、固定修订配置与许可。核查日期为 2026-10-16（Australia/Perth）；这是本轮同级候选调查，不声称穷尽所有模型。原始元数据摘要见 preparation/candidate-survey-2026-10-16.json。

| 模型 | 核查结果 | 本轮处理 |
| --- | --- | --- |
| [microsoft/Phi-4-mini-instruct](https://huggingface.co/microsoft/Phi-4-mini-instruct) | 3.8B；MIT；官方支持语言含 zh/en/ja；model_type=phi3，Phi3ForCausalLM；修订 cfbefacb99257ffa30c83adab238a50856ac3083 | 新增为第三个 4B 级候选，提供不同于 Qwen3 的实现架构；全部精度、真实通路、三语与并载检查待执行 |
| [Qwen/Qwen3.5-4B](https://huggingface.co/Qwen/Qwen3.5-4B) | 确有更新的 4B 权重；Apache-2.0；官方 4B 配置说明为 Gated DeltaNet 与 Gated Attention 交替的混合结构，不能当作原 Qwen3；修订 851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a | 调查记录保留；本轮三款候选采用两个已准备的 Qwen3 与 Phi。当前为 TTS 固定的 Transformers 4.57.3 不含 qwen3_5 配置映射，支持需独立新版环境/进程及混合状态缓存适配；这是工程范围选择，不是性能淘汰结论 |
| [google/gemma-3-4b-it](https://huggingface.co/google/gemma-3-4b-it) | 官方标注 Gemma 专用许可，下载需接受条款；未经认证读取配置返回 401 | 不满足本轮要求的 MIT/Apache 等宽松许可筛选；未尝试绕过门控，不纳入 |

Phi 的官方模型卡明确列出 Chinese、English、Japanese；该声明仅满足候选筛选，不代替三语实测。其固定修订 LICENSE 已读取并以 SHA-256 登记，原许可文本保留在 probe/licenses/Phi-4-mini-instruct-MIT.txt。这里的 Phi-4-mini 不是新于 Qwen3.5 的模型；新增依据是不同架构、同量级、宽松许可和三语支持。当前环境 CONFIG_MAPPING 实查 phi3=True、qwen3_5=False。现有 path_probe.py 仍仅支持 Qwen3，Phi 需要另行核对 forward 和缓存后适配，不会直接套用 Qwen3 包装或声称已测。

manifest.json 与 config/models.lock.json 已同步为三款 4B 级候选、一个 8B 对照和一个 TTS 负载，共五条，schema 校验与两份清单一致性检查通过。当前没有最终载体选型。

**TTS 边界：Qwen3-TTS 0.6B Base 仅用于 P0-07 并载显存测量；正式音色和变体由 P4-03 决定，不构成选型结论。** 此说明已写入两份登记的 TTS role，先前“concurrent TTS”是历史简写，不授予正式音色选择权限。

已有权重分段下载继续。Phi 配置准备已启动，Phi 权重安排在当前下载进程结束后顺序执行，避免叠加下载并发；缓存保持 Git 忽略。开始真实模型/性能测量前另行通知用户，本轮清单更新没有启动测量窗口。

## 2026-10-17 真实权重通路与并载探针推进

- 新增 Phi3 decoder 显式 forward 分支，保留 attention/MLP 两处原始 dropout 运算；注入位置仍为 attention 残差相加之后、下一归一化之前。来源 Transformers 4.57.3，Apache-2.0；没有新增依赖。
- `preparation/tiny-phi-cpu-path-v1.json`：Phi CPU 结构夹具的零调制 prefill/decode logits 最大差均为 0，改变调制后 KV 被丢弃，与重新计算 logits 最大差为 0。这不替代 Phi 真实权重验收。
- `preparation/qwen4b-int4-path-v1.json`：RTX 3080 上 Qwen3-4B int4 实测的上述三个 logits 最大差均为 0，KV 失效成立。注入原始向量和浮点误差保存于该 JSON；生产容差仍未据此宣告冻结。
- 新增 `latency_probe.py`：本地离线加载真实 TTS Base 全部参数到 GPU；greedy、eager、独立 KV、一次预热后 20 次测量，最多 96 token / 30 秒。首句测到句末标点，TTFT 单列，逐条保留实际汉字数，不筛除不合二十字的输出。全 GPU 显存采样包含其他桌面分配，不等同于精确 Nemu 归因，也不声称采样是全 GPU 峰值。
- 当前缓存核查仅找到四个 Qwen 模型的完整权重校验清单；Phi 原下载最后记录为 TLS 失败，未找到两片完整权重，已询问是否有其他下载位置。未改候选登记、未以缺失权重判定模型性能失败。
- 本轮未打开 PR、未触发 CI；冻结规格和已有裁决不变。

### 第一组并载数据（2026-10-17，非完成验收）

`preparation/qwen4b-int4-concurrent-v1.json` 包含一次预热和 20 次正式样本；真实 TTS 参数全部位于 cuda:0，舞台硬件渲染证据为 `preparation/hardware-render-v3.json`。Qwen3-4B int4 首句 p50=800.344800001767 ms。每条实际输出均为 14 个汉字，故不能作为二十字回答验收，也不能据此签发载体选择。全 GPU 离散显存样本 6432–6452 MiB，不是精确 Nemu 峰值。该组已结束，渲染进程退出码 0。后续需固定二十字文本夹具并继续所有配置、方向检查和最终归因。

### 本轮脚本验证与复现入口

- `uv run --project workers/model-adapter/probe --locked pytest workers/model-adapter/probe/test_injection.py -q`：4 passed（包含 Qwen3 / Phi3 两分支）。
- `uv run --project workers/model-adapter/probe --locked ruff check workers/model-adapter/probe`：通过。
- `uv run --project workers/model-adapter/probe --locked mypy workers/model-adapter/probe/injection.py workers/model-adapter/probe/path_probe.py workers/model-adapter/probe/latency_probe.py workers/model-adapter/probe/run_matrix.py workers/model-adapter/probe/test_injection.py`：5 个源文件通过。
- 新增固定二十字夹具（`--fixed-twenty`）：今天的努力值得肯定请带着信心继续向前走吧。只用于受控生成性能观察，不替代自由回答/方向流畅度检查；逐条记录是否准确输出，不能强制截断输出以伪造合格。
- 矩阵入口：`uv run --project workers/model-adapter/probe --locked python workers/model-adapter/probe/run_matrix.py --output reports/P0-07/matrix-<新编号>`。每个配置分别启动进程，保留失败退出码和日志，渲染在 finally 中关闭。该入口目前只覆盖通路/并载性能矩阵，尚未实现方向研究的一条命令全量复现，因此任务未完成。

## 2026-10-17 Phi 权重续传完成

用户确认此前尚未下载完整，授权继续续传。后台下载现已成功结束：两片权重大小与固定修订元数据一致，下载器的整片 SHA-256 校验均通过；证据见 `preparation/phi-weights-verified-20261017.json`。至此五个模型都有完整权重校验清单。此次仅核查下载，不启动 GPU 测量；下一测量窗口前另行通知。

## 2026-10-17 完整矩阵与脚本检查（追加）

十项并载矩阵均执行结束，各项 path/concurrent/render 退出码均为 0；原始证据见 matrix-v1/，汇总见 matrix-v1-summary.json 和 benchmark.md。三款 4B 级候选各测 bf16/int8/int4，8B 对照测 int4。每项 20 个非预热样本，Qwen3-4B bf16 与 Instruct-2507 bf16 各有一次未完成二十字，其余八项完成 20/20；失败样本保留在统计内。

主代理判定：本轮没有可选定载体。三款 4B 级 int4 的 p50 分别为 Qwen3-4B 662.2309000013047 ms、Instruct-2507 666.2554500071565 ms、Phi-4-mini 673.1948500018916 ms；8B int4 为 658.8728499918943 ms。十项均超过 600 ms，退出码 0 不表示验收通过。

十项零调制 prefill/decode 与改变调制后 fresh 对照最大 logits 差均为 0，缓存失效均成立。8B int4 注入最大绝对误差 0.001953125，其余九项为 0。生产容差未自行冻结，保持 measured-not-accepted。

显存边界：PyTorch 峰值仅覆盖本进程分配器；全 GPU 离散采样包括桌面，不能作为精确 Nemu 峰值归因。Qwen bf16 的 WDDM 现场读数显示共享内存使用，转录与限制见 matrix-v1/wddm-live-observation.json。int8 时间包含高频 dtype 转换日志开销，不全部归因于模型计算。渲染启动时检查硬件和帧推进，未记录结束帧计数，不能证明全时段帧率。

Luna/max 子代理按固定命令执行：Ruff 通过，Mypy 8 个文件通过，pytest 4 passed，汇总脚本退出码 0。原始命令、时间和 stdout/stderr 见 preparation/checks-v2-{ruff,mypy,pytest,summary}.log。主代理读取汇总并核对任务阈值，未委派验收判断。

run_all.py 已串接全矩阵和三款方向检查，命令见 probe README；尚未从头执行该全量入口，不宣称整条命令已验证。方向研究在通知后的独占 GPU 窗口进行，结果随后追加。没有创建 PR 或触发 CI。

上述 WDDM 证据的完整相对路径为 matrix-v1/Qwen--Qwen3-4B-bf16/wddm-live-observation.json。

## 2026-10-17 方向研究前两项的主代理阅读记录

已逐条阅读两款 Qwen 的各 60 条输出（每语言 10 个匹配 seed × 零/正调制），原始文本、token、U 与合成校准提示位于 direction-v1/。两款均有三语文本变化，但这不等于稳定的预期语义方向：Qwen3-4B 中文有语气词/标点变化、英文局部扩句、日文祝贺扩句；Instruct-2507 也多为祝贺措辞或表情变化。基线提示本身就是庆祝场景，零调制已偏兴奋，存在天花板效应。部分中文/日文输出夹入英文。主代理不据此认证稳定的兴奋方向或无流畅度损失，也不签发 M7 结论。

第二项模型已生成完整 JSON，但外层 PowerShell 写 meta 尾部发生 AppendAllLines overload 错误；模型生成和日志封装失败分开记录。保留原产物，不重跑覆盖。Phi 结果随后追加。

## 2026-10-17 Phi 方向停止标记缺陷（追加诊断）

主代理逐条阅读 Phi v1 的 60 条输出，发现回答后继续生成无关提示。固定修订 generation_config.json 的 eos_token_id=[200020,199999]，tokenizer.eos_token_id 仅为199999；手写循环遗漏200020。按角色名启动的 Luna 子代理只读统计证实：三语言各20/20在首个EOS后仍有token（原始核查见 ../agent-delegation/2026-10-17/native-role-trial.log）。因此 v1 的后续跑题不能直接归因于模型或注入；不据此作负面选型。

主代理新增 stop_tokens.py，在方向和延迟探针中读取完整 model.generation_config.eos_token_id，记录实际停止集合；不改阈值、注入、seed、采样或预算。v1 原始数据不覆盖，CPU检查后另存 Phi 复测。现有延迟矩阵是否遇到该token另核对，不因修复而冒称旧数据在新版脚本上生成。

### EOS 修复后的静态检查

初次 Mypy 检查失败（checks-v3-mypy.log）：nn.Module 的动态 generation_config 属性被类型系统视为 Tensor | Module。主代理改为读取固定修订目录中的 generation_config.json，同一来源的完整 EOS 列表，避免不安全的动态属性推断。修复后 checks-v4-ruff.log 全通过，checks-v4-mypy.log 为9个文件通过，checks-v4-pytest.log 为4 passed，checks-v4-eos.log 验证单EOS/多EOS及无效输入拒绝，全部退出0。v3失败保留，未跳过或改测试。

### 方向 v1 计数和延迟数据影响范围

| 模型 | zh / en / ja 有 token 变化的配对数（每语言10对） | U 算子范数 |
| --- | --- | --- |
| Qwen3-4B | 5 / 4 / 4 | 0.99899999960289 |
| Qwen3-4B-Instruct-2507 | 5 / 3 / 1 | 0.998999995734334 |
| Phi-4-mini v1（停止条件缺陷，非有效流畅度结论） | 8 / 6 / 5 | 0.99899999904788 |

Luna只读计数来自各JSON的token_ids和u_operator_norm；主代理已读全部180条文本。Qwen各语言均20条eos；Phi v1的token_limit计数zh16/en7/ja8。第二款Qwen模型子进程退出码因包装写meta失败未保留，记为未知，不能从JSON完整推断为0；第一/第三项模型退出码0。

对十项 concurrent.json 的全部21条样本（含预热）检查，200020出现次数全部为0。旧延迟矩阵未遇到此漏判EOS，数值仍按原数据记录；不把它说成修复后的重新测量。原始文件共约32MB，其中Instruct-2507 int8/concurrent.log约18MB，主要为重复告警，保留完整日志。只读文本扫描未发现签名URL/凭据赋值样式；权重仍在忽略路径。

## 2026-10-17 Phi 方向 v2 与本轮结论

修复EOS后单独复测Phi，证据 direction-v2/microsoft--Phi-4-mini-instruct.json 及同名前缀日志。Python退出0，60条输出均以eos结束，主代理逐条读完：此前回答结束后的无关指令续写已消失。中文仍有“迫不及忙”等不自然词语、日文仍有“増強”等不合场景用语，而且零/正调制双方都有；不能把这些基线质量问题归因于注入。观察到部分扩句/措辞变化，尚不证明稳定朝“兴奋”移动，更不代表跨场景或M7结论。

本轮新增的规则配置与真实子代理试用已完成；P0-07仍为部分完成。十项矩阵和三款三语方向观察已有原始数据，但没有满足全部选择条件的候选。保持600ms与9GB条件、冻结规格及已裁决事项不变，不把候选写成正式选型。全量入口尚未从头端到端执行，生产容差/精确峰值归因仍待闭合；不宣称任务验收完成。

下一步：按任务书“都不满足”分支等待用户选择BLOCKED.md中的后续路线。建议先评估现有实现优化并保持验收线；不擅自更换架构、增加硬件或将模式B当作严格载体通过。当前仅推送任务分支供审阅，不开PR、不触发CI、不启动下一张卡。需要Windows CI时提前明确通知runner任务与提交。

Phi v2 机械核对补充：zh/en/ja各20条、各10对，无缺对/重复；token不同对数为7/5/5，首个EOS均为200020，60条均无EOS后token。统计见 direction-v2/microsoft--Phi-4-mini-instruct.analysis-meta.json。子进程已退出，测量窗口结束。
