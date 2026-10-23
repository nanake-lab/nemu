# P2-05 尾巴、猫耳与弹簧骨调校 — 报告

## 结果
部分完成（见 BLOCKED.md）。三组尾形、三组弹簧预设及实际评审夹具已交付；用户最终选定 **A 偏粗短**，取代先前 B。A1.4/A1.5 结构错误为 0。手感、无高频抖动和正式管线挂接未完成，不签发整卡通过，不转为可审查。

## 交付物
- `tools/asset-pipeline/tail/`：候选配置生成器、候选尾巴阶段、完整候选管线入口及说明。
- `tools/asset-pipeline/springs/`：产物/权限/表情审计、逐关节速度证据摘要。
- `apps/stage/test/jitter/`：真实 three-vrm 30 秒固定步长拖动诊断、A 候选交互评审夹具和只写候选的保存 API；没有修改评审台生产源码。
- `content/_candidates/P2-05/`（独立私有内容仓库）：三种尾形×三组弹簧配置，共 9 份；5 个实际导出模型（3 尾形配标准预设，以及标准尾形配另两预设）；色板/比例与源 SHA、manifest、原始 VRoid 的逐字节候选快照、用户选择记录。正式 `content/nemu/build.yaml` 与模型没有改动。
- `tail-candidates-comparison.png`、`visual-v4/`：最终外形对照图及 18 张白/黑背景真实 GPU 评审台截图。
- `selected-A-review-controls.png`、`preview-controls-verification.json`：A 候选四类关节控件、预设实际参数和候选保存的核验。
- `pipeline-v4/<候选>/`：各阶段原始日志和全部 lint 诊断。
- `tasks/requests/P2-05-review-spring-inspector.md`、`P2-05-tail-stage-integration.md`：两项卡外正式代码的范围请求。

## 输入与保护
工作目录 `D:/Dev/nemu-p205`，任务分支 `task/P2-05-tail-springs`，最新 main 基线 `806e09b180ddbea015bd34f0bb868657980fbf4d`（包含 #47 ADR 0015）。主目录已有的 P2-03 分支、日志与研究报告保留。

新源模型只读：`D:/Dev/nemu/content/nemu/source/nemu-vroid-export.vrm`。SHA-256 在开工与当前均为 `d3f4799c37ecfdd1806ef93264b06f5d979b697a592d4422884046b594e1cb2b`。原源文件原有的未提交修改不纳入本卡提交；同一字节内容只复制到 `_candidates/P2-05/source/`，便于复现，不覆盖源文件。

bible 尾长范围按 1.58m 的 .35/.40/.45，最宽按 .12/.105/.09，分别为 55.3/63.2/71.1cm 与 18.96/16.59/14.22cm。色板自己标为 `draft_approximate_not_pixel_sampled`，没有把近似颜色说成原画取样。尾根位于骨盆后侧，曲线/宽度/颜色全部在候选配置，12 节尾骨。猫耳映射通过实际骨链及加权顶点范围核对：左 `J_Sec_Hair1_24/2_24`，右 `J_Sec_Hair1_23/2_23`；没有把其他头发骨随便标成猫耳。

用户最终原话：“粗短，我要这个，我刚才说错了”。A 选定仅作为后续外形基准；没有把自动化测试保存的 standard 配置当成用户认可弹簧手感。

## 契约相关修改（ADR 0015 第 1 条，单列）
- `export_vrm.py`：导出后分别检查 `licenseUrl == https://vrm.dev/licenses/1.0/` 与 `otherLicenseUrl == meta.contentLicenseUrl`；空内容许可拒绝。检查函数可在无 Blender 环境测试，实际 bpy 导出仍由原阶段执行。
- asset-lint A1.5：错误/缺失 VRM 公共许可地址报 E；缺失、空白或非字符串 `otherLicenseUrl` 报 E；修复建议使用正确字段。
- 合法夹具写不同的两种许可字段；8 个新增 Node 负向用例分别破坏一个字段且要求唯一对应错误。样例 `build.example.yaml` 改用 `contentLicenseUrl: LICENSE.md`，补空值负向测试。Python 同时覆盖导出检查的缺失、错值、空白及配置空值。
- 5 个实模型逐项比较 avatarPermission、commercialUsage、allowRedistribution、modification、creditNotation 及各使用限制，无差异。原模型已显式给出的限制全部保留；没有用新内容许可去放宽它们。LICENSE.md 沿用内容包“License pending”，`redistributable=false`，不自行决定最终授权。
- 没有改 CONTRACTS、原始规格、锁文件或新增运行时依赖。

