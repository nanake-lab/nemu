# P2-05 尾巴阶段的正式管线挂接请求

2026-10-21，P2-05 候选验证发现：源 VRoid 使用合并图集。`build_tail.py` 复制头发材质，尾巴新 UV 不对应图集位置，会采到衣服、皮肤和透明区域。仅删除贴图又使原本依赖贴图的白色 emissive/shade 因子变成白色发光。

本卡在授权的 `tools/asset-pipeline/tail/` 下实现候选阶段，保留头发 MToon 响应，仅对新尾巴移除原图集和白色发光，使用配置色板，不修改任何源材质。完整候选管线仍执行所有原阶段和原样 asset-lint，A1.3 保持非零。截图与审计见 `reports/P2-05/`。

请求扩展目录范围到 `tools/asset-pipeline/build_tail.py` 及其配置 schema（如需新参数），让正式 `pnpm asset:build` 复用这一材质处理。可以选择直接调用 tail 模块，或在 schema 中显式登记纹理模式和发光参数。用户选择候选之前不动正式 build.yaml，不移入正式资产目录。当前未修改这两个文件。
