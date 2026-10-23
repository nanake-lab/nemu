# Nemu 核心契约 v2.2

本文件定义 Nemu 各进程之间、代码与资产之间的全部接口。

- `packages/contracts`（TypeScript 类型 + zod 校验）逐条实现第 1、2、3、7 节，以及第 4.1 节的应用内总线。
- 终端与 Echo Core 之间的消息（第 4.2–4.4 节）、生成接口（第 5 节）、行动与权限（第 6 节）、主动边界（第 8 节）的权威定义在 Echo Core 的 IDL（`core/contracts/`，任务卡 P5-01）中。本文件规定 IDL 必须包含的 Nemu 专用消息种类和字段；TypeScript 类型由 IDL 生成，不得手写。
- Echo Core 本身的合同以 `docs/echo-core/` 为准，本文件不重复、不修改它们。

修改规则：

- **只增不改。** 已有字段、事件名、标记语法不能改名或改语义；需要变化时新增字段或新版本号。
- 任何修改都要先写 ADR（见 AGENTS.md 第 7 节），由用户批准。
- 版本号写在每条消息的 `v` 字段（应用内）或 `schema_version`（IDL）里。本文件为 v2.2：
  - v2 相对 v1：删除事件总线中的 Agent 事件和身体情绪模型，新增终端协议、身体读出和机会规则；
  - v2.2（P5-22，ADR 0007、0009–0011）：新增终端后继消息、三项身体读出、主动边界、成长通知与管理查询、载荷绑定审批及社交行动。旧消息和旧舞台输入保留解码；不静默改写已有字节。
  - v2.1（ADR 0005，只增）：应用内总线新增 `chat.send`、`memory.retract`、`core.query`、`core.queryResult`、`desktop.observation`、`codex.session`；观察事件新增 `user_retraction`；终端协议新增 `query` 与 `query.result`；运行状态新增谱系、诞生时间、自主行动开关；身体读出新增 `posture`（用户确认 E5 时明确包括姿态）。

---

## 1. 资产契约（Asset Contract）

运行时只接受满足本节规则的角色。`tools/asset-lint` 逐条检查：标 **E** 的违反即报错（构建失败），标 **W** 的只警告。

### 1.1 文件

| 规则 | 级别 |
| --- | --- |
| 格式为 VRM 1.0（`VRMC_vrm` 扩展，specVersion 1.0）。VRM 0.x 必须先经管线转换 | E |
| 脚底在 y = 0，面朝 +Z | E |
| 实测身高与 `nemu.profile.json` 的 `heightM` 相差 ≤ 2 cm（身高本身是设计决定，不设范围） | E |
| 三角面 ≤ 100,000 | E |
| 三角面 ≤ 70,000（目标值） | W |
| 单张贴图 ≤ 2048×2048 | E |
| 贴图显存估算（按 RGBA8 + mipmap 计算）≤ 256 MB | E |
| 贴图 ≤ 16 张 | W |
| 材质 ≤ 12 个 | W |
| 文件大小 ≤ 60 MB | W |
| 骨骼 ≤ 250 根 | W |
| 面部网格的形态键 ≤ 120 个 | W |
| 全部材质使用 MToon 1.0（`VRMC_materials_mtoon`），或显式标注的 unlit | E |

### 1.2 人形骨骼

VRM 1.0 规范的全部必需骨骼（E），以及以下可选骨骼在 Nemu 里也是必需的（E）：

- `chest`、`upperChest`（呼吸动画需要）；
- `neck`；
- `leftShoulder`、`rightShoulder`；
- `leftToes`、`rightToes`；
- 双手全部手指骨骼（手势需要）。

VRoid Studio 导出的模型默认包含这些骨骼。

### 1.3 表情

必需的预设表情（E）：

`neutral`、`happy`、`angry`、`sad`、`relaxed`、`surprised`、`aa`、`ih`、`ou`、`ee`、`oh`、`blink`、`blinkLeft`、`blinkRight`

覆盖规则（E）：

- `happy`、`angry`、`sad`、`relaxed`、`surprised`：`overrideMouth = "none"`。按 VRM 1.0 规范，`blend` 会让口型随情绪权重升高而被压低，`block` 会直接屏蔽口型，两者都会让她「笑着说话时嘴不动」。说话时嘴部的优先级改由运行时 L3 层处理：说话期间把表情权重上限压到 0.6（可在评审台调整）。
- `happy`、`relaxed`：`overrideBlink = "blend"`，防止笑眼和眨眼叠加导致眼皮穿插。
- `surprised`：`overrideBlink = "block"`。
- `aa`、`ih`、`ou`、`ee`、`oh`：`isBinary = false`。

视线：使用骨骼型 lookAt（`type = "bone"`），带合理的角度范围映射（W）。

Nemu 自定义表情（E），名称使用 camelCase，可以用形态键、材质颜色绑定或贴图变换绑定实现：

| 名称 | 含义 | 建议实现 |
| --- | --- | --- |
| `nemuSmug` | 得意 | 形态键组合 |
| `nemuCatMouth` | 猫嘴 ω | 形态键 |
| `nemuSparkle` | 星星眼 | 贴图变换绑定（切换瞳孔高光区域） |
| `nemuBlush` | 脸红 | 材质颜色绑定（脸颊叠加层） |
| `nemuTeary` | 泪光 | 贴图变换绑定 + 形态键 |
| `nemuSleepy` | 困倦 | 形态键 |
| `nemuPout` | 嘟嘴 | 形态键 |
| `nemuAngryMark` | 生气井字 | 材质颜色绑定（隐藏贴片显隐） |

这些表情是她可用的**表达词汇**，不是她的内部状态。何时使用由生成的表演提示和身体读出决定。

### 1.4 弹簧骨（`VRMC_springBone`）

| 链 | 要求 | 级别 |
| --- | --- | --- |
| 猫耳 | 左右各 1 条，每条 ≥ 2 个关节 | E |
| 尾巴 | 1 条，≥ 8 个关节，骨骼名记入 profile | E |
| 头发 | ≥ 6 条链 | W |
| 裙摆 | ≥ 8 条链 | W |

