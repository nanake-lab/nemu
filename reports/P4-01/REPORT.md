# P4-01 语音识别与语言识别 — 报告

## 结果
当前（2026-10-23，ADR 0017 合并后）：本卡功能与本机验收完成。沿用原参数、原 60 条有效测量，VAD 中位 320.16 ms ≤350 ms、开口到 voice.bargeIn 中位 285.26 ms ≤300 ms，均通过；持续 200 ms 门限不变。提前误触发 1/60（1.67%），按 ADR 只报告。既有识别重计分、语言判定和静音 CPU 也通过。旧目标下 FAIL 与原始记录保留，当前结论见 latency-acceptance-adr0017.json。最新 main 合并后回归与 CI 状态见文末；本卡中难度按当前规则无需审查，由用户合并，不取下一卡。

此前结果（原记录保留）：
部分完成：用户已验证全部转写，60 条用户录音已登记，完整 150 条识别评测通过；SenseVoice 开发许可方案已决定。已生成 60/60 独立模型参考（全量上下文稳定性复核仅 39/60 满足门槛，参考可靠性未通过）；执行方已完成 60 条完整实时延迟测量，但 VAD 中位 320.1 ms、59 条打断超限，延迟验收未通过，不再等待用户提供或确认录音。不得据此宣称整卡验收完成。

## 同步与认领
- 从 `origin/main` 创建 `task/P4-01-asr`，独立工作树 `C:\Users\nanake\.codex\worktrees\p4-01-asr\nemu`。
- 草稿 PR：https://github.com/nanake-lab/nemu/pull/44，已标记 `needs-user`。
- P2-03 的 `D:\Dev\nemu` 工作区、未提交文件和资产没有改动。
- 用户指示不合并、不取下一张卡；难度中，按 2026-10-23 用户指示和当前 AGENTS 第 6 节不需要审查，由用户合并。

## 交付物
- `packages/voice/web/mic.ts`、`mic-worklet.ts`：音频独占采集权限、回声消除/降噪/自动增益、16 kHz 单声道、20 ms Int16 PCM；关闭时释放设备与 AudioContext。
- `packages/voice/src/asr/`：SenseVoice int8、Silero VAD、AsrProvider 与未接通的云端桩、语言映射标记、播放中的持续语音打断、vadEnd/asrDone。
- `apps/desktop/src/voice/`：utilityProcess 入口、模型哈希校验、启动与退出清理、最多三次退避重启；主进程转发控制、麦克风和音频通道，发布经过 contracts 校验的 AppBus 事件。
- 只发布 CONTRACTS 4.1 的 `voice.recognized`；没有实现 Echo Core 观察事件、网关或权威存储接入。
- `config/models.lock.json`、schema 和 ASR `manifest.json`：只追加 ASR 条目。
- `public-data.lock.json`：FLEURS 固定修订、测试集原顺序前 30 条/语言、参考稿、样本哈希与源文件哈希。没有按 ASR 效果挑选或排除样本。
- `READING.md`：中英日各 20 句、文件命名与文字稿约定。仓库外已建立对应文字稿目录。

## 命令
```powershell
pnpm asr:models       # 本机下载并校验三份模型文件；CI 拒绝下载
pnpm asr:public       # 本机固定 FLEURS 数据下载并校验；需要 uv
pnpm asr:user         # 初始化/登记用户录音与相邻文字稿，不覆盖已有文字稿
pnpm eval:asr         # 一条命令输出三语 CER/WER、语言准确率到 evaluation.json；需要 ffmpeg
pnpm measure:asr      # 独占窗口内测真实 utilityProcess 静音 CPU 60 秒
pnpm asr:latency      # 使用仓库外 timing-references.json 的独立起止参考，实时 20 ms 输入测 VAD/打断
```
模型：`D:\NemuData\models\asr`，可用 `NEMU_ASR_MODELS` 覆盖。
音频：`D:\NemuData\asr-evaluation`，可用 `NEMU_ASR_DATA` 覆盖。用户录音、用户文字稿、user.json、FLEURS 音频和 Parquet 均在仓库外。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| 模型下载/哈希 | 三文件已校验；SenseVoice int8 239233841 字节 | lock、manifest、model-hashes.json |
| 公开数据准备 | 90 条，三语各 30，三个 Parquet SHA-256 与官方 LFS 元数据一致 | public-download.log、public-data.lock.json |
| P4-01 单测 / 整仓 Vitest | 整仓 40 文件、494/494，通过（最新 main 同步后） | hybrid-v2-after-final-sync-tests.log |
| TypeScript 类型检查 | 通过（最新 main 同步后） | hybrid-v2-after-final-sync-typecheck.log |
| ESLint | 通过（最新 main 同步后） | hybrid-v2-after-final-sync-lint.log |
| Python mypy --strict / 辅助测试 | 本轮 3 文件通过，完整辅助测试 18 项通过；历史工具检查另外保留 | hybrid-v2-final-mypy.log、hybrid-v2-final-python-tests.log |
| workspace build | 同步 main 并补齐桌面工作区依赖后通过，路径函数已打入主进程包 | final-resume-build.log |
| 完整 Electron E2E（构建后 Playwright） | 4 通过，专门静音 CPU 测量 1 项按设计跳过且随后单独实测通过；已正确初始化 run.json，含 OS 快照，整体退出 0 | resume-e2e.log、resume-e2e-*/ |
| 真实麦克风/utility 通道测试 | 通过：50 packets/s、异步关闭，未读取真实用户麦克风，使用 Chromium fake device | voice-e2e-v5.log、e2e-v5/ |
| Electron 首次测试 | 失败，原始日志保留。与 CI 窗口重叠且临时目录前缀不符合隔离约束；不能据失败推断单一原因 | voice-e2e.log、e2e/ |
| Electron 第二次测试 | 安静窗口检查把 PowerShell 空列表 null 当成占用，已修正；等待最终复跑 | voice-e2e-v2.log、e2e-v2/ |
| 各语言 50 条/150 条语言准确率 | 人工注记修正后的既有推断重计分：zh CER 6.29%、en WER 6.69%、ja CER 7.59%，LID 149/150=99.33%，通过；非新推断 | evaluation-human-reference-v2.json、annotation-text-rescore.log |
| 公开集 CER/WER 与 LID | 中文 74/989 = 7.48%，英文 54/664 = 8.13%，日文 126/1438 = 8.76%；LID 90/90 = 100% | public90-before-user-verification.json、public-evaluation.log |
| 独立语音起止参考 | 当前冻结 38 自动＋22 人工；原抽检 4/5 在 80 ms 内，ja-013 用户确认人工起点，原失败保留；自动其余点未经逐条人工校准 | hybrid-reference-registration-v2.json、MANUAL-AUDIT-RESULT.md |
| VAD 中位延迟 ≤350 ms（ADR 0017） | **PASS：320.16 ms**；有效全 60 条，原始旧目标 FAIL 保留 | latency-acceptance-adr0017.json、latency-attempt-2026-10-22T09-39-20-062Z.json |
| 打断中位延迟 ≤300 ms（ADR 0017） | **PASS：285.26 ms**；提前误触发 1/60=1.67%，只报告；持续 200 ms 不变 | latency-acceptance-adr0017.json、FORMAL-LATENCY-HYBRID-V2.md |
| utilityProcess 静音 CPU ≤2% | 迁移后复测 60.73 秒；平均 0.11191%、峰值 0.14086%，通过。此前结果另存，未覆盖历史证据 | idle-cpu.json、resume-idle.log、measurement-*/ |

