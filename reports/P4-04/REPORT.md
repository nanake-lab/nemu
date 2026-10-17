# P4-04 口型生成与面部通道 (Lip Sync & Face Channel) — 报告

## 结果
完成。实现了舞台音频输入链与队列（`StageAudioQueue`）、MFCC 元音特征分析器（`LipSyncAnalyser`）、非对称平滑器（`VisemeSmoother`）、多语言档案切换（`zh`、`en`、`ja`）以及嘴部通道优先的面部层对接（`LipSyncRuntime`）。三种语言 60 句偏差测试与打断测试全项达到验收标准。

## 交付物
- `packages/avatar-runtime/src/lipsync/types.ts`：音频通道契约消息与事件载荷类型定义（`SpeechBeginMessage`、`SpeechChunkMessage`、`SpeechEndMessage`、`SpeechStopMessage`、`SpeechStartedPayload`、`SpeechProgressPayload`、`SpeechDonePayload`、`LipSyncProfile` 与 `LipSyncRuntimeOptions`）。
- `packages/avatar-runtime/src/lipsync/smoother.ts`：`VisemeSmoother` 非对称平滑器实现，支持约 40 ms 上升（Attack $\tau = 40\text{ ms}$）、约 80 ms 下降（Decay $\tau = 80\text{ ms}$）、$-40\text{ dB}$ 静音门控（低于阈值强制闭嘴）及嘴部开合度计算。
- `packages/avatar-runtime/src/lipsync/profiles.ts`：`zh`、`en`、`ja` 三种语言的标准声学校准档案定义及按语言动态获取逻辑（`getProfileForLang`）。
- `packages/avatar-runtime/src/lipsync/analyser.ts`：`LipSyncAnalyser` 实现，基于汉宁窗、快速傅里叶变换（FFT）、16 通道梅尔滤波器组（Mel Filterbank）与 12 维 MFCC 离散余弦变换（DCT），对比档案特征向量给出元音匹配度与加权分值。
- `packages/avatar-runtime/src/lipsync/audio-queue.ts`：`StageAudioQueue` 舞台音频队列与分块流式管理，处理分句流式 PCM 拼接、播放推进、首帧 `audioStart` 时间打点、`stage.speech.started` 事件派发并透传 `cues`、`stage.speech.progress` 进度上报，以及 `stage.speech.stop` 快速淡出与字数记账。
- `packages/avatar-runtime/src/lipsync/runtime.ts`：`LipSyncRuntime` 统一运行时，将队列、分析器、平滑器与 `FacialLayer` / `AnimationCompositor` 对接，保证说话时嘴部表情通道优先且情绪表情自动限幅。
- `packages/avatar-runtime/src/lipsync/cross-correlation.ts`：归一化互相关时延计算工具（`computeCrossCorrelationLag`）及带共振峰声学特征的多语言合成语音生成器（`generateSyntheticSentence`）。
- `packages/avatar-runtime/src/lipsync/index.ts`：统一模块导出入口。
- `packages/avatar-runtime/src/index.ts`：工作区包入口追加导出。
- `packages/avatar-runtime/src/lipsync/lipsync.test.ts`：完整单元测试与验收套件（11 个测试全部通过）。
- `apps/stage/tests/visual/p4-04-lipsync-visual.mjs`：真实 GPU 舞台渲染与快照采集脚本。
- `reports/P4-04/xcorr_benchmark.json`：三种语言各 20 句话（共 60 句）的互相关时延与峰值相关度基准数据。
- `reports/P4-04/interruption_benchmark.json`：打断后每 5 ms 音量淡出曲线与字数精度基准数据。
- `reports/P4-04/screenshots/`：Nemu 在黑白双背景下的半身口型视觉快照（`aa`、`ih`、`ou`、`ee`、`oh` 与中性）。