必需碰撞体（E）挂在以下骨骼上：

- `head`：球体；
- `neck`、`upperChest`、`leftUpperArm`、`rightUpperArm`、`leftLowerArm`、`rightLowerArm`、`hips`、`leftUpperLeg`、`rightUpperLeg`：胶囊体。

头发、耳朵、尾巴、裙摆的弹簧链都要引用包含这些碰撞体的碰撞体组。

### 1.5 元数据

`meta.name = "Nemu"`；`authors`、`licenseUrl` 必填（E）。`licenseUrl` 指向内容包的许可文件。

> **2026-10-21 追加（ADR 0015）：** 按 VRM 1.0 规范，`licenseUrl` 固定为 VRM 公共许可地址 `https://vrm.dev/licenses/1.0/`。上一句所说的内容包许可文件写在 `otherLicenseUrl`，必填（E）。

### 1.6 `nemu.profile.json`

与 `nemu.vrm` 放在同一目录，描述运行时需要、但 VRM 规范里没有的身体信息。

**这个文件由资产管线生成，不手工编辑。** 管线从模型上实测身高、头顶偏移和骨骼名，其余字段从 `content/nemu/build.yaml` 的对应小节读取。

```ts
interface NemuProfile {
  version: 2;
  model: string;                 // 相对路径，如 "nemu.vrm"
  heightM: number;               // 实测身高
  headTopOffsetM: number;        // 头顶到 head 骨骼的偏移，用于气泡定位
  hitRegions: Array<{
    id: 'head' | 'ears' | 'tail' | 'body' | 'handL' | 'handR' | 'skirt';
    bone: string;                // VRM 人形骨骼名或节点名
    shape: 'sphere' | 'capsule';
    radiusM: number;
    lengthM?: number;            // capsule 才需要
    offsetM?: [number, number, number];
  }>;
  gazeLimits: { yawDeg: number; pitchUpDeg: number; pitchDownDeg: number };
  framing: Record<'fullBody' | 'upperBody', {
    fovDeg: number; distanceM: number; targetBone: string; offsetM?: [number, number, number];
  }>;
  rig: {
    earBonesL: string[]; earBonesR: string[];
    tailBones: string[];             // 从根到尖
    earBackMaxDeg: number;           // 耳朵向后压的最大角度（由身体读出或表演提示驱动）
    tailSwingDeg: [number, number];  // 尾巴摆幅范围：[最小, 最大]
  };
  emotionMap: Record<Emotion, Array<{ expression: string; weight: number }>>;   // 表演提示 emo → 表情组合
  visemeMap: { A: 'aa'; I: 'ih'; U: 'ou'; E: 'ee'; O: 'oh' };
  lipSyncProfiles: Record<'zh' | 'en' | 'ja', string>;  // 每种语言的 wLipSync 档案相对路径
  idleSet: Array<{ motionId: string; weight: number; cooldownS: number }>;
}
```

v1 中的 `emotionImpulse` 与 `baseMood` 已删除：身体不再维护情绪状态（ADR 0002）。

### 1.7 内容包目录

`content/` 是独立的私有仓库 `nemu-content`，以 git 子模块挂在代码仓里，由它自己的 `.gitattributes` 负责 LFS 规则。代理生成、还没经过用户确认的文件放在 `content/_candidates/`。

AI 模型的锁定文件 `config/models.lock.json` 放在代码仓里，不放内容包，这样「不含 Nemu 的公开版」也能独立运行。

```text
content/nemu/
├─ LICENSE.md                 # Nemu 形象、声音与核心配置的授权
├─ bible/                     # 设定集：原画、三视图、表情表、色板、人设种子
├─ source/                    # nemu.vroid、贴图源文件（LFS）
├─ build.yaml                 # 资产管线配置
├─ nemu.vrm                   # 管线产物
├─ nemu.profile.json
├─ motions/
│  ├─ manifest.json
│  └─ *.vrma
├─ voice/
│  ├─ voice.json              # 音色 id、音色描述、每种语言 × 每种语气的参考音频、云端指令表
│  └─ ref/<lang>/*.wav
├─ lipsync/<lang>.json        # 每种语言的 wLipSync 档案
├─ core/
│  ├─ genesis/                # genesis 清单（θ 初值、x 初值、A_x、载体与 U/W、各读出函数），带版本和摘要
│  ├─ protocol/               # 协议 p：身份、说话风格、边界、表演提示语法、三种语言的规则
│  └─ exemplars/              # 模式 B 的语气样例库（J_text），带版本
└─ manifest.json              # 全部资产的来源与许可清单
```

`core/` 下的文件改动一律视为「定稿」（AGENTS.md 第 6 节合并规则）：genesis 清单只能通过显式迁移修改。

资产清单条目：

```ts
interface AssetManifestEntry {
  path: string;
  kind: 'model' | 'motion' | 'texture' | 'voice' | 'core' | 'other';
  source: string;            // 来源说明或 URL
  license: string;           // SPDX 标识或 "custom:<说明>"
  redistributable: boolean;  // 能否进入公开仓库或公开发布包
  author?: string;
  notes?: string;
}
```

### 1.8 动作槽位

动作库 v1 共 30 个槽位。槽位 id 是跨模块的稳定名称；每个槽位在 `motions/manifest.json` 中对应一个 VRMA 文件。三类使用方式：

- 物理状态机按 id 取片段；
- 生成的表演提示 `act` 只能使用标为「可表演」的 id；
- 核心的身体动作请求只能使用标为「可请求」的 id。

