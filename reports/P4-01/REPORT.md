# P4-01 语音识别与语言识别 — 报告

## 结果
部分完成：实现、公开集评测与静音 CPU 测量已完成；待用户录音、独立语音时标的延迟测量及 SenseVoice 模型许可确认。不得据此宣称 P4-01 全部验收通过。

## 同步与认领
- 从 `origin/main` 创建 `task/P4-01-asr`，独立工作树 `C:\Users\nanake\.codex\worktrees\p4-01-asr\nemu`。
- 草稿 PR：https://github.com/nanake-lab/nemu/pull/44，已标记 `needs-user`。
- P2-03 的 `D:\Dev\nemu` 工作区、未提交文件和资产没有改动。
- 用户指示不合并、不取下一张卡；难度中，交 Claude 审查。

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
模型：`%LOCALAPPDATA%\Nemu\models\asr`，可用 `NEMU_ASR_MODELS` 覆盖。
音频：`%LOCALAPPDATA%\Nemu\asr-evaluation`，可用 `NEMU_ASR_DATA` 覆盖。用户录音、用户文字稿、user.json、FLEURS 音频和 Parquet 均在仓库外。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| 模型下载/哈希 | 三文件已校验；SenseVoice int8 239233841 字节 | lock、manifest、model-hashes.json |
| 公开数据准备 | 90 条，三语各 30，三个 Parquet SHA-256 与官方 LFS 元数据一致 | public-download.log、public-data.lock.json |
| P4-01 单测 / 整仓 Vitest | 16/16；整仓 32 文件、459/459 | unit-v3.log、final-test.log |
| TypeScript 类型检查 | 通过 | final-typecheck.log |
| ESLint | 通过 | final-lint.log |
| Python mypy --strict | 通过 | mypy.log |
| workspace build | 通过；含最终 Worklet 修复的桌面构建通过 | build-final.log、desktop-build-v4.log |
| 完整 pnpm test:e2e | 4 通过、1 个专门的静音 CPU 测量按设计跳过（已单独实测通过）；附加 OS 快照因没有初始化 run.json 报错，测试本身退出 0；Windows CI 将按标准初始化后重跑 | full-e2e.log、full-e2e/ |
| 真实麦克风/utility 通道测试 | 通过：50 packets/s、异步关闭，未读取真实用户麦克风，使用 Chromium fake device | voice-e2e-v5.log、e2e-v5/ |
| Electron 首次测试 | 失败，原始日志保留。与 CI 窗口重叠且临时目录前缀不符合隔离约束；不能据失败推断单一原因 | voice-e2e.log、e2e/ |
| Electron 第二次测试 | 安静窗口检查把 PowerShell 空列表 null 当成占用，已修正；等待最终复跑 | voice-e2e-v2.log、e2e-v2/ |
| 各语言 50 条/150 条语言准确率 | 待用户录音；当前用户录音 0/60，不能签发完整验收 | user-recording-status.log |
| 公开集 CER/WER 与 LID | 中文 74/989 = 7.48%，英文 54/664 = 8.13%，日文 126/1438 = 8.76%；LID 90/90 = 100% | evaluation.json、public-evaluation.log |
| VAD 中位延迟 ≤300 ms | 待独立语音结束参考点与安静窗口；不以能量阈值或 Silero 本身输出替代最后音节结束 | 待测 |
| 打断延迟 ≤250 ms | 逻辑测试验证 ≥200 ms 持续语音及每次话语一次打断；本机真实延迟未测 | unit-v3.log；待测 |
| utilityProcess 静音 CPU ≤2% | 60.92 秒、真实采集静音 PCM；均值 0.107%、最大 0.126%，通过 | idle-cpu.json、idle-measurement.log、measurement/ |

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
| SenseVoice int8 + tokens | 2365baeacb507f821a0c8120fcee3d484dba7a07 | custom:FunASR-Model-1.1 | **阻塞：不是项目列出的宽松许可**；redistributable=false 表示本项目尚未批准打包分发，不代表对原协议作法律结论 |
| Silero VAD | 1e261b036686cd0017d500ee96acd1c4ba572a9d | MIT | 模型与源许可已登记 |
| Google FLEURS 中/英/日 | Parquet 168de341b3db6859a9bac1c50a2ef5e3b47647e0 | CC-BY-4.0 | 公开测试数据；音频不入库，保留 Google/FLEURS 归属与来源 |

许可来源：
- sherpa 包元数据与官方源码：https://github.com/k2-fsa/sherpa-onnx
- SenseVoice 转换文件的 LICENSE：https://huggingface.co/csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/blob/2365baeacb507f821a0c8120fcee3d484dba7a07/LICENSE
- 原权重卡：https://huggingface.co/FunAudioLLM/SenseVoiceSmall
- FunASR 模型协议：https://github.com/modelscope/FunASR/blob/main/MODEL_LICENSE
- Silero：https://github.com/snakers4/silero-vad/blob/1e261b036686cd0017d500ee96acd1c4ba572a9d/LICENSE
- 数据集：https://huggingface.co/datasets/google/fleurs （FLEURS: Few-shot Learning Evaluation of Universal Representations of Speech, Google, 2022）

SenseVoice 源码的 MIT 不能替代权重许可证；转换仓库的 LICENSE 转向 FunASR 模型协议。协议含归属、模型名、行为与自动终止等条款，因此没有把它标成 MIT/Apache。按用户本次明确指示加 needs-user，等待用户决定；不自动换模型。

## 待用户确认
- [x] （已决定，2026-10-21）SenseVoice 现阶段继续用于开发；将来不随应用打包，由每位用户自行下载模型；正式发布前（P7）重新评估许可。模型登记保持 `redistributable=false`，注明“用户自行下载，不随应用分发”，保留 FunASR 要求的出处与模型名。
  - 原记录（保留作历史，已由上述决定取代，不再作为当前许可阻塞）：（阻塞）SenseVoice 自定义模型许可下是否同意本项目继续使用及未来分发方案。
- [ ] （阻塞）待用户三语各 20 条录音，完整 150 条验收尚不能完成。
- [ ] （阻塞）VAD/打断物理延迟的独立语音时标仍需补齐，完整 150 条验收仍需用户录音；CI 以最终提交为准。
- [ ] （不阻塞）可选 Nemu 唤醒词默认关闭。当前实现为常开模式下识别结果的 Nemu 文本门控，不宣称专用声学关键词模型或唤醒时延验收。

## 后续建议（不在本任务范围内）
- Echo Core 网关把 voice.recognized 转为 observation.user_utterance，留后续卡。
- P4-06 接真实语音输出时复核 started/done 生命周期和端到端打断淡出。
- P4-02 也在独立工作树对 apps/desktop 做集成；合并时按规则保留双方，不能覆盖另一张卡的启动/守护改动。

## 下一步
实现提交：`4d97e30`。Windows CI 与 Claude 审查均针对提交后的 PR，最终状态见 PR 检查。
已等待 P2-03 Blender 与 P4-02 Electron 重负载结束并复核进程，再串行完成公开集和静音 CPU；最后完成回归后转可审查，交 Claude 审查实现。不合并，不取下一张卡。等待用户录音与模型许可决定。


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
