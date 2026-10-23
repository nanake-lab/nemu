# P2-05: 评审台弹簧面板与当前 three-vrm API/管线 YAML 不一致

来源：P2-05 实机调校准备，2026-10-21。

请求目录：`apps/stage/src/review/spring-bones.ts` 及对应测试（P1-03 维护范围）；本卡没有修改这些文件。

现状：锁定的 three-vrm-springbone 3.5.5 使用 `joint.settings.stiffness/dragForce/gravityPower/gravityDir`。现有面板把参数写在 joint 顶层 `stiffnessForce/dragForce/...`，`in` 检查不成立时静默不应用。其导出片段为 `rig.springBones`，资产管线 schema 要求 `springs.chains` 的每条骨链完整参数，两者不能直接合并。

期望接口：四类实际骨链可分别调节，基于 `joint.settings` 修改，导出可由当前 `build.schema.json` 校验的链片段，保留骨名、center、hitRadius 与碰撞组。明确恢复原参数和重置模拟的行为，并覆盖当前锁定 API 的实对象测试。

本卡继续完成候选生成、真实评审台截图和专用抖动证据；上述 UI 失配不能被算作用户已完成四类手感验收。可由 Antigravity 在评审台维护范围修复，或用户扩大本卡范围后处理。