| 类别 | 槽位 id | 可表演（`act`） | 可请求（身体动作） | 使用者 |
| --- | --- | --- | --- | --- |
| 待机（4） | `idle_stand`、`idle_relaxed`、`idle_hands_behind`、`idle_look_around` | 否 | 否 | 物理状态机 |
| 待机小动作（6） | `fidget_stretch`、`fidget_yawn`、`fidget_tail_play`、`fidget_ear_scratch`、`fidget_hum`、`fidget_sway` | 否 | 是 | 待机反射、身体动作请求 |
| 说话手势（6） | `wave`、`nod`、`shake_head`、`think`、`explain`、`cheer` | 是 | 是 | 表演提示、身体动作请求 |
| 情绪反应（4） | `react_surprised`、`react_happy_hop`、`react_pout`、`react_sigh` | 是 | 是 | 表演提示、反射 |
| 桌面（4） | `sit_edge_loop`、`sit_to_stand`、`walk`、`dangle` | 否 | 否 | 物理状态机 |
| 状态（6） | `sleep_loop`、`wake_up`、`working_loop`、`peek_loop`、`fall_loop`、`land` | 否 | 否 | 物理状态机 |

---

## 2. 舞台协议（Stage Protocol）

舞台渲染进程与外界之间的消息。消息外壳：

```ts
interface StageMessage<T extends string, P> { v: 2; id: string; type: T; payload: P }
```

**三条通道**，都由主进程用 `MessageChannel` 创建，再把两端分发出去：

| 通道 | 两端 | 承载 |
| --- | --- | --- |
| 控制通道 | 主进程 ↔ 舞台 | 2.1 的命令（`stage.speech.*` 除外）和 2.2 的事件（`stage.speech.*` 除外） |
| 音频通道 | 语音进程 ↔ 舞台 | `stage.speech.*` 命令与事件 |
| 麦克风通道 | 舞台 → 语音进程 | `{ v: 2; seq: number; pcm: Int16Array }`，16 kHz 单声道，每块 20 ms |

消息本身与传输方式无关：它们都是可以 JSON 序列化的对象（音频数据为二进制数组）。如果将来把渲染进程换成非 Electron 程序，同样的消息可以改走本地 WebSocket 或命名管道。

### 2.1 命令（发给舞台）

| type | payload | 说明 |
| --- | --- | --- |
| `stage.loadAvatar` | `{ vrmUrl: string; profileUrl: string }` | 加载或替换角色 |
| `stage.setPlacement` | `{ displayId: number; anchor: Anchor; transitionMs?: number }` | 设定角色位置和锚点 |
| `stage.setPhysicalState` | `{ state: PhysicalState; reason: string }` | 强制切换物理状态，只用于调试和测试 |
| `stage.perform` | `{ cue: PerformanceCue }` | 立即播放一个表演提示 |
| `stage.action` | `{ actionId: string; action: BodyAction }` | 核心的身体动作请求（经主进程转发） |
| `stage.readout` | `{ readout: BodyReadout }` | 身体读出（第 7 节） |
| `stage.event` | `{ event: AppBusEvent }` | 主进程转发的应用内总线事件，范围见 2.3 |
| `stage.speech.begin` | `{ outputId: string; chunkIndex: number; sentenceId: string; text: string; lang: Lang; cues: PerformanceCue[]; sampleRate: number }` | 一句话开始（音频通道）；舞台在这句开始播放的那一刻触发 `cues` |
| `stage.speech.chunk` | `{ sentenceId: string; pcm: Float32Array }` | 音频分块，可转移对象（音频通道） |
| `stage.speech.end` | `{ sentenceId: string }` | 这句的音频发完了（音频通道） |
| `stage.speech.stop` | `{ fadeMs: number }` | 打断：淡出并清空队列（音频通道） |
| `stage.gaze` | `{ target: GazeTarget; holdMs?: number }` | 视线 |
| `stage.setQuality` | `{ fpsActive: number; fpsIdle: number; fpsSleep: number; msaa: 0 \| 4 }` | 画质与帧率 |
| `stage.mic.set` | `{ enabled: boolean; mode: 'pushToTalk' \| 'continuous' }` | 麦克风 |
| `stage.desktop.update` | `{ platforms: Platform[]; caret?: Rect; dnd: DndState }` | 桌面几何快照（节流后推送，只给身体用） |

```ts
type Lang = 'zh' | 'en' | 'ja';

type Anchor =
  | { kind: 'free'; x: number; y: number }                              // 物理像素，屏幕坐标
  | { kind: 'windowEdge'; hwnd: string; offsetX: number }               // 坐在窗口顶边
  | { kind: 'screenEdge'; side: 'left' | 'right' | 'bottom'; along: number }
  | { kind: 'taskbar'; along: number };

type GazeTarget = 'user' | 'cursor' | 'away' | 'window' | { x: number; y: number };

interface Platform { id: string; hwnd: string; x1: number; x2: number; y: number; z: number } // 可站立的水平边
interface Rect { x: number; y: number; w: number; h: number }

type PhysicalState =
  | 'idle' | 'listening' | 'speaking' | 'sitting' | 'walking'
  | 'dragged' | 'falling' | 'sleeping' | 'peeking' | 'hidden';

type BodyAction =
  | { kind: 'motion'; motionId: string }                          // 1.8 中标为「可请求」的槽位
  | { kind: 'goTo'; anchor: Anchor }                               // 走过去或移过去
  | { kind: 'sit'; platformId: string }
  | { kind: 'sleep' } | { kind: 'wake' }
  | { kind: 'peek'; side: 'left' | 'right' }
  | { kind: 'gaze'; target: GazeTarget; holdMs?: number };

type HitRegionId = NemuProfile['hitRegions'][number]['id'];
```

坐标约定：所有屏幕坐标都是**物理像素**、虚拟桌面坐标系；逻辑像素与物理像素的换算只允许在 `packages/desktop-sense` 和 `apps/desktop` 中出现。

### 2.2 事件（舞台发出）

