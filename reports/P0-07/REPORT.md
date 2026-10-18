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

## 2026-10-17 用户选择路线1：int4优化

用户明确选择保持600ms、仅优化int4，暂不再投入bf16/int8；保留显式forward包装、固定残差注入位置。任何编译、量化或内核替换之后都重跑零调制、注入误差、缓存失效探针。本段解除此前等待后续路线的状态，不代表验收通过。

主代理将4B/8B约18token/s作为固定开销假设的线索，不能据此排除所有算力/带宽影响。静态检查发现：每token创建a和重算Ua；prefill默认计算所有位置logits；eager注意力由多个小算子组成。安装的bitsandbytes 0.50.2在无梯度路径派发专用gemm_4bit，实际CUDA开销需profiler确认，不先假定完整反量化路径。

计划先做独立诊断，再依次比较SDPA、只保留末位置logits、请求内固定Ua缓存；只使用既有依赖。正式重测保持模型修订、NF4/bf16计算、TTS与渲染并载、同一二十字夹具、greedy、一次预热+20次独立KV、96token/30s预算和600ms。诊断profiler不进入正式计时，不删慢/失败样本。GPU开始前通知用户，runner在线期间仅准备代码。

依据：总方案§19要求优化后重跑路径；算子§13规定固定请求内m及缓存合同。PyTorch官方SDPA文档说明可按CUDA条件选择融合内核，是否实际采用和加速均以本机证据为准：https://docs.pytorch.org/docs/stable/generated/torch.nn.functional.scaled_dot_product_attention 。本机API以锁定torch2.10.0/transformers4.57.3源码核对。

另按用户要求，将AGENTS第9节和.codex/agents配置从P0-07拆出独立小PR先合并；此处模型优化不进入该PR。

### int4优化准备状态

已新增可选SDPA、末位置logits和请求内Ua缓存；默认仍保留eager参考路径。缓存首次计算仍在计时forward内，结束清除，不跨请求复用，不对m=0跳过注入。另新增profile_int4.py独立CPU/CUDA诊断，不用profiler耗时做验收。无新增依赖或编译器。Ruff和9文件Mypy通过，原始日志在optimization-preparation/；尚未执行本轮GPU探针/重测，不能声称有性能收益。

main的舞台代码自旧矩阵已有更新。规则PR合并并同步main后，先在同一版本重测int4参考路径，再比较SDPA/lean，避免跨版本渲染负载混淆。零调制探针对比相同后端/量化/末位置logits设置的未包装基准，不冒称跨内核逐位一致。后续保持所有原始失败和数值误差，生产容差仍不自行冻结。

## 2026-10-18 规则PR合并及新测量准备

独立PR #17已在最终head19c7ad9上获得Ubuntu/Windows全绿（run39045178103），main当时仍为ae803df；主代理按第6节核对并squash合并为9e12ba7。合并后的main CI run39045585344同样completed/success。规则已存在于main，P0-07与origin/main的AGENTS.md/.codex/agents差异为空。

P0-07同步main为c267c49；仅有AGENTS末尾空行冲突，采用main全文，其他目录正常合入。runner API offline/busy=false，本机Runner进程计数0；用户已获安静窗口通知。pnpm12.11.2使用install --frozen-lockfile --ignore-scripts补齐main依赖链接（4个包复用缓存，未更新锁）。随后CPU回归和GPU诊断按次序执行，尚不报告新测量结论。

### 优化准备的 CPU 回归（追加）

Luna/max 子代理按既定命令完成 pytest（4 passed）、Qwen3 tiny CPU 与 Phi3 tiny CPU 的 SDPA+lean 路径探针，三条命令均退出0。两份 JSON 的 zero_prefill_logits_max_abs_error、zero_decode_logits_max_abs_error、changed_vs_fresh_logits_max_abs_error 均为0.0，changed_cache_discarded 均为true。主代理核对字段及原始日志；证据在 optimization-preparation/pytest-v1.log、tiny-qwen-sdpa-lean.{log,json}、tiny-phi-sdpa-lean.{log,json}。这些 CPU 小夹具不证明真实 int4 GPU 的数值误差或性能，真实权重探针和并载测量另行执行。

