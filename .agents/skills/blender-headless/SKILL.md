---
name: blender-headless
description: 编写或运行 Nemu 资产管线（tools/asset-pipeline）中的 Blender 脚本时使用。说明无头运行方式、API 版本核对、确定性要求和禁止事项。
---

# Blender 无头脚本

## 运行方式

```powershell
pnpm asset:build                                  # 完整管线
pnpm asset:step -- tail <in.blend> <out.blend>    # 只跑一步，调试用
```

Blender 可执行文件的位置从环境变量 `NEMU_BLENDER` 读取；没有设置时，在 `C:\Program Files\Blender Foundation\` 下查找。`tools/asset-pipeline/blender-version.txt` 只记录锁定的版本号，运行前会核对实际版本是否一致，不一致就报错。

每一步都是独立脚本，用 `blender -b <输入.blend> --python <脚本.py> -- <参数>` 运行，从 `sys.argv` 中 `--` 之后读参数，把结果另存为新的 `.blend`。不要在一个脚本里串起所有步骤。

## 编写规则

1. **先核对 API。** 模型容易写出 Blender 2.8–4.x 的旧接口。不确定某个 `bpy` 属性或算子时，先用 `blender -b --python-expr "import bpy; print(dir(bpy.types.<类型>))"` 查当前版本，或查官方 API 文档；不要猜。
2. **不用依赖界面上下文的算子。** 无头模式下没有 3D 视图，`bpy.ops` 中需要 `context.area` 的算子会失败。优先直接操作数据（`bpy.data`、`bmesh`）；确实要用算子时，用 `bpy.context.temp_override(...)` 提供所需上下文。
3. **确定性。** 不用随机数（必须用时固定种子）；遍历对象时按名称排序；导出时关闭写入时间戳的选项。连续运行两次，产物应当相同。
4. **参数全部来自 `build.yaml`。** 脚本里不出现颜色、长度、刚度等「审美」常数；缺少参数就报错并说明需要哪个字段。
5. **VRM Add-on 的接口。** VRM 相关数据（人形骨骼、表情、弹簧骨、MToon）通过 VRM Add-on for Blender 的属性读写（`armature.data.vrm_addon_extension`）。先检查插件版本是否与 `blender-version.txt` 旁边记录的版本一致。
6. **失败要说清楚。** 抛出的错误信息包含：哪一步、哪个对象或骨骼、期望什么、实际是什么。

## 测试

- 纯逻辑（解析配置、计算骨链长度和权重衰减）写成普通 Python 函数，用 pytest 在没有 Blender 的环境里测试。
- 需要 Blender 的部分，用 `tests/fixtures/` 里的小模型在本机跑集成测试；CI 中可选。

## 禁止

- 资产管线里不能使用 Blender MCP；MCP 只用于交互式检查，且必须开启安全模式。
- 不能修改 `content/nemu/source/` 下的源文件；只读源文件，产物写到 `.cache/` 和最终输出路径。
- 不能在脚本中联网下载任何东西。