| type | payload | 说明 |
| --- | --- | --- |
| `stage.ready` | `{ avatarName: string; boundsPx: Rect }` | 角色加载完成 |
| `stage.hitTest` | `{ overCharacter: boolean }` | 光标是否在角色像素上，主进程据此切换穿透 |
| `stage.hit` | `{ region: HitRegionId; kind: 'click' \| 'doubleClick' \| 'hoverStart' \| 'hoverEnd' \| 'pat' }` | 用户与角色交互 |
| `stage.drag` | `{ phase: 'start' \| 'move' \| 'end'; screenX: number; screenY: number; vx: number; vy: number }` | 拖拽 |
| `stage.bounds` | `{ screenRect: Rect }` | 角色在屏幕上的包围盒（节流到 20 Hz） |
| `stage.frameStats` | `{ fps: number; p95FrameMs: number; state: PhysicalState }` | 每 5 秒一次 |
| `stage.actionResult` | `{ actionId: string; status: 'succeeded' \| 'failed' \| 'cancelled'; detail?: string }` | 身体动作的结果，由主进程作为执行结果上报核心 |
| `stage.speech.started` | `{ sentenceId: string; at: number }` | 这句开始播放（音频通道） |
| `stage.speech.progress` | `{ sentenceId: string; playedMs: number }` | 播放进度（音频通道） |
| `stage.speech.done` | `{ sentenceId: string; interrupted: boolean; playedChars: number }` | 一句播放结束（音频通道）；`playedChars` 是实际播出的字数，用于投递回执 |
| `stage.error` | `{ code: string; message: string }` | 错误 |

### 2.3 转发给舞台的应用内总线事件

主进程把下列应用内总线事件包进 `stage.event` 转发给舞台，供物理状态机和反射使用。舞台只能读，不能发布这些事件。

`voice.vad`、`voice.bargeIn`、`output.state`、`core.runState`、`desktop.dnd`、`desktop.idle`

新增转发项属于「只增」变更，同样需要 ADR。

---

## 3. 表演协议（Performance Protocol）

生成的流式文本里插入表演标记。解析器把标记从文本中剥离，挂到所在的句子上。

### 3.1 语法

```text
[[kind:value]]  或  [[kind:value:intensity]]
```

| kind | value | intensity | 例子 |
| --- | --- | --- | --- |
| `emo` | `Emotion` 枚举 | 0–1，默认 0.6 | `[[emo:happy:0.8]]` |
| `act` | 1.8 中标为「可表演」的槽位 id | 无 | `[[act:wave]]` |
| `look` | `user` `cursor` `away` `screen` `window` | 无 | `[[look:user]]` |
| `pause` | 毫秒，≤ 2000 | 无 | `[[pause:400]]` |
| `voice` | `whisper` `excited` `calm` `sleepy` | 无 | `[[voice:whisper]]` |

```ts
type Emotion =
  | 'neutral' | 'happy' | 'angry' | 'sad' | 'relaxed' | 'surprised'
  | 'smug' | 'shy' | 'sleepy' | 'pout' | 'excited' | 'worried';
```

`Emotion` 是表达词汇，不是她的内部状态坐标。

### 3.2 解析规则

1. 标记永远不会被念出来，也不会显示在气泡字幕里。
2. 标记作用于它所在的句子；出现在句末标点之后的标记，作用于下一句。
3. 每句最多 3 个标记，多余的忽略。
4. 未知的 kind 或 value 忽略，并记一条 `warn` 日志，不能报错中断。
5. 解析器必须支持流式：遇到 `[[` 开头但还没闭合的片段时，最多缓冲 48 个字符；超过仍未闭合，就把它当普通文本处理。
6. 分句规则按语言：
   - 中文、日文：遇到 `。！？!?…～~` 或换行就切；单句超过 40 个字时，在最近的 `，、,；;` 处切；首句 ≤ 20 字时立即切出。
   - 英文：遇到 `. ! ? …` 后跟空白、或换行时切。小数点、常见缩写（`Mr.`、`e.g.`、`i.e.`、`etc.` 等，清单写在代码里并有测试）、省略号中间不切。单句超过 25 个词时，在最近的 `,`、`;`、`—` 处切；首句 ≤ 8 个词时立即切出。
   - 语言由该段文本的主要字符判定；混合语言的句子按出现的第一个句末标点规则处理。

### 3.3 数据类型

```ts
interface PerformanceCue {
  kind: 'emo' | 'act' | 'look' | 'pause' | 'voice';
  value: string;
  intensity?: number;
}

interface PerformanceSentence {
  id: string;
  outputId: string;
  index: number;
  lang: Lang;
  text: string;              // 已剥离标记
  cues: PerformanceCue[];
  ttsStyle: string;          // 语气键：该句 voice 标记的值；没有 voice 标记时取 emo 的值；都没有时为 "neutral"
}
```

`ttsStyle` 只是一个键。各 TTS 供应商自己决定怎样体现它：本地 Qwen3-TTS Base 模型按 `voice.json` 选用这种语言、这种语气的参考音频；支持文字指令的云端 TTS 按 `voice.json` 的指令表生成指令。

### 3.4 给模型的说明

表演标记的说明是协议 p 的一部分（`content/nemu/core/protocol/`），由 P6-01 的协议构建器生成。可用的 `act` 列表从动作清单的「可表演」槽位生成，不手写。

---

## 4. 应用内总线与终端协议

### 4.1 应用内总线

只承载界面层事件，不承载她的状态，也不是她的记忆。主进程是枢纽，负责转发和记录（环形缓冲，默认保留最近 5000 条，只用于调试）。

```ts
interface AppBusEvent<T extends string, P> {
  v: 2;
  id: string;            // ULID
  ts: number;            // 毫秒时间戳，统一用 performance.timeOrigin + performance.now()
  source: 'main' | 'stage' | 'panel' | 'voice';
  type: T;
  payload: P;
  corr?: string;         // 关联 id，如 outputId
}
```