## 候选专用处理与临时配置
初轮直接复制头发材质也复制了合并图集，尾巴新 UV 采到服装/皮肤/透明区，截图失败。只去贴图时，原本依赖图集的白色发光/阴影因子又使新尾巴变白。这些失败日志、截图与中间产物保留，未交作合格外形。

最终尾巴阶段在授权 `tail/` 内复用原几何算法，平滑中心线与网格法线，只对新尾巴去掉不匹配的图集和白色发光，使用色板 tailBase/hairShadow/outline，保留源头发 MToon 的其余响应。原模型材质没有列入调色配置。新尾巴是供选比例的纯色体块，未增加毛束细节。候选完整管线复用原全部阶段、profile、manifest 与原样 asset-lint；只替换候选尾巴步骤。**原 `pnpm asset:build` 入口尚未挂接该步骤**，已写范围请求，不能说正式通道已修复。

`expressions.preset/custom` 均为空；`materials` 只配置本卡新尾巴，源材质没有任何参数覆盖；`profile.emotionMap` 各项为空，`idleSet=[]`，口型路径仅为未生成的临时引用。相机/命中区域/耳尾读出范围明确为临时评审值，没有假装 P2-06 或口型卡已经完成。源表情 override 和自定义表情名称逐项审计保持不变。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| 开工源 `pnpm asset:lint ... --json` | 原规则 6 E / 3 W；4 E 是 A1.3，2 E 是 A1.4；未忽略原始非零退出码 | baseline-original.log |
| 5 个完整候选构建 | 所有阶段成功到 lint；每个全量 lint 为 4 E / 1 W，E 均为原 A1.3；A1.4/A1.5 E=0 | pipeline-v4/、product-audit-v4.json |
| 使用权限、表情定义与元数据检查 | 5/5 无权限/表情差异；公共许可固定，内容许可 LICENSE.md | product-audit-v4.json |
| `pnpm --filter @nemu/asset-lint test` | 33/33 通过 | license-tests-final.log |
| pytest（Python 3.12，既有工具环境） | 31/31 通过 | pytest-final.log |
| `mypy --strict` / ruff | 22 文件通过 / 通过 | mypy.log、ruff.log |
| `pnpm lint` / `pnpm typecheck` | 通过 / 通过 | lint.log、typecheck.log |
| `pnpm test` | 443/443 通过 | tests.log |
| 白/黑、正面/3/4/背面截图 | 18 张实际 RTX 3080 GPU 渲染；冻结循环、固定姿态/镜头；非视觉基准更新 | visual-v4/、对照图 |
| 30秒固定步长拖动 + 5秒回稳 | 5 套×171 个有效弹簧关节均无非有限数；仅为诊断，**未判无高频抖动通过** | visual-v4/jitter-summary.json、逐帧 json.gz |
| A 独立评审夹具 | 轻盈/标准/沉稳均实际修改尾关节 settings；保存后过原 schema；dragForce=2 被 400 拒绝 | preview-controls-verification.json |
| 源模型与正式内容保护 | 源 SHA 不变；原未提交源改动保持不纳入提交；正式 build.yaml/模型未改 | source-structure.json、内容仓库 diff |

全部 lint 的非零不是本卡全量通过。A1.3 三条眨眼 override 和八个自定义表情缺失，严格留给 P2-06。唯一 W 是 277 skin joints 超过 250 的目标；源为 215，新尾 12，另 50 是源节点导出时纳入 skin，没有移除源骨。见 `skin-joint-diff.json`。

## 抖动边界与调度
诊断按 120Hz，hips 在固定 Root 参考系内执行 x=.24*sin(2π1.3t)、y=.12*sin(2π2.1t)，头/脊柱同时受控转动，30秒后归零再回稳5秒。使用原 VRM 弹簧中心，不声称覆盖 OS 鼠标拖拽或正式身体输入通道。记录每个关节局部角速度和每秒 RMS。部分头发峰值达 316–366rad/s，A 尾巴峰值约81.47rad/s；不能用有限值或周期性本身替代高频抖动验收。原头碰撞体位置回代试验未解决全部高角速度，失败/未决数据保留。后续按 A 调校碰撞体并建立明确的频率证据，不自行放宽阈值。

