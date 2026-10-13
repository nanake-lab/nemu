---
name: stage-visual-check
description: 渲染、动画、表情、口型、界面类任务做视觉验证时使用。说明如何在浏览器里确定性地渲染 Nemu 舞台、截图、与基准图比对，以及截图要交给用户看什么。
---

# 舞台视觉验证

## 何时使用

任何会改变画面的任务：渲染器、光照、材质、动画层、表情、口型、气泡、评审台、面板界面。

## 步骤

1. 启动舞台：`pnpm dev:stage`（默认 `http://localhost:5174`，以终端输出为准）。
2. 打开确定性模式：在 URL 后加 `?deterministic=1`。此模式冻结时间、固定随机种子、关闭弹簧骨随机扰动，同样的参数每次渲染结果完全一致。
3. 用 URL 参数指定要看的画面：
   - `bg=white|black|gradient|wallpaper:<名称>`
   - `framing=fullBody|upperBody`
   - `expr=<表情名>:<权重>`，可以重复多个
   - `motion=<动作id>&t=<秒>`：定格在动作的某一时刻
   - `layers=L0,L1,L2,L3,L4`：只开启指定的动画层
   - `model=<路径>`：默认是内容包里的 `nemu.vrm`，测试时可以换成示例模型
4. 给人看的截图：Antigravity 用 `/browser` 打开上述页面截图（真实 GPU 渲染）。Codex 运行 `pnpm test:visual -- --update-candidates`，截图输出到 `reports/<任务号>/visual/`。
5. 与基准图比对：`pnpm test:visual`。基准图在 `apps/stage/tests/visual/__golden__/`。

## 两种截图不要混用

- **基准图比对**一律在软件渲染（Chromium 的 SwiftShader）下进行，`pnpm test:visual` 会强制这个模式，本机和 CI 的结果才能一致。基准图也必须在这个模式下生成。
- **`/browser` 截图**走真实 GPU，用来给用户看真实效果，不能拿来更新基准图，也不能和基准图做像素比对。

## 规则

- **不能自己更新基准图。** 画面确实需要改变时，把新旧对比图（同一参数下并排）放进报告的「待用户确认」，由用户批准后再更新。
- 截图必须同时包含白色和黑色背景，否则看不出透明边缘的问题。
- 比较「好不好看」不是你的职责；你的职责是证明「与预期一致」，或者把候选方案并排摆好让用户选。
- 每张截图的文件名包含全部 URL 参数，例如 `bg-black_framing-upperBody_expr-happy-0.8.png`，方便复现。

## 常见问题

- 截图边缘出现白边或黑边：检查 `premultipliedAlpha`、材质的 alpha 模式、是否误加了后处理。
- 两次截图不一致：确认 URL 中有 `deterministic=1`，并且没有依赖 `Date.now()` 或 `Math.random()` 的代码绕过了确定性时钟。