| type | payload |
| --- | --- |
| `voice.vad` | `{ state: 'speechStart' \| 'speechEnd' }` |
| `voice.bargeIn` | `{}` |
| `voice.recognized` | `{ utteranceKey; text; lang: Lang; durationMs; confidence? }`（由主进程转成 4.2 的观察事件） |
| `output.state` | `{ outputId; state: 'streaming' \| 'speaking' \| 'done' \| 'cancelled' }` |
| `output.chunk` | `{ outputId; chunkIndex; text; cues }`（给面板显示用的镜像） |
| `core.notice` | `{ noticeId; kind; at; summary; lRef }`（`lRef` 为 L 对象的可解析引用字符串，主进程从 IDL ObjectRef 转换；只通知，无回应字段） |
| `growth.control` | `{ action: 'pause' \| 'resume' }`（二次确认后交 P5-05 类型化管理入口；不含复核结论） |
| `approval.request.v2` | `{ approvalId; actionId; tier; title; detail; expiresAt; actionPayload: Uint8Array; payloadDigest: Uint8Array }`（原样载荷和 32 字节摘要，替代新审批的旧总线消息） |
| `approval.decision.v2` | `{ approvalId; decision; payloadDigest: Uint8Array }`（必须回传所见载荷的摘要） |
| `core.runState` | `{ states: string[]; speechMode: SpeechMode; strict: boolean; lineage: Lineage; bornAt?: string; autonomy: 'active' \| 'paused'; reasons: string[] }` |
| `approval.request` | `{ approvalId; actionId; tier: ToolTier; title; detail; expiresAt }`（给面板的审批卡） |
| `approval.decision` | `{ approvalId; decision: 'allowOnce' \| 'allowAlways' \| 'deny' }`（面板 → 主进程 → 核心） |
| `chat.send` | `{ text }`（面板里打字对她说话；主进程转成 4.2 的 `user_text` 观察事件） |
| `memory.retract` | `{ targetRefs: string[]; note? }`（面板里让她忘掉某条记录；主进程转成 4.2 的 `user_retraction` 观察事件） |
| `core.query` | `{ queryId; kind: QueryKind; filter?: QueryFilter; limit: number }`（面板 → 主进程 → 核心的只读查询） |
| `core.queryResult.v2` | `{ queryId; kind: QueryKind; status; items: QueryItem[]; growthEntries: EchoCore.LedgerEntry[] }`（新返回入口，保留完整 L；IDL 原样类型，经结构化克隆传输） |
| `core.queryResult` | `{ queryId; status: QueryStatus; items: QueryItem[] }`（核心 → 主进程 → 面板） |
| `desktop.dnd` | `{ state: DndState }` |
| `desktop.idle` | `{ idleMs }` |
| `desktop.observation` | `{ kind: DesktopObservationKind; processName?; title?; dnd?: DndState; idleMs? }`（桌面感知发出的低频语义事件；主进程转成 4.2 的 `desktop` 观察事件，`title` 只在 T1 授权后填写） |
| `codex.session` | `{ event: 'SessionStart' \| 'PermissionRequest' \| 'Stop'; sessionId; cwdLabel?; summary? }`（用户自己的 Codex 会话钩子，默认关闭；主进程转成 4.2 的 `codex_session` 观察事件） |
| `metrics.latency` | `{ utteranceKey; marks: LatencyMarks }` |
| `stage.ready`、`stage.hit`、`stage.drag`、`stage.frameStats`、`stage.error` | 同第 2 节，由主进程从控制通道转发 |

```ts
type DndState = 'none' | 'fullscreen' | 'presentation' | 'busy' | 'meeting' | 'quietHours';
type ToolTier = 'T0' | 'T1' | 'T2' | 'T3';
type SpeechMode = 'A_LOCAL_STRICT' | 'B_REMOTE_PROJECTED' | 'C_DRAFT_REWRITE';
type Lineage = 'REHEARSAL' | 'LIFE';                 // 谱系（ADR 0005）
type DesktopObservationKind =
  | 'foreground_changed' | 'idle_started' | 'idle_ended' | 'dnd_changed'
  | 'session_locked' | 'session_unlocked' | 'meeting_started' | 'meeting_ended';
type QueryKind = 'memory' | 'actions' | 'run_history' | 'growth';
type QueryStatus = 'ok' | 'no_match' | 'index_lagging' | 'denied' | 'object_missing' | 'budget_exhausted';
interface QueryFilter { text?: string; from?: string; to?: string; kinds?: string[]; includeRetracted?: boolean } // includeRetracted 只用于审计视图
interface QueryItem { ref: string; at: string; kind: string; summary: string; source: string; retracted: boolean }

interface LatencyMarks {
  vadEnd?: number; asrDone?: number; received?: number; committed?: number;
  firstChunk?: number; ttsFirstChunk?: number; audioStart?: number;
}
```

面板始终分别显示：谱系（排练/正式）、说话模式与严格性、情绪（接通，或 m = 0 及原因）、成长（排练中关闭、开启、已暂停、复核中）。`run_state` 的业务字段不增改，状态通过 `states[]` 表达：`NO_GROWTH` 配 `GROWTH_OFF_REHEARSAL`、`GROWTH_PAUSED`、`GROWTH_REVIEW_PENDING`，情绪降级时显示 `MODULATION_NOT_ENABLED` 或 `CORE_UNMODULATED` 及原因。没有关闭或暂停标记且正式谱系完整运行时显示成长开启。多个状态取能力交集，不能用单个「正常」掩盖其他状态。

`approval.request` / `approval.decision` 的旧总线载荷保留兼容，新审批使用 `.v2` 后继事件，旧载荷不能用于新的执行授权。ADR 0017 的语音早期压音量事件尚未授权，本卡不添加该事件。

`QueryResultV2` 经 `queryResultToPanel(query, result)` 映射到 `core.queryResult.v2`：核对 query_id，明确映射状态和查询种类；普通 QueryItem 仅去除 schema_version，全部业务字段保留；growth_entries 逐条用生成的 LedgerEntry 校验并原样放入 growthEntries（包括证据引用、bigint 与 Uint8Array）。growth 时 items 必须为空，其余查询时 growthEntries 必须为空。旧 core.queryResult 保留兼容，新成长结果不能经旧事件丢弃 L 后发送。

### 4.2 终端 → 核心：观察与执行结果

全部使用 Echo Core 的 EventEnvelope（字段见 v0.6.4 第 4 节、总方案第 9 节）。终端为每条记录生成稳定的 `stable_source_key`（ULID），重发时必须复用，由 Γ 做幂等判定。终端只报告发生了什么，不解释「这件事应使她改变多少」。

