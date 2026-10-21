# Blender 无头资产管线

Blender **5.2.2**、VRM 扩展 **4.10.2**，两个锁分别存放在相邻的版本文件。每个步骤启动独立的 `blender -b [input.blend] --python script.py -- ...`，步骤之间保存 `.blend`；所有缓存默认在本目录 `.cache/`，不提交。开发机只用已安装工具和缓存，不联网下载。

## 使用

Python 3.12，PyYAML 6.0.3（MIT）、jsonschema 4.26.0（MIT）。测试工具使用 pytest 9.1.1（MIT）、mypy 1.20.2（MIT）、ruff 0.15.6（MIT）。本卡脚本不自动安装软件。

```powershell
uv venv tools/asset-pipeline/.venv
uv pip install --offline --python tools/asset-pipeline/.venv/Scripts/python.exe PyYAML==6.0.3 jsonschema==4.26.0
# 缓存缺失即停止，需另行准备环境。不要为本卡联网下载。
pnpm asset:build
pnpm asset:build -Config tools/asset-pipeline/build.example.yaml -Output tools/asset-pipeline/.cache/sample
pnpm asset:step -- tail in.blend out.blend -Config tools/asset-pipeline/build.example.yaml
```

`NEMU_BLENDER` 指向已有 Blender；未设置时，只在 `C:\Program Files\Blender Foundation\` 查找。`NEMU_ASSET_PYTHON` 可指向已有依赖的 Python；默认使用本目录 `.venv/Scripts/python.exe`。仓库已有 Node、pnpm、`@nemu/contracts`、asset-lint 也是运行前提。工具拒绝在已有 Blender、Electron、Runner.Worker 进程运行时开始，并使用跨输出目录的排他锁。锁因进程被强制终止而残留时，先检查锁内 PID 已退出，再手动删除该锁。

`--Config` 文件中的 `source` 相对仓库根目录；`-Output` 相对调用目录，默认本工具 `.cache/build`。本卡将 `content/` 和 `content-template/` 强制设为只读目标，源 VRM 永远只读。输出为 `nemu.vrm`、`nemu.profile.json`、`manifest.json`、逐步骤日志和 `.cache/NN-step.blend`。源码不下载资产，不通过 Blender MCP 操作。

## 配置

`build.schema.json` 要求所有小节和可调参数显式提供，拒绝重复键、非有限数及未知字段。`build.example.yaml` 是仓库样例 VRM 的机制测试配置，数值不是 Nemu 外观候选；不能复制到正式内容包后声称已获审美认可。它生成一条测试尾巴并测试三类表情绑定。

- `tail.curve` 为局部形状折线，按弧长归一到 `lengthM`，再产生 `boneCount` 节骨链。截面使用 Blender round bevel，分辨率由 `sectionResolution` 控制；`radiusCurve` 是弧长分数到米的分段线性曲线。`tipShape` 支持 flat/taper，后者的末端半径由 `tipRadiusM` 提供。网格生成 UV，权重在相邻骨骼间连续归一，不使用自动权重求解器的环境相关结果。
- `springs` 显式指定 append/replace、碰撞体、链和现有链是否挂新碰撞组。骨名可写实际名称或 VRM humanoid role；`["$tail"]` 展开为本次实测生成的尾骨。replace 只作用于缓存中的场景。
- `expressions` 的 preset 覆盖规则和 custom 的 morph/color/texture-transform 绑定完全来自配置。没有自动猜测形态键或角色情绪。
- `materials` 按名称指定 MToon 参数；未列出的材质保持源模型参数。新增尾巴材质复制指定头发材质，再应用 YAML 颜色。
- `textures.maxSize` 不大于 2048，保留长宽比，打包已使用贴图，移除无用户的贴图。
- `meta` 写入名称、版本、作者和内容许可地址。扩展导出的 VRM 1.0 `licenseUrl` 固定为 VRM 公共许可。脚本校验导出的 URL 与配置相同；不同则明确失败，要求先解决 CONTRACTS 1.5 与扩展的字段冲突。附加许可也写入 `otherLicenseUrl`；源模型的授权限制不被放宽。本卡样例使用原模型公共许可，两个 URL 相同。
- `profile` 的形状与词汇由现有 `@nemu/contracts` 验证；生成身高、头顶偏移和实际耳/尾骨名，其余字段从配置读取。JSON 测量包含节点变换，与 asset-lint 的 accessor bounds 判据一致。

最终总是调用未修改的 `pnpm asset:lint`。任何错误都会使正式构建失败；本工具没有忽略规则的生产开关。示例模型不具备的角色规则由报告逐条分类，保留 asset-lint 原始非零退出码。

## 无 Blender 的验证

在有 pytest/mypy/ruff 的 Python 3.12 环境执行：

```powershell
python -m pytest tools/asset-pipeline/tests
python -m mypy --strict --config-file tools/asset-pipeline/pyproject.toml tools/asset-pipeline tools/texture-fix/texture_fix.py
python -m ruff check tools/asset-pipeline tools/texture-fix
```

纯逻辑与 Blender 解耦；bpy 动态属性在本机 RNA 探针中核对并做集成验收。配置里的 profile 通过 Node 调用契约模块校验，不复制另一份跨进程类型。GPU/桌面/视觉负载与本管线串行。