已读取 P4-01 与 P4-02 最新状态：GPU 工作结束，正式重测等待另约；每次 Blender/浏览器前检查实际进程，未见活动测量或 Runner.Worker。重负载始终串行。`resource-window.json` 记录线程证据；候选入口也补进程守卫，但它不代替对预约窗口的确认。独立评审夹具默认暂停，播放会占用 GPU 35 秒，使用时不要重叠 P4 的后续安静窗口。

## 新增依赖
无新增应用运行时依赖。使用既有 Blender 5.2.2 / VRM Add-on 4.10.2、PyYAML/jsonschema、pytest/mypy/ruff、three-vrm 3.5.5、Vite、Playwright；对照图仅用桌面已带 Pillow 拼接截图。

## 待用户确认
- [x] 尾形最终选 A 偏粗短（取代 B）。
- [ ] （阻塞）四类弹簧手感选择/调整与认可；可使用独立评审夹具，保存只进入候选目录。
- [ ] （阻塞）实际抖动验收完成；当前高角速度证据未闭合。
- [ ] （阻塞）正式尾巴入口所需 `build_tail.py`/schema 范围扩展，或由对应维护方完成挂接；见请求文件。
- [ ] （不阻塞）正式评审台 spring-bones.ts 的旧 API 与导出片段不匹配，由维护方按请求修复；当前独立夹具可完成候选调校。

## 后续建议（不在本任务范围内）
P2-06 处理 A1.3、真实 emotionMap、源材质；口型卡填入真实三语档案；正式评审台维护方修复参数写入与 YAML 导出。没有顺手改其他卡的目录。

## 下一步
PR #48 保持草稿和 needs-user；不合并、不取下一卡。等待 A 的弹簧反馈，并继续本卡尚未完成的碰撞/抖动与正式挂接。全部验收完成后才合并最新 origin/main、重跑检查并转为可审查。

## 交付快照补充
独立评审夹具地址为 `http://127.0.0.1:5186/test/jitter/review.html`，默认暂停；保存会再次调用原 `build.schema.json` 和 `@nemu/contracts`，并登记候选 manifest。自动化保存的两份 user-tuned 文件是控件测试证据，不是用户选择。此前批量后台启动组合命令被客户端策略拒绝，随后分开写文件并使用可停止的前台进程已成功，不存在遗留权限阻塞。

最终 v4 逐帧原始证据以 `.json.gz` 随报告保存；无压缩原件和较早 v1/v3/头碰撞体试验的大体量逐帧记录仍留在本机 D 盘 `reports/P2-05/` 和工具 `.cache/`，按本目录 .gitignore 不提交。旧失败日志和白色发光/图集失败截图仍随报告保留。

内容候选快照已先推到私有 nemu-content 的 `task/P2-05-tail-springs`：`03742ab5a7350b565a1229b11d2730f656b52279`，6 个 LFS 对象共约73MB（5模型+源候选快照）。内容仓库只提交 `_candidates/P2-05/`；原 `nemu/source/nemu-vroid-export.vrm` 仍是原有未提交修改。主仓库随后更新本卡 gitlink；未合并内容分支或主 PR。

## 本次发布状态
主代码提交 `fbcd820ccd49372ab177f9e95c03dde2011f2b6d` 已推送到 PR #48；Ubuntu CI 运行中（39629389607），Windows 按草稿规则跳过，不称当前提交的 CI 全绿。根测试产生的八份临时进程 stdout 日志已按绝对路径核对后移入 `unit-test-process-logs/`，主目录原有 logs 不动。评审服务器仅为本卡可停止前台进程，默认暂停渲染；其保存动作不会推送或定稿内容。

频率诊断已补充到 `visual-v4/frequency-diagnostics.json`：取驱动第1–30秒、去掉角速度模长的直流分量，统计>15Hz能量。A 尾巴约13.6%，其余三类分别约0.1%（耳）、4.6%（头发）、0.4%（裙摆）。15Hz只是报告频段，不是新增或放宽的验收阈值；角速度模长还包含绝对值谐波和非线性接触，不能单凭该比值裁决通过。保留这些证据，后续优先定位尾链接触及速度尖峰。

## 2026-10-23 日常待机评审夹具

按用户要求，只在 `apps/stage/test/jitter/` 接入已合并 P3-01 的 `ProceduralLayer` 和 `FacialLayer`，复用呼吸、重心摆动、鼠标视线与眨眼；不修改生产评审台和动画运行时。可选缓慢转身仅为夹具程序化预览（16 秒周期、左右各 20 度），不是正式动作资产。无新增依赖、源模型或候选参数变更。