| kind | 子类型 | payload 字段 | 来源认识论类型 |
| --- | --- | --- | --- |
| `observation` | `user_utterance` | `text`、`lang`、`asr_confidence`、`duration_ms`、`audio_ref?` | 用户陈述（由 IRL 细分为 user_assertion 或 user_self_report） |
| `observation` | `user_text` | `text` | 同上 |
| `observation` | `user_retraction` | `target_refs[]`（事件或记录的引用）、`note?` | 用户陈述（纠错）。由事实视图的纠错链确定性处理，不需要 IRL（E10） |
| `observation` | `touch` | `region`、`kind`（click、doubleClick、pat、dragStart、dragEnd、fling）、`speed?` | system_record |
| `observation` | `desktop` | `kind`（foreground_changed、idle_started、idle_ended、dnd_changed、session_locked、session_unlocked、meeting_started、meeting_ended）、`process_name?`、`title?`（需要 T1 授权）、`dnd?`、`idle_ms?` | system_record |
| `observation` | `codex_session`（可选，需授权） | `event`（SessionStart、PermissionRequest、Stop）、`session_id`、`cwd_label?`、`summary?` | tool_output |
| `execution_result` | `speech_delivery` | `action_id`、`output_id`、`delivered_chunks`、`played_chars`、`status`（DELIVERED、PARTIAL、CANCELLED、UNKNOWN_DELIVERY） | system_record |
| `execution_result` | `body_action` | `action_id`、`status`、`detail?` | system_record |
| `maintenance` | `terminal` | `kind`（started、stopping、mode_switched、clock_epoch）、`detail` | system_record |

高频几何信息（窗口矩形、平台、光标插入点、拖拽过程中的坐标）只在终端内部使用，不作为事件发给核心。

### 4.3 核心 → 终端

| 消息 | 字段 | 说明 |
| --- | --- | --- |
| `receipt` | `stable_source_key`、`stage`（RECEIVED_DURABLE、CORE_COMMITTED、FILTERED、REJECTED）、`event_id?`、`reason?` | 两种回执必须区分：前者只表示已持久化接收 |
| `output.chunk` | `output_id`、`action_id`、`chunk_index`、`text`（含表演标记）、`lang`、`snapshot_id`、`speech_mode`、`strict` | 一次生成的流式输出块；语音进程负责分句与合成 |
| `output.end` | `output_id`、`reason`（done、cancelled、error） | 输出结束 |
| `action.request` | `action_id`、`credential`、`action`（BodyAction）、`expires_at` | 身体动作请求；终端执行前检查凭证未过期 |
| `approval.request`（IDL `ApprovalRequestV2`） | `approval_id`、`action_id`、`tier`、`title`、`detail`、`expires_at_ns`、`action_payload`、`payload_digest` | 行动载荷原样显示；后继替代旧 ApprovalRequest，新请求不可省略载荷与摘要 |
| `run_state` | `states[]`、`speech_mode`、`strict`、`lineage`（REHEARSAL、LIFE）、`born_at?`、`autonomy`（active、paused）、`reasons[]` | 运行状态变化时推送 |
| `readout`（IDL `ReadoutV2`） | `body` oneof（`v1: BodyReadout` 或 `v2: BodyReadoutV2`）、`snapshot_id`、`projected_at_ns` | 必须且只能选择一项；P5-19 起核心只发 v2，舞台 P3-06 前旧入口保持可用 |
| `notice` | `notice_id`、`kind`（review_hold、slow_drift、growth_paused、growth_resumed、migration、law_version）、`at`、`summary`、`l_ref: ObjectRef` | 核心按 L 条目确定性生成摘要；只有通知，没有回应或投票字段 |
| `query.result`（IDL `QueryResultV2`） | `query_id`、`status`、`items[]`、`growth_entries: LedgerEntry[]` | 只读查询的结果（`QueryStatus`、`QueryItem` 见 4.1）。`status` 必须区分无命中、索引落后、无权限、对象缺失、预算耗尽，不能都返回空集（SYS05） |

终端 → 核心另有以下控制消息（旧消息保留解码，新功能使用后继消息）：

- `approval.decision`（IDL `ApprovalDecisionV2`：`approval_id`、`decision`、`payload_digest`）：与请求及实际待执行载荷摘要不一致即拒绝；
- `output.cancel`（`output_id`）：打断时发送，随后补发 `speech_delivery` 执行结果；
- `growth.control`（`GrowthControl`：`action` 只允许 pause、resume）：面板二次确认后由 P5-05 认证的类型化管理命令执行。复核结论只用 `echoctl growth review`（P5-15），没有按结果投票的面板入口。
- `query`（IDL `QueryV2`：`query_id`、`kind`、`filter?`、`limit`）：类型化的只读管理查询（总方案第 21 节），用途为 `user_view`，留审计，不改变任何状态。`memory` 只返回能进入 c_task 的内容；`actions` 返回 Q 中的行动记录（含被拒绝的）；`run_history` 返回运行状态与模式切换；`growth` 返回 L 条目（R、θ 变化和证据组引用、重算、迁移、暂停、恢复与复核），不能送入 c_core/c_task。非 growth 查询的 `growth_entries` 为空，growth 查询的 `items` 为空；状态仍需区分错误与无命中。

IDL 兼容：`ReadoutV2`、`RunStateV2`、`QueryV2`、`QueryResultV2`、`ApprovalRequestV2`、`ApprovalDecisionV2` 使用新冻结类型标签，旧字段原号保留或 reserved；新字段号大于前任最大号。`RunStateV2` 改用只含 REHEARSAL/LIFE 的 `TerminalLineage`，不把内部 EXPERIMENT 暴露给终端；旧 RunState 仍可解码历史数据。QueryKindV2 新增 growth，旧 QueryKind 不改。所有 schema_version 仍为 1，新语义由新类型标签区分。

### 4.4 投递语义

- 已经播出的内容是已发生的输出，不能在之后假装撤回。打断时，终端如实回报实际播出的范围。
- 终端无法确认是否播出时，回报 `UNKNOWN_DELIVERY`。
- 结构重放不会重新发送任何输出块。