CER/WER 采用总编辑距离/总参考单位数；NFKC、小写、去 Unicode 标点与符号，中文日文逐 Unicode 字符、英文按空白分词。没有繁简替换、数字语义替换或删除失败样本。公开集和用户集计数分别记录；不足每语 30+20 时状态固定为“待用户录音”，不会返回完整 PASS。


### 故障与修正记录
- `e2e-v3`：最终打包的类变为匿名 class，原先把 class.toString() 当声明拼入 Worklet 会造成无类名声明的语法错误。改为显式 `const PcmFramer = <class expression>`；`e2e-v5` 已实测连续 PCM 帧与关闭设备。
- `e2e-v4`：PCM 已正常输出，但本卡新增测试错误地把示例角色 `PROFILE_FETCH_FAILED` 也当成语音失败。最终测试只断言 MIC_/ASR_ 错误为零；全部 stage.error 仍保存到 `e2e-v5/voice-stage-errors.json`，没有修复或隐藏该角色路径问题。
- 首次 Python urllib 下载遇到 SSL EOF；原报错留在工具执行记录。下载器改用 curl 的有限重试并仍执行完整 SHA-256，三个源 Parquet 和 90 个样本最终全部校验成功。
- 校验子代理通过 routine_reporter 角色调用，按要求读取项目 TOML；只运行 lint/typecheck/Vitest，无修改或验收裁决。没有把它表述成已验证项目 TOML 自动发现能力。

## 规格对照
| 条款 | 实现 | 验证 |
| --- | --- | --- |
| CONTRACTS 2.0 麦克风 16 kHz/20 ms/seq | microphoneMessageSchema、MicController、VoiceRuntime | 重采样 16/44.1/48 kHz、序号跳变、非法输入测试 |
| CONTRACTS 2.1 stage.mic.set | 主进程同时发送 utility 与舞台；默认关闭 | PTT 尾句、关闭/权限竞态测试；最终 Electron 待测 |
| CONTRACTS 2.2 音频播放事件 | utility 追踪 stage.speech.started/done | 打断测试 |
| CONTRACTS 2.3、4.1 voice.* | contracts 类型和 zod 校验，不修改契约 | runtime.test.ts |
| CONTRACTS 5.4 AsrProvider | SenseVoiceProvider、CloudAsrProvider 桩 | provider 与 runtime 测试 |
| 网关尚未完成 | 只到 AppBus | 本卡没有核心接入 |

## 新增依赖、模型和数据许可
| 名称 | 版本/修订 | 许可证 | 用途 |
| --- | --- | --- | --- |
| sherpa-onnx-node / sherpa-onnx-win-x64 | 1.13.8 | Apache-2.0 | 本地 VAD/ASR，CPU provider |
| zod / @nemu/contracts | 4.6.7 / workspace | 已有依赖 | 边界校验与权威类型 |
| tsx | 4.21.0 | MIT | 开发/评测 CLI，不是应用运行时依赖 |
| pyarrow | 23.0.1 | Apache-2.0 | 本机公开数据 Parquet 读取；PEP 723 固定版本 |
| SenseVoice int8 + tokens | 2365baeacb507f821a0c8120fcee3d484dba7a07 | custom:FunASR-Model-1.1 | 已决定：开发使用、用户自行下载，不随应用分发，P7 再评估；redistributable=false；原阻塞记录见下文 |
| Silero VAD | 1e261b036686cd0017d500ee96acd1c4ba572a9d | MIT | 模型与源许可已登记 |
| Google FLEURS 中/英/日 | Parquet 168de341b3db6859a9bac1c50a2ef5e3b47647e0 | CC-BY-4.0 | 公开测试数据；音频不入库，保留 Google/FLEURS 归属与来源 |

许可来源：
- sherpa 包元数据与官方源码：https://github.com/k2-fsa/sherpa-onnx
- SenseVoice 转换文件的 LICENSE：https://huggingface.co/csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/blob/2365baeacb507f821a0c8120fcee3d484dba7a07/LICENSE
- 原权重卡：https://huggingface.co/FunAudioLLM/SenseVoiceSmall
- FunASR 模型协议：https://github.com/modelscope/FunASR/blob/main/MODEL_LICENSE
- Silero：https://github.com/snakers4/silero-vad/blob/1e261b036686cd0017d500ee96acd1c4ba572a9d/LICENSE
- 数据集：https://huggingface.co/datasets/google/fleurs （FLEURS: Few-shot Learning Evaluation of Universal Representations of Speech, Google, 2022）

SenseVoice 源码的 MIT 不能替代权重许可证；转换仓库的 LICENSE 转向 FunASR 模型协议。协议含归属、模型名、行为与自动终止等条款，因此没有把它标成 MIT/Apache。原先按用户指示加 needs-user 等待决定；2026-10-21 用户已决定继续开发使用、用户自行下载且不随应用分发，正式发布前 P7 再评估。needs-user 按用户明确指示继续保留至整卡完成。

## 待用户确认
- [x] （已决定，2026-10-21）SenseVoice 现阶段继续用于开发；将来不随应用打包，由每位用户自行下载模型；正式发布前（P7）重新评估许可。模型登记保持 `redistributable=false`，注明“用户自行下载，不随应用分发”，保留 FunASR 要求的出处与模型名。
  - 原记录（保留作历史，已由上述决定取代，不再作为当前许可阻塞）：（阻塞）SenseVoice 自定义模型许可下是否同意本项目继续使用及未来分发方案。