### Qwen3-4B int4 eager 算子诊断

独占 GPU、runner 离线下，profile_int4.py 退出0；原始日志 optimization-preparation/profile-qwen4-eager-v1.log，原始 Chrome 轨迹 optimization-profile/qwen4-eager-v1/trace.json（53,312,249 bytes），聚合 profile.json。一次预填充加8次decode，轨迹包含21,186个CUDA kernel事件；cudaLaunchKernel调用21,033次，其self CPU合计约137.7ms；bitsandbytes::gemm_4bit调用2,268次。decode使用专用gemm_4bit_simt，预填充出现sm80内核，不能把当前实现描述成每层先展开完整bf16权重再GEMM。

这支持“小算子和派发开销值得优化”的方向，但不证明GPU算力/带宽完全无影响。聚合表里decode/prefill的device范围包含整个阶段，不能当作单个GPU内核、不能与子事件求和。profiler下decode约92ms，受采样扰动，不与旧18token/s直接比较，更不用于600ms验收。先在同一main版本重测eager参考，再测SDPA及请求内固定Ua/末位置logits选项。

### SDPA 与 lean 的同版本对照

Qwen3-4B int4，同一舞台版本/固定夹具/并载协议：eager p50=658.94605ms（opt-reference-qwen4-v1），SDPA=653.33925ms（opt-sdpa-qwen4-v1），SDPA+lean=655.67910ms（opt-sdpa-lean-qwen4-v1）。每组均20/20固定句匹配、20汉字、相同12token，全部路径/并载/渲染子进程退出0；零调制/注入/缓存比较误差0，缓存失效true。三组均未达到600ms，不把小差值认定为稳定收益。

SDPA profiler（optimization-profile/qwen4-sdpa-v1）实际出现324次 aten::_scaled_dot_product_attention_math，23,607个CUDA kernel、23,526次cudaLaunchKernel；不是融合attention。因此下一项只使用已有PyTorch的原生RMSNorm，针对eager下每forward约145次RMS归约的小算子链进行替换；保留权重、epsilon、原dtype中间舍入位置和显式注入。另记录内核替换前后logits差异，不能以同内核零调制为0掩盖跨内核差异。无新增依赖，依据 https://raw.githubusercontent.com/pytorch/pytorch/v2.10.0/aten/src/ATen/native/layer_norm.cpp 。目前仅准备代码，尚无原生RMS性能结论。

原生RMS准备首次Mypy失败已保留在 optimization-preparation/native-rms-check-v1-mypy.log：latency_probe.py 的 manifest JSON revision 动态Any使local()返回类型不明确。主代理新增字符串边界校验，未改测试或数值逻辑；修复后另存v2检查，不覆盖首次失败。

### 原生RMS首次真实权重结果及同时段参考

v2 Ruff、Mypy、pytest（4 passed）与Qwen/Phi tiny CPU均退出0。子代理额度中断后先核对已有日志，没有重复运行已完成CPU任务。恢复后的原生RMS并载路径位于 opt-native-rms-qwen4-v1，p50=538.8758499975665ms，20/20固定句，三子进程退出0；145个载体RMSNorm替换，TTS不变。零调制prefill/decode、注入误差、changed-vs-fresh、native-vs-original的5-token夹具比较均为0，changed_cache_discarded=true。

中断后额外测量同时间段eager参考v2（opt-reference-qwen4-v2）：p50=654.7183500006213ms，20/20固定句，误差0/缓存失效true。单项优化相对该参考约17.7%提速；首次满足延迟阈值。仍需全候选/8B优化矩阵、三语方向、显存与完整入口证据，不提前选型。所有原始日志保留。

原生RMS诊断轨迹补充：profile.json 中 aten::_fused_rms_norm 调用1,305次，对应实际CUDA kernel名称为 vectorized_layer_norm_kernel 的 RMS 模板实例（符号含 Lb1），不是字面包含rms的名称。kernel总数13,356，cudaLaunchKernel调用13,203，对比eager诊断21,186/21,033明显减少；不可将“按rms字符串搜索kernel为0”解释成未融合。诊断和完整原始轨迹位于 optimization-profile/qwen4-native-rms-v1/，profiler不参与正式验收计时。