---

## 5. 生成接口、说话模式与助手

### 5.1 生成接口

```text
generate(TaskInput, ModulationEnvelope) -> PlanAndCarrierReceipt
```

- **TaskInput：**
  - 当前输入 o；
  - 协议 p 的版本与内容；
  - c_task（FactProjection）；
  - 快照 id；
  - 最大 token 数与最长时长。
- **ModulationEnvelope：** 按《核心算子》第 13.2 节。
- **PlanAndCarrierReceipt：**
  - 输出块序列；
  - 可选的行动计划；
  - `speech_mode`、`strict`；
  - 模型与 tokenizer 哈希；
  - 载体回执（模式 A、C）或投影版本（模式 B）；
  - 中间结果引用（模式 C 的草稿）；
  - 用量。

三种模式的定义、限制与验收见 ADR 0003。任何模式下，生成都不得读取 x、R、θ、H 的原始内容；模型只能看到 o、p、c_task，以及 m 或它的投影。

### 5.2 文字投影 J_text（模式 B）

```ts
interface ExemplarBank {
  version: string;
  metric: 'l2';                      // 固定度量
  k: number;                         // 每次选取的样例数
  entries: Array<{ id: string; m: number[]; lang: Lang; text: string; neutral: boolean }>;
}
// projectText(m, lang, bank) -> { exemplarIds: string[]; promptBlock: string; projectionVersion: string }
```

纯函数，相同输入得到相同输出；结果写入生成回执。

### 5.3 助手与社交行动

| 行动种类 | 参数 | 默认档位 | 默认权限 |
| --- | --- | --- | --- |
| `helper.ask` | `question`、`context_refs[]`、`tier?` | `everyday` | T0（不读个人数据时）或 T1 |
| `helper.task` | `goal`、`inputs[]`（经授权的文件或文档引用）、`tier?`、`workspace` | `work` | T2；其中的命令执行与对外操作按 T3 逐项审批 |
| `helper.draft` | 模式 C 内部使用，不作为独立行动 | `everyday` | — |
| `social.message` | `SocialAction` 的 kind 与原样 action_payload；她以 Nemu 身份发出的消息 | — | T2，可设为信任 |
| `social.commit` | `SocialAction` 的 kind 与原样 action_payload；约定、付款、代表 u* 表态等会让 u* 承担的事 | — | T3，不能设为信任 |

档位到模型的映射在 `config/helpers.json`：

```json
{ "everyday": "gpt-6-luna", "work": "gpt-6.1-sol", "hard": "gpt-6-astra",
  "provider": "codex-app-server", "budgets": { "autonomousPerDay": 30 } }
```

模型名以 Codex 当前提供的为准，由 P0-08 核对后写入。

### 5.4 语音供应商接口

```ts
interface TtsProvider {
  id: string;
  synthesize(req: { text: string; lang: Lang; style: string; voiceId: string; sampleRate: 24000 }, signal: AbortSignal):
    AsyncIterable<Float32Array>;   // style 即 PerformanceSentence.ttsStyle
  warmup?(): Promise<void>;
}

interface AsrProvider {
  id: string;
  start(opts: { lang: 'auto' | Lang; mode: 'pushToTalk' | 'continuous' }): AsrSession;
}
interface AsrSession {
  feed(pcm: Int16Array): void;
  onVad(cb: (s: 'speechStart' | 'speechEnd') => void): void;
  onResult(cb: (r: { text: string; lang: Lang; confidence?: number }) => void): void;
  close(): void;
}
```

---

## 6. 行动与权限分级

所有对外行动都经过 Echo Core 的行动闸门：说话、身体动作、助手、Office、文件、网络。行动闸门按以下分级决定是否需要用户（权限管理由 W22 实现）。

| 级别 | 定义 | 运行规则 |
| --- | --- | --- |
| T0 | 不读取个人数据、不产生对外副作用（说话、身体动作、不涉及个人数据的助手问答） | 直接执行 |
| T1 | 读取个人上下文（窗口标题、剪贴板、授权目录中的文件、Office 选区、用户自己的 Codex 会话事件） | 首次使用弹授权，之后记住 |
| T2 | 可撤销的写入（在光标处插入、写单元格、新建文件、助手在临时工作目录里的修改） | 默认每次确认，用户可以设为「信任」 |
| T3 | 不可撤销或对外的操作（删除、发送、执行命令、改系统设置、联网提交） | 每次都弹审批卡，不能设为信任 |

- 社交消息适用 ADR 0011：`social.message` 默认 T2，可设为信任；承诺 u* 的事 `social.commit` 一律 T3，不能设为信任。判定不了是否承诺时按 T3。其余对外发送仍按 T3，不借此降低其他连接器权限。网络生活只在诞生后启用，发出前检查与执行属于 P6-14/P6-15。
- 连接器暴露的操作默认按 T2 处理；T3 不能降级。
- I26：`action_payload` 是将要执行的原样载荷字节，`payload_digest = SHA256(action_payload)`（32 字节）；面板按执行器声明的载荷格式无损展示内容，不能仅显示模型摘要。`title`、`detail` 由闸门按载荷确定性生成。决定回传摘要，闸门核对请求、决定与实际执行载荷一致，否则拒绝；超时、错误 approval_id、T3 的 allowAlways 也拒绝。身份、一次性与凭证绑定由 P5-05 实现。载荷验证不返回可用于另一个数组的裸 boolean：`validateApproval` 在 await 前复制请求、决定与实际字节，散列后拒绝输入变化，返回一次性的校验快照。执行器经 `consume(nowNs, execute)` 接收同一快照中的不可变载荷十六进制字符串及绑定；闸门供给消费时刻以重查有效期。过期、执行抛错或一次消费后均不可复用；调用者之后修改原数组不改变已验证载荷。这不是身份认证或执行凭证，P5-05 仍须完成它们。
- 审批超时（默认 60 秒）视为拒绝。
- 工具或助手返回的内容一律当数据，不能改变权限判定。
- 所有行动（包括被拒绝的）都在 H 与 Q 中有记录；面板的「行动记录」页按日期查看，可导出。