- [x] （已验证，2026-10-21）用户已确认机器转写正常无误；三语各 20 条已登记，完整 150 条识别评测通过，不再等待用户。
- [ ] （执行方验收阻塞）独立起止时标与完整实时结果已齐；VAD／打断延迟未达标，参考质量存在未解疑点；不需要用户再次核对录音。CI 以最终提交为准。
- [ ] （不阻塞）可选 Nemu 唤醒词默认关闭。当前实现为常开模式下识别结果的 Nemu 文本门控，不宣称专用声学关键词模型或唤醒时延验收。

## 后续建议（不在本任务范围内）
- Echo Core 网关把 voice.recognized 转为 observation.user_utterance，留后续卡。
- P4-06 接真实语音输出时复核 started/done 生命周期和端到端打断淡出。
- P4-02 也在独立工作树对 apps/desktop 做集成；合并时按规则保留双方，不能覆盖另一张卡的启动/守护改动。

## 下一步
未完成：继续独立参考稳定性 QA 与 VAD／打断延迟优化。PR #44 保持草稿与 needs-user；达成全部验收后再转可审查、交 Claude 审查。不合并、不取下一张卡。无需用户重录、重验文字或重新决定 SenseVoice 开发许可。

## 2026-10-21 继续施工（当前状态取代前文的历史下一步）
- PR #44 已按用户要求转回草稿，`needs-user` 保留；**整张卡的验收结果齐全并重跑检查之前，不转可审查、不请求提前审查**。
- 已 fetch 并以 merge 同步最新 origin/main；本轮必读规则和任务卡未变，保留 P2-03 的根脚本追加。
- 已实际运行 `pnpm asr:user`。本机目录 `C:\Users\nanake\AppData\Local\Nemu\asr-evaluation\user\` 下 zh/en/ja 各 20 个同名 `.txt`，录音仍为 0/60；原文字稿未覆盖。用户录完通知后再登记和评测。
- 当前许可状态：开发使用方案已决定；前文“等待许可决定”等表述保留为历史，已被本次用户决定取代。`redistributable=false` 不变，不把此决定扩大成应用打包分发许可。P7 必须再次评估。
- 独立参考方案见 `ALIGNMENT-PREREGISTRATION.md`。`qwen-asr==0.0.6`（Apache-2.0）与 `Qwen/Qwen3-ForcedAligner-0.6B`（Apache-2.0，修订 c7cbfc2048c462b0d63a45797104fc9db3ad62b7）；Torch 2.10.0+cpu、transformers 4.57.6，完整环境由 ASR 子目录的 uv.lock 固定。这是本机评测工具，不加入应用运行时依赖或运行时模型锁。
- 已核验官方 wheel SHA-256 `b9c55a38413298f3a990a4475467399daec6e8f4172363053fc42e2166c2dfd3`，固定模型全部文件哈希；本轮未加载模型、未运行用户音频对齐或延迟测量。
- 官方配置的 timestamp_segment_time 为 80 ms。这是模型输出的时间网格，不是经过校准的真实边界误差上限。最后词/字结束点作为最后音节结束的模型估计，不宣称人工真实标注；临界案例随结果完整报告。
- 待用户录音后：冻结全部 60 条用户 WAV 与人工文字稿的哈希 → 独立强制对齐 → 仓库外生成 timing-references.json → 150 条 CER/WER/LID → 安静窗口延迟测量。无缺样本、无按结果筛选、无能量阈值、无 Silero/SenseVoice 参考。失败保留并阻塞整卡。
- 本轮准备检查与未来真实验收严格区分；之前 CI/静音 CPU/公开集数值属于先前提交，不自动冒充本轮完整验收。

### 本轮准备验证（2026-10-21）
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| pnpm asr:user | 已建立 zh/en/ja 各 20 份文字稿；0/60 WAV，等待用户 | 本机仓库外 user/ 和 user.json；READING.md |
| pnpm lint | 通过；首次发现 replay.ts 的 finally 抛出 lint 错误，已把检查移到清理后修复 | continue-lint.log |
| pnpm typecheck | 通过 | continue-typecheck.log |
| pnpm exec vitest run packages/voice/src/asr | 4 文件、19 项通过 | continue-asr-tests.log |
| python -m unittest discover -s packages/voice/src/asr/alignment -p test_reference_data.py | 4 项通过 | alignment-reference-tests.log |
| mypy 1.19.1 --strict（对齐器及辅助脚本） | 3 文件通过 | alignment-mypy.log |
| 独立对齐输入 --preflight | 预期拒绝：registered=0，未生成参考时标 | alignment-preflight.log |
| uv lock | 已固定 96 个包；未安装完整对齐环境或加载权重 | alignment-lock.log、alignment/uv.lock |

追加命令 `pnpm asr:align:models` 下载并逐文件校验独立对齐模型，`pnpm asr:align` 在全部 60 条录音登记后生成仓库外参考。对齐器真实运行、完整 150 条识别和延迟验收均待录音，不能用辅助单测替代。正式测量前本次方法文件必须已提交且无未提交修改；测量脚本拒绝旧式自声明时标、缺样本、哈希变化和追赶式突发 PCM 重放。

当前 PR 保持草稿且保留 needs-user。录完通知后再进入真实评测；全部验收有结果后重新同步 main、串行重跑必要检查，再转可审查并通知用户。

## 2026-10-21 连续录音接收与导入准备
- 已收到用户单个 M4A，389.504 秒、48 kHz、双声道 AAC-LC，4,748,270 字节。原文件保留；另存仓库外 imports/58a8fe9653a2cdac/original.m4a，原件与副本 SHA-256 一致：58a8fe9653a2cdac58475a53aeea3e2b132594e641b94efc3e31178e033d87f0。
- 规范整轨只转换为 16 kHz 单声道 PCM16，SHA-256：ae86600f0bf28cd2eabaeb9b7133d7001e0e4f29553c693c15ee8eb805fab2ed。未裁静音、未归一化；有损 AAC 来源不能被描述成无损采集。
- 连续输入规则在 CONTINUOUS-IMPORT-PREREGISTRATION.md 追加预登记：独立工具定位、固定顺序、保留最多 1 秒前后余量、逐样本切分与哈希、全部 60 条覆盖；参考时标仍在正式逐句输入冻结后独立重新生成。
- 当前正在确认用户实际录制顺序与是否重读。候选音频不写正式 user.json；全部验收仍未完成，PR 保持草稿与 needs-user。对齐工具环境安装/模型下载只在本机进行。

### 连续录音工具实际环境与本轮检查
| 本机评测包 | 实际版本 | 许可证（包元数据） |
| --- | --- | --- |
| qwen-asr | 0.0.6 | Apache-2.0 |
| torch | 2.10.0+cpu | BSD-3-Clause |
| transformers | 4.57.6 | Apache-2.0 |
| nagisa | 0.2.11 | MIT |
| dynet38 | 2.2 | Apache-2.0 |
| numpy | 2.2.6 | BSD-3-Clause |
| soundfile | 0.13.1 | BSD-3-Clause |
| accelerate | 1.12.0 | Apache-2.0 |

独立环境仅用于本机评测；不进应用运行时依赖或打包。`Qwen3ForcedAligner` 实际导入成功；nagisa 有厂商代码的无效转义 SyntaxWarning，未修改其源码。模型权重仍须完整下载、按已登记 SHA-256 验证后才可加载。

合并 origin/main（f0303c2）后串行验证：`pnpm lint`、`pnpm typecheck` 通过，`pnpm test` 33 个文件 462 项通过；证据为 import-main-lint.log、import-main-typecheck.log、import-main-tests.log。连续切分辅助测试 5 项通过、Python 严格类型检查 3 个文件通过，见 continuous-import-tests.log 和 continuous-import-mypy.log。没有把这些检查写成录音/语言/延迟验收通过。

### 连续录音当前阻塞状态（2026-10-21）
已接收并保护原始连续 M4A，等待用户确认 zh20→en20→ja20 的录制顺序及是否重读/跳句。正式 user.json 尚未改变，当前没有可验收的 60 条切片或有效 timing-references.json。

独立对齐工具已安装且导入成功，但 1,835,544,544 字节的模型权重尚在缓慢下载，未校验完成前不加载。原始 curl 下载保留继续；额外官方 HF HTTP/Xet 下载尝试未产出完整文件，已停止重复下载以免占用带宽，日志保留在仓库外 imports/58a8fe9653a2cdac/。固定模型来源、修订和 SHA-256 不变；不以未验证权重、能量阈值或被测系统替代参考。

下一步：确认录制内容 → 完成模型哈希核验 → 依预登记生成全部切分候选及外部切分记录 → 登记输入 → 独立逐句对齐 → 150 条评测 → 安静窗口延迟测量。PR #44 保持草稿及 needs-user；当前未宣称拆分或整卡验收完成。

### 用户补充与核对稿（取代上一节待顺序回复的状态）
用户确认按表顺序录制，有个别重读，要求转写核对。已用现有本机 SenseVoice 完成 18 个固定窗口的私有机器核对草稿；混语窗口有明显漏识别，不能当真实文字稿。稿件没有用于评分，也未写正式用户文字稿或时标。根据各窗口对应句子序号，输入看起来按表格逐行 zh/en/ja 交替；方法补充 v1.2 在任何独立定位/延迟结果之前提交，候选按此假设定位，全 60 条独立覆盖和重读歧义仍须检查。现阶段无需用户重录。

### 首次独立定位结果与保留的失败
独立 Qwen 模型全部文件 SHA-256 校验通过并已在 CPU 加载。v1.2 粗定位的前三行 9 句给出了有效区间，但第 4 行的 23 单位中出现 2 个零时长单位，严格校验停止；没有生成有效 60 条切片或时标。见 continuous-rowwise-failure.json；完整 raw、音频窗口和 traceback 均在仓库外 import-attempt-20261021T121716Z 和 import-rowwise-attempt.log。未删除失败或放宽规则。随后先提交 v1.3 对应语言逐句定位方法，再对全部 60 句从头重跑；不是延迟验收重试择优。

### 第二次定位失败与候选边界
逐句 v1.3 在第 4 个候选再次因零时长单位停止，日志 import-sentence-attempt.log 和全部 raw 保留在仓库外。混语上下文只是可能原因，未被第二次尝试证实。没有运行延迟。

继续不受影响的音频整理：方法 v2 只生成带零时长警告的可逆候选切片，不签发时间参考。正式逐句 boundaries 函数、测试及全部验收阈值不变；正式时标仍可能阻塞。不得把候选文件存在写成独立时标通过。

### 当前可核对交付物（取代前文下载中／0 切片状态）
Qwen3 ForcedAligner 权重及全部配置已完整下载并按 manifest 的 SHA-256 校验，通过实际 CPU 加载。没有改应用运行时依赖；额外 HTTP/Xet 下载已停止，完整原下载成功，日志保留。

连续录音已整理为全部 60 个候选 WAV，zh/en/ja 各 20 条，文件按 001–020 编号；源音频、逐样本切分、哈希及全部失败尝试保留。候选位于仓库外 imports/58a8fe9653a2cdac/split/，试听与“原提示／机器转写”对照位于同目录 REVIEW.html / REVIEW.md。切片哈希与区间见 continuous-candidates.json，不含用户音频或实际转写。

逐句机器稿 60/60 已生成；按既有归一化，33 条与提示完全一致，27 条不同。这不是识别验收：不同可能是模型错误或实际改读。尤其日语第 8 句机器稿出现开头改口；必须按用户选择决定保留实际重读还是取最后一遍完整朗读，不能按识别得分挑选。机器稿不自动覆盖人工文字稿。

19 个候选的粗定位存在零时长单位警告，全部保留；没有把候选区间签发为有效 timing-references.json。正式 user.json 尚未变更，完整 150 条与独立时标／延迟仍未验收。已向用户询问重读处理规则，候选与核对稿已具体可审阅。PR #44 继续草稿、needs-user；不合并，不取卡。

## 2026-10-21 用户已验证文字稿（当前决定，取代此前待确认）
用户明确确认：“我已验证，机器转写正常无误。”录音／文字稿／重读处理不再等待用户，不再作为用户阻塞。保留全部 60 个完整切片及其中重读，以用户已核实的逐句机器稿登记实际文字；不根据识别得分挑选或裁掉重读。原提示文字备份、原录音、机器初稿和所有失败尝试均保留在仓库外。

60 条音频已进入本机 user/zh、user/en、user/ja，每种 001–020；下一步立即登记哈希并跑完整 150 条。文字来源明确为“被测系统草稿经用户验证”，不把用户集称作独立盲测；独立 90 条公开集继续单列。文字验证不验证任何时间点，所有时标仍须独立强制对齐。

尚未完成的是执行方的独立起止时标、完整识别评测与延迟测量。此前“待用户确认重读”等记录保留为历史，已被本次确认解除；不再据此停工或要求用户重录。PR #44 仍保持草稿和 needs-user，直到整卡验收及检查齐备。

## 2026-10-21 当前验收与测量流程修正
完整 150 条结果：zh CER 6.29%、en WER 6.69%、ja CER 7.52%；语言识别 149/150 = 99.33%。见 evaluation.json 与 eval150-user-verified.log。公开 90 条独立参考保持不变；用户 60 条参考为用户明确验证的逐句稿，源草稿来自同一识别器，不能把用户集称为独立盲测。原录音、原朗读稿和用户验证哈希均保留。

独立 v1 全量对齐 37/60 通过额外的严格词元校验、23/60 被拒绝：22 条为零时长词元，1 条末端超出文件 40 ms。完整数值见 strict-alignment-diagnostics.json，原始全部 60 条输出在仓库外，不删除失败。原始数据不证明用户录音有问题；v1 每个词元必须正时长是执行方加入的额外规则。

已在任何实际延迟测量之前登记 v2：保留全部量化时间点及警告，验证完整语音区间和单调／有限／边界合法性；全部规范 WAV 统一追加 1 秒数字静音，原文件不改，独立工具与被测系统使用同一 v2 WAV。旧函数和全部旧断言保留，v2 单独版本、路径、文件快照及测试；不改 250/300 ms 验收阈值，不删样本，不使用能量或被测时间点。见 ALIGNMENT-PREREGISTRATION-V2.md。准备验证：9 项 Python 测试、严格 mypy、lint/typecheck、19 项 ASR 单测通过；证据 alignment-v2-*.log。

历史结果栏原文保留：部分完成：实现、公开集评测与静音 CPU 测量已完成；待用户录音、独立语音时标的延迟测量及 SenseVoice 模型许可确认。不得据此宣称 P4-01 全部验收通过。该状态已由本节与结果栏的新状态取代。


## 2026-10-21 独立时标完成与测量窗口中断
v2 全部数值有效，但孤立输入部分首单位被定位为 0，未用于被测延迟；v3 增加统一公开语音上下文与首尾数字静音，59/60 有效，日语 016 复合词拆分导致跨段错配，按规则拒绝整套发布。两轮原始输出、诊断和输入快照均保留在仓库外，均未用于签发延迟通过。

v4 在 c224bf2 提交后、首次实际延迟之前，按目标原语言分词并固定上下文单位边界。60/60 参考有效，20 条含量化零时长单位警告，所有词元保留，整句起止合法；不使用能量阈值或被测系统时间点。目标音频、60 条完整录音和已验证文字均保持冻结。方法与诊断见 ALIGNMENT-PREREGISTRATION-V4.md、alignment-preregistration-v4.json、alignment-v4-evidence.json、alignment-v4-diagnostics.json；全部 timing-references.json 与原始词元留在仓库外。

独立工具 qwen-asr 0.0.6 和 Qwen3-ForcedAligner-0.6B（Apache-2.0，固定修订与哈希见 manifest）仅用于本机评测，未进入应用运行时。80 ms 是模型输出网格，不是经人工校准的误差上限；临界样本必须保留，不能据此宣称参考是真实毫秒精度。

首次 pnpm asr:latency 在启动前安静检查通过，但完成 15/60 后检测到 uv/Python 测试或模型任务，立即停止。状态 INVALID，原因是资源窗口中断，不能把部分数值或缺失的 45 条当成最终验收，也不因该失败修改代码或阈值。原始日志及唯一尝试文件完整保留：latency-v4-run.log、latency-attempt-2026-10-21T13-53-12-367Z.json。待其他任务结束后从头重测全部 60 条。当前录音和文字稿没有待用户确认项，PR #44 仍为草稿，needs-user 保留。

测量覆盖 20 ms PCM 输入到本机 VoiceRuntime AppBus 回调；没有把它表述成物理麦克风驱动到 UI 的全链路测量。真实 Electron utilityProcess 的采集／通道和静音 CPU 单独有对应证据。


## 2026-10-21 D 盘迁移（用户明确指示）
默认缓存统一为 D:\NemuData，模型及完整评测资料按 SHA-256 验证，评测环境按原锁文件在 D 盘重建且 94 个依赖版本一致。原生 ASR/VAD 和独立 Qwen3 对齐均在 D 盘实际运行通过后，已清理 C 盘三处 Nemu 专用副本。详见 [CACHE-MIGRATION.md](CACHE-MIGRATION.md) 与 cache-migration-*.json；没有更改冻结输入、时标或验收阈值。


## 2026-10-21 同步与迁移后复验
已合入 origin/main 806e09b（ADR 0015，资产许可字段追加），本卡语音契约未变。CI 暴露桌面导入 @nemu/voice 缺工作区依赖，已补齐 apps/desktop/package.json、生成锁文件，并把路径函数打入 Electron 主进程包；未新增第三方运行时包。失败日志 cache-migration-ci-failure.log 保留。当前 lint、类型、469 项单测、Python 严格检查和 11 项辅助测试、整仓构建、4 项 Electron E2E、静音 CPU 复测及 150 条识别全部通过。E2E 的专用 CPU 项在普通套件内明确跳过，随后单独实测通过。完整延迟测量进行中；不得把尚未得到的结果记为通过。


## 2026-10-22 完整实时延迟结果及当前限制

生产实现按原参数完成全部 60 条，状态 FAIL：中位 VAD 结束延迟 320.091 ms，打断 59 条超过 250 ms、最大 1340.490 ms。无缺事件，最大单包重放迟到 17.351 ms，未发生资源冲突或超过 20 ms 的无效重放。user-en-014 的 VAD 结束 -639.492 ms、user-ja-014 的打断 -119.875 ms，全部原值保留，不归零、不删样本。该结果相对冻结的独立模型时标，不把模型估计当人工校准的真实边界。

前一轮完成 48 条后因测量器误把 ASR_LANGUAGE_MAPPED 当致命错误而 INVALID，原始数据保留。该代码是约定的语言映射诊断；已先提交 MEASUREMENT-DIAGNOSTIC-ADDENDUM.md 与回归测试，仅此诊断非致命且逐条保留，未知和实际管线错误仍致命。本轮完整数据只在 user-ja-008 记录该标记。没有根据结果删掉重读或改变文字。

### 离线诊断，不是验收

固定五组全局参数均覆盖全部 60 条。baseline 中位 VAD 320 ms / 59 条打断超限；首窗补偿候选 280 ms / 47 条；后验 0.35 为 280 ms / 45 条，后验 0.2 为 280 ms / 42 条；较短结束窗 220 ms / 45 条。各组均有提前值，均未解决问题。完整逐条值在 vad-diagnostic-grid.json，计划在任何该组结果之前提交。

持续窗门控候选的前一组意图使用原生零等待，实则 C API 将数值零当默认值，原生日志证实实际使用 0.25/0.5 秒，故不能当该算法的有效验证。三组结果原样保留。随后先提交哨兵修正，非零 1e-6 秒在 16 kHz 整型样本计数为零；3200 确认样本门槛及短噪声、分离脉冲和 reset 断言不变。475 项单测通过。有效配置的前两组仍分别有 47/45 条超限；第三组被无法读取命令行的后台 Python 进程打断，进度文件标 INCOMPLETE，不作全组三组通过结论。生产 SenseVoiceProvider 未引用候选类，未应用这些诊断参数。

诊断工具初版遇到最长重读片段的 float PCM 超出子进程默认 1 MiB 输出缓冲；修正为 32 MiB 后全部片段保留，失败日志仍在。另一轮在读取不到后台进程信息时被资源保护停止；没有为此放松资源检查。各次日志、原生配置与逐组进度均保存。

### 独立参考质量复核

对全部 60 条统一比较粗定位与正式 v4 上下文结果，见 timing-quality-comparison.json。粗定位不是有效真值，不能替换正式参考；该比较不读取 DUT 时间点。英文 014 的两端分别相差约 -1020/+980 ms，表明存在明显上下文敏感性，不能只用 80 ms 网格解释，亦不能自动按更有利于延迟的方向移动点位。

已只下载三个独立 CTC 候选的小型配置与词表，登记固定修订、哈希和 Apache-2.0 来源，未下载或使用大型权重（候选权重总计约 2.93 GB）。现有已确认稿不经改写时，英文、中文、日文完整字符覆盖分别 19/20、18/20、16/20；日文稿部分混合字符超出该日语模型词表。不能删除不覆盖句、用未知标记顶替或擅改用户已确认稿来制造全量时标。因此未把该候选当解决方案，未增加应用运行时依赖。见 ctc-character-preflight.json、alignment/ctc-candidates.manifest.json。

后续必须先取得全量可靠且独立的时间参考或明确其不确定性，再验证持续语音判定优化。可在同一工具上对全部 60 条做预登记的同语言双锚点一致性检查；若仍不稳定，再评估覆盖混合文字的独立音素级工具。任何新参考方法都须在再次测量前提交，旧参考和失败结果保留，不以 DUT 结果选点，不以能量阈值或 Silero/SenseVoice 输出替代真值。

当前不是等待用户录音、重读处理或 SenseVoice 开发许可；这些已决定。待解决的是执行方的延迟性能及时间参考质量。PR #44 保持草稿和 needs-user，不请求 Claude 提前审查，不合并，不取下一张卡。


### 2026-10-22 诊断补齐与最终状态
有效零样本门控的第三组只续跑原定 0.2 配置全部 60 条，结果为 42 条打断超限、2 条提前、1 条提前结束，最大打断约 1300 ms，仍不解决问题。原有效前两组的 47/45 条超限及全部原始数据保留，未覆盖或选择成功样本。故所有候选均未进入生产实现。

针对 CIM 进程退出竞态，未知工作进程只触发一次新的完整快照；只有新快照证明已无未知进程且没有重负载才继续。仍存在的信息不明进程或新出现的重负载仍会阻止执行，四项测试证明该边界。此前中断仍是中断，不追溯为有效。最终静态检查前已再次确认安静，不再把过去中断写作当前用户资源待办。

最终本机 lint、typecheck、479 项单测及整仓构建见 final-resume-*.log；Python 8 文件严格检查与 11 项辅助测试、Electron 4 项功能验证及 60 秒 CPU 复测证据保持。最后 fetch 的 origin/main 没有新提交，当前已含 806e09b。整卡仍未完成，下一步是独立参考稳定性复核及满足既定持续语音语义的低延迟实现，而不是要求用户重录或重新核实已确认稿。

### 历史下一步原文（保留记录，已被当前状态取代）
## 下一步
实现提交：`4d97e30`。Windows CI 与 Claude 审查均针对提交后的 PR，最终状态见 PR 检查。
已等待 P2-03 Blender 与 P4-02 Electron 重负载结束并复核进程，再串行完成公开集和静音 CPU；最后完成回归后转可审查，交 Claude 审查实现。不合并，不取下一张卡。等待用户录音与模型许可决定。

### 2026-10-22 独立参考全量稳定性复核（已完成，未通过）

继续任务前已 fetch 并 merge 最新 origin/main（bba68cc，合并提交 3ee272f）；规则更新不改变本卡验收。用户明确要求的整卡完成后审查、不合并、不取下一张卡继续执行。

先提交方法和脚本 ac11d72，再执行全部 60 条、每条两个固定同语言锚点，共 120 次 CPU 对齐。工具、模型、参数、输入及 80 ms 一致性门槛见 REFERENCE-STABILITY-PLAN.md / reference-stability-plan.json；实际结果见 reference-stability-20261022T030747Z.json。没有读取 DUT 延迟来选点，没有改转写或跳过重读，没有更换正式参考。

| 语言 | 稳定 / 总数 | 含无效上下文的录音 | 有效双上下文最大起点差 | 最大终点差 |
| --- | --- | --- | --- | --- |
| 中文 | 19/20 | 0 | 980 ms | 60 ms |
| 英文 | 8/20 | 3 | 1000 ms | 1000 ms |
| 日文 | 12/20 | 0 | 1180 ms | 60 ms |

共 39/60 稳定、21/60 不稳定；120 次中有 3 次边界越界，44 次包含零时长单位警告，均保留。示例：中文 018 两个起点约 980 / 1960 ms；英文 014 两个终点约 5120 / 6120 ms。不能把这种差异解释为单纯 80 ms 量化误差。双上下文一致也不等于真实物理边界准确；本轮已经足以否定该固定上下文方案作为全量可靠验收参考。

完整音频、逐单位原始结果和文字只在 D:/NemuData/asr-evaluation/reference-stability-20261022T030747Z/；仓库只保存编号、时间值、哈希、警告和来源提交。原 timing-references.json SHA-256 仍为 d6889a9d9f2fd7d102e517d914da83f85e96648196579913f5d530a5dae03409，未替换。旧延迟结果保留为相对旧模型参考的实测失败，不能继续当作已验证物理边界下的唯一性能结论；本轮没有重跑或宣告延迟通过。

启动及运行中资源保护未触发冲突，使用 D 盘已有 Qwen 权重和 D 盘环境；没有新增模型权重，没有 GPU 测量。全部数据处理不触及用户正式存储。

### 替代工具文字预检（尚未选用声学模型）

仅用 uroman 1.3.1.1 对已确认文字执行字符覆盖预检，未读取音频或 DUT 时间。见 alignment/mms_text_preflight.py、mms-text-preflight.json：49/60 可直接落入 MMS_FA 字母词表，11/60 罗马化结果仍含 0、1 或 5。uroman 也会把原文中的部分汉字数词转换为数字，因此不能只处理原稿中显式的阿拉伯数字。没有删除数字、未知字符替换、修改已确认稿或声称字符覆盖等于对齐有效。必须先建立可审计的语言相关发音转换和全量覆盖，再考虑下载权重及预登记声学验证。

此处的 uroman 是本机评测候选辅助工具，不是应用依赖；没有改动原 94 包对齐环境或根运行时清单。首次探查 1.3.1 收到 yanked 提示，正式可重跑脚本固定文档修正版 1.3.1.1，结果由该版本产生。缓存和临时环境全部在 D:/NemuData/cache/uv 与 D:/NemuData/tmp。

| 评测辅助工具 | 版本 | 许可证与要求 | 状态 |
| --- | --- | --- | --- |
| uroman | 1.3.1.1 | MIT 文本附项目发表致谢要求：This project uses the universal romanizer software 'uroman' written by Ulf Hermjakob, USC Information Sciences Institute (2015-2020) | 仅本机文字预检；未进入应用 |

候选声学模型 MMS_FA 的官方说明为 [CC-BY-NC-4.0](https://docs.pytorch.org/audio/2.7.0/generated/torchaudio.pipelines.MMS_FA.html)，不是宽松模型许可，未下载、未采用、不会随应用分发。官方 TorchAudio 旧 API 另有版本兼容约束，不可直接在现有 torch 2.10 环境中假设可用。另一条可研究路线是 [MFA](https://montreal-forced-aligner.readthedocs.io/en/latest/getting_started.html) 及其音素词典；目前没有安装或声称其能处理全部混合文字，不把研究候选写成通过证据。

本轮回归：lint、typecheck、36 文件 479 项测试通过；新两文件 mypy --strict 通过；辅助测试 12 项通过。证据 reference-stability-lint.log、reference-stability-typecheck.log、reference-stability-tests.log、reference-stability-mypy.log、reference-stability-python-tests.log。首次 mypy 未设置模块搜索路径而报 cache_paths 找不到；修正 MYPYPATH 后同一源文件通过，此为检查配置问题，未添加 ignore 或改类型门槛。

已将这份 SHA 的参考登记为拒用，loadTimingReferences 在启动测量前明确报 QA failed；旧文件和全部历史失败保留。该拒用不允许未知参考绕过已有清单、来源与哈希校验。回归新增两项，最终 37 文件 481 项通过，覆盖已知失败哈希被拒绝及非同一哈希不被误归类。

当前下一步：独立参考方案仍需解决全量发音转换与可靠性验证，之后才能确定 VAD / 打断优化及重跑实际验收。当前不要求用户重录或再次确认转写；不把技术问题转为用户许可或录音等待。PR #44 保持 draft + needs-user，不转可审查。

### 2026-10-22 用户指定收敛方案（当前状态，取代前面的工具搜索下一步）

已按用户最新决定停止新对齐工具/模型/权重搜索、下载与MFA安装，停止延迟优化。既有模型、0.5概率阈值、≥200 ms持续语音门限、300/250 ms验收指标全部不改。39条双锚一致固定主锚参考，21条改为用户人工标注，另一次性随机抽5条稳定录音盲检。预登记提交f9e42c5先于页面生成和标注；具体规则是第一个/最后一个能听到的语音音，包括清辅音。重读保留，随机集合不重抽。

- 盲标页已生成：D:/NemuData/asr-evaluation/manual-labels/annotation.html，26条混排，不含自动点位或DUT输出。操作见ANNOTATION.md。音频、批准文字和频谱只嵌入仓库外本地HTML；CSP禁联网，无服务器/上传/外部资源。JSON用显式文件选择器保存到D盘，不回退到C盘下载目录。
- `pnpm asr:annotate`复建页面；`pnpm asr:annotations <最终JSON>`验证26条身份、音频与文字哈希、坐标和final状态，报告5条人工−自动起止偏差；>80 ms逐条报告。只生成仓库外的60条候选，不覆盖旧参考、不自动启动测量或解禁旧参考。
- 39条既有原始trace描述性重算：VAD结束中位319.80 ms、p95 380.49 ms；打断中位300.30 ms、p95 1192.29 ms；负结束/负打断均0，结束>300有29条，打断>250有36条。一条打断小于名义200 ms及五条长尾的参考质量疑点均保留，不猜原因或移动点位。见STABLE39-LATENCY.md / stable39-latency.json。
- 旧日志没有原生推理、处理和事件发送独立时标；这三项明确为未采集，JSON为null，未填0或编造比例。事件ts在publish内schema.parse/emit之前采样，不能冒充实际收包或完整IPC耗时。
- 门限与目标的可选方案、代价已列N0-P0-09-LATENCY-OPTIONS.md，并以只追加方式链接进reports/P0-09/AGENDA.md第2、3项；tasks/requests/P4-01-to-P0-09-latency-budget.md记录评审接口。没有实施任何候选方案。

本轮lint、typecheck、38文件487项单测、16项Python辅助测试、新增4文件mypy --strict通过，见manual-*.log。六项页面状态测试使用合成媒体与独立DOM桩，不是浏览器测试；26份嵌入WAV SHA/格式/长度及频谱PNG结构、数据字段白名单通过，见manual-page-integrity.json。浏览器自动化明确拒绝file://协议（仅允许HTTP/HTTPS），没有绕过或开启服务器，因此真实浏览器画面、声音和文件选择器尚未经自动实机验证。这一限制已在用户操作文档说明。

annotation/index.html为715行独立离线模板，包含可访问的操作控件、样式和无外部依赖的内联ES模块。单文件有意让生成品可以从file://直接打开，避免服务/网络和新打包依赖；它不进入应用运行时入口。其状态/数据安全逻辑有独立合成媒体测试。新增运行时依赖：无。频谱仅用已有numpy 2.2.6（BSD-3-Clause），PNG编码由标准库完成；没有新模型资产。

再次fetch后合入origin/main的bf69c32（仅新N0议程）；本卡规则、任务卡、契约和架构无变更。主代理已核对routine_reporter只读返回的39条集合、逐条值、线性分位及计数，无差异。

当前下一步：等待用户导出26条final JSON；先导入并报告所有>80 ms抽检偏差，处理后冻结混合参考，在无Blender/Electron/Playwright/GPU测量/Runner.Worker的窗口从头完整测60条。此次39条分析不当作正式结果。PR保持草稿与needs-user，不合并、不取下一张卡。

本轮测试生成的8个已归类logs/*.log保留在工作树，未提交；其D盘只读副本及SHA见manual-generated-test-logs.json。自动审批拒绝了原归档/清理命令，返回理由仅为“blocked by policy”；随后只复制证据，不移动或删除原文件。该情况不影响标注页、分析或PR草稿状态。最终页面字节级校验见manual-page-integrity.json；校验器首次把CRLF文本归一化后重新编码而造成SHA断言失败，改为直接校验原始字节后通过，页面未改动，记录保留于manual-page-integrity-checker-note.log。


## 2026-10-22 人工标注与原参数正式测量（当前结论）

- 人工标注及五条抽检已处理：原抽检 4/5 在 80 ms 内，ja-013 自动起点相差 5454.875 ms，用户明确确认人工正确。原失败保留，38 自动＋22 人工冻结后从头测 60 条。详见 MANUAL-AUDIT-RESULT.md、HYBRID-REFERENCE-V2.md。
- 正式结果与阶段耗时见 FORMAL-LATENCY-HYBRID-V2.md、formal-hybrid-v2-summary.json。有效完整结果 FAIL；首轮被 CI 中断为 INVALID，分开保留。没有改门限、模型、目标、挑选事件或删样本。
- 当前识别值：zh CER 6.29%、en WER 6.69%、ja CER 7.59%，LID 99.33%；这是人工注记修正后的既有 150 条推断重计分，非新模型运行。新增命令 `pnpm eval:asr:annotations`，来源和音频哈希校验后生成版本化私有 manifest 与公开结果，原文件不覆盖。
- 静态/单元回归：lint、typecheck、40 文件 494 项 Vitest、18 项 Python 测试、3 文件 mypy strict 通过；同步最新 main 后检查另留 hybrid-v2-after-final-sync-*.log。没有新增依赖或模型。本轮只读核验按 routine_reporter 角色进行，360 个逐条数值一致，不由子代理裁决验收。
- 与 main 同步到 a5a3091；本卡 AGENTS/契约/架构/任务卡无新变化。9fbc213 合并期间共享 package.json 的本地错误提交与显式修复记录见 MANUAL-AUDIT-RESULT.md；不重写历史。

### 待用户确认（当前）
- [x] 2026-10-21 已决定：SenseVoice 用于开发，每位用户自行下载，不随应用分发，redistributable=false；保留 FunASR 出处和模型名，P7 正式发布前重评。原许可阻塞记录保留。
- [x] 2026-10-22 用户完成 26 条标注，并明确确认 ja-013 在 6.695 秒之前无语音、人工起点正确。无需再次标注或重录。
- [x] 原阻塞已解决（2026-10-23）：ADR 0017 已合并，VAD ≤350 ms、打断中位 ≤300 ms，当前原参数实测均通过；提前误触发仅报告，无需继续等待 N0 决定。原方案和旧 FAIL 数据保留。

### 下一步（当前）
等待 N0/P0-09 评审决定；PR #44 保持草稿及 needs-user，不提交 Claude 提前审查、不合并、不取下一卡。用户原始录音、文字及标注保存在 D 盘仓库外；工作树生成的 logs/ 已归类且未提交，不清理原件。

## 2026-10-23 ADR 0017 验收更新与收尾（当前状态）

已 fetch 并 merge origin/main 的 1f7d473，包含 ADR 0017、P4-01 卡追加说明和 N0 决定；本轮不修改契约、ADR 或运行参数。ADR 正文的背景仍写 39 自动＋21 人工，本卡最终冻结证据实际为 38＋22（ja-013 经用户复听确认后改用人工），沿用已登记 SHA，不覆盖原文或参考。

| 当前卡片验收 | 实测 | 结论 |
| --- | --- | --- |
| 中文 CER ≤10% | 74/1177=6.29% | 通过 |
| 英文 WER ≤10% | 54/807=6.69% | 通过 |
| 日文 CER ≤10% | 127/1674=7.59% | 通过 |
| 150 条 LID ≥98% | 149/150=99.33% | 通过 |
| VAD 说完中位 ≤350 ms | 320.1581 ms | 通过 |
| 开口→voice.bargeIn 中位 ≤300 ms | 285.2591 ms | 通过 |
| 提前误触发条数与比例（只报告） | 1/60=1.67%，user-ja-013 | 已报告，不作为失败门槛 |
| 静音 CPU ≤2% | 先前有效 60.73 秒实测均值 0.11191%、峰值 0.14086% | 通过，未冒充新测量 |

**提前误触发原因：** ja-013 人工起点 6694.875 ms，用户明确复听确认此前没有语音。Silero 首次 speechStart 在 1420.504 ms，将非语音区间误判为语音；现有持续判定约 200.024 ms 后发出首个 bargeIn（1620.528 ms），比真实开口早 5074.347 ms。可确定是该非语音段的 VAD 假阳性触发打断，不能确定具体是何种声音，也没有证据归因为门限缩短。保留首个事件及负值，不换取后一次事件或排除样本。

**验收重判与测量的区别：** 新报告 latency-acceptance-adr0017.json 绑定原始唯一尝试 SHA，按新中位目标重新判定，没有重放、模型推断或修改 60 条数值；原 latency.json、唯一尝试和 formal-hybrid-v2-summary.json 的旧目标 FAIL 保持历史原样。可用 `pnpm exec tsx packages/voice/src/asr/reassess-latency.ts` 重现；未来 `pnpm asr:latency` 也按 ADR 0017 判定。新增边界测试覆盖含负打断/长尾的中位数、等于上限、超过上限、缺事件、重复 ID、非有限值及提前结束；不放宽 20 ms 迟包等测量有效性约束。

原许可、用户录音、人工时标、等待 N0 的阻塞均已解决；不需要用户再次录音/标注。SenseVoice 保持 redistributable=false、用户自行下载、不随应用分发、保留 FunASR 模型名与出处，P7 许可复评作为后续发布工作。本轮无新增运行时依赖、模型或下载。物理麦克风、跨进程传输和扬声器停止的完整延迟仍未由本次重放测得；保留原报告边界，不把它冒充 ADR 第 3 项的扬声器停止验收（后续语音输出卡负责）。

检查证据：adr0017-lint.log、adr0017-typecheck.log、adr0017-tests.log、adr0017-build.log；Python 与 Electron 检查另存 adr0017-*.log/e2e。中难度无需审查；PR 转为可审查后等待 CI，全部绿色后由用户合并，执行方不合并、不取下一卡。needs-user 仅标记由用户合并，不再代表待补录音或待决定延迟目标。
