# P3-06 读出观察材料

以 `REPORT.md` 为验收结论。`preview-final/` 是供用户看的白/黑背景最终候选；最终交付视频在 `videos-final/`，人工观看不阻塞。

| 组 | facing | tension | energy |
| --- | --- | --- | --- |
| A | 0.85 | -0.65 | 0.25 |
| B | 0.25 | 0.70 | 0.85 |

映射版本 `body-animation-v2-candidate-1`，参数是候选初值、待校准。最终由用户或 P5-17 确定；没有更新正式内容包或 genesis。

最终视频：

- [A：180.375 秒](videos-final/readout-A_facing-0.85_tension--0.65_energy-0.25_180s.webm)
- [B：180.452 秒](videos-final/readout-B_facing-0.25_tension-0.7_energy-0.85_180s.webm)

## 观察条件与限制

- 使用已有 Nemu 模型/profile，只读提供给本机无界面 Chromium；资产哈希见各次 manifest。
- 源 profile 的 `idleSet` 为空。因此录制脚本装入一个仅供观察的合成基础姿态（双臂放低，固定片段），不把它登记为正式动作库或已选定动作。呼吸、重心、视线和耳尾仍由真实动画层与两组三项读出驱动。没有以视频证明正式动作库、小动作选择或整日自然程度。
- 使用 SwiftShader；它是可复现的功能/视觉观察，不是 RTX 3080 性能验收。
- `preview/` 保留首次 T 姿势/脚部被工具栏遮挡的观察；`preview-idle/` 保留合成夹具手臂旋转方向错误的观察，不能作为交付画面。`preview-final/` 修正符号、取景并增加“双手低于上臂”的几何断言，不能删去前面的原始材料冒充一次成功。
- `visual/golden/comparison.json` 是原有 18 组舞台视觉回归的比较数据；基准没有修改。
- `videos/` 是未通过的首次录制：墙钟计时 180 秒，但解码只有 179.602/179.539 秒，且透明 WebGL 画布录制有边缘花块。`checks/videos-final.log` 是发现花块后中断的一轮，只录了约 30 秒，不计验收。`video-probe/` 是修正后约 4 秒的短片能力验证，不是三分钟交付。
- 最终录制先把 WebGL 画布合成到不透明 2D 画布，使用浅灰白底；增加一秒缓冲以补偿编码首帧延迟。`videos-final/verification.json` 核对文件哈希、实际解码时长至少 180 秒，并保存 0.1、90、179 秒抽帧；原始失败视频不冒充最终交付。

## 复现

所有路径/环境变量只对当前 PowerShell 进程生效，未写用户级或系统级设置。

```powershell
$env:PLAYWRIGHT_BROWSERS_PATH = 'D:\Dev\cache\playwright'
$env:NEMU_P306_VRM = 'D:\Dev\nemu\content\nemu\nemu.vrm'
$env:NEMU_P306_PROFILE = 'D:\Dev\nemu\content\nemu\nemu.profile.json'
pnpm test:p306:preview
pnpm record:p306
pnpm test:p306:visual
```

录制前/每 30 秒检查 `Runner.Worker`；有活动作业就停止，不能与本机其他重负载并发。源文件以后若变化，按 manifest 哈希识别来源，不把后续资产当成本次验证的同一输入。重新执行时应先使用新的报告目录，保留原始证据。