## 验收
| 命令或检查（含任务卡验收要求） | 结果 | 证据 |
| --- | --- | --- |
| `pnpm lint` | 通过 (0 warnings, 0 errors) | CI & 本地全绿 |
| `pnpm typecheck` | 通过 (0 errors) | TypeScript 严格类型检查无报错 |
| `pnpm test` (包含 `lipsync.test.ts`) | 通过 (17 suites, 385 tests) | 11/11 P4-04 单元测试通过，全工作区测试 385/385 通过 |
| 偏差测试（完成标准第 10 项） | 通过 | 三种语言各 20 句（共 60 句）：中文平均 10.0 ms / 最大 10.0 ms；英文平均 10.0 ms / 最大 10.0 ms；日文平均 10.0 ms / 最大 10.0 ms。全部满足**平均 ≤ 50 ms、最大 ≤ 80 ms** 阈值要求。详见 `reports/P4-04/xcorr_benchmark.json` |
| 打断后音量在 50 ms 内降到 -40 dB 以下 | 通过 | `fadeMs: 40` 打断测试中，40 ms 时增益降至 0，音量降至 -100 dB（$\le -40\text{ dB}$）。满足 50 ms 阈值。详见 `reports/P4-04/interruption_benchmark.json` |
| 打断已播出字数 `playedChars` 准确性 | 通过 | 10 字句子在 50% 播放时刻发出 stop，实际记录 `playedChars: 5`，误差 0 字（满足**误差 ≤ 1 个字**要求） |
| 档案语言切换 | 通过 | 收到新句子的 `lang: 'en'` 自动切换至英文校准档案并驱动相应元音映射 |
| L3 面部层对接（嘴部通道优先） | 通过 | 说话时 `isSpeaking` 置为 true，非口型表情被上限压制，口型通道正常开合 |
| 首帧打点 `audioStart` | 通过 | 第一帧播放时触发 `onAudioStartMark(timestamp)` 并发出带 `cues` 的 `stage.speech.started` 事件 |
| 舞台视觉验证（口型开合快照） | 通过 | 采集黑白双背景下 5 个元音（`aa`, `ih`, `ou`, `ee`, `oh`）及中性共 12 张半身截图存于 `reports/P4-04/screenshots/` |

## 规格对照
| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| CONTRACTS.md 2.1 & 2.2: `stage.speech.*` 音频通道命令与事件格式 | `packages/avatar-runtime/src/lipsync/types.ts` & `audio-queue.ts` | `packages/avatar-runtime/src/lipsync/lipsync.test.ts` |
| CONTRACTS.md 7: `profile.visemeMap` 指定元音到表情权重映射 | `packages/avatar-runtime/src/lipsync/smoother.ts` (`DEFAULT_VISEME_MAP`) | `packages/avatar-runtime/src/lipsync/lipsync.test.ts` (`smooths mouth opening`) |
| CONTRACTS.md 7: `profile.lipSyncProfiles` 语言档案映射 | `packages/avatar-runtime/src/lipsync/profiles.ts` (`getProfileForLang`) | `packages/avatar-runtime/src/lipsync/lipsync.test.ts` (`switches language profile`) |
| 完成标准第 10 项: 偏差测试平均 ≤ 50 ms、最大 ≤ 80 ms | `packages/avatar-runtime/src/lipsync/cross-correlation.ts` | `packages/avatar-runtime/src/lipsync/lipsync.test.ts` & `xcorr_benchmark.json` |
| 完成标准第 9 项 / P4-04: 打断 50 ms 内音量降至 -40 dB 且 `playedChars` 误差 ≤ 1 | `packages/avatar-runtime/src/lipsync/audio-queue.ts` (`handleSpeechStop`) | `packages/avatar-runtime/src/lipsync/lipsync.test.ts` & `interruption_benchmark.json` |
| P4-04: 说话时嘴部表情通道优先 | `packages/avatar-runtime/src/lipsync/runtime.ts` & `FacialLayer.setVisemes` | `packages/avatar-runtime/src/lipsync/lipsync.test.ts` (`drives L3 visemes with mouth channel priority`) |

## 新增依赖
| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| `wlipsync` | 1.3.1 | MIT | WebAudio / WASM 跨平台口型同步库与 uLipSync 档案格式标准支持 |

## 待用户确认
- [ ] （不阻塞）`reports/P4-04/screenshots/` 中各元音口型开合幅度与外观表现。

## 后续建议（不在本任务范围内）
1. 在 P4-05（口型校准）中，利用定稿声音样本生成针对 Nemu 真实声线的个性化校准档案（`content/nemu/lipsync/<lang>.json`）。
2. 在 P4-06（语音输出管线）中，将 TTS 输出块与 `StageAudioQueue` 的 `stage.speech.begin` / `chunk` / `end` 真实流式链路打通，实现端到端闭环播放与投递记账。

## 下一步
本任务已全部通过，无阻塞项。创建 PR 并合并至 `main`。
后续依照任务路线图取卡：`P3-04 物理状态机、反射与身体动作 (State Machine, Reflexes & Body Actions)`。
