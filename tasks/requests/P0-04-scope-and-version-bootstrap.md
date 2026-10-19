# P0-04 范围与 Blender 版本锁启动请求

## 原因

P0-04 任务卡列出既有 `docs/SETUP.md`，但根 `AGENTS.md` 只允许对既有 `docs/**` 追加报告、请求和 ADR，不允许改正文。当前实现因此没有改动 `docs/SETUP.md`。

此外，P0-04 要求检查 `tools/asset-pipeline/blender-version.txt`，该文件属于 P2-03 交付；P2-03 依赖 P0-05 第二部分，而 P0-05 第二部分又要求 P0-04 doctor 全部通过。没有初始精确版本锁时，doctor 无法通过这项检查。当前 Blender 4.5.14 也低于项目要求的 5.1，不能直接用它建立合格锁。

## 期望决策

请明确以下两点：

1. 是否允许 P0-04 按任务卡权限更新 `docs/SETUP.md`；若不允许，请指定承接 VRoid 导出说明的任务或文件。
2. 用户安装合格的 Blender 5.1+ 后，由谁建立第一份精确版本锁：用户在 P0-05 第二部分提供并登记，还是另行调整 P2-03 的依赖/交付顺序。不得降低 `>=5.1` 或跳过精确版本比较。

收到决定前，本分支保留 doctor 的严格检查结果，不修改 `docs/SETUP.md` 或 `tools/asset-pipeline/`。