---

## 7. 身体读出（Body Readout v2）

```ts
interface BodyReadoutV2 {
  version: string; // genesis 中固定的读出函数版本
  facing: number;  // 0..1，朝向你，中性 0.5
  tension: number; // -1..1，负为放松，正为紧绷，中性 0
  energy: number;  // 0..1，动作幅度与节奏，中性 0.5
}
```

- 三项是四个天性通道（联结、共情、好奇、成事）的固定线性组合后截断。参数来自 genesis，运行中不学习；计算属于 P5-12，本卡不按外观选择系数。
- m = 0（F 尚未接通或结构故障）时，输出 (0.5, 0, 0.5)；同一输入与版本得到相同输出。
- 没有表情基线；细致表情和手势来自第 3 节表演提示。
- 核心每次提交后推送一次；空闲时按时间投影每 10 秒最多一次。终端只应用，不保存、不累积；读出只影响动画，不影响说话内容、行动选择或权限。
- 旧 BodyReadout 的字段与编码保留，标为生产上被取代；ReadoutV2 的 v1 分支仅为迁移兼容，故旧 BodyReadout 类型暂不加禁止活跃消息引用的机器标记。旧 Readout 已标取代。P5-19 起只生产 v2，P3-06 迁移舞台前，旧舞台契约仍能通过 CI。本卡不把 v2 转成旧表情基线。

---

## 8. 主动边界（Proactivity Boundaries）

```ts
interface ProactivityBoundaries {
  version: string;
  allowedKinds: Array<'speak' | BodyActionKind | 'helper.ask'>;
  cooldownS: Partial<Record<string, number>>;
  gates: Array<'dnd' | 'quietHours' | 'budget' | 'mode' | 'minInterval'>;
  budgets: { minIntervalS: number; speakPerHour: number; quietDayFactor: number; autonomousHelperPerDay: number };
}
```

`config/opportunities.json` 是固定、版本化的边界表，修改需要 ADR。权威消息为 terminal.proto 的 ProactivityBoundaries、ProactivityBudgets、ProactivityCooldown，种类与门控由 ProactiveKind/ProactiveGate 定义。字段限制、JSON 枚举拼写、camelCase 名称及 cooldownS 映射均在 IDL 注解中：资源省略 schema_version，投影时补 1；cooldownS 的字典投影为按 kind 为键的 cooldowns 集合。TypeScript 资源类型由 IDL 生成，校验器按同一 descriptor 构造；`proactivityToWire` 转换后再次校验权威消息，禁止另起手写规则。次数预算以 uint32 记录，秒数与 quietDayFactor 为有限 binary64，范围按 IDL。它只规定允许的行动种类、冷却、门控和预算；没有 trigger、candidate 或「什么情况做什么」。主动强度来自天性的固定读出，乘运行门控后 thinning 调度（P5-20）；模型在 m 下提出做什么或什么也不做（P6-01）。不得把这张表用作行为脚本。

默认最小间隔 180 秒、每小时说话 2 次、安静当天预算系数 0.5、自主助手每日 30 次（沿用本文件 5.3 的 helpers 预算）。当前仓库还没有 config/helpers.json；该文件由其所属卡创建时必须与此值一致。冷却表先为空，不从旧触发规则推断每类行动的新冷却；共用最小间隔与预算照常生效。允许的身体种类沿用 BodyActionKind 的 motion、goTo、sit、sleep、wake、peek、gaze。

- 勿扰不为 none 或处于安静时段时，所有主动说话候选拒绝；权限、模式、预算、最小间隔门控仍由运行时执行，不交给模型。
- 最小间隔内的候选拒绝，不推迟到间隔末尾。
- 原 helper_done 归助手任务后续（P6-06），说出口时仍受门控；codex_needs_you 产生一次立即主动时机，仍受全部门控（P5-20）；其余七条触发规则删除，由她的主动产生行为。拖拽导致离开睡眠属于物理状态转换（P3-04）。

---

## 追加（2026-10-20）：v2.0 对本文件的影响

本节只追加，上文保留。v2.0 核心规格（ADR 0009、0010、0011）合并后，下列各节由任务卡 P5-22 改写，改写需要用户批准。在那之前，与 v2.0 冲突的地方以 v2.0 为准：

- **第 7 节 身体读出：** 只保留朝向你、放松或紧绷、精力三项，由天性通道的固定线性组合得出；没有表情基线，表情只来自表演提示（v2.0 第 9.3 节，差异登记 V2-05）。
- **第 8 节 机会规则：** 改为主动边界表，只写允许的行动种类、冷却、门控与预算，不写「什么情况做什么」（v2.0 第 9.4 节，V2-06）。
- **4.1、4.3：** 运行状态的显示增加情绪与成长；新增通知 `notice`、查询种类 `growth`、成长的暂停与恢复；审批请求带行动载荷的摘要（I26）。
- **5.3、第 6 节：** 新增网络生活的社交行动种类，让你承担的事一律按 T3 审批（ADR 0011）。

## 追加（2026-10-22）：P5-22 兼容与迁移说明

上面的 2026-10-20 追加作为历史授权依据保留。v2.2 的契约修订随本 PR 请求用户批准；审批前保持 needs-user，不自行合并。

- 既有 CanonicalV1 字节、类型标签与旧黄金向量不变；无存储被打开、重建或迁移。上线前依 ADR 0007 做备份并选择支持后继消息的双方版本；回退时恢复旧应用和备份，后继字节保留原标签，绝不冒充旧消息解码。
- `ReadoutV2.body` 按 ADR 0007 §6 编码：选中字段号 u32 小端，再接该值，不写每个成员的 presence 字节；必填缺失、多选、未知字段号、尾随字节拒绝。其余编码规则不变。
- 后继审批载荷字节与 SHA-256 绑定的具体表示在 PR 的规格对照和差异登记中供 Claude 与用户审查。