入口仍为 `http://127.0.0.1:5186/test/jitter/review.html`。默认暂停，勾选「开启待机」后连续播放；可再勾选「加入缓慢转身」。预设及滑块修改后恢复待机，比较从相同阶段开始；停止、关闭待机、隐藏页面均取消动画循环。每帧先恢复基础姿态，再叠加 L2/L3，将规范骨姿态传到实际骨骼后仅更新一次弹簧求解器，避免程序化偏移累计和双重物理更新。眨眼使用固定种子，便于三组预设复现比较。

已另查 P4-01/P4-02 会话最新状态：均空闲，P4-02 最新记录明确本次未启动 GPU、短测仍待另约窗口；P4-01 正式测量已结束。进程检查未见 Runner.Worker、Blender 或活动 Python 测量进程。浏览器检查与静态/单元检查串行。

浏览器集成证据：`idle-preview/checks.json`、`idle-preview-check.log`，三组预设各 900 帧固定步长；胸部呼吸最大旋转约 0.02094 rad、重心位移最大 0.005 m、转身约 0.3491 rad、眨眼最大权重 1，视线分级跟随、鼠标输入、切换预设保持待机、停止取消循环、诊断与待机互斥均通过。白/黑背景截图在 `idle-preview/`；截图是固定步长回放到 15 秒的定格，开关保持暂停，不是视觉基准更新。

本次只是可交互评审夹具接线通过，不能替代弹簧高频抖动验收和用户手感认可。PR #48 继续保持草稿和 needs-user；用户看过动态后再选择预设。静态与单元检查日志：`idle-preview-lint.log`、`idle-preview-typecheck.log`、`idle-preview-tests.log`。

## 2026-10-23 在「沉稳」基础上增加重量感候选

用户选择「沉稳」作为调校基础，并要求提高重力和阻尼、保持中等刚度，分部位观察跟随惯性与回稳。本轮保留原沉稳 A，另做 B 加重·留惯性、C 加重·快收稳两组候选；命名代表调校方向，最终手感尚未获用户认可。参数、配置路径、同动作视频与复现步骤见 [HEAVY-COMPARISON.md](HEAVY-COMPARISON.md)。

三组完整 build 配置经既有 Python 加载器校验，参数及配置仅写入 `_candidates/P2-05/springs/`，已登记来源/许可/不可分发。内容仓库提交 `ff52c045ce3f7af47429c5ff8f40884710b2b158` 已单独推送，未包含源 VRM 的原有未提交修改（源 SHA-256 仍为 D3F4799C37ECFDD1806EF93264B06F5D979B697A592D4422884046B594E1CB2B）。没有修改生产评审台或动画运行时，没有新增应用运行时依赖。录制使用机器已安装的 FFmpeg 9.0 作为开发工具。

单组夹具默认改为原沉稳，另提供 B/C；并排夹具默认暂停，三组共享固定动作时序、固定种子和固定物理步长。仅使用实际 `joint.settings` 的刚度、阻尼、重力进行调校；无独立质量参数、无给次级骨骼人为添加迟滞。首轮录制中断原件保留，关闭 HMR 后等待活动 Windows CI 结束再重录；此次对比不替代原定正式抖动验收，PR 仍保持草稿和 needs-user。

完整重录及检查已完成：1920×1080、24 秒、30 fps、720 帧 H.264 对比视频约 6.59 MB；ffprobe 全帧解码与浏览器播放通过，实际关节参数核验、同动作响应差异、播放暂停归零及单组新预设切换通过。白/黑背景最终截图已查看。`pnpm lint`、`pnpm typecheck` 与 443 项单测全部通过，日志及验证 JSON 均在本卡报告目录。用户现在可以直接在并排页面或视频中选择 A/B/C，也可分部位组合；此次未定稿任何新候选。

## 2026-10-23 用户选择 B

用户原话：「选B吧，其实我还是没看出来区别」。弹簧预设选择记录为 B `heavyBalanced`（加重·留惯性），作为后续调校和技术验收基础；尾巴外形仍为 A 偏粗短，两者编号不混淆。本条覆盖旧「等待弹簧预设选择」状态，无需用户再次比较三组。用户明确未感到明显区别，不据此宣称重量感或可感知改善已获认可，也不把选择替代正式抖动验收。

选择写入候选 user-selection.json 和参数汇总，单组评审夹具默认同步到 B；保存时全部 K/D/G 数值保持不变。此轮仅记录选择和默认值，核对 JSON、脚本语法和页面默认项，不重录或更改既有视频。候选未移入正式目录，正式管线挂接和技术验收仍待完成，PR 保持草稿。