## 2026-10-18 完整 int4 原生RMS复测

单命令 `workers/model-adapter/probe/.venv/Scripts/python.exe workers/model-adapter/probe/run_all.py --int4-only --native-rms --output reports/P0-07/opt-native-rms-all-v1` 已实际执行，UTC 04:26:03.5423967–04:34:19.3954781，退出0。四项矩阵的path/concurrent/render均0，三项方向进程均0，见 exit-codes.json 与 optimization-preparation/run-all-native-rms-v1.log。仍为eager、不lean，保留显式forward、层号、rank8、掩码、NF4和全部生成预算。无新增依赖。

| 模型 | 首句p50 ms | 固定句/20 | 整卡离散显存 MiB | 载体+TTS torch reserved峰值 bytes | 内核替换前后logits最大差 prefill/decode |
| --- | ---: | ---: | ---: | ---: | --- |
| Qwen3-4B | 546.9559 | 20 | 6636–6637 | 5263851520 | 0 / 0 |
| Qwen3-4B-Instruct-2507 | 540.7334 | 20 | 6634–6635 | 5261754368 | 0.234375 / 0.125 |
| Phi-4-mini-instruct | 593.8206 | 20 | 6826–6837 | 5463080960 | 0 / 0 |
| Qwen3-8B（对照） | 542.63375 | 20 | 9867 | 8797552640 | 0 / 0 |

四模型相同数值内核下的零调制prefill/decode误差、changed-vs-fresh误差都为0，缓存失效均true。注入测量三款4B均为0误差；8B预填充误差0.001953125、observed_l2=0.501953125，decode误差0。不能把相同内核的零调制为0写成所有内核替换前后相等，Instruct已明确观察到差异。

整卡MiB数据包括桌面其他进程，且为离散采样；torch reserved峰值不含Chromium/驱动全部开销。两者分别列出，不据此冒称Nemu精确连续峰值。9867MiB约9.64GiB（10.35GB），不是9.87GB；不能直接把整卡采样当作8B的Nemu归因。

### 三语人工阅读（主代理）

主代理已逐条读完新三模型各60条输出。两款Qwen三语均有措辞/语气词/标点/扩句变化，未见批量语法崩溃；Qwen3-4B中文仍夹英语，日文seed735的正调制出现不自然的第三人称措辞。Instruct整体措辞更规整，但其RMS内核差异尚不能忽略。Phi中文“迫不及忙”等不自然措辞存在于零/正两侧；日文seed734正调制语义偏离，其他若干句也不自然。基线庆祝场景已偏兴奋，不能从这些变化推出稳定、独立验证的兴奋方向；本轮仍是信息性观察，不签发M7结论。

当前结论：保持600ms的优化路线已产生可复现的达线配置，原“全部超过600ms”不再代表最新结果。一条命令的实际执行证据已补齐。完整P0-07仍为部分完成：原生RMS跨内核误差与生产数值容差、Nemu显存归因、方向观察的质量边界需在最终选型前处理。暂以Qwen3-4B作为进一步核验的优先候选（本夹具内核差异0、延迟有余量），不是正式选型，不修改候选状态为selected。不开P0-07 PR、不触发CI、不开始下一卡。

测量结束后检查Runner/Worker、Electron、probe Python、render_load/Playwright Node相关进程均为0，安静窗口结束。后续GPU测量另行通知，Windows CI需要runner时提前列明提交和作业。

方向机械核对补充：每模型每语言20条、10对；Qwen3-4B zh/en/ja token变化配对5/2/3，Instruct-2507为5/3/2，Phi为6/2/6。全部180条以EOS结束，EOS后token数均0。完整文本已经由主代理逐条核对，统计不替代上述语义阅读。轨迹压缩归档校验见 optimization-profile/ARCHIVE.md，原始JSON保留本机，提交gzip无损副本，解压SHA256一致。
