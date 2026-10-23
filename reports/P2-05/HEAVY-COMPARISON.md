# P2-05 加重弹簧候选对比

用户选「沉稳」作为基础；以下 B/C 尚未选定，全部保留在 `content/_candidates/P2-05/`。A 偏粗短尾巴外形继续沿用已选版本。本轮只改评审夹具、候选配置与报告；未改动画运行时、生产评审台、源 VRM 或正式 build.yaml。

## 观看

- 视频：`heavy-comparison/A-B-C_same-motion_24s_30fps.mp4`，1920×1080、30 fps、24 秒、原速、无音频。三列同时显示同一动作，参数始终列在各自下方。
- 在线入口：`http://127.0.0.1:5186/test/jitter/heavy-comparison.html`。默认暂停，可以播放或点击「对比视频」。
- 单组调校：`http://127.0.0.1:5186/test/jitter/review.html`，默认「A 原沉稳」，可切换 B/C。
- 0–4 秒待机呼吸，4–12 秒缓慢转身，12–15 秒快速拖动，15–24 秒停止拖动后的回稳；呼吸、眨眼全程继续。每组使用相同固定种子的 P3-01 L2/L3、同一机位/模型/光照、相同 4 秒预热、120 Hz 物理子步；没有给尾巴、猫耳、头发或裙摆人为添加延迟或另行驱动动画。视频为固定时间步离线逐帧录制，不按渲染耗时调整速度。

## 参数

K = stiffness，D = dragForce，G = gravityPower；重力方向统一向下，碰撞体、关节半径和模型均不变。

| 部位 | A 原沉稳 K / D / G | B 加重·留惯性 K / D / G | C 加重·快收稳 K / D / G |
| --- | --- | --- | --- |
| 尾巴 | 1.755 / .750 / .035 | 1.200 / .830 / .090 | .950 / .880 / .140 |
| 猫耳 | 2.295 / .750 / .000 | 2.100 / .880 / .006 | 2.000 / .920 / .010 |
| 头发 | 1.080 / .750 / .020 | .950 / .840 / .045 | .850 / .890 / .065 |
| 裙摆 | 1.350 / .750 / .040 | .950 / .860 / .085 | .800 / .900 / .120 |

B 以中等刚度配更高阻尼和重力，作为保留些跟随惯性的起点；C 再降低刚度，提高阻尼与重力，作为更快收住摆动的起点。猫耳刚度仍明显高于其他部位，只给少量重力。这些名称是调校方向，不是已获认可的手感结论。VRM 弹簧接口没有独立质量或「晚半拍」时间参数；本轮未修改求解器，也没有用动画假造迟滞。

候选完整配置：`springs/heavy-review-original-calm.yaml`、`springs/heavy-review-balanced.yaml`、`springs/heavy-review-damped.yaml`；参数汇总 `springs/heavy-review-parameters.json`，均通过既有配置加载器验证并登记来源和不可分发状态。原「沉稳」参数没有覆盖。

## 复现与证据边界

先运行 `node apps/stage/test/jitter/serve-review.mjs`，再单独运行 `node apps/stage/test/jitter/record-heavy-comparison.mjs`。准备候选用 `prepare-heavy-candidates.mjs`，只写候选目录与本卡 preview-data。所有工作目录和本轮浏览器临时目录均在 D 盘。

资源窗口另查 P4-01/P4-02 会话，均空闲；前者最近测量已结束，后者最新明确未启动 GPU、短测另约。录制前无 Runner.Worker/Blender 或活动测量进程。现有 Android bridge、看板和 Krita MCP 服务未终止。记录使用 RTX 3080 的 ANGLE D3D11；录制、静态检查、单元测试串行。

首轮在 270 帧之后发生开发服务器热重载（录制期间修改了另一夹具入口），浏览器导航销毁执行上下文，中断原始日志及未完成 MP4 保存在 `heavy-comparison/invalid-attempt-1/`，不作为交付视频。随后也发现 CI Worker 正在执行，因此等待其结束再从 0 帧重录；夹具服务器关闭 HMR，录制期间不修改画面代码。

`heavy-comparison/capture.json` 记录帧数、渲染器、分区关节数量、文件哈希及错误；`tip-samples.json` 保留三组逐帧端点坐标。白/黑背景截图与视频一起留在报告目录，早期 probe 截图保留原样。证据只用于比较候选，未签发高频抖动验收，也未改变任何阈值。仍需用户选择、后续正式管线与抖动工作；PR #48 保持草稿及 needs-user。

## 本轮检查结果

- 完整重录成功：720 帧、24.000 秒、1920×1080、H.264、30 fps、约 6.59 MB；`ffprobe -count_frames` 解码全部 720 帧通过。文件 SHA-256 在 `capture.json`。
- 浏览器实际播放 MP4 通过，MIME 为 video/mp4；三列实际尾巴参数与表格一致，相同动作下端点响应不同，播放/暂停/归零及单组 B/C 切换通过。证据 `heavy-comparison/verification.json`、`heavy-comparison-check.log`。
- `pnpm lint`、`pnpm typecheck`、`pnpm test`（29 文件、443 项）通过；日志 `heavy-comparison-{lint,typecheck,tests}.log`。白/黑背景的最终截图已人工查看；本轮没有更新任何视觉基准。
- 内容候选提交 `ff52c045ce3f7af47429c5ff8f40884710b2b158` 已推送，源 VRM 哈希仍与本轮开始一致。三份完整 build 配置保留原模板全部必需节，因此各超过 400 行；未拆减或改动其他卡的临时小节。

## 用户选择更新

2026-10-23 用户选择 B 加重·留惯性，并说明未明显看出区别。B 作为后续调校基础，单组夹具默认改为 B，三列对比及视频原件保持不变。无需用户再次比较；抖动技术验收和正式管线工作继续独立完成。
